//! Presuda na slobodni odgovor (DEMO §7, ARHITEKTURA §8.3): oblik koji AI vraća, provjera, upis i neutralna presuda.
//! AI-ju se ne vjeruje: kvaliteta se računa iz ispunjenih kriterija, a oblik se provjerava prije upisa.

use serde::{Deserialize, Serialize};

use crate::error::GameError;
use crate::event::{AiMeta, AiSource, AnswerRule, CriterionVerdict, GameEvent, VerdictRecorded};
use crate::game::{Game, RoundStatus};
use crate::ids::HotelId;
use crate::own_words::{
    CLICK_MAX_QUALITY, NEUTRAL_OWN_WORDS_QUALITY, PASTED_QUALITY, PASTED_RATIONALE, copied_answer,
    pasted_answers,
};
use crate::round::RoundKey;

/// Razlog neutralne presude kad rok zatvaranja istekne (ARHITEKTURA §8.3 korak 5).
pub const NEUTRAL_DEADLINE: &str = "AI presuda nije stigla do kraja zatvaranja mjeseca; neutralna presuda po pravilu, na pregled profesoru.";
/// Razlog neutralne presude kad AI ne uspije ni nakon ponovljenih pokušaja.
pub const NEUTRAL_FAILED: &str = "AI presuda nije uspjela ni nakon ponovljenih pokušaja; neutralna presuda po pravilu, na pregled profesoru.";

/// Kvaliteta neutralne presude: srednja ocjena, pa učinak `(q − 3)` je 0 (mehanike `unanswered` bez nagrade i kazne).
const NEUTRAL_QUALITY: u8 = 3;

/// Oblik koji AI mora vratiti (JSON Schema u `ai` je zrcalo ovoga); ne vjeruje mu se dok ne prođe `validate_verdict`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerdictDraft {
    pub criteria: Vec<CriterionVerdict>,
    pub nearest_answer: u8,
    pub rationale: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VerdictError {
    /// Situacija nema slobodni odgovor (ili je nepoznata).
    NotFreeText,
    CriteriaCount(usize),
    CriterionNumber(u8),
    DuplicateCriterion(u8),
    EmptyReason(u8),
    NearestAnswer(u8),
    EmptyRationale,
    /// Tekst za studenta spominje redni broj ponuđenog odgovora ili kriterija (ADR-025).
    InternalNumber,
}

/// Presuda koja je prošla provjeru oblika; kvaliteta je izračunata, ne preuzeta.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidVerdict {
    pub criteria: Vec<CriterionVerdict>,
    pub nearest_answer: u8,
    pub quality: u8,
    pub rationale: String,
}

/// Ocjena je broj ispunjenih kriterija; 0 ili 1 → 1 (DEMO §7).
pub fn quality_from(criteria: &[CriterionVerdict]) -> u8 {
    let met = criteria.iter().filter(|c| c.met).count();
    u8::try_from(met).unwrap_or(5).clamp(1, 5)
}

/// Spominje li tekst redni broj ponuđenog odgovora ili kriterija („ponuđenom odgovoru 0", „kriterije 1 i 3",
/// „2. kriterij", „[2]"). Na ekranu ni ponuđeni odgovori ni kriteriji nemaju brojeve, pa studentu ništa ne znače.
fn names_internal_number(text: &str) -> bool {
    // Padežni oblici imenica; „odgovornost", „odgovorio" nisu poziv na ponuđeni odgovor.
    const NOUNS: [&str; 14] = [
        "odgovor",
        "odgovora",
        "odgovoru",
        "odgovorom",
        "odgovori",
        "odgovore",
        "odgovorima",
        "kriterij",
        "kriterija",
        "kriteriju",
        "kriterijem",
        "kriteriji",
        "kriterije",
        "kriterijima",
    ];
    let digits = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
    let bare = |w: &str| {
        w.trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase()
    };
    let words: Vec<&str> = text.split_whitespace().collect();
    words.iter().enumerate().any(|(i, w)| {
        let raw = w.trim_end_matches(['.', ',', ';', ':', '!', '?']);
        if raw.starts_with(['[', '('])
            && raw.ends_with([']', ')'])
            && digits(&raw[1..raw.len() - 1])
        {
            return true;
        }
        if !NOUNS.contains(&bare(w).as_str()) {
            return false;
        }
        // Redni broj ispred imenice: „2. kriterij".
        let ordinal_before = i > 0 && words[i - 1].strip_suffix('.').is_some_and(&digits);
        ordinal_before
            || words[i + 1..]
                .iter()
                .map(|n| bare(n))
                .find(|n| !matches!(n.as_str(), "br" | "broj" | "pod"))
                .is_some_and(|n| digits(&n))
    })
}

impl Game {
    /// Provjera presude po šifri situacije, bez hotela (sadržaj S1–S4, mjerilo, živi test): za šifru primjerka
    /// uzima prvi hotel koji ga ima. Kad je hotel poznat, treba `validate_verdict_for`.
    pub fn validate_verdict(
        &self,
        code: &str,
        draft: VerdictDraft,
    ) -> Result<ValidVerdict, VerdictError> {
        let free = self.ai_entry(code).is_some_and(|a| a.free_text);
        let answers = self
            .situation_round(code)
            .ok()
            .map(|(_, t)| t.answers.len());
        check_draft(free, answers, draft)
    }

    /// Provjera presude na slobodni odgovor hotela `hotel` (njegov primjerak situacije).
    pub fn validate_verdict_for(
        &self,
        hotel: HotelId,
        code: &str,
        draft: VerdictDraft,
    ) -> Result<ValidVerdict, VerdictError> {
        let free = self.ai_entry_for(hotel, code).is_some_and(|a| a.free_text);
        let answers = self
            .situation_round_for(hotel, code)
            .ok()
            .map(|(_, t)| t.answers.len());
        check_draft(free, answers, draft)
    }
}

/// Oblik presude: slobodan odgovor (`free`), broj ponuđenih odgovora situacije (`answers`), pet kriterija, obrazloženje.
fn check_draft(
    free: bool,
    answers: Option<usize>,
    draft: VerdictDraft,
) -> Result<ValidVerdict, VerdictError> {
    let (true, Some(answers)) = (free, answers) else {
        return Err(VerdictError::NotFreeText);
    };
    if draft.criteria.len() != 5 {
        return Err(VerdictError::CriteriaCount(draft.criteria.len()));
    }
    let mut seen = [false; 5];
    for c in &draft.criteria {
        if !(1..=5).contains(&c.n) {
            return Err(VerdictError::CriterionNumber(c.n));
        }
        let slot = &mut seen[usize::from(c.n) - 1];
        if *slot {
            return Err(VerdictError::DuplicateCriterion(c.n));
        }
        *slot = true;
        if c.reason.trim().is_empty() {
            return Err(VerdictError::EmptyReason(c.n));
        }
    }
    if usize::from(draft.nearest_answer) >= answers {
        return Err(VerdictError::NearestAnswer(draft.nearest_answer));
    }
    if draft.rationale.trim().is_empty() {
        return Err(VerdictError::EmptyRationale);
    }
    if names_internal_number(&draft.rationale)
        || draft
            .criteria
            .iter()
            .any(|c| names_internal_number(&c.reason))
    {
        return Err(VerdictError::InternalNumber);
    }
    let mut criteria = draft.criteria;
    criteria.sort_by_key(|c| c.n);
    let quality = quality_from(&criteria);
    Ok(ValidVerdict {
        criteria,
        nearest_answer: draft.nearest_answer,
        quality,
        rationale: draft.rationale,
    })
}

impl Game {
    /// Mjesec i je li presuda već upisana za slobodni odgovor hotela.
    fn response_slot(&self, hotel: HotelId, code: &str) -> Result<(RoundKey, bool), GameError> {
        let h = self.hotel(hotel)?;
        let (m, _) = self.situation_round_for(hotel, code)?;
        let fr = h
            .responses
            .get(&(m, code.to_owned()))
            .ok_or(GameError::UnknownSituation)?;
        Ok((m, fr.verdict.is_some()))
    }

    /// Odgovor postoji i još nema presudu (ni AI-jevu ni neutralnu).
    pub fn response_pending(&self, hotel: HotelId, code: &str) -> Result<bool, GameError> {
        let (_, has_verdict) = self.response_slot(hotel, code)?;
        Ok(!has_verdict)
    }

    /// Što je student napisao u odnosu na ponuđene odgovore (ADR-047, B13). Zalijepljeno ≥ 2 odgovora vrijedi uz oba
    /// pravila; prepisan jedan odgovor samo uz `OwnWords`. Provjera zalijepljenog ide prije prepisivanja.
    fn text_check(&self, hotel: HotelId, code: &str, m: RoundKey) -> Result<TextCheck, GameError> {
        let h = self.hotel(hotel)?;
        let Some(fr) = h.responses.get(&(m, code.to_owned())) else {
            return Ok(TextCheck::Own);
        };
        let t = self
            .hotel_situations(h, m)
            .into_iter()
            .find(|t| t.code == code)
            .ok_or(GameError::UnknownSituation)?;
        let labels: Vec<&str> = t.answers.iter().map(|a| a.label.as_str()).collect();
        if pasted_answers(&fr.text, &labels) {
            return Ok(TextCheck::Pasted);
        }
        if self.answer_rule() == AnswerRule::OwnWords
            && let Some(i) = copied_answer(&fr.text, &labels)
            && let Some(a) = t.answers.get(i)
            && let Ok(idx) = u8::try_from(i)
        {
            return Ok(TextCheck::Copied(idx, a.quality.min(CLICK_MAX_QUALITY)));
        }
        Ok(TextCheck::Own)
    }

    /// Upis presude: `VerdictRecorded` dok odgovor čeka (runda `Open`/`Closing`), inače `VerdictLate`.
    pub fn record_verdict(
        &self,
        hotel: HotelId,
        code: &str,
        v: ValidVerdict,
        meta: AiMeta,
    ) -> Result<GameEvent, GameError> {
        let (m, has_verdict) = self.response_slot(hotel, code)?;
        let mut rec = VerdictRecorded {
            hotel_id: hotel,
            month: m,
            situation: code.to_owned(),
            criteria: v.criteria,
            nearest_answer: Some(v.nearest_answer),
            quality: v.quality,
            rationale: v.rationale,
            source: AiSource::Ai,
            meta: Some(meta),
            review: false,
            copied: None,
            pasted: false,
        };
        match self.text_check(hotel, code, m)? {
            TextCheck::Own => {}
            TextCheck::Copied(i, q) => {
                rec.nearest_answer = Some(i);
                rec.quality = q;
                rec.copied = Some(i);
            }
            TextCheck::Pasted => {
                rec.criteria = Vec::new();
                rec.nearest_answer = None;
                rec.quality = PASTED_QUALITY;
                rec.rationale = PASTED_RATIONALE.to_owned();
                rec.pasted = true;
            }
        }
        let waiting = matches!(self.status(m)?, RoundStatus::Open | RoundStatus::Closing);
        Ok(if waiting && !has_verdict {
            GameEvent::VerdictRecorded(rec)
        } else {
            GameEvent::VerdictLate(rec)
        })
    }

    /// Neutralna presuda po pravilu (ARHITEKTURA §8.3 korak 5): mehanike `unanswered`, na pregled. Kvaliteta je 3
    /// (`Classic`) ili 4 (`OwnWords`, ADR-047 t. 4); prepisan odgovor vrijedi kao klik, zalijepljeni kao šutnja 3.
    pub fn neutral_verdict(
        &self,
        hotel: HotelId,
        code: &str,
        reason: &str,
    ) -> Result<GameEvent, GameError> {
        let (m, _) = self.response_slot(hotel, code)?;
        let quality = match self.answer_rule() {
            AnswerRule::Classic => NEUTRAL_QUALITY,
            AnswerRule::OwnWords => NEUTRAL_OWN_WORDS_QUALITY,
        };
        let mut rec = VerdictRecorded {
            hotel_id: hotel,
            month: m,
            situation: code.to_owned(),
            criteria: Vec::new(),
            nearest_answer: None,
            quality,
            rationale: reason.to_owned(),
            source: AiSource::Fallback,
            meta: None,
            review: true,
            copied: None,
            pasted: false,
        };
        match self.text_check(hotel, code, m)? {
            TextCheck::Own => {}
            TextCheck::Copied(i, q) => {
                rec.nearest_answer = Some(i);
                rec.quality = q;
                rec.copied = Some(i);
            }
            TextCheck::Pasted => {
                rec.quality = PASTED_QUALITY;
                rec.rationale = PASTED_RATIONALE.to_owned();
                rec.review = false;
                rec.pasted = true;
            }
        }
        Ok(GameEvent::VerdictRecorded(rec))
    }

    /// Slobodni odgovori u mjesecu `m` bez presude: hotelima redom slota, situacijama redom iz sadržaja.
    pub fn pending_verdicts(&self, m: impl Into<RoundKey>) -> Vec<(HotelId, String)> {
        let m = m.into();
        let mut out = Vec::new();
        for h in &self.hotels {
            for t in self.hotel_situations(h, m) {
                if h.responses
                    .get(&(m, t.code.clone()))
                    .is_some_and(|fr| fr.verdict.is_none())
                {
                    out.push((h.setup.hotel_id, t.code.clone()));
                }
            }
        }
        out
    }
}

/// Ishod provjere teksta slobodnog odgovora (vidi `Game::text_check`).
enum TextCheck {
    Own,
    /// Prepisan ponuđeni odgovor (redni broj, kvaliteta kao klik).
    Copied(u8, u8),
    Pasted,
}
