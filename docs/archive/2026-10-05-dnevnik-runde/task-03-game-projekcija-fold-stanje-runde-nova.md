### Task 3: `game` — projekcija (fold), stanje runde, nova igra, potezi robota i naredbe

**Files:**
- Create: `crates/game/src/game.rs` (Game, HotelRecord, RoundStatus, fold/apply, upiti)
- Create: `crates/game/src/command.rs` (`new_game`, `robot_moves`, `set_plan`, `choose_answer`, `close_round`, `plan_limits`)
- Modify: `crates/game/src/lib.rs` (`pub mod game; pub mod command;` i re-exporti)
- Test: `crates/game/tests/round.rs`

**Interfaces:**
- Consumes: Task 2 (`GameEvent`, `Recorded`, `Actor`, DTO-i, `GameError`), `content::Snapshot::build`,
  `sim_hotel::{HotelState::from_profile, RefTable::compute, start_indices, MonthPlan::{default_for, validate},
  Strategy::{plan, choose}, Profile, Category}`, `engine::{Rng, label}`.
- Produces:
  - `pub enum RoundStatus { Scheduled, Open, Closing, Resolved }` (`Clone, Copy, Debug, PartialEq, Eq`).
  - `pub struct HotelRecord { pub setup: HotelSetup, pub profile: Profile, pub start_state: HotelState,
    pub start_indices: IndexSet, pub plans: BTreeMap<Month, MonthPlan>, pub answers: BTreeMap<(Month, String), u8> }`.
  - `pub struct Game { pub id: GameId, pub seed: u64, pub months: Vec<Month>, pub content: content::Content,
    pub hotels: Vec<HotelRecord>, pub closed: BTreeMap<Month, Actor>, pub resolved: BTreeMap<Month, RoundResolved>,
    pub last_seq: i64, pub history: Vec<Recorded>, refs: BTreeMap<Category, RefTable> }` (`Clone, Debug`).
  - `Game::fold(&[Recorded]) -> Result<Game, GameError>`, `Game::apply(&mut self, Recorded) -> Result<(), GameError>`,
    `hotel(HotelId) -> Result<&HotelRecord, GameError>`, `current_month() -> Option<Month>`,
    `status(Month) -> Result<RoundStatus, GameError>`, `state_at(HotelId, Month) -> Result<HotelState, GameError>`,
    `latest_state(HotelId) -> Result<HotelState, GameError>`, `effective_plan(HotelId, Month) -> Result<MonthPlan, GameError>`,
    `situations_in(Month) -> Vec<&SituationTemplate>`, `unanswered_count(HotelId, Month) -> Result<u8, GameError>`,
    `refs(Category) -> Result<&RefTable, GameError>`.
  - `command::new_game(game_id: GameId, seed: u64, snapshot: &Snapshot, guest: GuestId, hotel_ids: [HotelId; 4])
    -> Result<Vec<(Actor, GameEvent)>, GameError>`;
    `Game::robot_moves(&self, Month) -> Result<Vec<(Actor, GameEvent)>, GameError>`;
    `Game::set_plan(&self, HotelId, Month, MonthPlan) -> Result<GameEvent, GameError>`;
    `Game::choose_answer(&self, HotelId, &str, u8) -> Result<GameEvent, GameError>`;
    `Game::close_round(&self, Month) -> Result<Option<GameEvent>, GameError>` (`None` = već zatvoreno ili obračunato);
    `pub struct PlanLimits { pub price_min_cents: i64, pub price_max_cents: i64, pub marketing_max_cents: i64,
    pub seasonal_min: u16, pub seasonal_max: u16, pub wage_min_pct: u16, pub wage_max_pct: u16,
    pub renovation_available: bool }` i `plan_limits(&HotelState, Month, &Catalog) -> Result<PlanLimits, GameError>`.
  - Pravila nove igre (ADR-015 t. 13, Leon 05.10.: **različiti profili**): igrač dobiva
    `profile_at(Rng::derive(seed, &[label("game:profile")]).range(0, 3))`; roboti (slotovi 2–4) dobivaju
    `profile_at(rot + slot)` uz `rot = Rng::derive(seed, &[label("game:robot-profiles")]).range(0, 3)`, pa su među
    robotima **sva tri profila** A, B, C (`profile_at(n)`: `n % 3` → 0 A, 1 B, 2 C); slotovi: 1 čovjek „Tvoj hotel", 2 robot `stedisa` „Štediša", 3 robot `brizni`
    „Brižni", 4 robot `lovac` „Lovac na popunjenost"; mjeseci `[Jun, Jul]`.
  - Kocka robota: plan `Rng::derive(seed, &[label("game:robot-plan"), mjesec.position(), slot])`, odgovor
    `Rng::derive(seed, &[label("game:robot-answer"), mjesec.position(), slot, label(šifra)])` (sve `u64`).

- [ ] **Step 1: Testovi (crveni).** `crates/game/tests/round.rs`:

```rust
use content::{Snapshot, repo_dir};
use game::command::*;
use game::dto::*;
use game::game::*;
use game::*;
use sim_hotel::{Month, MonthPlan, PlanError};
use uuid::Uuid;

pub fn hotel_ids() -> [HotelId; 4] {
    [11, 12, 13, 14].map(|n| HotelId(Uuid::from_u128(n)))
}
pub const GUEST: GuestId = GuestId(Uuid::from_u128(99));

/// Nova igra kao zapisani događaji (redni brojevi od 1).
pub fn created(seed: u64) -> Vec<Recorded> {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    new_game(GameId(Uuid::from_u128(1)), seed, &snap, GUEST, hotel_ids())
        .unwrap()
        .into_iter()
        .zip(1..)
        .map(|((actor, event), seq)| Recorded { seq, actor, event })
        .collect()
}

pub fn push(log: &mut Vec<Recorded>, actor: Actor, event: GameEvent) {
    let seq = log.last().map_or(1, |r| r.seq + 1);
    log.push(Recorded { seq, actor, event });
}

fn me() -> HotelId {
    hotel_ids()[0]
}

#[test]
fn new_game_writes_created_then_robot_moves_for_june() {
    let log = created(42);
    let kinds: Vec<(&str, String)> = log.iter().map(|r| (r.event.kind(), r.actor.as_db())).collect();
    assert_eq!(kinds[0], ("GameCreated", "system".into()));
    assert_eq!(log.len(), 1 + 3 * 3, "svaki robot: PlanSet + odgovor na S1 i S3");
    for (i, robot) in ["robot:stedisa", "robot:brizni", "robot:lovac"].iter().enumerate() {
        let k: Vec<&str> = log[1 + i * 3..4 + i * 3].iter().map(|r| r.event.kind()).collect();
        assert_eq!(k, ["PlanSet", "AnswerChosen", "AnswerChosen"], "{robot}");
        assert!(log[1 + i * 3..4 + i * 3].iter().all(|r| r.actor.as_db() == *robot));
    }
    let GameEvent::GameCreated(c) = &log[0].event else { panic!() };
    assert_eq!(c.ruleset_id, c.snapshot.fingerprint());
    assert_eq!(c.months, [MonthCode::Jun, MonthCode::Jul]);
    let mut robot_profiles: Vec<&str> = c.hotels[1..].iter().map(|h| h.profile.as_str()).collect();
    robot_profiles.sort_unstable();
    assert_eq!(robot_profiles, ["A", "B", "C"], "roboti pokrivaju sva tri profila");
    assert!(["A", "B", "C"].contains(&c.hotels[0].profile.as_str()));
    assert_eq!(c.hotels[0].controller, Controller::Human { guest_id: GUEST });
}

#[test]
fn statuses_follow_the_calendar() {
    let g = Game::fold(&created(42)).unwrap();
    assert_eq!(g.current_month(), Some(Month::Jun));
    assert_eq!(g.status(Month::Jun), Ok(RoundStatus::Open));
    assert_eq!(g.status(Month::Jul), Ok(RoundStatus::Scheduled));
    assert_eq!(g.status(Month::Aug), Err(GameError::UnknownMonth));
    assert_eq!(g.unanswered_count(me(), Month::Jun), Ok(2));
    assert_eq!(g.unanswered_count(hotel_ids()[1], Month::Jun), Ok(0));
}

#[test]
fn untouched_plan_is_default_and_last_plan_wins() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    let s = g.state_at(me(), Month::Jun).unwrap();
    let default = MonthPlan::default_for(&s, Month::Jun, &g.content.catalog).unwrap();
    assert_eq!(g.effective_plan(me(), Month::Jun).unwrap(), default);
    let mut p1 = default.clone();
    p1.marketing_cents = 500_000;
    let ev = g.set_plan(me(), Month::Jun, p1.clone()).unwrap();
    push(&mut log, Actor::Guest(GUEST), ev);
    let mut p2 = default.clone();
    p2.wage_pct = 105;
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.set_plan(me(), Month::Jun, p2.clone()).unwrap());
    assert_eq!(Game::fold(&log).unwrap().effective_plan(me(), Month::Jun).unwrap(), p2);
}

#[test]
fn invalid_plan_is_rejected_with_plan_error() {
    let g = Game::fold(&created(42)).unwrap();
    let mut p = g.effective_plan(me(), Month::Jun).unwrap();
    p.price_cents += 50;
    assert_eq!(g.set_plan(me(), Month::Jun, p), Err(GameError::Plan(PlanError::PriceNotWholeEuro)));
}

#[test]
fn answers_once_per_situation() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.choose_answer(me(), "S1", 0).unwrap());
    let g = Game::fold(&log).unwrap();
    assert_eq!(g.choose_answer(me(), "S1", 1), Err(GameError::AlreadyAnswered));
    assert_eq!(g.choose_answer(me(), "S3", 9), Err(GameError::UnknownAnswer));
    assert_eq!(g.choose_answer(me(), "S2", 0), Err(GameError::RoundNotOpen), "S2 je srpanj");
    assert_eq!(g.choose_answer(me(), "S4", 0), Err(GameError::UnknownSituation), "kolovoz nije u igri");
    assert_eq!(g.choose_answer(me(), "S9", 0), Err(GameError::UnknownSituation));
    assert_eq!(g.unanswered_count(me(), Month::Jun), Ok(1));
}

#[test]
fn stale_commands_are_rejected() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    let ev = g.close_round(Month::Jun).unwrap().expect("otvorena runda se zatvara");
    push(&mut log, Actor::Guest(GUEST), ev);
    let g = Game::fold(&log).unwrap();
    assert_eq!(g.status(Month::Jun), Ok(RoundStatus::Closing));
    assert_eq!(g.close_round(Month::Jun), Ok(None), "drugi klik ne piše ništa");
    let p = g.effective_plan(me(), Month::Jun).unwrap();
    assert_eq!(g.set_plan(me(), Month::Jun, p), Err(GameError::RoundNotOpen));
    assert_eq!(g.choose_answer(me(), "S1", 0), Err(GameError::RoundNotOpen));
    assert_eq!(g.close_round(Month::Jul), Err(GameError::RoundNotOpen), "srpanj još nije otvoren");
}

#[test]
fn fold_rejects_broken_logs() {
    let mut log = created(42);
    log[3].seq = 99;
    assert!(matches!(Game::fold(&log), Err(GameError::Corrupt(_))));
    let mut log = created(42);
    let GameEvent::GameCreated(c) = &mut log[0].event else { panic!() };
    let mut v = serde_json::to_value(&c.snapshot).unwrap();
    v["catalog"]["economy"]["hire_cents"] = serde_json::json!(1);
    c.snapshot = serde_json::from_value(v).unwrap(); // ruleset_id ostaje stari
    match Game::fold(&log) {
        Err(GameError::Corrupt(m)) => assert!(m.contains("otisak"), "{m}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn plan_limits_are_exactly_the_validation_bounds() {
    let g = Game::fold(&created(42)).unwrap();
    let s = g.state_at(me(), Month::Jun).unwrap();
    let cat = &g.content.catalog;
    let l = plan_limits(&s, Month::Jun, cat).unwrap();
    let base = MonthPlan::default_for(&s, Month::Jun, cat).unwrap();
    let with = |f: &dyn Fn(&mut MonthPlan)| {
        let mut p = base.clone();
        f(&mut p);
        p.validate(&s, Month::Jun, cat)
    };
    assert_eq!(with(&|p| p.price_cents = l.price_min_cents), Ok(()));
    assert_eq!(with(&|p| p.price_cents = l.price_min_cents - 100), Err(PlanError::PriceOutOfRange));
    assert_eq!(with(&|p| p.price_cents = l.price_max_cents), Ok(()));
    assert_eq!(with(&|p| p.price_cents = l.price_max_cents + 100), Err(PlanError::PriceOutOfRange));
    assert_eq!(with(&|p| p.seasonal_target = l.seasonal_min), Ok(()));
    assert_eq!(with(&|p| p.seasonal_target = l.seasonal_max), Ok(()));
    assert!(with(&|p| p.seasonal_target = l.seasonal_max + 1).is_err());
    assert_eq!(with(&|p| p.wage_pct = l.wage_max_pct), Ok(()));
    assert_eq!(with(&|p| p.wage_pct = l.wage_max_pct + 1), Err(PlanError::WageOutOfRange));
    assert_eq!(with(&|p| p.marketing_cents = l.marketing_max_cents), Ok(()));
    assert!(l.renovation_available);
}
```
  (`seasonal_min` može biti 0 pa se `- 1` ne testira; `seasonal_max + 1` pada ili na `SeasonalOutOfRange` ili na
  `SeasonalStepTooBig`.)

- [ ] **Step 2: Pokreni** `cargo test -p game --test round; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 3: `game.rs` — fold i upiti.**

```rust
use std::collections::BTreeMap;

use content::Content;
use sim_hotel::{
    Category, HotelState, IndexSet, Month, MonthPlan, Profile, RefTable, SituationTemplate, start_indices,
};

use crate::actor::Actor;
use crate::error::{GameError, corrupt};
use crate::event::{GameCreated, GameEvent, HotelSetup, Recorded, RoundResolved};
use crate::ids::{GameId, HotelId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundStatus {
    /// ZAKAZANA (ARHITEKTURA §8.1): prethodni mjesec još nije obračunat.
    Scheduled,
    Open,
    /// ZATVARANJE: gumb pritisnut, posao obračuna čeka u redu.
    Closing,
    Resolved,
}

// HotelRecord i Game točno kako stoji u Interfaces (derive `Clone, Debug`).

impl Game {
    pub fn fold(records: &[Recorded]) -> Result<Game, GameError> {
        let (first, rest) = records.split_first().ok_or_else(|| corrupt("prazan dnevnik"))?;
        let GameEvent::GameCreated(c) = &first.event else {
            return Err(corrupt("prvi događaj nije GameCreated"));
        };
        if first.seq != 1 {
            return Err(corrupt("GameCreated nije na rednom broju 1"));
        }
        let mut g = Game::created(c, first)?;
        for r in rest {
            g.apply(r.clone())?;
        }
        Ok(g)
    }

    fn created(c: &GameCreated, r: &Recorded) -> Result<Game, GameError> {
        if c.snapshot.fingerprint() != c.ruleset_id {
            return Err(corrupt("otisak pravila se ne slaže sa snimkom"));
        }
        let content = c.snapshot.build().map_err(|e| corrupt(format!("snimak: {e:?}")))?;
        let mut setups: Vec<HotelSetup> = c.hotels.clone();
        setups.sort_by_key(|h| h.slot);
        let mut refs = BTreeMap::new();
        let mut hotels = Vec::with_capacity(setups.len());
        for setup in setups {
            let profile = Profile::from_code(&setup.profile)
                .ok_or_else(|| corrupt(format!("profil {}", setup.profile)))?;
            let spec = content.catalog.profile(profile).map_err(|e| corrupt(format!("{e:?}")))?;
            let state = HotelState::from_profile(spec).map_err(|e| corrupt(format!("{e:?}")))?;
            if !refs.contains_key(&state.category) {
                let t = RefTable::compute(&content.catalog, state.category).map_err(|e| corrupt(format!("{e:?}")))?;
                refs.insert(state.category, t);
            }
            let table = refs.get(&state.category).ok_or_else(|| corrupt("referentna tablica"))?;
            let start = start_indices(&state, &content.catalog, table).map_err(|e| corrupt(format!("{e:?}")))?;
            hotels.push(HotelRecord {
                setup,
                profile,
                start_state: state,
                start_indices: start,
                plans: BTreeMap::new(),
                answers: BTreeMap::new(),
            });
        }
        Ok(Game {
            id: c.game_id,
            seed: c.seed,
            months: c.months.iter().map(|&m| m.into()).collect(),
            content,
            hotels,
            closed: BTreeMap::new(),
            resolved: BTreeMap::new(),
            last_seq: 1,
            history: vec![r.clone()],
            refs,
        })
    }

    /// Primijeni jedan zapisani događaj. Pravila su provjerena pri upisu; prekršaj ovdje znači pokvaren dnevnik.
    pub fn apply(&mut self, r: Recorded) -> Result<(), GameError> {
        if r.seq != self.last_seq + 1 {
            return Err(corrupt(format!("redni broj {} nakon {}", r.seq, self.last_seq)));
        }
        match &r.event {
            GameEvent::GameCreated(_) => return Err(corrupt("drugi GameCreated")),
            GameEvent::PlanSet(p) => {
                let m = p.month.into();
                self.expect_open(m)?;
                let plan = MonthPlan::from(&p.plan);
                self.hotel_mut(p.hotel_id)?.plans.insert(m, plan);
            }
            GameEvent::AnswerChosen(a) => {
                let m = a.month.into();
                self.expect_open(m)?;
                let h = self.hotel_mut(a.hotel_id)?;
                if h.answers.insert((m, a.situation.clone()), a.answer).is_some() {
                    return Err(corrupt(format!("dvostruki odgovor na {}", a.situation)));
                }
            }
            GameEvent::RoundClosed(c) => {
                let m = c.month.into();
                self.expect_open(m)?;
                self.closed.insert(m, r.actor.clone());
            }
            GameEvent::RoundResolved(rr) => {
                let m = rr.month.into();
                if self.status(m)? != RoundStatus::Closing {
                    return Err(corrupt("obračun runde koja nije zatvorena"));
                }
                self.resolved.insert(m, rr.clone());
            }
        }
        self.last_seq = r.seq;
        self.history.push(r);
        Ok(())
    }

    fn expect_open(&self, m: Month) -> Result<(), GameError> {
        match self.status(m)? {
            RoundStatus::Open => Ok(()),
            s => Err(corrupt(format!("događaj za rundu u stanju {s:?}"))),
        }
    }

    pub fn current_month(&self) -> Option<Month> {
        self.months.iter().copied().find(|m| !self.resolved.contains_key(m))
    }

    pub fn status(&self, m: Month) -> Result<RoundStatus, GameError> {
        if !self.months.contains(&m) {
            return Err(GameError::UnknownMonth);
        }
        Ok(if self.resolved.contains_key(&m) {
            RoundStatus::Resolved
        } else if self.closed.contains_key(&m) {
            RoundStatus::Closing
        } else if self.current_month() == Some(m) {
            RoundStatus::Open
        } else {
            RoundStatus::Scheduled
        })
    }

    /// Stanje hotela na POČETKU mjeseca `m`: početno stanje ili `after` iz obračuna prethodnog mjeseca.
    pub fn state_at(&self, hotel: HotelId, m: Month) -> Result<HotelState, GameError> {
        let h = self.hotel(hotel)?;
        let pos = self.months.iter().position(|&x| x == m).ok_or(GameError::UnknownMonth)?;
        let Some(prev) = pos.checked_sub(1).and_then(|p| self.months.get(p)) else {
            return Ok(h.start_state.clone());
        };
        let rr = self.resolved.get(prev).ok_or(GameError::RoundNotOpen)?;
        let res = rr
            .hotels
            .iter()
            .find(|x| x.hotel_id == hotel)
            .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))?;
        HotelState::try_from(&res.report.after)
    }
}
```
  Ostatak `impl Game` (svaki kratko, bez novih pravila):
  - `hotel(id)` / `hotel_mut(id)` (privatno) → `GameError::UnknownHotel` ako ga nema;
  - `latest_state(id)`: `after` zadnjeg obračunatog mjeseca (redoslijedom `months`) ili `start_state`;
  - `effective_plan(id, m)`: `plans.get(&m)` ili `MonthPlan::default_for(&self.state_at(id, m)?, m, &catalog)`
    (greška kataloga → `corrupt`);
  - `situations_in(m)`: `self.content.situations.iter().filter(|t| t.month == m).collect()` (redoslijed iz snimka);
  - `unanswered_count(id, m)`: broj predložaka mjeseca bez ključa `(m, code)` u `answers`, `u8::try_from` (→ `corrupt`);
  - `refs(c)`: `self.refs.get(&c).ok_or_else(|| corrupt("referentna tablica"))`.

- [ ] **Step 4: `command.rs` — nova igra, roboti, naredbe, granice plana.**

```rust
use engine::calc as c;
use engine::{Rng, label};
use sim_hotel::{Catalog, HotelState, Month, MonthPlan, Profile};

use crate::actor::{Actor, strategy_from_code};
use crate::dto::{MonthCode, PlanDto};
use crate::error::{GameError, corrupt};
use crate::event::*;
use crate::game::{Game, RoundStatus};
use crate::ids::{GameId, GuestId, HotelId};

const MONTHS: [Month; 2] = [Month::Jun, Month::Jul];
const ROBOTS: [(&str, &str); 3] = [("stedisa", "Štediša"), ("brizni", "Brižni"), ("lovac", "Lovac na popunjenost")];

fn pos(m: Month) -> Result<u64, GameError> {
    u64::try_from(m.position()).map_err(|_| corrupt("mjesec"))
}

pub fn new_game(
    game_id: GameId,
    seed: u64,
    snapshot: &content::Snapshot,
    guest: GuestId,
    hotel_ids: [HotelId; 4],
) -> Result<Vec<(Actor, GameEvent)>, GameError> {
    fn profile_at(n: u64) -> Profile {
        match n % 3 {
            0 => Profile::A,
            1 => Profile::B,
            _ => Profile::C,
        }
    }
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
            controller: Controller::Robot { strategy: code.into() },
        });
    }
    let created = GameCreated {
        game_id,
        seed,
        ruleset_id: snapshot.fingerprint(),
        core_version: sim_hotel::CORE_VERSION.into(),
        snapshot: snapshot.clone(),
        months: MONTHS.iter().map(|&m| m.into()).collect(),
        hotels,
    };
    let first = Recorded { seq: 1, actor: Actor::System, event: GameEvent::GameCreated(created) };
    let game = Game::fold(std::slice::from_ref(&first))?;
    let mut out = vec![(first.actor, first.event)];
    out.extend(game.robot_moves(Month::Jun)?);
    Ok(out)
}

impl Game {
    /// Roboti igraju istim naredbama kao čovjek (DEMO §1.3): PlanSet i AnswerChosen za mjesec koji se upravo otvara.
    pub fn robot_moves(&self, m: Month) -> Result<Vec<(Actor, GameEvent)>, GameError> {
        let cat = &self.content.catalog;
        let mut out = Vec::new();
        for h in &self.hotels {
            let Controller::Robot { strategy } = &h.setup.controller else { continue };
            let st = strategy_from_code(strategy).ok_or_else(|| corrupt(format!("strategija {strategy}")))?;
            let id = h.setup.hotel_id;
            let state = self.state_at(id, m)?;
            let slot = u64::from(h.setup.slot);
            let mut rng = Rng::derive(self.seed, &[label("game:robot-plan"), pos(m)?, slot]);
            let plan = st.plan(&state, m, cat, &mut rng).map_err(|e| corrupt(format!("{e:?}")))?;
            plan.validate(&state, m, cat).map_err(|e| corrupt(format!("robot {strategy}: {e:?}")))?;
            out.push((Actor::Robot(st), GameEvent::PlanSet(PlanSet { hotel_id: id, month: m.into(), plan: (&plan).into() })));
            for t in self.situations_in(m) {
                let mut rng = Rng::derive(self.seed, &[label("game:robot-answer"), pos(m)?, slot, label(&t.code)]);
                let i = st.choose(&t.answers, cat, state.category, &mut rng).map_err(|e| corrupt(format!("{e:?}")))?;
                let answer = u8::try_from(i).map_err(|_| corrupt("indeks odgovora"))?;
                out.push((Actor::Robot(st), GameEvent::AnswerChosen(AnswerChosen {
                    hotel_id: id,
                    month: m.into(),
                    situation: t.code.clone(),
                    answer,
                })));
            }
        }
        Ok(out)
    }

    pub fn set_plan(&self, hotel: HotelId, m: Month, plan: MonthPlan) -> Result<GameEvent, GameError> {
        if self.status(m)? != RoundStatus::Open {
            return Err(GameError::RoundNotOpen);
        }
        let state = self.state_at(hotel, m)?;
        plan.validate(&state, m, &self.content.catalog).map_err(GameError::Plan)?;
        Ok(GameEvent::PlanSet(PlanSet { hotel_id: hotel, month: m.into(), plan: (&plan).into() }))
    }

    pub fn choose_answer(&self, hotel: HotelId, code: &str, answer: u8) -> Result<GameEvent, GameError> {
        self.hotel(hotel)?;
        let t = self
            .content
            .situations
            .iter()
            .find(|t| t.code == code && self.months.contains(&t.month))
            .ok_or(GameError::UnknownSituation)?;
        if self.status(t.month)? != RoundStatus::Open {
            return Err(GameError::RoundNotOpen);
        }
        if usize::from(answer) >= t.answers.len() {
            return Err(GameError::UnknownAnswer);
        }
        let h = self.hotel(hotel)?;
        if h.answers.contains_key(&(t.month, t.code.clone())) {
            return Err(GameError::AlreadyAnswered);
        }
        Ok(GameEvent::AnswerChosen(AnswerChosen { hotel_id: hotel, month: t.month.into(), situation: t.code.clone(), answer }))
    }

    /// `None` = runda je već zatvorena ili obračunata (drugi klik vraća stanje, ne piše ništa).
    pub fn close_round(&self, m: Month) -> Result<Option<GameEvent>, GameError> {
        match self.status(m)? {
            RoundStatus::Open => Ok(Some(GameEvent::RoundClosed(RoundClosed { month: m.into() }))),
            RoundStatus::Closing | RoundStatus::Resolved => Ok(None),
            RoundStatus::Scheduled => Err(GameError::RoundNotOpen),
        }
    }
}

// PlanLimits (polja iz Interfaces, derive Clone, Copy, Debug, PartialEq, Eq).

/// Granice izbornika radnji za ekran — iste brojke koje `MonthPlan::validate` provjerava (test ih veže).
pub fn plan_limits(s: &HotelState, m: Month, cat: &Catalog) -> Result<PlanLimits, GameError> {
    let spec = cat.category(s.category).map_err(|e| corrupt(format!("{e:?}")))?;
    let a = &cat.actions;
    let reference = i128::from(spec.price_ref_cents.get(m));
    let calc = |e: engine::calc::CalcError| corrupt(format!("{e:?}"));
    // cijena u centima mora biti cijeli euro: najmanji euro ≥ min %, najveći euro ≤ max %
    let lo_eur = c::div_floor(c::add(c::mul(i128::from(a.price_min_pct), reference).map_err(calc)?, 9_999).map_err(calc)?, 10_000).map_err(calc)?;
    let hi_eur = c::div_floor(c::mul(i128::from(a.price_max_pct), reference).map_err(calc)?, 10_000).map_err(calc)?;
    Ok(PlanLimits {
        price_min_cents: c::narrow(c::mul(lo_eur, 100).map_err(calc)?).map_err(calc)?,
        price_max_cents: c::narrow(c::mul(hi_eur, 100).map_err(calc)?).map_err(calc)?,
        marketing_max_cents: a.marketing_max_cents,
        seasonal_min: s.seasonal.saturating_sub(a.seasonal_step),
        seasonal_max: s.seasonal.saturating_add(a.seasonal_step).min(a.seasonal_max),
        wage_min_pct: a.wage_min_pct,
        wage_max_pct: a.wage_max_pct,
        renovation_available: !s.renovated,
    })
}
```
  (`MonthCode`/`PlanDto` u `use` služe pretvorbama `.into()`; ako ih clippy prijavi kao neiskorištene, ukloni ih.)

- [ ] **Step 5: Pokreni** `cargo test -p game; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 6: Provjere i commit** — `crates/game/src/game.rs crates/game/src/command.rs crates/game/src/lib.rs
  crates/game/tests/round.rs`, poruka „Dnevnik 3/12: game — fold, stanje runde, nova igra, roboti i naredbe".

---

