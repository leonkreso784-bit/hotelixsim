//! Ulazi za AI sloj (DEMO korak 4): sve što `ai` smije vidjeti gradi `game` iz dnevnika i sadržaja, a `ai` ne
//! poznaje ništa drugo. Mentor ne dobiva `DebriefInput` nego ljudske činjenice iz `mentor.rs`.

use crate::round::RoundKey;
use serde::Serialize;

use crate::dto::{LineDto, MonthCode, PlanDto, indices_dto};
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
    /// Šifra (npr. `S2`) samo za povezivanje s uzrocima u retcima; mentoru se ne šalje.
    pub code: String,
    pub title: String,
    pub quality: u8,
    pub free_text: bool,
    pub rationale: Option<String>,
    /// Neutralna presuda (AI nije stigao): ocjena 3 po pravilu.
    pub neutral: bool,
    /// Nitko nije odgovorio do zatvaranja (ocjena 1, DEMO §1.2).
    pub unanswered: bool,
    /// Oznaka ponuđenog odgovora koji je student izabrao; `None` za slobodan odgovor i bez odgovora.
    pub chosen_label: Option<String>,
}

/// Odlasci zaposlenih jednog odjela u krugu (samo v2); ime je iz sadržaja, ne šifra.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DepartmentDepartures {
    pub name: String,
    pub count: u16,
}

/// Ulaz za mentora nakon obračuna mjeseca.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DebriefInput {
    pub round: RoundKey,
    /// Idući mjesec igre; `None` = ovo je bio zadnji.
    pub next_round: Option<RoundKey>,
    /// Prošli mjesec igre; `None` = ovo je prvi mjesec, a „prije" je početno stanje hotela.
    pub prev_round: Option<RoundKey>,
    /// Vrsta kruga: "month" | "week".
    pub round_kind: String,
    /// Prvi i zadnji dan kruga: (dan u mjesecu od 1, mjesec).
    pub first_date: (u8, MonthCode),
    pub last_date: (u8, MonthCode),
    pub hotel_name: String,
    pub situations: Vec<DebriefSituation>,
    /// Pet indeksa (stotinke) na početku mjeseca, redoslijedom `INDEX_ORDER`.
    pub indices_before: [u16; 5],
    pub indices_after: [u16; 5],
    pub progress_hundredths: i32,
    /// Blagajna na kraju mjeseca.
    pub cash_end_cents: i64,
    /// Plan koji je stvarno obračunat i uobičajena cijena sobe za taj mjesec (kategorija hotela).
    pub plan: PlanDto,
    pub price_ref_cents: i64,
    /// Sezonci prije mjeseca (plan kaže koliko ih je poslije).
    pub seasonal_before: u16,
    /// Opterećenje osoblja (prodane noći po zaposlenom na dan, stotinke) prije i poslije mjeseca.
    pub load_before_hundredths: u16,
    pub load_after_hundredths: u16,
    /// Stopa odlazaka (baze, 300 = 3 % osoblja u mjesecu) prije i poslije mjeseca; ulazi u zadržavanje (Z).
    pub turnover_before_bp: u16,
    pub turnover_after_bp: u16,
    /// Odlasci po odjelu u krugu, od najvećeg broja (pa po šifri odjela); odjeli bez odlazaka se ne navode.
    /// Prazno za v1 igru i za krug bez odlazaka; zbroj je hotelski broj odlazaka.
    pub department_departures: Vec<DepartmentDepartures>,
    /// Plaće (% uobičajenih) kakve je hotel imao na početku kruga i kakve je imao na početku igre (profil), da mentor
    /// ne pripisuje studentu stanje koje je preuzeo.
    pub wage_before_pct: u16,
    pub wage_start_pct: u16,
    /// Plan prošlog kruga; `None` u prvom krugu (prije njega je samo početno stanje profila).
    pub plan_before: Option<PlanDto>,
    pub lines: Vec<LineDto>,
}

impl Game {
    pub fn assess_input(&self, hotel: HotelId, code: &str) -> Result<AssessInput, GameError> {
        let h = self.hotel(hotel)?;
        let (m, t) = self.situation_round(code)?;
        let fr = h
            .responses
            .get(&(m, code.to_owned()))
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
        let (_, t) = self.situation_round(code)?;
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

    pub fn debrief_input(
        &self,
        hotel: HotelId,
        m: impl Into<RoundKey>,
    ) -> Result<DebriefInput, GameError> {
        let m = m.into();
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
            .rounds
            .iter()
            .position(|&x| x == m)
            .ok_or(GameError::UnknownMonth)?;
        let prev_round = pos.checked_sub(1).and_then(|p| self.rounds.get(p));
        let indices_before = match prev_round {
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
            let t = self
                .situations_in(m)
                .into_iter()
                .find(|t| t.code == s.code)
                .ok_or_else(|| corrupt("situacija iz obračuna nije u sadržaju"))?;
            let chosen_label = match s.answer {
                Some(i) => Some(
                    t.answers
                        .get(usize::from(i))
                        .ok_or_else(|| corrupt("odgovor iz obračuna nije u sadržaju"))?
                        .label
                        .clone(),
                ),
                None => None,
            };
            let verdict = if s.free_text {
                h.responses
                    .get(&(m, s.code.clone()))
                    .and_then(|fr| fr.verdict.as_ref())
            } else {
                None
            };
            situations.push(DebriefSituation {
                code: s.code.clone(),
                title: t.title.clone(),
                quality: s.quality,
                free_text: s.free_text,
                rationale: verdict.map(|v| v.rationale.clone()),
                neutral: verdict.is_some_and(|v| v.nearest_answer.is_none()),
                unanswered: s.answer.is_none() && !s.free_text,
                chosen_label,
            });
        }
        let category = self.state_at(hotel, m)?.category();
        let price_ref_cents =
            self.model()
                .price_ref_cents(category, &self.round_info(m)?, &self.content)?;
        let report = &res.report;
        let department_departures = self.department_departures(hotel, res)?;
        let plan_before = match prev_round {
            None => None,
            Some(prev) => self
                .resolved
                .get(prev)
                .and_then(|r| r.hotels.iter().find(|x| x.hotel_id == hotel))
                .map(|x| x.plan.clone()),
        };
        let period = self.round_info(m)?.period;
        let (fd, fm) = period.first_date();
        let (ld, lm) = period.last_date();
        Ok(DebriefInput {
            round: m,
            next_round: pos.checked_add(1).and_then(|p| self.rounds.get(p)).copied(),
            prev_round: prev_round.copied(),
            round_kind: period.kind.code().to_owned(),
            first_date: (fd, fm.into()),
            last_date: (ld, lm.into()),
            hotel_name: h.setup.name.clone(),
            situations,
            indices_before,
            indices_after: res.indices,
            progress_hundredths: res.progress_hundredths,
            cash_end_cents: report.after.cash_cents,
            plan: res.plan.clone(),
            price_ref_cents,
            seasonal_before: report.before.seasonal,
            load_before_hundredths: report.before.load_hundredths,
            load_after_hundredths: report.after.load_hundredths,
            turnover_before_bp: report.before.turnover_bp,
            turnover_after_bp: report.after.turnover_bp,
            department_departures,
            wage_before_pct: report.before.wage_pct,
            wage_start_pct: h.start_state.summary().wage_pct,
            plan_before,
            lines: report.lines.clone(),
        })
    }
}
