use sim_hotel::{SituationOutcome, progress_pts};
use sim_hotel_v2::PeriodReport;

use crate::actor::Actor;
use crate::dto::indices_dto;
use crate::error::{GameError, corrupt};
use crate::event::{GameEvent, HotelResolution, Recorded, RoundResolved, SituationResult};
use crate::game::{Game, HotelRecord, RoundStatus};
use crate::round::RoundKey;
use crate::verdict::NEUTRAL_DEADLINE;

/// Kvaliteta situacije prepuštene voditelju odjela (Leon P4): bolja od šutnje (1), slabija od srednje (3).
pub const DELEGATED_QUALITY: u8 = 2;

/// Ishod odluke posla obračuna.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolveStep {
    Done(Vec<(Actor, GameEvent)>),
    /// Runda je zatvorena, ali presude za slobodne odgovore još nisu stigle i rok nije prošao.
    WaitForVerdicts(usize),
}

impl Game {
    /// Obračun hotela `h` u krugu `m` iz dnevnika: rezultati situacija za izvještaj i ishodi za jezgru. Isti račun za
    /// obračun kruga i za izvlačenje idućeg (koje gleda cijeli obračun prošlog kruga).
    fn hotel_outcomes(
        &self,
        h: &HotelRecord,
        m: RoundKey,
    ) -> Result<(Vec<SituationResult>, Vec<SituationOutcome>), GameError> {
        let model = self.model();
        let mut outcomes = Vec::new();
        let mut results = Vec::new();
        for t in self.hotel_situations(h, m) {
            let key = (m, t.code.clone());
            let no_penalty = self.no_penalty(h, m, &t.code);
            if let Some(&i) = h.answers.get(&key) {
                let a = t
                    .answers
                    .get(usize::from(i))
                    .ok_or_else(|| corrupt("indeks odgovora"))?;
                results.push(SituationResult {
                    code: t.code.clone(),
                    answer: Some(i),
                    quality: a.quality,
                    free_text: false,
                    delegated: false,
                });
                let o = SituationOutcome::from_answer(&t.code, a);
                outcomes.push(model.situation_outcome(o, no_penalty));
            } else if let Some(fr) = h.responses.get(&key) {
                // Slobodan odgovor: mehanike najbližeg ponuđenog odgovora, kvaliteta iz presude. Neutralna
                // presuda (`None`) nosi mehanike `unanswered` s kvalitetom 3: učinak `(q − 3)` je 0, a trošak
                // kompenzacije iz `unanswered` je 0 €. Situacija „samo nagrada“ (v2, S-4) ni tada ne snižava
                // pokazatelje (`Model::situation_outcome`).
                let v = fr
                    .verdict
                    .as_ref()
                    .ok_or_else(|| corrupt("presuda nedostaje"))?;
                let mechanics = match v.nearest_answer {
                    Some(i) => t
                        .answers
                        .get(usize::from(i))
                        .ok_or_else(|| corrupt("indeks odgovora"))?
                        .mechanics
                        .clone(),
                    None => t.unanswered.clone(),
                };
                results.push(SituationResult {
                    code: t.code.clone(),
                    answer: None,
                    quality: v.quality,
                    free_text: true,
                    delegated: false,
                });
                let o = SituationOutcome {
                    code: t.code.clone(),
                    mechanics,
                    quality: v.quality,
                };
                outcomes.push(model.situation_outcome(o, no_penalty));
            } else if h.delegated.contains(&key) {
                // Prepuštena voditelju: mehanike `unanswered` (isti trošak i smjer) uz kvalitetu 2, bez AI-ja.
                let o = SituationOutcome {
                    code: t.code.clone(),
                    mechanics: t.unanswered.clone(),
                    quality: DELEGATED_QUALITY,
                };
                results.push(SituationResult {
                    code: t.code.clone(),
                    answer: None,
                    quality: o.quality,
                    free_text: false,
                    delegated: true,
                });
                outcomes.push(model.situation_outcome(o, no_penalty));
            } else {
                let o = SituationOutcome::unanswered(t);
                results.push(SituationResult {
                    code: t.code.clone(),
                    answer: None,
                    quality: o.quality,
                    free_text: false,
                    delegated: false,
                });
                outcomes.push(model.situation_outcome(o, no_penalty));
            }
        }
        Ok((results, outcomes))
    }

    /// Cijeli obračun (v2) obračunatog kruga `m` za hotel `h`, ponovno izračunat iz dnevnika (stanje, plan i
    /// ishodi krugu su isti kao pri obračunu): ulaz izvlačenja idućeg kruga (popunjenost i opterećenje odjela).
    pub(crate) fn period_report(
        &self,
        h: &HotelRecord,
        m: RoundKey,
    ) -> Result<PeriodReport, GameError> {
        let id = h.setup.hotel_id;
        let state = self.state_at(id, m)?;
        let plan = self.effective_plan(id, m)?;
        let (_, outcomes) = self.hotel_outcomes(h, m)?;
        self.model().period_report(
            &state,
            &self.round_info(m)?,
            &plan,
            &outcomes,
            &self.content,
        )
    }

    /// Obračun runde za sve hotele, redom po slotu, iz snimka pravila igre (ARHITEKTURA §7.5; DEMO §1.2).
    pub fn resolve_round(&self, m: impl Into<RoundKey>) -> Result<RoundResolved, GameError> {
        let m = m.into();
        if self.status(m)? != RoundStatus::Closing {
            return Err(corrupt("obračun runde koja nije zatvorena"));
        }
        let info = self.round_info(m)?;
        let model = self.model();
        let err = |e: &dyn std::fmt::Debug| corrupt(format!("{e:?}"));
        let mut hotels = Vec::with_capacity(self.hotels.len());
        for h in &self.hotels {
            let id = h.setup.hotel_id;
            let state = self.state_at(id, m)?;
            let plan = self.effective_plan(id, m)?;
            let (results, outcomes) = self.hotel_outcomes(h, m)?;
            let refs = self.refs(state.category())?;
            let res = model.resolve(&state, &info, &plan, &outcomes, &self.content, refs)?;
            let progress = progress_pts(&h.start_indices, &res.indices).map_err(|e| err(&e))?;
            hotels.push(HotelResolution {
                hotel_id: id,
                plan: plan.to_dto(),
                situations: results,
                report: res.report,
                indices: indices_dto(&res.indices),
                progress_hundredths: progress,
                detail: res.detail,
            });
        }
        Ok(RoundResolved { month: m, hotels })
    }

    /// Odluka posla obračuna (red poslova, ADR-015 t. 8): idempotentna po stanju runde. Dok presude za slobodne
    /// odgovore nisu stigle i rok nije prošao, čeka; po isteku roka upisuje neutralne presude i obračunava.
    pub fn resolve_job(
        &self,
        m: impl Into<RoundKey>,
        deadline_passed: bool,
    ) -> Result<ResolveStep, GameError> {
        let m = m.into();
        match self.status(m)? {
            RoundStatus::Resolved => Ok(ResolveStep::Done(Vec::new())),
            RoundStatus::Closing => {
                let current = self.model().core_version();
                if self.core_version != current {
                    return Err(GameError::CoreMismatch(format!(
                        "igra je stvorena jezgrom {}, a radi {}",
                        self.core_version, current
                    )));
                }
                let pending = self.pending_verdicts(m);
                if !pending.is_empty() && !deadline_passed {
                    return Ok(ResolveStep::WaitForVerdicts(pending.len()));
                }
                let mut next = self.clone();
                let mut out = Vec::new();
                for (hotel, code) in pending {
                    let ev = next.neutral_verdict(hotel, &code, NEUTRAL_DEADLINE)?;
                    next.apply(Recorded {
                        seq: next.last_seq + 1,
                        actor: Actor::System,
                        event: ev.clone(),
                    })?;
                    out.push((Actor::System, ev));
                }
                let rr = next.resolve_round(m)?;
                next.apply(Recorded {
                    seq: next.last_seq + 1,
                    actor: Actor::System,
                    event: GameEvent::RoundResolved(rr.clone()),
                })?;
                out.push((Actor::System, GameEvent::RoundResolved(rr)));
                if let Some(n) = next.current_round() {
                    out.extend(next.open_round(n)?);
                }
                Ok(ResolveStep::Done(out))
            }
            RoundStatus::Open | RoundStatus::Scheduled => {
                Err(corrupt("posao obračuna za rundu koja nije zatvorena"))
            }
        }
    }
}
