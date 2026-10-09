//! Presuda na slobodni odgovor (DEMO §7, ARHITEKTURA §8.3): oblik koji AI vraća, provjera, upis i neutralna presuda.
//! AI-ju se ne vjeruje: kvaliteta se računa iz ispunjenih kriterija, a oblik se provjerava prije upisa.

use serde::{Deserialize, Serialize};

use crate::error::GameError;
use crate::event::{AiMeta, AiSource, CriterionVerdict, GameEvent, VerdictRecorded};
use crate::game::{Game, RoundStatus};
use crate::ids::HotelId;
use sim_hotel::Month;

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

impl Game {
    pub fn validate_verdict(
        &self,
        code: &str,
        draft: VerdictDraft,
    ) -> Result<ValidVerdict, VerdictError> {
        let free = self.content.ai.get(code).is_some_and(|a| a.free_text);
        let answers = self
            .content
            .situations
            .iter()
            .find(|t| t.code == code && self.months.contains(&t.month))
            .map(|t| t.answers.len());
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

    /// Mjesec i je li presuda već upisana za slobodni odgovor hotela.
    fn response_slot(&self, hotel: HotelId, code: &str) -> Result<(Month, bool), GameError> {
        let h = self.hotel(hotel)?;
        let m = self.situation_month(code)?;
        let fr = h
            .responses
            .get(&(m, code.to_owned()))
            .ok_or(GameError::UnknownSituation)?;
        Ok((m, fr.verdict.is_some()))
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
        let rec = VerdictRecorded {
            hotel_id: hotel,
            month: m.into(),
            situation: code.to_owned(),
            criteria: v.criteria,
            nearest_answer: Some(v.nearest_answer),
            quality: v.quality,
            rationale: v.rationale,
            source: AiSource::Ai,
            meta: Some(meta),
            review: false,
        };
        let waiting = matches!(self.status(m)?, RoundStatus::Open | RoundStatus::Closing);
        Ok(if waiting && !has_verdict {
            GameEvent::VerdictRecorded(rec)
        } else {
            GameEvent::VerdictLate(rec)
        })
    }

    /// Neutralna presuda po pravilu (ARHITEKTURA §8.3 korak 5): mehanike `unanswered`, kvaliteta 3, na pregled.
    pub fn neutral_verdict(
        &self,
        hotel: HotelId,
        code: &str,
        reason: &str,
    ) -> Result<GameEvent, GameError> {
        let (m, _) = self.response_slot(hotel, code)?;
        Ok(GameEvent::VerdictRecorded(VerdictRecorded {
            hotel_id: hotel,
            month: m.into(),
            situation: code.to_owned(),
            criteria: Vec::new(),
            nearest_answer: None,
            quality: NEUTRAL_QUALITY,
            rationale: reason.to_owned(),
            source: AiSource::Fallback,
            meta: None,
            review: true,
        }))
    }

    /// Slobodni odgovori u mjesecu `m` bez presude: hotelima redom slota, situacijama redom iz sadržaja.
    pub fn pending_verdicts(&self, m: Month) -> Vec<(HotelId, String)> {
        let mut out = Vec::new();
        for h in &self.hotels {
            for t in self.situations_in(m) {
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
