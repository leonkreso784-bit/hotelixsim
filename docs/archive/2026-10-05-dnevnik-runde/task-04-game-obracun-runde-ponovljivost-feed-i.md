### Task 4: `game` — obračun runde, ponovljivost, feed i pogledi

**Files:**
- Create: `crates/game/src/resolve.rs` (`resolve_round`, `resolve_job`)
- Create: `crates/game/src/feed.rs` (`FeedItem`, `Game::feed`)
- Create: `crates/game/src/view.rs` (`PublicHotel`, `Seat`, `MonthScore`, `PrivateHotel`, `RealNumbers`, `LastMonth`, `PlanState`)
- Modify: `crates/game/src/lib.rs`
- Test: `crates/game/tests/resolve.rs` (koristi pomoćnike iz `tests/round.rs` — premjesti `hotel_ids`, `GUEST`,
  `created`, `push` u `crates/game/tests/common/mod.rs` i u oba testa `mod common; use common::*;`)

**Interfaces:**
- Consumes: Task 3 (`Game`, `RoundStatus`, `robot_moves`, `state_at`, `effective_plan`, `situations_in`, `refs`,
  `plan_limits`), `sim_hotel::{resolve_month, month_indices, progress_pts, SituationOutcome::{from_answer, unanswered}}`.
- Produces:
  - `Game::resolve_round(&self, Month) -> Result<RoundResolved, GameError>` (samo za `Closing`; hoteli po slotu).
  - `Game::resolve_job(&self, Month) -> Result<Vec<(Actor, GameEvent)>, GameError>`: `Resolved` → prazno;
    `Closing` → `[(System, RoundResolved)]` + `robot_moves(idući mjesec)` ako idući postoji; `Open`/`Scheduled` → `Corrupt`.
  - `pub enum FeedItem` (`Clone, Debug, PartialEq, Eq`):
    `HotelReceived { seq: i64, name: String, profile: String }` ·
    `PlanChanged { seq: i64, month: MonthCode, plan: PlanDto, by: Actor }` ·
    `AnswerChosen { seq: i64, month: MonthCode, situation: String, title: String, answer: u8, label: String, by: Actor }` ·
    `RoundClosed { seq: i64, month: MonthCode, by: Actor }` ·
    `SituationIgnored { seq: i64, month: MonthCode, situation: String, title: String }` ·
    `MonthResolved { seq: i64, month: MonthCode, indices: [u16; 5], progress_hundredths: i32 }` ·
    `Change { seq: i64, month: MonthCode, line: LineDto }`; `Game::feed(&self, HotelId) -> Result<Vec<FeedItem>, GameError>`.
  - `view`: `pub enum Seat { Human, Robot { strategy: String } }` (bez `guest_id` — tuđi gost se ne otkriva);
    `pub struct MonthScore { month: MonthCode, indices: [u16; 5], progress_hundredths: i32 }`;
    `pub struct PublicHotel { hotel_id: HotelId, slot: u8, name: String, seat: Seat, start: [u16; 5], months: Vec<MonthScore> }`;
    `pub struct RealNumbers { rooms: u16, permanent: u16, seasonal: u16, wage_pct: u16, morale_pts: i32,
    experience_pts: i32, equipment_pts: i32, satisfaction_pts: i32, rating: u16, cash_cents: i64, load_hundredths: u16 }`;
    `pub struct LastMonth { month: MonthCode, sold: u32, available: u32, rooms_revenue_cents: i64, gop_cents: i64, goppar_cents: i64 }`;
    `pub struct PlanState { month: MonthCode, plan: PlanDto, limits: PlanLimits }`;
    `pub struct PrivateHotel { profile: String, now: RealNumbers, last_month: Option<LastMonth>, plan: Option<PlanState> }`
    (sva polja `pub`, derive `Clone, Debug, PartialEq, Eq`);
    `Game::public_hotel(&self, HotelId) -> Result<PublicHotel, GameError>`, `Game::private_hotel(&self, HotelId) -> Result<PrivateHotel, GameError>`
    (`plan` = `Some` samo dok je trenutni mjesec `Open`).

- [ ] **Step 1: Testovi (crveni).** `crates/game/tests/resolve.rs`:

```rust
mod common;
use common::*;
use content::{Snapshot, repo_dir};
use game::command::new_game;
use game::dto::*;
use game::feed::FeedItem;
use game::game::*;
use game::*;
use sim_hotel::Month;
use uuid::Uuid;

fn me() -> HotelId {
    hotel_ids()[0]
}

/// Lipanj: igrač odgovori na S1 (indeks 0), S3 ostavi; zatvori.
fn june_closed(seed: u64) -> Vec<Recorded> {
    let mut log = created(seed);
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.choose_answer(me(), "S1", 0).unwrap());
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.close_round(Month::Jun).unwrap().unwrap());
    log
}

fn run_job(log: &mut Vec<Recorded>, m: Month) {
    for (a, e) in Game::fold(log).unwrap().resolve_job(m).unwrap() {
        push(log, a, e);
    }
}

#[test]
fn june_resolution_covers_four_hotels_and_unanswered_is_quality_one() {
    let g = Game::fold(&june_closed(42)).unwrap();
    let rr = g.resolve_round(Month::Jun).unwrap();
    assert_eq!(rr.hotels.iter().map(|h| h.hotel_id).collect::<Vec<_>>(), hotel_ids());
    let mine = &rr.hotels[0];
    assert_eq!(mine.situations[0], SituationResult { code: "S1".into(), answer: Some(0), quality: 5 });
    assert_eq!(mine.situations[1], SituationResult { code: "S3".into(), answer: None, quality: 1 });
    assert!(rr.hotels[1..].iter().all(|h| h.situations.iter().all(|s| s.answer.is_some())));
}

#[test]
fn unanswered_s1_costs_satisfaction() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.close_round(Month::Jun).unwrap().unwrap());
    let rr = Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap();
    let line = rr.hotels[0]
        .report
        .lines
        .iter()
        .find(|l| l.var == VarCode::Satisfaction && l.cause == CauseDto::Situation("S1".into()))
        .expect("neodgovorena S1 ima redak zadovoljstva");
    assert_eq!(line.amount, -400);
}

#[test]
fn replay_gives_byte_identical_round_resolved() {
    let log = june_closed(42);
    let first = Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap();
    // kao iz baze: JSON → tipizirano → JSON
    let stored: RoundResolved = serde_json::from_value(serde_json::to_value(&first).unwrap()).unwrap();
    let again = Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap();
    assert_eq!(serde_json::to_vec(&again).unwrap(), serde_json::to_vec(&first).unwrap());
    assert_eq!(serde_json::to_vec(&stored).unwrap(), serde_json::to_vec(&first).unwrap());
    let other = Game::fold(&june_closed(42)).unwrap().resolve_round(Month::Jun).unwrap();
    assert_eq!(other, first, "isto sjeme, iste naredbe → isti obračun");
}

#[test]
fn resolution_uses_snapshot_not_files() {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let mut v = serde_json::to_value(&snap).unwrap();
    for p in ["A", "B", "C"] {
        let f = v["profiles"][p]["fixed_cents"].as_i64().unwrap();
        v["profiles"][p]["fixed_cents"] = serde_json::json!(f + 100_000);
    }
    let changed: Snapshot = serde_json::from_value(v).unwrap();
    let log_of = |s: &Snapshot| {
        let mut log: Vec<Recorded> = new_game(GameId(Uuid::from_u128(1)), 42, s, GUEST, hotel_ids())
            .unwrap()
            .into_iter()
            .zip(1..)
            .map(|((actor, event), seq)| Recorded { seq, actor, event })
            .collect();
        let g = Game::fold(&log).unwrap();
        push(&mut log, Actor::Guest(GUEST), g.close_round(Month::Jun).unwrap().unwrap());
        Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap()
    };
    let a = log_of(&snap);
    let b = log_of(&changed);
    // fiksni troškovi dolaze iz snimka igre, ne iz datoteka u content/hotel
    assert_eq!(b.hotels[0].report.before.fixed_cents, a.hotels[0].report.before.fixed_cents + 100_000);
    assert_ne!(b.hotels[0].report.money.gop_cents, a.hotels[0].report.money.gop_cents);
}

#[test]
fn june_resolution_opens_july_with_robot_moves() {
    let mut log = june_closed(42);
    let before = log.len();
    run_job(&mut log, Month::Jun);
    let kinds: Vec<&str> = log[before..].iter().map(|r| r.event.kind()).collect();
    assert_eq!(kinds[0], "RoundResolved");
    assert_eq!(kinds.len(), 1 + 3 * 2, "svaki robot: PlanSet + S2");
    let g = Game::fold(&log).unwrap();
    assert_eq!(g.current_month(), Some(Month::Jul));
    assert_eq!(g.status(Month::Jul), Ok(RoundStatus::Open));
    let GameEvent::RoundResolved(rr) = &log[before].event else { panic!() };
    assert_eq!(HotelStateDto::from(&g.state_at(me(), Month::Jul).unwrap()), rr.hotels[0].report.after);
}

#[test]
fn resolve_job_twice_is_noop() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    assert_eq!(Game::fold(&log).unwrap().resolve_job(Month::Jun).unwrap(), vec![]);
}

#[test]
fn july_resolution_finishes_game() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.close_round(Month::Jul).unwrap().unwrap());
    let before = log.len();
    run_job(&mut log, Month::Jul);
    assert_eq!(log.len(), before + 1, "nakon srpnja nema novih poteza robota");
    let g = Game::fold(&log).unwrap();
    assert_eq!(g.current_month(), None);
    assert_eq!(g.close_round(Month::Jul), Ok(None));
    assert_eq!(g.public_hotel(me()).unwrap().months.len(), 2);
    assert_eq!(g.private_hotel(me()).unwrap().plan, None, "igra je gotova");
}

#[test]
fn feed_is_chronological_and_only_mine() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    let feed = Game::fold(&log).unwrap().feed(me()).unwrap();
    assert!(matches!(feed[0], FeedItem::HotelReceived { seq: 1, .. }));
    assert!(feed.windows(2).all(|w| seq(&w[0]) <= seq(&w[1])));
    assert!(feed.iter().any(|f| matches!(f, FeedItem::AnswerChosen { situation, by: Actor::Guest(_), .. } if situation == "S1")));
    assert!(feed.iter().any(|f| matches!(f, FeedItem::SituationIgnored { situation, .. } if situation == "S3")));
    assert!(feed.iter().any(|f| matches!(f, FeedItem::MonthResolved { month: MonthCode::Jun, .. })));
    assert!(feed.iter().any(|f| matches!(f, FeedItem::Change { line, .. } if line.cause == CauseDto::Situation("S1".into()))));
    assert!(!feed.iter().any(|f| matches!(f, FeedItem::PlanChanged { by: Actor::Robot(_), .. })), "tuđi potezi nisu u mom feedu");
}

fn seq(f: &FeedItem) -> i64 {
    match f {
        FeedItem::HotelReceived { seq, .. }
        | FeedItem::PlanChanged { seq, .. }
        | FeedItem::AnswerChosen { seq, .. }
        | FeedItem::RoundClosed { seq, .. }
        | FeedItem::SituationIgnored { seq, .. }
        | FeedItem::MonthResolved { seq, .. }
        | FeedItem::Change { seq, .. } => *seq,
    }
}

#[test]
fn views_split_public_and_private() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    let g = Game::fold(&log).unwrap();
    let p = g.public_hotel(hotel_ids()[1]).unwrap();
    assert_eq!(p.seat, game::view::Seat::Robot { strategy: "stedisa".into() });
    assert_eq!(p.months.len(), 1);
    let mine = g.private_hotel(me()).unwrap();
    let GameEvent::RoundResolved(rr) = &g.history.iter().rev().find(|r| r.event.kind() == "RoundResolved").unwrap().event else { panic!() };
    assert_eq!(mine.now.cash_cents, rr.hotels[0].report.after.cash_cents);
    assert_eq!(mine.last_month.as_ref().unwrap().month, MonthCode::Jun);
    let plan = mine.plan.expect("srpanj je otvoren");
    assert_eq!(plan.month, MonthCode::Jul);
}
```

- [ ] **Step 2: Pokreni** `cargo test -p game --test resolve; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 3: `resolve.rs`.**

```rust
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
                        let a = t.answers.get(usize::from(i)).ok_or_else(|| corrupt("indeks odgovora"))?;
                        results.push(SituationResult { code: t.code.clone(), answer: Some(i), quality: a.quality });
                        outcomes.push(SituationOutcome::from_answer(&t.code, a));
                    }
                    None => {
                        let o = SituationOutcome::unanswered(t);
                        results.push(SituationResult { code: t.code.clone(), answer: None, quality: o.quality });
                        outcomes.push(o);
                    }
                }
            }
            let report = resolve_month(&state, m, &plan, &outcomes, cat).map_err(|e| err(&e))?;
            let idx = month_indices(&report, cat, self.refs(state.category)?).map_err(|e| err(&e))?;
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
        Ok(RoundResolved { month: m.into(), hotels })
    }

    /// Odluka posla obračuna (red poslova, ADR-015 t. 8): idempotentna po stanju runde.
    pub fn resolve_job(&self, m: Month) -> Result<Vec<(Actor, GameEvent)>, GameError> {
        match self.status(m)? {
            RoundStatus::Resolved => Ok(Vec::new()),
            RoundStatus::Closing => {
                let rr = self.resolve_round(m)?;
                let mut next = self.clone();
                next.apply(Recorded { seq: self.last_seq + 1, actor: Actor::System, event: GameEvent::RoundResolved(rr.clone()) })?;
                let mut out = vec![(Actor::System, GameEvent::RoundResolved(rr))];
                if let Some(n) = next.current_month() {
                    out.extend(next.robot_moves(n)?);
                }
                Ok(out)
            }
            RoundStatus::Open | RoundStatus::Scheduled => Err(corrupt("posao obračuna za rundu koja nije zatvorena")),
        }
    }
}
```

- [ ] **Step 4: `feed.rs`.** `Game::feed(hotel)` prolazi `self.history` redom i za **taj** hotel izvodi stavke:
  `GameCreated` → `HotelReceived { seq, name, profile }`; `PlanSet` tog hotela → `PlanChanged { by: r.actor }`;
  `AnswerChosen` tog hotela → `AnswerChosen` s `title` i `label` iz predloška snimka (nepoznata šifra → `corrupt`);
  `RoundClosed` → `RoundClosed { by }` (zatvaranje je zajedničko, vidi ga svaki hotel); `RoundResolved` → za
  `HotelResolution` tog hotela redom: `SituationIgnored` za svaki `answer: None` (s naslovom), zatim `MonthResolved`,
  zatim `Change` za svaki redak `report.lines` (redoslijed iz jezgre). Sve stavke iz istog događaja nose isti `seq`.
  Nepoznat hotel → `GameError::UnknownHotel`.

- [ ] **Step 5: `view.rs`.** Strukture iz Interfaces. `public_hotel(id)`: `seat` iz `Controller` (bez `guest_id`),
  `start = indices_dto(&h.start_indices)`, `months` = za svaki obračunati mjesec redom `months` njegov `MonthScore`.
  `private_hotel(id)`: `profile = h.profile.code()`, `now` iz `latest_state(id)`; `last_month` iz zadnjeg obračunatog
  mjeseca (`report.market.sold`, `report.market.available`, `report.money.rooms_revenue_cents`, `gop_cents`,
  `goppar_cents`); `plan` = `Some(PlanState { month, plan: effective_plan, limits: plan_limits(state_at, …) })` samo
  ako je `current_month()` u stanju `Open`, inače `None`.

- [ ] **Step 6: Pokreni** `cargo test -p game; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 7: Provjere i commit** — `crates/game/src crates/game/tests`, poruka
  „Dnevnik 4/12: game — obračun runde, ponovljivost, feed i pogledi".

---
