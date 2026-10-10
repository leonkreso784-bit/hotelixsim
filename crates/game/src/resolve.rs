use sim_hotel::{SituationOutcome, progress_pts};

use crate::actor::Actor;
use crate::dto::indices_dto;
use crate::error::{GameError, corrupt};
use crate::event::{GameEvent, HotelResolution, Recorded, RoundResolved, SituationResult};
use crate::game::{Game, RoundStatus};
use crate::round::RoundKey;
use crate::verdict::NEUTRAL_DEADLINE;

/// Ishod odluke posla obračuna.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolveStep {
    Done(Vec<(Actor, GameEvent)>),
    /// Runda je zatvorena, ali presude za slobodne odgovore još nisu stigle i rok nije prošao.
    WaitForVerdicts(usize),
}

impl Game {
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
            let mut outcomes = Vec::new();
            let mut results = Vec::new();
            for t in self.situations_in(m) {
                let key = (m, t.code.clone());
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
                    });
                    outcomes.push(SituationOutcome::from_answer(&t.code, a));
                } else if let Some(fr) = h.responses.get(&key) {
                    // Slobodan odgovor: mehanike najbližeg ponuđenog odgovora, kvaliteta iz presude. Neutralna
                    // presuda (`None`) nosi mehanike `unanswered` s kvalitetom 3: učinak `(q − 3)` je 0, a trošak
                    // kompenzacije iz `unanswered` je 0 €.
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
                    });
                    outcomes.push(SituationOutcome {
                        code: t.code.clone(),
                        mechanics,
                        quality: v.quality,
                    });
                } else {
                    let o = SituationOutcome::unanswered(t);
                    results.push(SituationResult {
                        code: t.code.clone(),
                        answer: None,
                        quality: o.quality,
                        free_text: false,
                    });
                    outcomes.push(o);
                }
            }
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
                    out.extend(next.robot_moves(n)?);
                }
                Ok(ResolveStep::Done(out))
            }
            RoundStatus::Open | RoundStatus::Scheduled => {
                Err(corrupt("posao obračuna za rundu koja nije zatvorena"))
            }
        }
    }
}
