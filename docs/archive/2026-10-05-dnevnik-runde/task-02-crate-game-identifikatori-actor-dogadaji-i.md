### Task 2: Crate `game` — identifikatori, actor, događaji i DTO-i

**Files:**
- Create: `crates/game/Cargo.toml`, `crates/game/src/{lib,ids,actor,dto,event,error}.rs`
- Modify: `Cargo.toml` (member `crates/game`; workspace deps `game = { path = "crates/game" }`,
  `uuid = { version = "1", features = ["serde"] }`)
- Modify: `scripts/check-core-deps.mjs` (pravilo čistoće za `game`)
- Test: `crates/game/tests/events.rs`

**Interfaces:**
- Consumes: `content::Snapshot`, `sim_hotel::{HotelState, MonthPlan, MonthReport, Month, Strategy, Category, IndexSet, Index}`.
- Produces (sve `pub`, re-export iz `lib.rs`):
  - `GameId(pub Uuid)`, `HotelId(pub Uuid)`, `GuestId(pub Uuid)` — `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
    Serialize, Deserialize`, `#[serde(transparent)]`; `GameId::stream(self) -> String` (`"game:<uuid>"`).
  - `enum Actor { Guest(GuestId), Robot(Strategy), System }` s `as_db(&self) -> String` i `parse(&str) -> Option<Actor>`;
    `strategy_from_code(&str) -> Option<Strategy>`.
  - `dto`: `MonthCode`, `TrainingCode`, `MaintenanceCode`, `IndexCode`, `INDEX_ORDER: [IndexCode; 5]`, `PlanDto`,
    `HotelStateDto`, `PendingDto`, `GroupBlockDto`, `MarketDto`, `PeopleDto`, `MoneyDto`, `GuestDto`, `VarCode`,
    `CauseDto`, `UnitCode`, `LineDto`, `MonthReportDto`, `indices_dto(&IndexSet) -> [u16; 5]`.
  - `event`: `SCHEMA_VER: i32 = 1`, `GameCreated`, `HotelSetup`, `Controller`, `PlanSet`, `AnswerChosen`, `RoundClosed`,
    `RoundResolved`, `HotelResolution`, `SituationResult`, `enum GameEvent`, `GameEvent::{kind, to_payload, from_record}`,
    `EventError`, `struct Recorded { seq: i64, actor: Actor, event: GameEvent }`.
  - `error`: `enum GameError` (cijeli popis ovdje, koriste ga Task 3–4).

- [ ] **Step 1: Crate i ovisnosti.** `crates/game/Cargo.toml`:

```toml
[package]
name = "game"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
engine.workspace = true
sim-hotel.workspace = true
content.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true

[lints]
workspace = true
```
  `lib.rs`:

```rust
//! Igra kao dnevnik događaja (ARHITEKTURA §9.1, ADR-015): događaji, projekcija (fold), runde, roboti i obračun runde.
//! Čisto: bez baze, mreže, sata i vlastite slučajnosti — sve što treba dolazi iz dnevnika i snimka pravila.

pub mod actor;
pub mod dto;
pub mod error;
pub mod event;
pub mod ids;

pub use actor::{Actor, strategy_from_code};
pub use error::GameError;
pub use event::*;
pub use ids::{GameId, GuestId, HotelId};
```

- [ ] **Step 2: Testovi (crveni).** `crates/game/tests/events.rs`:

```rust
use content::{Snapshot, repo_dir};
use game::dto::*;
use game::*;
use serde_json::json;
use sim_hotel::{HotelState, Month, MonthPlan, Profile, Strategy, resolve_month};
use uuid::Uuid;

fn ids() -> (GameId, HotelId, GuestId) {
    (GameId(Uuid::from_u128(1)), HotelId(Uuid::from_u128(2)), GuestId(Uuid::from_u128(3)))
}

fn plan() -> PlanDto {
    PlanDto {
        price_cents: 14_500,
        marketing_cents: 500_000,
        seasonal_target: 18,
        wage_pct: 103,
        training: TrainingCode::Basic,
        maintenance: MaintenanceCode::Regular,
        renovation: false,
        reply_reviews: true,
    }
}

fn june_report() -> (HotelState, sim_hotel::MonthReport) {
    let c = Snapshot::read(&repo_dir()).unwrap().build().unwrap();
    let s = HotelState::from_profile(c.catalog.profile(Profile::C).unwrap()).unwrap();
    let p = MonthPlan::default_for(&s, Month::Jun, &c.catalog).unwrap();
    let r = resolve_month(&s, Month::Jun, &p, &[], &c.catalog).unwrap();
    (s, r)
}

fn samples() -> Vec<GameEvent> {
    let (g, h, guest) = ids();
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let (_, r) = june_report();
    vec![
        GameEvent::GameCreated(GameCreated {
            game_id: g,
            seed: u64::MAX,
            ruleset_id: snap.fingerprint(),
            core_version: sim_hotel::CORE_VERSION.into(),
            snapshot: snap,
            months: vec![MonthCode::Jun, MonthCode::Jul],
            hotels: vec![
                HotelSetup { hotel_id: h, slot: 1, name: "Tvoj hotel".into(), profile: "C".into(),
                             controller: Controller::Human { guest_id: guest } },
                HotelSetup { hotel_id: HotelId(Uuid::from_u128(4)), slot: 2, name: "Štediša".into(),
                             profile: "C".into(), controller: Controller::Robot { strategy: "stedisa".into() } },
            ],
        }),
        GameEvent::PlanSet(PlanSet { hotel_id: h, month: MonthCode::Jun, plan: plan() }),
        GameEvent::AnswerChosen(AnswerChosen { hotel_id: h, month: MonthCode::Jun, situation: "S1".into(), answer: 2 }),
        GameEvent::RoundClosed(RoundClosed { month: MonthCode::Jun }),
        GameEvent::RoundResolved(RoundResolved {
            month: MonthCode::Jun,
            hotels: vec![HotelResolution {
                hotel_id: h,
                plan: plan(),
                situations: vec![
                    SituationResult { code: "S1".into(), answer: Some(2), quality: 1 },
                    SituationResult { code: "S3".into(), answer: None, quality: 1 },
                ],
                report: MonthReportDto::from(&r),
                indices: [5000, 5100, 4900, 5000, 5200],
                progress_hundredths: -37,
            }],
        }),
    ]
}

#[test]
fn every_event_roundtrips_through_record() {
    for e in samples() {
        let back = GameEvent::from_record(e.kind(), SCHEMA_VER, e.to_payload()).unwrap();
        assert_eq!(back, e, "{}", e.kind());
    }
}

#[test]
fn record_kinds_are_stable_names() {
    let kinds: Vec<&str> = samples().iter().map(GameEvent::kind).collect();
    assert_eq!(kinds, ["GameCreated", "PlanSet", "AnswerChosen", "RoundClosed", "RoundResolved"]);
}

#[test]
fn bad_records_are_rejected() {
    let e = &samples()[1];
    assert!(matches!(GameEvent::from_record("Nepoznat", 1, e.to_payload()), Err(EventError::UnknownKind(_))));
    assert!(matches!(GameEvent::from_record("PlanSet", 2, e.to_payload()), Err(EventError::UnsupportedVersion { .. })));
    let mut p = e.to_payload();
    p["visak"] = json!(1);
    assert!(matches!(GameEvent::from_record("PlanSet", 1, p), Err(EventError::Payload { .. })));
}

#[test]
fn actor_roundtrip_and_rejects() {
    let (_, _, guest) = ids();
    let mut all = vec![Actor::Guest(guest), Actor::System];
    all.extend(Strategy::ALL.iter().map(|s| Actor::Robot(*s)));
    for a in all {
        assert_eq!(Actor::parse(&a.as_db()), Some(a.clone()), "{}", a.as_db());
    }
    assert_eq!(Actor::Guest(guest).as_db(), "guest:00000000-0000-0000-0000-000000000003");
    assert_eq!(Actor::Robot(Strategy::Saver).as_db(), "robot:stedisa");
    for bad in ["", "guest:xyz", "robot:nepoznat", "System", "guest:"] {
        assert_eq!(Actor::parse(bad), None, "{bad}");
    }
}

#[test]
fn state_and_plan_dtos_roundtrip_to_core() {
    let (s, r) = june_report();
    assert_eq!(HotelState::try_from(&HotelStateDto::from(&s)).unwrap(), s);
    assert_eq!(HotelState::try_from(&HotelStateDto::from(&r.after)).unwrap(), r.after);
    let p = MonthPlan::from(&plan());
    assert_eq!(PlanDto::from(&p), plan());
}

#[test]
fn report_dto_survives_json() {
    let (_, r) = june_report();
    let d = MonthReportDto::from(&r);
    let back: MonthReportDto = serde_json::from_value(serde_json::to_value(&d).unwrap()).unwrap();
    assert_eq!(back, d);
    assert_eq!(serde_json::to_value(CauseDto::Situation("S1".into())).unwrap(), json!({"situation": "S1"}));
    assert_eq!(serde_json::to_value(CauseDto::Price).unwrap(), json!("price"));
}
```

- [ ] **Step 3: Pokreni** `cargo test -p game; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 4: `ids.rs`, `actor.rs`, `error.rs`.**

```rust
// ids.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GameId(pub Uuid);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HotelId(pub Uuid);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GuestId(pub Uuid);

impl GameId {
    /// Jedan tok po igri: ukupan redoslijed svih događaja igre (ADR-015 t. 4).
    pub fn stream(self) -> String {
        format!("game:{}", self.0)
    }
}
```

```rust
// actor.rs
use sim_hotel::Strategy;
use uuid::Uuid;

use crate::ids::GuestId;

/// Tko je izazvao događaj; u bazi kao tekst `guest:<uuid>` · `robot:<kod>` · `system` (CHECK u migraciji 0002).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Actor {
    Guest(GuestId),
    Robot(Strategy),
    System,
}

pub fn strategy_from_code(code: &str) -> Option<Strategy> {
    Strategy::ALL.into_iter().find(|s| s.code() == code)
}

impl Actor {
    pub fn as_db(&self) -> String {
        match self {
            Actor::Guest(g) => format!("guest:{}", g.0),
            Actor::Robot(s) => format!("robot:{}", s.code()),
            Actor::System => "system".to_owned(),
        }
    }

    pub fn parse(s: &str) -> Option<Actor> {
        if s == "system" {
            return Some(Actor::System);
        }
        if let Some(id) = s.strip_prefix("guest:") {
            return Uuid::parse_str(id).ok().map(|u| Actor::Guest(GuestId(u)));
        }
        s.strip_prefix("robot:").and_then(strategy_from_code).map(Actor::Robot)
    }
}
```

```rust
// error.rs
use sim_hotel::PlanError;

/// Greške pravila igre. `Corrupt` znači da dnevnik ili snimak krše pravilo koje je provjereno pri upisu — to je
/// greška sustava (500), nikad korisnika.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameError {
    UnknownHotel,
    UnknownMonth,
    UnknownSituation,
    UnknownAnswer,
    RoundNotOpen,
    AlreadyAnswered,
    Plan(PlanError),
    Corrupt(String),
}

pub(crate) fn corrupt(msg: impl Into<String>) -> GameError {
    GameError::Corrupt(msg.into())
}
```

- [ ] **Step 5: `dto.rs`.** Svaki DTO: `#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]` i
  `#[serde(deny_unknown_fields)]` (strukture) ili `#[serde(rename_all = "snake_case")]` (enumi; `MonthCode` i
  `IndexCode` kako je navedeno). Polja **točno** ovim imenima i tipovima (zrcalo jezgre, polje za polje):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MonthCode { Jun, Jul, Aug, Sep }
// From<Month> for MonthCode i From<MonthCode> for Month (iscrpni match, bez `_`).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingCode { None, Basic, Full }          // ↔ sim_hotel::Training
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceCode { Deferred, Regular, Intensive } // ↔ sim_hotel::Maintenance

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexCode { G, R, Z, O, F }
/// Redoslijed `Index::ALL`; nizovi `[u16; 5]` u događajima slijede ga.
pub const INDEX_ORDER: [IndexCode; 5] = [IndexCode::G, IndexCode::R, IndexCode::Z, IndexCode::O, IndexCode::F];
pub fn indices_dto(s: &IndexSet) -> [u16; 5] {
    Index::ALL.map(|i| s.get(i).hundredths())
}

pub struct PlanDto { pub price_cents: i64, pub marketing_cents: i64, pub seasonal_target: u16, pub wage_pct: u16,
                     pub training: TrainingCode, pub maintenance: MaintenanceCode, pub renovation: bool,
                     pub reply_reviews: bool }
// From<&MonthPlan> for PlanDto, From<&PlanDto> for MonthPlan.

pub struct PendingDto { pub experience_pts: i32, pub morale_pts: i32, pub renovation: bool }
pub struct GroupBlockDto { pub month: MonthCode, pub room_nights: u32, pub price_cents: i64 }
pub struct HotelStateDto {
    pub category: String, pub rooms: u16, pub permanent: u16, pub seasonal: u16, pub wage_pct: u16,
    pub extra_wage_cents: i64, pub morale_pts: i32, pub experience_pts: i32, pub equipment_pts: i32,
    pub satisfaction_pts: i32, pub rating: u16, pub cash_cents: i64, pub fixed_cents: i64,
    pub load_hundredths: u16, pub turnover_bp: u16, pub renovated: bool, pub pending: PendingDto,
    pub groups: Vec<GroupBlockDto>,
}
// From<&HotelState> for HotelStateDto (category = Category::code()).
// TryFrom<&HotelStateDto> for HotelState, Error = GameError: nepoznata kategorija → corrupt("kategorija <x>").

pub struct MarketDto { pub rooms_out: u16, pub available: u32, pub f_price_bp: i32, pub f_reputation_bp: i32,
    pub f_marketing_bp: i32, pub base_demand: u32, pub demand: u32, pub staff_capacity: u32,
    pub group_room_nights: u32, pub sold: u32, pub group_sold: u32, pub individual_sold: u32,
    pub group_revenue_cents: i64 }
pub struct PeopleDto { pub hires: u16, pub released: u16, pub staff: u16, pub load_hundredths: u16,
    pub load_penalty_pts: i32, pub morale_target_pts: i32, pub turnover_bp: u16, pub departures: u16,
    pub experience_mix_pts: i32 }
pub struct MoneyDto { pub rooms_revenue_cents: i64, pub other_revenue_cents: i64, pub labor_cents: i64,
    pub costs_cents: i64, pub gop_cents: i64, pub capex_cents: i64, pub interest_cents: i64, pub goppar_cents: i64 }
pub struct GuestDto { pub quality_pts: i32, pub value_pts: i32, pub satisfaction_target_pts: i32, pub rating_target: u16 }
// From<&MarketOutcome>, From<&PeopleOutcome>, From<&MoneyOutcome>, From<&GuestOutcome> — samo u smjeru jezgra → DTO.

#[serde(rename_all = "snake_case")]
pub enum VarCode { Sold, Revenue, Costs, Cash, Staff, Morale, Experience, Equipment, Satisfaction, Rating }
#[serde(rename_all = "snake_case")]
pub enum CauseDto { Baseline, Price, Reputation, Marketing, Groups, Capacity, OtherRevenue, Wage, Load, Training,
    Hires, Release, Turnover, Maintenance, Renovation, ReplyReviews, FixedCosts, Interest, Situation(String) }
#[serde(rename_all = "snake_case")]
pub enum UnitCode { RoomNights, Cents, People, Points, RatingHundredths, Bp }
pub struct LineDto { pub var: VarCode, pub cause: CauseDto, pub amount: i64, pub unit: UnitCode }
// From<&Var>, From<&Cause>, From<&Unit>, From<&Line> — iscrpni matchevi bez `_`, da nova varijanta u jezgri
// ne prevede tiho.

pub struct MonthReportDto { pub month: MonthCode, pub before: HotelStateDto, pub after: HotelStateDto,
    pub market: MarketDto, pub people: PeopleDto, pub money: MoneyDto, pub guests: GuestDto,
    pub lines: Vec<LineDto> }
// From<&MonthReport> for MonthReportDto.
```

- [ ] **Step 6: `event.rs`.**

```rust
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::actor::Actor;
use crate::dto::{MonthCode, MonthReportDto, PlanDto};
use crate::ids::{GameId, GuestId, HotelId};

/// Verzija oblika svih događaja; promjena oblika = nova verzija i „podizanje" starih pri čitanju (ARHITEKTURA §9.1).
pub const SCHEMA_VER: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameCreated {
    pub game_id: GameId,
    pub seed: u64,
    /// `snapshot.fingerprint()`; fold ga provjerava.
    pub ruleset_id: String,
    pub core_version: String,
    pub snapshot: content::Snapshot,
    pub months: Vec<MonthCode>,
    pub hotels: Vec<HotelSetup>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotelSetup {
    pub hotel_id: HotelId,
    /// 1..=4; određuje redoslijed obračuna i tokove kocke (nikad uuid ni vrijeme dolaska, ARHITEKTURA §7.2).
    pub slot: u8,
    pub name: String,
    /// "A" | "B" | "C"
    pub profile: String,
    pub controller: Controller,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Controller {
    Human { guest_id: GuestId },
    /// `strategy` = `Strategy::code()`
    Robot { strategy: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanSet { pub hotel_id: HotelId, pub month: MonthCode, pub plan: PlanDto }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnswerChosen { pub hotel_id: HotelId, pub month: MonthCode, pub situation: String, pub answer: u8 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundClosed { pub month: MonthCode }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundResolved { pub month: MonthCode, pub hotels: Vec<HotelResolution> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotelResolution {
    pub hotel_id: HotelId,
    /// Plan koji je stvarno obračunat (zadnji `PlanSet` ili zadani iz DEMO §6.1).
    pub plan: PlanDto,
    pub situations: Vec<SituationResult>,
    pub report: MonthReportDto,
    /// Pet indeksa nakon mjeseca, redoslijedom `INDEX_ORDER`.
    pub indices: [u16; 5],
    pub progress_hundredths: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SituationResult {
    pub code: String,
    /// `None` = nitko nije odgovorio do zatvaranja (kvaliteta 1, DEMO §1.2).
    pub answer: Option<u8>,
    pub quality: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameEvent {
    GameCreated(GameCreated),
    PlanSet(PlanSet),
    AnswerChosen(AnswerChosen),
    RoundClosed(RoundClosed),
    RoundResolved(RoundResolved),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventError {
    UnknownKind(String),
    UnsupportedVersion { kind: String, version: i32 },
    Payload { kind: String, message: String },
}

/// Zapisan događaj: redni broj u toku igre i tko ga je izazvao.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorded {
    pub seq: i64,
    pub actor: Actor,
    pub event: GameEvent,
}

fn body<T: DeserializeOwned>(kind: &str, v: serde_json::Value) -> Result<T, EventError> {
    serde_json::from_value(v).map_err(|e| EventError::Payload { kind: kind.to_owned(), message: e.to_string() })
}

impl GameEvent {
    pub fn kind(&self) -> &'static str {
        match self {
            GameEvent::GameCreated(_) => "GameCreated",
            GameEvent::PlanSet(_) => "PlanSet",
            GameEvent::AnswerChosen(_) => "AnswerChosen",
            GameEvent::RoundClosed(_) => "RoundClosed",
            GameEvent::RoundResolved(_) => "RoundResolved",
        }
    }

    pub fn to_payload(&self) -> serde_json::Value {
        let v = match self {
            GameEvent::GameCreated(e) => serde_json::to_value(e),
            GameEvent::PlanSet(e) => serde_json::to_value(e),
            GameEvent::AnswerChosen(e) => serde_json::to_value(e),
            GameEvent::RoundClosed(e) => serde_json::to_value(e),
            GameEvent::RoundResolved(e) => serde_json::to_value(e),
        };
        v.expect("događaj nema mapu s ne-tekstualnim ključem pa se uvijek serijalizira")
    }

    pub fn from_record(kind: &str, schema_ver: i32, payload: serde_json::Value) -> Result<GameEvent, EventError> {
        if schema_ver != SCHEMA_VER {
            return Err(EventError::UnsupportedVersion { kind: kind.to_owned(), version: schema_ver });
        }
        Ok(match kind {
            "GameCreated" => GameEvent::GameCreated(body(kind, payload)?),
            "PlanSet" => GameEvent::PlanSet(body(kind, payload)?),
            "AnswerChosen" => GameEvent::AnswerChosen(body(kind, payload)?),
            "RoundClosed" => GameEvent::RoundClosed(body(kind, payload)?),
            "RoundResolved" => GameEvent::RoundResolved(body(kind, payload)?),
            other => return Err(EventError::UnknownKind(other.to_owned())),
        })
    }
}
```
  (`GameId`, `GuestId` su u `use` zbog polja; `use crate::actor::Actor` zbog `Recorded`.)

- [ ] **Step 7: Pokreni** `cargo test -p game; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 8: Brana čistoće u `check-core-deps.mjs`.** Izdvoji poziv `cargo tree` u funkciju `tree(pkg)` koja vraća
  `Set` imena (s istom obradom grešaka: izlaz 2). Postojeća petlja nad `CORE` je koristi. Dodaj iza nje:

```js
// `game` je čist sloj iznad jezgre (ADR-015): smije serde i uuid, ali nikad bazu, mrežu ni async izvođač.
const PURE = {
  game: ["tokio", "sqlx", "sqlx-core", "sqlx-postgres", "axum", "hyper", "reqwest", "mio", "socket2"],
};
for (const [pkg, banned] of Object.entries(PURE)) {
  const names = tree(pkg);
  for (const b of banned) {
    if (names.has(b)) {
      console.error(`NIJE DOPUŠTENO: ${pkg} ovisi o ${b}`);
      found += 1;
    }
  }
  console.log(`${pkg}: provjereno ${names.size} paketa (čist sloj)`);
}
```
  Ručna proba (ne ide u CI): privremeno dodaj `tokio.workspace = true` u `crates/game/Cargo.toml`,
  `node scripts/check-core-deps.mjs; echo EXIT=$?` → `NIJE DOPUŠTENO: game ovisi o tokio`, `EXIT=1`; vrati datoteku
  (`git diff --stat -- crates/game/Cargo.toml` prazno nakon vraćanja u odnosu na tvoju verziju).

- [ ] **Step 9: Pokreni** `node scripts/check-core-deps.mjs; echo EXIT=$?` → `game: provjereno … (čist sloj)`,
  `prekršaja: 0`, `EXIT=0`; zatim provjere iz Global Constraints.

- [ ] **Step 10: Commit** — `Cargo.toml Cargo.lock crates/game scripts/check-core-deps.mjs`, poruka
  „Dnevnik 2/12: crate game — događaji, DTO-i, actor i brana čistoće".

---

