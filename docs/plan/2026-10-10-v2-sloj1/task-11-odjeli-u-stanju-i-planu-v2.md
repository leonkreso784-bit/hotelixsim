### Task 11: Odjeli u stanju i planu v2 (oblik, Smještaj, roboti, DTO)

**Veličina:** M, 3–4 h · **Ovisi o:** Task 9, Task 10 · **Paralelno s:** ništa iz jezgre (Task 13 smije) · **Model:** sonnet

**Cilj:** v2 stanje drži **odjele** umjesto agregata `permanent/seasonal/equipment_pts/fixed_cents`; plan drži
**postavke po odjelu**; v1 sažetak se izvodi (MODEL_V2 §3.8). Obračun u ovom tasku koristi samo odjel Smještaj (ostali
se nose, a prihod im dolazi u Tasku 12); degenerirana ekvivalencija mora ostati zelena.

**Files:**
- Modify: `crates/sim-hotel-v2/src/{state,plan,resolve,market,indices,robots,season,explain}.rs`,
  `crates/sim-hotel-v2/tests/{common/mod.rs,period.rs}`, `crates/game/src/{model,dto_v2,dto,game}.rs` (`game.rs`: samo
  `Game::plan_from_dto` — dopuna odjela, vidi niže), `crates/game/tests/model_v2.rs` (jedan novi test),
  `crates/app/src/error.rs` (samo mapiranje šest novih varijanti `sim_hotel_v2::PlanError` — vidi niže)
- Create: `crates/sim-hotel-v2/tests/departments_state.rs`
- Mehaničke izmjene literala (Global Constraints; novo polje `PlanDto.departments: BTreeMap::new()`, očekivanja se ne
  mijenjaju): `crates/api/src/schema.rs:176` (`impl From<&PlanBody> for PlanDto`), `crates/ai/tests/request.rs:152`,
  `crates/game/tests/events.rs:18`, `crates/game/tests/mentor.rs:32` i `crates/game/tests/mentor.rs:724` (retci prema
  `main` @ 5d44614 — ako ih je raniji task pomaknuo, isti literal `PlanDto {` u istoj datoteci).
- Ne dirati: `department.rs` (osim ako fali `pub` — dodaj), `crates/content/**`, `crates/api/**` (osim gornjeg
  retka), `crates/game/src/{view,ai_input,kpi}.rs` (`summary` ostaje bez kataloga, pa se pozivi ne mijenjaju), web.

**Interfaces:**
- Consumes: Task 9 (`DeptCode`, `DeptState`, `DeptPlan`, `allocate_seasonal`, `Catalog::{dept, profile_v2}`),
  Task 10 (stvarni sadržaj za igru).
- Produces:

```rust
// crates/sim-hotel-v2/src/state.rs
pub struct HotelState {
    pub category: Category, pub rooms: u16, pub wage_pct: u16, pub extra_wage_cents: i64,
    pub morale_pts: i32, pub experience_pts: i32, pub satisfaction_pts: i32, pub rating: u16, pub cash_cents: i64,
    pub load_hundredths: u16, pub turnover_bp: u16, pub renovated: bool, pub pending: Pending, pub groups: Vec<GroupBlock>,
    pub departments: BTreeMap<DeptCode, DeptState>,
    /// Mjesečni stalni dio neraspoređenih odjela (iz profila) i mjesečne fiksne naknade.
    pub undistributed_fixed_cents: BTreeMap<DeptCode, i64>,
    pub fixed_charges_cents: BTreeMap<String, i64>,
}
impl HotelState {
    pub fn from_profile(cat: &Catalog, p: Profile) -> Result<HotelState, ResolveError>;
    pub fn staff(&self) -> u16;                 // Σ stalni + sezonci svih odjela
    pub fn rooms_state(&self) -> Result<&DeptState, ResolveError>;
    /// v1 sažetak: permanent/seasonal = zbrojevi; equipment = Smještaj ako nema drugih operativnih odjela
    /// (`DeptState.class` ∈ {FoodBeverage, OtherOperated}), inače div_round(7 000·Smještaj + 3 000·prosjek ostalih
    /// operativnih, 10 000); fixed_cents = Σ neraspoređeni + Σ naknade.
    /// **Potpis iz Taska 2 ostaje** (bez kataloga, bez `Result`) — `CoreState::summary(&self)` iz Taska 6 i njegovi
    /// pozivi u `game` se ne mijenjaju. Razred odjela čita iz `DeptState.class` (Task 9). Zbrojevi `saturating_add`;
    /// 70/30 kroz `engine::calc` s `.unwrap_or(stanje Smještaja)` (vrijednosti su u 0..=10 000 po `Catalog::validate`,
    /// pa ta grana ne radi — komentar to kaže).
    pub fn summary(&self) -> sim_hotel::HotelState;
}
// crates/sim-hotel-v2/src/plan.rs
pub struct PeriodPlan {
    pub price_cents: i64, pub marketing_cents: i64, pub wage_pct: u16, pub training: Training,
    pub renovation: bool, pub reply_reviews: bool,
    pub departments: BTreeMap<DeptCode, DeptPlan>,
}
impl PeriodPlan {
    pub fn seasonal_total(&self) -> u16;
    pub fn rooms_maintenance(&self) -> Maintenance;
    // default_for / validate / carry_into / limits — isti potpisi kao Task 3
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeptLimits { pub seasonal_min: u16, pub seasonal_max: u16, pub price_level_min_pct: u16, pub price_level_max_pct: u16, pub can_close: bool }
#[derive(Clone, Debug, PartialEq, Eq)] // bez `Copy` (mapa)
pub struct PlanLimits { /* Task 3 polja */ pub seasonal_step_total: u16, pub departments: BTreeMap<DeptCode, DeptLimits> }
pub enum PlanError { /* Task 3 … */ DepartmentUnknown, DepartmentMissing, DepartmentCannotClose, DepartmentSeasonalOutOfRange, ClosedWithSeasonal, PriceLevelOutOfRange }
```

Pravila plana: plan mora imati točno odjele stanja; Smještaj uvijek otvoren i razine cijene 100; `seasonal_target ≤
seasonal_max` odjela; zatvoren odjel ima 0 sezonaca i smije se zatvoriti samo uz `can_close`; razina cijene u
`[price_level_min_pct, price_level_max_pct]`; Σ |cilj − trenutno| po odjelima ≤ `coeffs.seasonal_step`.

**Redoslijed provjera u `validate`** (vraća se prva pogođena greška; `resolve_period` zove `validate` za svaki plan,
kao v1 `resolve.rs:451`): (1) provjere iz Taska 3 bez sezonaca — cijena (cijeli euro, raspon), marketing, plaća,
obnova; (2) **struktura**: odjel kojeg nema u stanju → `DepartmentUnknown`, odjel stanja kojeg nema u planu →
`DepartmentMissing`; (3) **pravila odjela**, redom koda odjela: zatvoren bez `can_close` (i zatvoren Smještaj) →
`DepartmentCannotClose`, zatvoren s `seasonal_target > 0` → `ClosedWithSeasonal`, `seasonal_target > seasonal_max`
odjela → `DepartmentSeasonalOutOfRange`, razina cijene izvan granica (Smještaj ≠ 100) → `PriceLevelOutOfRange`, pa
ukupno `seasonal_total() > base.actions.seasonal_max` → `SeasonalOutOfRange`; (4) **korak** (zadnji): Σ |cilj −
trenutno| > `coeffs(p.kind).seasonal_step` → `SeasonalStepTooBig`. Zato test koji zatvara odjel dobiva grešku odjela
čak i kad bi plan prešao i korak. Plan koji **mora proći** obračun smije u jednom krugu pomaknuti sezonce najviše za
korak (tjedan 5, mjesec 10): testovi koji režu odjel polaze od `common::hold_plan` (svi odjeli na trenutnim
sezoncima, Σ|Δ| = 0) i režu za najviše korak po krugu (`trenutno.saturating_sub(step)`).

Zadani plan:
`total = clamp(Σ avg_u16(plan_odjela, p), Σtrenutno − step, Σtrenutno + step)`, raspodjela `allocate_seasonal`, razine
100, održavanje redovno svugdje, otvorenost kao u stanju. `carry_into`: postavke odjela se prenose (zatvoreni ostaju
zatvoreni, sezonci stisnuti na `seasonal_max`). Robot: v1 strategija daje ukupne sezonce i razinu održavanja kao u
Tasku 3; raspodjela `allocate_seasonal` prema planu odjela; ista razina održavanja u svim odjelima; razina cijene 100
(Nasumični: jednoliko u granicama, u koracima od 5).

Obračun u ovom tasku: osoblje po odjelu iz plana (najam/puštanje po odjelu, moral puštanja kao v1 ako je ikoga
pušteno); kapacitet tržišta = osoblje Smještaja × dani × `capacity_tenths_per_staff_day` Smještaja / 10;
opterećenje i kazna morala iz Smještaja; oprema u obračunu = stanje Smještaja (pravila v1: trošenje, održavanje
razine Smještaja, obnova, situacije); plaće = Σ plaća svih odjela (svaki svojom `wage_cents`); fiksno = Σ
neraspoređeni (razmjerno); ostali prihod = Σ `ShareOfRooms` odjela. U degeneriranom katalogu to je točno v1.

`crates/game`: `HotelStateV2Dto` dobiva `departments: BTreeMap<String, DeptStateDto>`, `undistributed_fixed_cents`,
`fixed_charges_cents` (bez `permanent/seasonal/equipment_pts/fixed_cents`); `PlanDto` (v1 tip u `dto.rs`) dobiva
`#[serde(default, skip_serializing_if = "BTreeMap::is_empty")] pub departments: BTreeMap<String, DeptPlanDto>` — v1
JSON se ne mijenja (Task 1 test to provjerava); za v2 `PlanDto.seasonal_target = seasonal_total()` i
`maintenance = rooms_maintenance()`; `DeptPlanDto { open: bool, seasonal_target: u16, price_level_pct: u16,
maintenance: MaintenanceCode }`; `DeptStateDto` nosi i `class` (kod iz `UsaliClass::code`). `Model::plan_from_dto`
za v2 traži neprazne `departments` (`seasonal_target` i `maintenance` iz DTO-a se za v2 ne čitaju — izvode se iz
odjela). **`Game::plan_from_dto(h, r, d)`** (Task 6) za v2 s **praznim** `d.departments` prije toga upiše postavke
odjela iz `effective_plan(h, r)` — tako `PUT plans/{round}` bez odjela (do Taska 15 `PlanBody` ih nema; test Taska 7
`round_paths_accept_week_codes` to radi) i dalje prolazi i zadržava postavke odjela.

Greške: šest novih varijanti `sim_hotel_v2::PlanError` (`DepartmentUnknown` … `PriceLevelOutOfRange`) u
`crates/app/src/error.rs` (mapiranje `GameError::PlanV2` iz Taska 6) privremeno idu u `ErrorCode::Internal` s
komentarom „Task 15: `department_plan_invalid`" — kroz API su do Taska 15 nedostižne (odjeli se tada samo dopunjuju iz
važećeg plana); bez `_ =>` grane.

- [ ] **Step 1: Testovi (padaju)** — `tests/departments_state.rs`:

```rust
mod common;

use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::*;

fn code(s: &str) -> DeptCode {
    DeptCode(s.into())
}

#[test]
fn summary_counts_staff_from_departments() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let sum = s.summary();
    let base = c.base.profile(Profile::C).unwrap(); // testni profil C (30, 20), ne sadržaj (30, 28)
    assert_eq!((sum.permanent, sum.seasonal), (base.permanent, base.seasonal));
}

#[test]
fn summary_equipment_is_seventy_thirty_rooms_and_operated_departments() {
    let c = common::catalog_with_departments();
    let mut s = HotelState::from_profile(&c, Profile::C).unwrap();
    for (k, d) in s.departments.iter_mut() {
        d.condition_pts = if k.as_str() == "rooms" { 6_000 } else { 8_000 };
    }
    assert_eq!(s.summary().equipment_pts, 6_600);
}

#[test]
fn summary_fixed_is_undistributed_plus_charges() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let undist: i64 = s.undistributed_fixed_cents.values().sum();
    let charges: i64 = s.fixed_charges_cents.values().sum();
    assert_eq!(s.summary().fixed_cents, undist + charges);
}

#[test]
fn default_plan_moves_seasonal_within_step_toward_department_plans() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = period_by_code(PeriodKind::Week, "t01").unwrap();
    let plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    let moved: u16 = plan.departments.iter().map(|(k, d)| d.seasonal_target.abs_diff(s.departments[k].seasonal)).sum();
    assert!(moved <= c.coeffs(PeriodKind::Week).unwrap().seasonal_step);
    assert_eq!(plan.validate(&s, &p, &c), Ok(()));
}

#[test]
fn closing_a_department_that_cannot_close_is_rejected() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let mut plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    plan.departments.get_mut(&code("restaurant")).unwrap().open = false;
    plan.departments.get_mut(&code("restaurant")).unwrap().seasonal_target = 0;
    // Uz pomake zadanog plana ovo prelazi i korak 10 — pravilo odjela se provjerava prije koraka.
    assert_eq!(plan.validate(&s, &p, &c), Err(PlanError::DepartmentCannotClose));
}

#[test]
fn closed_department_with_seasonal_is_rejected() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let mut plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    let bar = plan.departments.get_mut(&code("lobby_bar")).unwrap();
    bar.open = false;
    bar.seasonal_target = 1; // izričito > 0, ne ovisi o raspodjeli zadanog plana
    assert_eq!(plan.validate(&s, &p, &c), Err(PlanError::ClosedWithSeasonal));
}

#[test]
fn hold_plan_is_valid_and_moves_nobody() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = period_by_code(PeriodKind::Week, "t01").unwrap();
    let plan = common::hold_plan(&s, &p, &c);
    assert!(plan.departments.iter().all(|(k, d)| d.seasonal_target == s.departments[k].seasonal));
    assert_eq!(plan.validate(&s, &p, &c), Ok(()));
}

#[test]
fn carried_plan_respects_department_limits() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let mut plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    plan.departments.get_mut(&code("spa")).unwrap().price_level_pct = 120;
    let carried = plan.carry_into(&s, &month_period(Month::Jul), &c).unwrap();
    assert_eq!(carried.departments[&code("spa")].price_level_pct, 120);
    assert_eq!(carried.validate(&s, &month_period(Month::Jul), &c), Ok(()));
}

#[test]
fn robots_produce_valid_department_plans_every_week() {
    let c = common::catalog_with_departments();
    let r = RefTable::compute(&c, Category::Four, &calendar::season_periods(PeriodKind::Week)).unwrap();
    for s in Strategy::ALL {
        run_season(Profile::A, s, 3, PeriodKind::Week, &c, &[], &r).unwrap();
    }
}
```

`tests/common/mod.rs` dobiva `pub fn catalog_with_departments() -> Catalog`: `degenerate(&test_catalog())` pa odjeli
`restaurant`, `lobby_bar`, `spa`, `parking`, `rooftop_bar` i neraspoređeni s brojkama iz `content/hotel/v2/departments.json`
(Task 10, prepisane u kod — ovaj paket ne smije ovisiti o `content`) i profili C/A s raspodjelom osoblja iz
`profiles.json` prilagođenom testnom katalogu (Σ = v1 testni profil: C 30 stalnih i 20 sezonaca,
`crates/sim-hotel-v2/tests/common/mod.rs` = kopija `crates/sim-hotel/tests/common/mod.rs:7-23`). U profilu C
`restaurant` ima 10 sezonaca i `lobby_bar` 3 (kao `profiles.json`), razliku do Σ nosi `rooms` — testovi Taska 12 na
to računaju. I `pub fn hold_plan(s: &HotelState, p: &Period, c: &Catalog) -> PeriodPlan`: `PeriodPlan::default_for`,
pa svakom odjelu `seasonal_target = s.departments[k].seasonal` (Σ|Δ| = 0; plan koji uvijek prolazi korak).
Ekvivalencijski testovi (Task 2–3) i dalje koriste `degenerate` i moraju ostati zeleni.

`crates/game/tests/model_v2.rs` — novi test (dopuna odjela u `Game::plan_from_dto`):

```rust
#[test]
fn v2_plan_dto_without_departments_takes_them_from_the_effective_plan() {
    let g = Game::fold(&v2_events(PeriodKind::Week)).unwrap();
    let h = common::hotel_ids()[0];
    let r = g.current_round().unwrap();
    let CorePlan::V2(eff) = g.effective_plan(h, r).unwrap() else { panic!("v2") };
    let mut dto = CorePlan::V2(eff.clone()).to_dto();
    assert!(!dto.departments.is_empty());
    dto.departments.clear();
    let CorePlan::V2(p) = g.plan_from_dto(h, r, &dto).unwrap() else { panic!("v2") };
    assert_eq!(p.departments, eff.departments);
}
```

- [ ] **Step 2:** `cargo test -p sim-hotel-v2` → FAIL (novi testovi i prevođenje).
- [ ] **Step 3: Implementiraj** jezgru, pa `crates/game` (DTO i `model.rs`); `cargo test --workspace` mora proći,
  uključujući `game --test equivalence`, `game --test model_v2`, `game --test v1_frozen`.
- [ ] **Step 4: Sve provjere** (i `probe-mutacija`) i commit „Sloj 1/11: odjeli u stanju i planu v2; v1 sažetak izveden".

---

