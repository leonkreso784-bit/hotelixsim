//! Ulazi za AI sloj (DEMO korak 4): sve što `ai` smije vidjeti gradi `game` iz dnevnika i sadržaja, a `ai` ne
//! poznaje ništa drugo. Plus mentor bez AI-ja, deterministički iz brojki (ARHITEKTURA §8.5 „bez AI-ja").

use serde::Serialize;
use sim_hotel::Month;

use crate::dto::{INDEX_ORDER, IndexCode, LineDto, MonthCode, indices_dto};
use crate::error::{GameError, corrupt};
use crate::game::{ChatLine, Game, RoundStatus};
use crate::ids::HotelId;

/// Ponuđeni odgovor kako ga vidi sudac: oznaka i brojevi kriterija koje ispunjava.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OfferedForJudge {
    pub index: u8,
    pub label: String,
    pub criteria: Vec<u8>,
}

/// Ulaz za presudu na slobodni odgovor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AssessInput {
    pub situation: String,
    pub title: String,
    pub rubric: Vec<String>,
    pub character: Option<String>,
    pub offered: Vec<OfferedForJudge>,
    pub transcript: Vec<ChatLine>,
    pub response: String,
}

/// Lik kakvog ga vidi AI (bez rezervnog odgovora).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CharacterCard {
    pub name: String,
    pub role: String,
    pub brief: String,
    pub opening: String,
}

/// Ulaz za odgovor lika; prijepis ne sadrži uvodnu rečenicu (ona je u `character.opening`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CharacterInput {
    pub situation: String,
    pub title: String,
    pub character: CharacterCard,
    pub transcript: Vec<ChatLine>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DebriefSituation {
    pub title: String,
    pub quality: u8,
    pub free_text: bool,
    pub rationale: Option<String>,
    /// Neutralna presuda (AI nije stigao): ocjena 3 po pravilu.
    pub neutral: bool,
}

/// Ulaz za mentora nakon obračuna mjeseca.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DebriefInput {
    pub month: MonthCode,
    pub hotel_name: String,
    pub situations: Vec<DebriefSituation>,
    /// Pet indeksa (stotinke) na početku mjeseca, redoslijedom `INDEX_ORDER`.
    pub indices_before: [u16; 5],
    pub indices_after: [u16; 5],
    pub progress_hundredths: i32,
    pub lines: Vec<LineDto>,
}

impl Game {
    pub fn assess_input(&self, hotel: HotelId, code: &str) -> Result<AssessInput, GameError> {
        let h = self.hotel(hotel)?;
        let m = self.situation_month(code)?;
        let fr = h
            .responses
            .get(&(m, code.to_owned()))
            .ok_or(GameError::UnknownSituation)?;
        let t = self
            .situations_in(m)
            .into_iter()
            .find(|t| t.code == code)
            .ok_or(GameError::UnknownSituation)?;
        let ai = self.content.ai.get(code).ok_or(GameError::NotFreeText)?;
        let offered = t
            .answers
            .iter()
            .zip(0u8..)
            .map(|(a, index)| OfferedForJudge {
                index,
                label: a.label.clone(),
                criteria: ai
                    .criteria
                    .get(usize::from(index))
                    .cloned()
                    .unwrap_or_default(),
            })
            .collect();
        Ok(AssessInput {
            situation: code.to_owned(),
            title: t.title.clone(),
            rubric: ai.rubric.clone(),
            character: ai.character.as_ref().map(|c| c.name.clone()),
            offered,
            transcript: self.chat(hotel, code)?,
            response: fr.text.clone(),
        })
    }

    pub fn character_input(&self, hotel: HotelId, code: &str) -> Result<CharacterInput, GameError> {
        let m = self.situation_month(code)?;
        let t = self
            .situations_in(m)
            .into_iter()
            .find(|t| t.code == code)
            .ok_or(GameError::UnknownSituation)?;
        let c = self
            .content
            .ai
            .get(code)
            .and_then(|a| a.character.as_ref())
            .ok_or(GameError::NotFreeText)?;
        Ok(CharacterInput {
            situation: code.to_owned(),
            title: t.title.clone(),
            character: CharacterCard {
                name: c.name.clone(),
                role: c.role.clone(),
                brief: c.brief.clone(),
                opening: c.opening.clone(),
            },
            transcript: self.chat(hotel, code)?,
        })
    }

    pub fn debrief_input(&self, hotel: HotelId, m: Month) -> Result<DebriefInput, GameError> {
        let h = self.hotel(hotel)?;
        if self.status(m)? != RoundStatus::Resolved {
            return Err(GameError::RoundNotOpen);
        }
        let rr = self.resolved.get(&m).ok_or(GameError::RoundNotOpen)?;
        let res = rr
            .hotels
            .iter()
            .find(|x| x.hotel_id == hotel)
            .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))?;
        let pos = self
            .months
            .iter()
            .position(|&x| x == m)
            .ok_or(GameError::UnknownMonth)?;
        let indices_before = match pos.checked_sub(1).and_then(|p| self.months.get(p)) {
            None => indices_dto(&h.start_indices),
            Some(prev) => {
                self.resolved
                    .get(prev)
                    .and_then(|r| r.hotels.iter().find(|x| x.hotel_id == hotel))
                    .ok_or_else(|| corrupt("prošli mjesec nije obračunat"))?
                    .indices
            }
        };
        let mut situations = Vec::with_capacity(res.situations.len());
        for s in &res.situations {
            let title = self
                .situations_in(m)
                .into_iter()
                .find(|t| t.code == s.code)
                .map(|t| t.title.clone())
                .ok_or_else(|| corrupt("situacija iz obračuna nije u sadržaju"))?;
            let verdict = if s.free_text {
                h.responses
                    .get(&(m, s.code.clone()))
                    .and_then(|fr| fr.verdict.as_ref())
            } else {
                None
            };
            situations.push(DebriefSituation {
                title,
                quality: s.quality,
                free_text: s.free_text,
                rationale: verdict.map(|v| v.rationale.clone()),
                neutral: verdict.is_some_and(|v| v.nearest_answer.is_none()),
            });
        }
        Ok(DebriefInput {
            month: m.into(),
            hotel_name: h.setup.name.clone(),
            situations,
            indices_before,
            indices_after: res.indices,
            progress_hundredths: res.progress_hundredths,
            lines: res.report.lines.clone(),
        })
    }
}

fn index_name(i: IndexCode) -> &'static str {
    match i {
        IndexCode::G => "Zadovoljstvo gostiju",
        IndexCode::R => "Recenzije",
        IndexCode::Z => "Zaposlenici",
        IndexCode::O => "Operativa",
        IndexCode::F => "Financije",
    }
}

/// Mjesec u lokativu („U lipnju…").
fn month_locative(m: MonthCode) -> &'static str {
    match m {
        MonthCode::Jun => "lipnju",
        MonthCode::Jul => "srpnju",
        MonthCode::Aug => "kolovozu",
        MonthCode::Sep => "rujnu",
    }
}

/// Stotinke kao „12,05"; `signed` dodaje „+" ili „−" ispred.
fn fmt_hundredths(v: i32, signed: bool) -> String {
    let sign = if v < 0 {
        "-"
    } else if signed {
        "+"
    } else {
        ""
    };
    let a = v.unsigned_abs();
    format!("{sign}{},{:02}", a / 100, a % 100)
}

/// Mentor bez AI-ja: deterministički tekst iz brojki (ARHITEKTURA §8.5 „bez AI-ja").
pub fn fallback_debrief(input: &DebriefInput) -> String {
    let mut out = format!(
        "U {} je tvoj napredak u odnosu na početak {} boda.\n",
        month_locative(input.month),
        fmt_hundredths(input.progress_hundredths, true)
    );
    let moved = INDEX_ORDER
        .iter()
        .zip(input.indices_before.iter().zip(&input.indices_after))
        .map(|(&code, (&b, &a))| (code, i32::from(b), i32::from(a)))
        .fold(
            None,
            |best: Option<(IndexCode, i32, i32)>, cur| match best {
                Some(b) if (b.2 - b.1).abs() >= (cur.2 - cur.1).abs() => Some(b),
                _ => Some(cur),
            },
        );
    if let Some((code, b, a)) = moved.filter(|&(_, b, a)| a != b) {
        out.push_str(&format!(
            "Najviše se pomaknuo indeks {}: {} → {}.\n",
            index_name(code),
            fmt_hundredths(b, false),
            fmt_hundredths(a, false)
        ));
    }
    if !input.situations.is_empty() {
        let parts: Vec<String> = input
            .situations
            .iter()
            .map(|s| {
                let note = if s.neutral {
                    " (neutralna ocjena, na pregledu)"
                } else {
                    ""
                };
                format!("„{}\" — ocjena {}/5{note}", s.title, s.quality)
            })
            .collect();
        out.push_str(&format!("Situacije: {}.\n", parts.join("; ")));
    }
    out.push_str("Mentor danas govori iz brojki; AI objašnjenje nije bilo dostupno.");
    out
}
