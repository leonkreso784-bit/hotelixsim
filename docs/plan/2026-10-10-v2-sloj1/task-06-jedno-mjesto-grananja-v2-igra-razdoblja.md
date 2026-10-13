### Task 6: Jedno mjesto grananja — v2 igra (razdoblja, prenošenje plana, roboti, verzija)

**Veličina:** L, 5–7 h · **Ovisi o:** Task 3, 4, 5 (spojeni) · **Paralelno s:** ništa (VRATA) · **Model:** opus
(arhitektonski rez kroz cijeli `game`)

**Cilj:** igra se stvara i igra na v1 ili v2 prema snimku; v2 igra ima 4 mjesečna ili 17 tjednih krugova, plan se
prenosi iz kruga u krug, roboti igraju, verzija jezgre v2 se provjerava; sve grananje je u `model.rs`.

**Files:**
- Create: `crates/game/src/model.rs`, `crates/game/src/dto_v2.rs`, `crates/game/tests/model_v2.rs`,
  `crates/game/tests/equivalence.rs`
- Modify: `crates/game/Cargo.toml` (`sim-hotel-v2.workspace = true`), `crates/game/src/{lib,game,event,command,resolve,
  view,ai_input,error,kpi}.rs`, `crates/app/src/error.rs`, `crates/api/src/problem.rs` (samo novi kodovi grešaka),
  minimalno `crates/app/src/*.rs` i `crates/api/src/*.rs` da se prevede (bez novih ruta i polja)
- Ne dirati: `crates/sim-hotel/**`, `crates/sim-hotel-v2/**` (ako fali javna funkcija → zapiši u izvještaj, dodaj
  najmanji getter), fixturei, `openapi.json` osim novih kodova grešaka.

**Interfaces:**
- Consumes: Task 3 (`resolve_period`, `PeriodPlan::{default_for, validate, carry_into, limits}`, `RefTable::compute`,
  `start_indices`, `period_indices`, `robot_plan`, `situations_for`), Task 4 (`Snapshot::model()`, `Content.v2`),
  Task 5 (`RoundKey`).
- Produces:

```rust
// crates/game/src/model.rs — JEDINO mjesto s `match` na model
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model { V1, V2 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoreState { V1(sim_hotel::HotelState), V2(Box<sim_hotel_v2::HotelState>) }
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorePlan { V1(sim_hotel::MonthPlan), V2(sim_hotel_v2::PeriodPlan) }
pub(crate) enum CoreRefs { V1(sim_hotel::RefTable), V2(sim_hotel_v2::RefTable) }
/// Jedan krug igre: ključ i kalendar (za v1 uvijek cijeli mjesec).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundInfo { pub key: RoundKey, pub period: sim_hotel_v2::Period }
pub(crate) struct Resolved { pub report: MonthReportDto, pub detail: Option<DetailDto>, pub indices: IndexSet }

impl Model {
    pub fn of(content: &content::Content, snapshot_model: content::Model) -> Model;
    pub fn core_version(self) -> &'static str;              // sim_hotel::CORE_VERSION | sim_hotel_v2::CORE_VERSION
    pub(crate) fn start_state(self, c: &Content, p: Profile) -> Result<CoreState, GameError>;
    pub(crate) fn refs(self, c: &Content, category: Category, rounds: &[RoundInfo]) -> Result<CoreRefs, GameError>;
    pub(crate) fn start_indices(self, s: &CoreState, c: &Content, r: &CoreRefs) -> Result<IndexSet, GameError>;
    pub(crate) fn default_plan(self, s: &CoreState, r: &RoundInfo, c: &Content) -> Result<CorePlan, GameError>;
    pub(crate) fn carry_plan(self, prev: &CorePlan, s: &CoreState, r: &RoundInfo, c: &Content) -> Result<CorePlan, GameError>;
    pub(crate) fn validate_plan(self, p: &CorePlan, s: &CoreState, r: &RoundInfo, c: &Content) -> Result<(), GameError>;
    pub(crate) fn robot_plan(self, st: Strategy, s: &CoreState, r: &RoundInfo, c: &Content, rng: &mut Rng) -> Result<CorePlan, GameError>;
    pub(crate) fn resolve(self, s: &CoreState, r: &RoundInfo, p: &CorePlan, o: &[SituationOutcome], c: &Content, refs: &CoreRefs) -> Result<Resolved, GameError>;
    pub(crate) fn state_after(self, res: &HotelResolution) -> Result<CoreState, GameError>;
    pub(crate) fn plan_from_dto(self, d: &PlanDto) -> Result<CorePlan, GameError>;
    pub(crate) fn plan_limits(self, s: &CoreState, r: &RoundInfo, c: &Content) -> Result<PlanLimits, GameError>;
    pub(crate) fn situations_in<'a>(self, r: &RoundInfo, all: &'a [SituationTemplate]) -> Vec<&'a SituationTemplate>;
}
impl CoreState { pub fn summary(&self) -> sim_hotel::HotelState; pub fn category(&self) -> Category; }
impl CorePlan { pub fn to_dto(&self) -> PlanDto; }
impl From<sim_hotel::MonthPlan> for CorePlan;

// crates/game/src/command.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameSettings { pub kind: sim_hotel_v2::PeriodKind }
/// v1: snimak `model = v1` i `kind = Month` → lipanj i srpanj (kao danas); v2: cijela sezona (🟨 P6).
pub fn new_game_with(game_id: GameId, seed: u64, snapshot: &content::Snapshot, guest: GuestId, hotel_ids: [HotelId; 4], settings: GameSettings) -> Result<Vec<(Actor, GameEvent)>, GameError>;
// `new_game(...)` ostaje i zove `new_game_with(.., GameSettings { kind: PeriodKind::Month })`.

// crates/game/src/event.rs — samo dodano
pub struct GameCreated { /* …postojeće… */
    /// v2: kalendar krugova u trenutku stvaranja (igra ga nikad ne računa ponovno). v1: prazno.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rounds: Vec<RoundDefDto>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundDefDto { pub code: RoundKey, pub kind: String, pub index: u8, pub start_day: u16, pub days: u8 }
pub struct HotelResolution { /* …postojeće… */
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<DetailDto>,
}

// crates/game/src/dto_v2.rs
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetailDto { pub after: HotelStateV2Dto }
/// Serde-zrcalo `sim_hotel_v2::HotelState` (polje za polje; Task 11 ga mijenja zajedno sa stanjem).
pub struct HotelStateV2Dto { /* polja v2 stanja */ }

// crates/game/src/error.rs — dodano
GameError::Outdated { created: String, current: String }   // v2 igra druge verzije jezgre v2
GameError::WrongModel                                      // metoda samo za drugi model
GameError::InvalidSettings                                 // npr. v1 + tjedan

// crates/game/src/game.rs
impl Game {
    pub fn model(&self) -> Model;
    pub fn round_info(&self, r: impl Into<RoundKey>) -> Result<RoundInfo, GameError>;
    pub fn rounds_info(&self) -> Vec<RoundInfo>;
    pub fn state_at(&self, h: HotelId, r: impl Into<RoundKey>) -> Result<CoreState, GameError>;
    pub fn latest_state(&self, h: HotelId) -> Result<CoreState, GameError>;
    pub fn effective_plan(&self, h: HotelId, r: impl Into<RoundKey>) -> Result<CorePlan, GameError>;
    pub fn set_plan(&self, h: HotelId, r: impl Into<RoundKey>, plan: impl Into<CorePlan>) -> Result<GameEvent, GameError>;
}
```

Pravila:
- **Model** dolazi iz snimka u `GameCreated` (`Snapshot::from_value(..).model()`); `core_version` mora biti
  `Model::core_version()`; za **v2** razliku `fold` vraća `GameError::Outdated` (🟨 P11), za **v1** sve ostaje kao
  danas (provjera samo u `resolve_job`).
- **Krugovi:** v1 → iz `GameCreated.months` (mjeseci) i `month_period`; v2 → iz `GameCreated.rounds` (spremljeni
  kalendar), a `months` sadrži iste kodove (provjera pri `fold`: isti popis, inače `corrupt`).
- **Plan:** v1 — zadnji `PlanSet` za taj krug ili **zadani** (kao danas); v2 — zadnji `PlanSet` za taj krug, inače
  plan **najbližeg ranijeg kruga** kroz `carry_into`, inače zadani.
- **Sažetak:** `HotelResolution.report` uvijek v1 oblika (`MonthReportDto::from(&report.summary)` za v2);
  `HotelResolution.plan = CorePlan::to_dto()`; `detail` samo za v2.
- **Situacije u krugu:** v1 po mjesecu; v2 `sim_hotel_v2::season::situations_for`.
- **Tokovi kocke robota:** `game:robot-plan` s `[pozicija kruga, slot]` (pozicija = indeks u popisu krugova; u v1 je
  jednaka poziciji mjeseca, pa su v1 nizovi isti).

- [ ] **Step 1: Testovi (padaju)** — `crates/game/tests/model_v2.rs` (pomoćnici u istoj datoteci):

```rust
mod common;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::*;
use sim_hotel_v2::PeriodKind;
use uuid::Uuid;

fn v2_events(kind: PeriodKind) -> Vec<Recorded> {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    new_game_with(GameId(Uuid::from_u128(1)), 42, &snap, common::GUEST, common::hotel_ids(), GameSettings { kind })
        .unwrap()
        .into_iter()
        .zip(1..)
        .map(|((actor, event), seq)| Recorded { seq, actor, event })
        .collect()
}

fn play_all(mut g: Game) -> Game {
    while let Some(r) = g.current_round() {
        let ev = g.close_round(r).unwrap().unwrap();
        common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else { panic!("čeka") };
        for (a, e) in evs {
            common::apply_ev(&mut g, a, e);
        }
    }
    g
}

#[test]
fn v2_weekly_game_has_17_rounds_and_resolves_all_with_robots() {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    assert_eq!(g.model(), Model::V2);
    assert_eq!(g.rounds.len(), 17);
    assert_eq!(g.rounds_info()[4].period.days, 7);
    let g = play_all(g);
    assert_eq!(g.resolved.len(), 17);
    let last = g.resolved.values().last().unwrap();
    assert!(last.hotels.iter().all(|h| h.detail.is_some()));
}

#[test]
fn v2_monthly_game_has_four_rounds() {
    let g = play_all(Game::fold(&v2_events(PeriodKind::Month)).unwrap());
    assert_eq!(g.resolved.len(), 4);
}

#[test]
fn v1_with_weeks_is_invalid_settings() {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let r = new_game_with(GameId(Uuid::from_u128(1)), 42, &snap, common::GUEST, common::hotel_ids(), GameSettings { kind: PeriodKind::Week });
    assert_eq!(r.unwrap_err(), GameError::InvalidSettings);
}

#[test]
fn v2_plan_carries_over_without_renovation() {
    let mut g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let h = common::hotel_ids()[0];
    let r1 = g.current_round().unwrap();
    let CorePlan::V2(mut p) = g.effective_plan(h, r1).unwrap() else { panic!("v2") };
    p.wage_pct = 105;
    p.renovation = true;
    let ev = g.set_plan(h, r1, CorePlan::V2(p.clone())).unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ev = g.close_round(r1).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(r1, false).unwrap() else { panic!() };
    for (a, e) in evs {
        common::apply_ev(&mut g, a, e);
    }
    let r2 = g.current_round().unwrap();
    let CorePlan::V2(carried) = g.effective_plan(h, r2).unwrap() else { panic!("v2") };
    assert_eq!(carried.wage_pct, 105);
    assert!(!carried.renovation);
}

#[test]
fn v1_round_without_plan_still_uses_the_default_plan() {
    let (mut g, h) = common::open_june_game();
    let mut p = match g.effective_plan(h, sim_hotel::Month::Jun).unwrap() {
        CorePlan::V1(p) => p,
        CorePlan::V2(_) => panic!("v1"),
    };
    p.wage_pct = 110;
    let ev = g.set_plan(h, sim_hotel::Month::Jun, p).unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ev = g.close_round(sim_hotel::Month::Jun).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(sim_hotel::Month::Jun, false).unwrap() else { panic!() };
    for (a, e) in evs {
        common::apply_ev(&mut g, a, e);
    }
    let CorePlan::V1(jul) = g.effective_plan(h, sim_hotel::Month::Jul).unwrap() else { panic!() };
    assert_eq!(jul.wage_pct, 100); // v1: zadani plan, ne prošlomjesečni
}

#[test]
fn v2_game_from_other_core_version_is_outdated() {
    let mut log = v2_events(PeriodKind::Week);
    let GameEvent::GameCreated(c) = &mut log[0].event else { panic!() };
    c.core_version = "2.0.0-staro".into();
    assert!(matches!(Game::fold(&log), Err(GameError::Outdated { .. })));
}

#[test]
fn v1_game_is_never_outdated_by_v2_version() {
    let (g, _) = common::open_june_game();
    assert_eq!(g.model(), Model::V1);
    assert_eq!(g.core_version, sim_hotel::CORE_VERSION);
}

#[test]
fn v2_situations_appear_in_the_round_containing_the_first_of_the_month() {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let in_round = |code: &str| g.situations_in(RoundKey::parse(code).unwrap()).len();
    assert!(in_round("t01") > 0); // lipanj
    assert_eq!(in_round("t02"), 0);
    assert!(in_round("t05") > 0); // srpanj
}

#[test]
fn dispatch_lives_in_one_place() {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in std::fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().unwrap() == "model.rs" {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for needle in ["Model::V1", "Model::V2", "CoreState::V1", "CoreState::V2", "CorePlan::V1", "CorePlan::V2", "CoreRefs::"] {
            assert!(!text.contains(needle), "{} sadrži {needle} — grananje ide u model.rs", path.display());
        }
    }
}
```

`crates/game/tests/equivalence.rs`:

```rust
mod common;

use content::{Snapshot, repo_dir};
use game::*;
use sim_hotel::Month;

/// v2 u degeneriranom katalogu (mjesečno, kao v1) daje iste obračune kao v1 za istu igru (MODEL_V2 §2 t. 6).
#[test]
fn v2_monthly_degenerate_game_matches_v1_game() {
    let v1 = Snapshot::read(&repo_dir()).unwrap();
    let v2 = v1.degenerate_v2();
    let play = |snap: &Snapshot| {
        let mut g = Game::fold(&common::new_game_events(snap)).unwrap();
        let h = common::hotel_ids()[0];
        let me = Actor::Guest(common::GUEST);
        let ev = g.choose_answer(h, "S1", 0).unwrap();
        common::apply_ev(&mut g, me.clone(), ev);
        let ev = g.choose_answer(h, "S3", 1).unwrap();
        common::apply_ev(&mut g, me.clone(), ev);
        for m in [Month::Jun, Month::Jul] {
            let ev = g.close_round(m).unwrap().unwrap();
            common::apply_ev(&mut g, me.clone(), ev);
            let ResolveStep::Done(evs) = g.resolve_job(m, false).unwrap() else { panic!() };
            for (a, e) in evs {
                common::apply_ev(&mut g, a, e);
            }
        }
        g
    };
    let (a, b) = (play(&v1), play(&v2));
    assert_eq!(b.model(), Model::V2);
    for m in [Month::Jun, Month::Jul] {
        let (ra, rb) = (&a.resolved[&m.into()], &b.resolved[&m.into()]);
        for (ha, hb) in ra.hotels.iter().zip(&rb.hotels) {
            assert_eq!(ha.plan, hb.plan, "{m:?}");
            assert_eq!(ha.report, hb.report, "{m:?}");
            assert_eq!(ha.indices, hb.indices, "{m:?}");
            assert_eq!(ha.progress_hundredths, hb.progress_hundredths, "{m:?}");
        }
    }
}
```

Napomena: `new_game_events` iz `common` zove `new_game` (mjesečne postavke). Za v1 snimak to je lipanj–srpanj, a za v2
snimak mjesečna v2 igra od 4 kruga (🟨 P6), pa test uspoređuje samo prva dva kruga. Referentna tablica u obje igre
pokriva sva četiri mjeseca, pa su i početni indeksi isti.

- [ ] **Step 2:** `cargo test -p game --test model_v2 --test equivalence` → FAIL.
- [ ] **Step 3: `model.rs`, `dto_v2.rs`, promjene u `game.rs`/`command.rs`/`resolve.rs`/`view.rs`/`ai_input.rs`/
  `kpi.rs`.** `HotelRecord.start_state: CoreState`, `plans: BTreeMap<RoundKey, CorePlan>`; `Game` dobiva
  `model: Model`, `rounds_info: Vec<RoundInfo>`, `refs: BTreeMap<Category, CoreRefs>`. Pogledi i mentor koriste
  `CoreState::summary()` (v1 oblik) — zato rade i za v2. `plan_limits` (v1 `PlanLimits` iz `command.rs`) za v2 dolazi
  iz `PeriodPlan::limits` kroz `Model::plan_limits`. `kpi.rs` (sloj 0): dani iz `round_info(..).period.days`.
- [ ] **Step 4: Greške.** `app::ErrorCode::{GameOutdated, InvalidSettings, WrongModel}`; `api::ProblemCode`:
  `game_outdated` 409 „Igra je stvorena starijom verzijom igre. Započni novu igru.", `invalid_settings` 422 „Te postavke
  igre nisu dopuštene.", `model_v1` 409 „Ova igra je na starom modelu i nema ovaj prikaz.". Regeneriraj
  `corepack pnpm gen:api` (u `openapi.json` se mijenja samo popis kodova grešaka).
- [ ] **Step 5:** `cargo test --workspace; echo EXIT=$?` → zeleno, uključujući `v1_frozen` (6) i sve stare testove.
- [ ] **Step 6: Sve provjere i commit** „Sloj 1/6: jedno mjesto grananja v1/v2; v2 igra po tjednima ili mjesecima".

**Prihvaćanje:** `dispatch_lives_in_one_place` zelen; ekvivalencija igre zelena; v1 fixturei nepromijenjeni;
`git diff --stat -- crates/sim-hotel` prazno. **Tek nakon spajanja ovog taska počinje Faza B.**

---

## Faza B — nakon vrata (Taskovi 7–22)

