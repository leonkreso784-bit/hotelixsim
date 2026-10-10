### Task 6: `app` — naredbe runde, zatvaranje i radnik obračuna

**Files:**
- Create: `crates/app/src/commands.rs`, `crates/app/src/jobs.rs`
- Modify: `crates/app/src/lib.rs` (`pub mod commands; pub mod jobs;` + `pub use`)
- Test: `crates/app/tests/rounds.rs`

**Interfaces:**
- Consumes: Task 5 (`Deps`, `Caller`, `Decision`, `NewJob`, `fold_raw`, `new_event`, `member_hotel`, `require_owner`,
  `round_view`, `situations`, `load_game`, `MAX_ATTEMPTS`), Task 3–4 (`set_plan`, `choose_answer`, `close_round`, `resolve_job`).
- Produces:
  - `set_plan(&Deps, &Caller, HotelId, MonthCode, PlanDto) -> Result<PlanDto, AppError>`
  - `answer(&Deps, &Caller, HotelId, &str, u8) -> Result<SituationsView, AppError>`
  - `round_status(&Deps, &Caller, GameId, MonthCode) -> Result<RoundView, AppError>`
  - `CloseOutcome { accepted: bool, round: RoundView }`; `close_round(&Deps, &Caller, GameId, MonthCode) -> Result<CloseOutcome, AppError>`
  - `RESOLVE_KIND = "resolve_round"`, `resolve_key(GameId, Month) -> String` (`"resolve:<uuid>:<jun|jul>"`),
    `ResolvePayload { month: MonthCode }` (serde), `LEASE_SECS: i32 = 30`,
    `run_next_job(&Deps) -> Result<bool, AppError>` (`true` = obradio posao; greška posla se zapiše u red pa vrati `Err`).

- [ ] **Step 1: Testovi (crveni).** `crates/app/tests/rounds.rs` (pomoćnici `setup`, `guest`, `robot_hotel` kao u
  `views.rs` — kopiraj ih, ne uvozi među test-datotekama):

```rust
use game::dto::MonthCode::{Aug, Jul, Jun};
use game::game::RoundStatus;

async fn events_of(mem: &Memory, g: GameId, kind: &str) -> usize {
    mem.events(g).iter().filter(|e| e.kind == kind).count()
}

#[tokio::test]
async fn plan_answer_close_then_worker_resolves_june() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let mut plan = my_hotel(&d, &me).await.unwrap().private.unwrap().plan.unwrap().plan;
    plan.marketing_cents = 500_000;
    assert_eq!(set_plan(&d, &me, s.hotel_id, Jun, plan.clone()).await.unwrap(), plan);
    answer(&d, &me, s.hotel_id, "S1", 0).await.unwrap();
    assert_eq!(round_status(&d, &me, s.game_id, Jun).await.unwrap().unanswered, 1);
    let c = close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert!(c.accepted);
    assert_eq!(c.round.status, RoundStatus::Closing);
    assert!(run_next_job(&d).await.unwrap());
    assert!(!run_next_job(&d).await.unwrap(), "red je prazan");
    assert_eq!(round_status(&d, &me, s.game_id, Jun).await.unwrap().status, RoundStatus::Resolved);
    let g = game_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(g.current.unwrap().month, Jul);
    assert!(g.hotels.iter().all(|h| h.months.len() == 1));
    assert_eq!(events_of(&mem, s.game_id, "RoundResolved").await, 1);
}

#[tokio::test]
async fn double_close_is_idempotent() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    assert!(close_round(&d, &me, s.game_id, Jun).await.unwrap().accepted);
    let again = close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert!(!again.accepted);
    assert_eq!(again.round.status, RoundStatus::Closing);
    assert_eq!(mem.jobs().len(), 1, "jedan posao po ključu");
    run_next_job(&d).await.unwrap();
    let after = close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert_eq!((after.accepted, after.round.status), (false, RoundStatus::Resolved));
    assert_eq!(events_of(&mem, s.game_id, "RoundClosed").await, 1);
}

#[tokio::test]
async fn plan_for_closed_month_rejected() {
    let (d, _) = setup();
    let (me, s) = guest(&d).await;
    let plan = my_hotel(&d, &me).await.unwrap().private.unwrap().plan.unwrap().plan;
    close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert_eq!(set_plan(&d, &me, s.hotel_id, Jun, plan).await.unwrap_err().code, ErrorCode::RoundNotOpen);
    assert_eq!(answer(&d, &me, s.hotel_id, "S1", 0).await.unwrap_err().code, ErrorCode::RoundNotOpen);
    assert_eq!(close_round(&d, &me, s.game_id, Jul).await.unwrap_err().code, ErrorCode::RoundNotOpen);
    assert_eq!(close_round(&d, &me, s.game_id, Aug).await.unwrap_err().code, ErrorCode::UnknownMonth);
}

#[tokio::test]
async fn crashed_worker_job_is_reclaimed_and_duplicate_job_is_noop() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    close_round(&d, &me, s.game_id, Jun).await.unwrap();
    let claimed = d.jobs.claim(30).await.unwrap().unwrap(); // radnik preuzme i „padne"
    assert_eq!(claimed.attempts, 1);
    assert!(!run_next_job(&d).await.unwrap(), "zakup još traje");
    mem.expire_leases();
    assert!(run_next_job(&d).await.unwrap());
    mem.push_job(NewJob { key: "dupli".into(), kind: RESOLVE_KIND.into(), game_id: s.game_id,
                          payload: serde_json::json!({"month": "jun"}) });
    assert!(run_next_job(&d).await.unwrap());
    assert_eq!(events_of(&mem, s.game_id, "RoundResolved").await, 1, "drugi posao ne obračunava ponovno");
    assert!(mem.jobs().iter().all(|j| j.status == app::memory::JobStatus::Done));
}

#[tokio::test]
async fn failing_job_is_recorded_and_eventually_failed() {
    let (d, mem) = setup();
    let (_, s) = guest(&d).await;
    mem.push_job(NewJob { key: "los".into(), kind: "nepoznat".into(), game_id: s.game_id, payload: serde_json::json!({}) });
    for _ in 0..MAX_ATTEMPTS {
        assert!(run_next_job(&d).await.is_err());
    }
    assert_eq!(mem.jobs()[0].status, app::memory::JobStatus::Failed);
    assert!(!run_next_job(&d).await.unwrap());
}

#[tokio::test]
async fn rights_by_role() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let (other, _) = guest(&d).await;
    let plan = my_hotel(&d, &me).await.unwrap().private.unwrap().plan.unwrap().plan;
    let robot = robot_hotel(&mem, s.game_id);
    assert_eq!(set_plan(&d, &me, robot, Jun, plan.clone()).await.unwrap_err().code, ErrorCode::NotYourHotel);
    assert_eq!(answer(&d, &me, robot, "S1", 0).await.unwrap_err().code, ErrorCode::NotYourHotel);
    assert_eq!(close_round(&d, &other, s.game_id, Jun).await.unwrap_err().code, ErrorCode::NotInGame);
    let anon = Caller { guest: None };
    assert_eq!(close_round(&d, &anon, s.game_id, Jun).await.unwrap_err().code, ErrorCode::GuestRequired);
    let mut bad = plan;
    bad.price_cents += 50;
    assert_eq!(set_plan(&d, &me, s.hotel_id, Jun, bad).await.unwrap_err().code, ErrorCode::PriceNotWholeEuro);
}

#[tokio::test]
async fn full_demo_season_june_and_july() {
    let (d, _) = setup();
    let (me, s) = guest(&d).await;
    close_round(&d, &me, s.game_id, Jun).await.unwrap();
    run_next_job(&d).await.unwrap();
    let v = situations(&d, &me, s.hotel_id).await.unwrap();
    assert_eq!(v.items.iter().map(|i| i.code.as_str()).collect::<Vec<_>>(), ["S2"]);
    close_round(&d, &me, s.game_id, Jul).await.unwrap();
    run_next_job(&d).await.unwrap();
    let g = game_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(g.current, None, "igra je gotova nakon srpnja");
    assert!(g.hotels.iter().all(|h| h.months.len() == 2));
    assert!(!close_round(&d, &me, s.game_id, Jul).await.unwrap().accepted);
}
```

- [ ] **Step 2: Pokreni** `cargo test -p app --test rounds; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 3: `commands.rs`.** Svaka naredba je **jedan** `decide`: provjera prava i pravila igre događa se nad
  tokom učitanim pod zaključavanjem, pa nema utrke između provjere i upisa.

```rust
use game::dto::{MonthCode, PlanDto};
use game::{Actor, GameId, HotelId};
use sim_hotel::{Month, MonthPlan};

use crate::views::{member_hotel, require_owner, round_view};
use crate::jobs::{RESOLVE_KIND, ResolvePayload, resolve_key};
use crate::{AppError, Caller, Decision, Deps, ErrorCode, NewJob, RawEvent, SituationsView, fold_raw, load_game, new_event};

async fn game_of_hotel(deps: &Deps, hotel: HotelId) -> Result<GameId, AppError> {
    deps.guests.game_of_hotel(hotel).await?.ok_or(AppError::new(ErrorCode::UnknownHotel))
}

pub async fn set_plan(deps: &Deps, caller: &Caller, hotel: HotelId, month: MonthCode, plan: PlanDto) -> Result<PlanDto, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let m: Month = month.into();
    let core = MonthPlan::from(&plan);
    deps.log
        .decide(game_id, Box::new(move |raw: &[RawEvent]| {
            let g = fold_raw(raw)?;
            require_owner(&g, guest, hotel)?;
            let ev = g.set_plan(hotel, m, core)?;
            Ok(Decision { events: vec![new_event(&Actor::Guest(guest), &ev)], ..Decision::default() })
        }))
        .await?;
    Ok(plan)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloseOutcome { pub accepted: bool, pub round: crate::RoundView }

/// „Zatvori mjesec" (BitLifeov „Age +"): `RoundClosed` i posao obračuna u istoj transakciji; drugi klik ne piše ništa.
pub async fn close_round(deps: &Deps, caller: &Caller, game_id: GameId, month: MonthCode) -> Result<CloseOutcome, AppError> {
    let guest = caller.require()?;
    let m: Month = month.into();
    let decided = deps
        .log
        .decide(game_id, Box::new(move |raw: &[RawEvent]| {
            let g = fold_raw(raw)?;
            member_hotel(&g, guest)?;
            match g.close_round(m)? {
                None => Ok(Decision::default()),
                Some(ev) => Ok(Decision {
                    events: vec![new_event(&Actor::Guest(guest), &ev)],
                    enqueue: Some(NewJob {
                        key: resolve_key(game_id, m),
                        kind: RESOLVE_KIND.to_owned(),
                        game_id,
                        payload: serde_json::to_value(ResolvePayload { month }).map_err(|e| AppError::internal(e.to_string()))?,
                    }),
                    finish_job: None,
                }),
            }
        }))
        .await?;
    let g = load_game(deps, game_id).await?;
    let mine = member_hotel(&g, guest)?;
    Ok(CloseOutcome { accepted: decided.appended > 0, round: round_view(&g, m, mine)? })
}
```
  - `answer(...)`: kao `set_plan`, s `g.choose_answer(hotel, &code, answer)` (kloniraj `code` u `String` prije
    `move`), pa vrati `situations(deps, caller, hotel).await`.
  - `round_status(...)`: `caller.require()`, `load_game`, `member_hotel`, `round_view(&g, month.into(), mine)`.
  - Redoslijed provjera u `decide`: `fold_raw` (nepostojeća igra → `UnknownGame`) → pravo (`member_hotel` /
    `require_owner`) → pravilo igre. Prazna igra se nikad ne stvara kroz `decide`.

- [ ] **Step 4: `jobs.rs`.**

```rust
use game::dto::MonthCode;
use game::GameId;
use serde::{Deserialize, Serialize};
use sim_hotel::Month;

use crate::{AppError, Decision, Deps, Job, RawEvent, fold_raw, new_event};

pub const RESOLVE_KIND: &str = "resolve_round";
/// Zakup posla (ARHITEKTURA §9.4): radnik koji ga ne završi u roku vraća ga u red.
pub const LEASE_SECS: i32 = 30;

pub fn resolve_key(game: GameId, m: Month) -> String {
    format!("resolve:{}:{}", game.0, m.code())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvePayload { pub month: MonthCode }

/// Jedan korak radnika: preuzmi posao, obradi ga, a grešku zapiši u red (ponovni pokušaj ili `failed`).
pub async fn run_next_job(deps: &Deps) -> Result<bool, AppError> {
    let Some(job) = deps.jobs.claim(LEASE_SECS).await? else { return Ok(false) };
    match run_job(deps, &job).await {
        Ok(()) => Ok(true),
        Err(e) => {
            deps.jobs.fail(job.id, format!("{e}")).await?;
            Err(e)
        }
    }
}

async fn run_job(deps: &Deps, job: &Job) -> Result<(), AppError> {
    if job.kind != RESOLVE_KIND {
        return Err(AppError::internal(format!("nepoznata vrsta posla {}", job.kind)));
    }
    let p: ResolvePayload = serde_json::from_value(job.payload.clone()).map_err(|e| AppError::internal(e.to_string()))?;
    let id = job.id;
    deps.log
        .decide(job.game_id, Box::new(move |raw: &[RawEvent]| {
            let g = fold_raw(raw)?;
            // idempotentno: već obračunata runda daje praznu odluku, ali posao se ipak zatvara
            let events = g.resolve_job(p.month.into())?.iter().map(|(a, e)| new_event(a, e)).collect();
            Ok(Decision { events, enqueue: None, finish_job: Some(id) })
        }))
        .await?;
    Ok(())
}
```

- [ ] **Step 5: Pokreni** `cargo test -p app; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 6: Provjere i commit** — `crates/app/src/commands.rs crates/app/src/jobs.rs crates/app/src/lib.rs
  crates/app/tests/rounds.rs`, poruka „Dnevnik 6/12: app — naredbe runde, zatvaranje i radnik obračuna".

---

