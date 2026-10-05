use sim_hotel::{Month, SituationOutcome, month_indices, progress_pts, resolve_month};

use crate::actor::Actor;
use crate::dto::{MonthReportDto, indices_dto};
use crate::error::{GameError, corrupt};
use crate::event::{GameEvent, HotelResolution, Recorded, RoundResolved, SituationResult};
use crate::game::{Game, RoundStatus};

impl Game {
    /// Obračun runde za sve hotele, redom po slotu, iz snimka pravila igre (ARHITEKTURA §7.5; DEMO §1.2).
    pub fn resolve_round(&self, m: Month) -> Result<RoundResolved, GameError> {
        if self.status(m)? != RoundStatus::Closing {
            return Err(corrupt("obračun runde koja nije zatvorena"));
        }
        let cat = &self.content.catalog;
        let err = |e: &dyn std::fmt::Debug| corrupt(format!("{e:?}"));
        let mut hotels = Vec::with_capacity(self.hotels.len());
        for h in &self.hotels {
            let id = h.setup.hotel_id;
            let state = self.state_at(id, m)?;
            let plan = self.effective_plan(id, m)?;
            let mut outcomes = Vec::new();
            let mut results = Vec::new();
            for t in self.situations_in(m) {
                match h.answers.get(&(m, t.code.clone())) {
                    Some(&i) => {
                        let a = t
                            .answers
                            .get(usize::from(i))
                            .ok_or_else(|| corrupt("indeks odgovora"))?;
                        results.push(SituationResult {
                            code: t.code.clone(),
                            answer: Some(i),
                            quality: a.quality,
                        });
                        outcomes.push(SituationOutcome::from_answer(&t.code, a));
                    }
                    None => {
                        let o = SituationOutcome::unanswered(t);
                        results.push(SituationResult {
                            code: t.code.clone(),
                            answer: None,
                            quality: o.quality,
                        });
                        outcomes.push(o);
                    }
                }
            }
            let report = resolve_month(&state, m, &plan, &outcomes, cat).map_err(|e| err(&e))?;
            let idx =
                month_indices(&report, cat, self.refs(state.category)?).map_err(|e| err(&e))?;
            let progress = progress_pts(&h.start_indices, &idx).map_err(|e| err(&e))?;
            hotels.push(HotelResolution {
                hotel_id: id,
                plan: (&plan).into(),
                situations: results,
                report: MonthReportDto::from(&report),
                indices: indices_dto(&idx),
                progress_hundredths: progress,
            });
        }
        Ok(RoundResolved {
            month: m.into(),
            hotels,
        })
    }

    /// Odluka posla obračuna (red poslova, ADR-015 t. 8): idempotentna po stanju runde.
    pub fn resolve_job(&self, m: Month) -> Result<Vec<(Actor, GameEvent)>, GameError> {
        match self.status(m)? {
            RoundStatus::Resolved => Ok(Vec::new()),
            RoundStatus::Closing => {
                let rr = self.resolve_round(m)?;
                let mut next = self.clone();
                next.apply(Recorded {
                    seq: self.last_seq + 1,
                    actor: Actor::System,
                    event: GameEvent::RoundResolved(rr.clone()),
                })?;
                let mut out = vec![(Actor::System, GameEvent::RoundResolved(rr))];
                if let Some(n) = next.current_month() {
                    out.extend(next.robot_moves(n)?);
                }
                Ok(out)
            }
            RoundStatus::Open | RoundStatus::Scheduled => {
                Err(corrupt("posao obračuna za rundu koja nije zatvorena"))
            }
        }
    }
}
