use engine::{Rng, label};
use sim_hotel::{Catalog, HotelState, Month, MonthPlan, Profile, euro_bounds, seasonal_range};

use crate::actor::{Actor, strategy_from_code};
use crate::error::{GameError, corrupt};
use crate::event::*;
use crate::game::{Game, RoundStatus};
use crate::ids::{GameId, GuestId, HotelId};
use crate::round::RoundKey;

const MONTHS: [Month; 2] = [Month::Jun, Month::Jul];
const ROBOTS: [(&str, &str); 3] = [
    ("stedisa", "Štediša"),
    ("brizni", "Brižni"),
    ("lovac", "Lovac na popunjenost"),
];

fn pos(r: RoundKey) -> Result<u64, GameError> {
    u64::try_from(r.require_month()?.position()).map_err(|_| corrupt("mjesec"))
}

fn profile_at(n: u64) -> Profile {
    match n % 3 {
        0 => Profile::A,
        1 => Profile::B,
        _ => Profile::C,
    }
}

/// Događaji nove igre: `GameCreated` pa potezi robota za lipanj (redni brojevi od 1 dodjeljuje pozivatelj).
pub fn new_game(
    game_id: GameId,
    seed: u64,
    snapshot: &content::Snapshot,
    guest: GuestId,
    hotel_ids: [HotelId; 4],
) -> Result<Vec<(Actor, GameEvent)>, GameError> {
    let profile = profile_at(Rng::derive(seed, &[label("game:profile")]).range(0, 3));
    let rot = Rng::derive(seed, &[label("game:robot-profiles")]).range(0, 3);
    let [h1, h2, h3, h4] = hotel_ids;
    let mut hotels = vec![HotelSetup {
        hotel_id: h1,
        slot: 1,
        name: "Tvoj hotel".into(),
        profile: profile.code().into(),
        controller: Controller::Human { guest_id: guest },
    }];
    for ((id, slot), (code, name)) in [h2, h3, h4].into_iter().zip(2_u8..).zip(ROBOTS) {
        hotels.push(HotelSetup {
            hotel_id: id,
            slot,
            name: name.into(),
            profile: profile_at(rot + u64::from(slot)).code().into(),
            controller: Controller::Robot {
                strategy: code.into(),
            },
        });
    }
    let value = snapshot.to_value();
    let created = GameCreated {
        game_id,
        seed,
        ruleset_id: content::fingerprint_value(&value),
        core_version: sim_hotel::CORE_VERSION.into(),
        snapshot: Box::new(value),
        months: MONTHS.iter().map(|&m| m.into()).collect(),
        hotels,
    };
    let first = Recorded {
        seq: 1,
        actor: Actor::System,
        event: GameEvent::GameCreated(created),
    };
    let game = Game::fold(std::slice::from_ref(&first))?;
    let mut out = vec![(first.actor, first.event)];
    out.extend(game.robot_moves(Month::Jun)?);
    Ok(out)
}

impl Game {
    /// Roboti igraju istim naredbama kao čovjek (DEMO §1.3): PlanSet i AnswerChosen za mjesec koji se upravo otvara.
    pub fn robot_moves(
        &self,
        r: impl Into<RoundKey>,
    ) -> Result<Vec<(Actor, GameEvent)>, GameError> {
        let r = r.into();
        let m = r.require_month()?;
        let cat = &self.content.catalog;
        let mut out = Vec::new();
        for h in &self.hotels {
            let Controller::Robot { strategy } = &h.setup.controller else {
                continue;
            };
            let st = strategy_from_code(strategy)
                .ok_or_else(|| corrupt(format!("strategija {strategy}")))?;
            let id = h.setup.hotel_id;
            let state = self.state_at(id, r)?;
            let slot = u64::from(h.setup.slot);
            let mut rng = Rng::derive(self.seed, &[label("game:robot-plan"), pos(r)?, slot]);
            let plan = st
                .plan(&state, m, cat, &mut rng)
                .map_err(|e| corrupt(format!("{e:?}")))?;
            plan.validate(&state, m, cat)
                .map_err(|e| corrupt(format!("robot {strategy}: {e:?}")))?;
            out.push((
                Actor::Robot(st),
                GameEvent::PlanSet(PlanSet {
                    hotel_id: id,
                    month: r,
                    plan: (&plan).into(),
                }),
            ));
            for t in self.situations_in(r) {
                let mut rng = Rng::derive(
                    self.seed,
                    &[label("game:robot-answer"), pos(r)?, slot, label(&t.code)],
                );
                let i = st
                    .choose(&t.answers, cat, state.category, &mut rng)
                    .map_err(|e| corrupt(format!("{e:?}")))?;
                let answer = u8::try_from(i).map_err(|_| corrupt("indeks odgovora"))?;
                out.push((
                    Actor::Robot(st),
                    GameEvent::AnswerChosen(AnswerChosen {
                        hotel_id: id,
                        month: r,
                        situation: t.code.clone(),
                        answer,
                    }),
                ));
            }
        }
        Ok(out)
    }

    pub fn set_plan(
        &self,
        hotel: HotelId,
        m: impl Into<RoundKey>,
        plan: MonthPlan,
    ) -> Result<GameEvent, GameError> {
        let m = m.into();
        if self.status(m)? != RoundStatus::Open {
            return Err(GameError::RoundNotOpen);
        }
        let state = self.state_at(hotel, m)?;
        plan.validate(&state, m.require_month()?, &self.content.catalog)
            .map_err(GameError::Plan)?;
        Ok(GameEvent::PlanSet(PlanSet {
            hotel_id: hotel,
            month: m,
            plan: (&plan).into(),
        }))
    }

    pub fn choose_answer(
        &self,
        hotel: HotelId,
        code: &str,
        answer: u8,
    ) -> Result<GameEvent, GameError> {
        let h = self.hotel(hotel)?;
        let t = self
            .content
            .situations
            .iter()
            .find(|t| t.code == code && self.rounds.contains(&t.month.into()))
            .ok_or(GameError::UnknownSituation)?;
        let round = RoundKey::from(t.month);
        if self.status(round)? != RoundStatus::Open {
            return Err(GameError::RoundNotOpen);
        }
        if usize::from(answer) >= t.answers.len() {
            return Err(GameError::UnknownAnswer);
        }
        let key = (round, t.code.clone());
        if h.responses.contains_key(&key) || h.answers.contains_key(&key) {
            return Err(GameError::AlreadyAnswered);
        }
        Ok(GameEvent::AnswerChosen(AnswerChosen {
            hotel_id: hotel,
            month: round,
            situation: t.code.clone(),
            answer,
        }))
    }

    /// `None` = runda je već zatvorena ili obračunata (drugi klik vraća stanje, ne piše ništa).
    pub fn close_round(&self, m: impl Into<RoundKey>) -> Result<Option<GameEvent>, GameError> {
        let m = m.into();
        match self.status(m)? {
            RoundStatus::Open => Ok(Some(GameEvent::RoundClosed(RoundClosed { month: m }))),
            RoundStatus::Closing | RoundStatus::Resolved => Ok(None),
            RoundStatus::Scheduled => Err(GameError::RoundNotOpen),
        }
    }
}

/// Granice izbornika radnji za ekran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanLimits {
    pub price_min_cents: i64,
    pub price_max_cents: i64,
    pub marketing_max_cents: i64,
    pub seasonal_min: u16,
    pub seasonal_max: u16,
    pub wage_min_pct: u16,
    pub wage_max_pct: u16,
    pub renovation_available: bool,
}

/// Iste brojke koje `MonthPlan::validate` provjerava (test ih veže); koristi jezgrine funkcije, ne kopije.
pub fn plan_limits(s: &HotelState, m: Month, cat: &Catalog) -> Result<PlanLimits, GameError> {
    let spec = cat
        .category(s.category)
        .map_err(|e| corrupt(format!("{e:?}")))?;
    let a = &cat.actions;
    let reference = i128::from(spec.price_ref_cents.get(m));
    let (lo_eur, hi_eur) = euro_bounds(
        reference,
        i128::from(a.price_min_pct),
        i128::from(a.price_max_pct),
    )
    .map_err(|e| corrupt(format!("{e:?}")))?;
    let cents = |eur: i128| {
        eur.checked_mul(100)
            .and_then(|c| i64::try_from(c).ok())
            .ok_or_else(|| corrupt("granica cijene"))
    };
    let (seasonal_min, seasonal_max) = seasonal_range(s, cat);
    Ok(PlanLimits {
        price_min_cents: cents(lo_eur)?,
        price_max_cents: cents(hi_eur)?,
        marketing_max_cents: a.marketing_max_cents,
        seasonal_min,
        seasonal_max,
        wage_min_pct: a.wage_min_pct,
        wage_max_pct: a.wage_max_pct,
        renovation_available: !s.renovated,
    })
}
