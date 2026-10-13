### Task 11: Odjeli u stanju i planu v2 (oblik, Smještaj, roboti, DTO)

**Veličina:** M, 3–4 h · **Ovisi o:** Task 9, Task 10 · **Paralelno s:** ništa iz jezgre (Task 13 smije) · **Model:** sonnet

**Cilj:** v2 stanje drži **odjele** umjesto agregata `permanent/seasonal/equipment_pts/fixed_cents`; plan drži
**postavke po odjelu**; v1 sažetak se izvodi (MODEL_V2 §3.8). Obračun u ovom tasku koristi samo odjel Smještaj (ostali
se nose, a prihod im dolazi u Tasku 12); degenerirana ekvivalencija mora ostati zelena.

**Files:**
- Modify: `crates/sim-hotel-v2/src/{state,plan,resolve,market,indices,robots,season,explain}.rs`,
  `crates/sim-hotel-v2/tests/{common/mod.rs,period.rs}`, `crates/game/src/{model,dto_v2,dto}.rs`
- Create: `crates/sim-hotel-v2/tests/departments_state.rs`
- Ne dirati: `department.rs` (osim ako fali `pub` — dodaj), `crates/content/**`, `crates/api/**`, web.

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
    /// v1 sažetak: permanent/seasonal = zbrojevi; equipment = Smještaj ako nema drugih operativnih odjela, inače
    /// div_round(7 000·Smještaj + 3 000·prosjek ostalih operativnih, 10 000); fixed_cents = Σ neraspoređeni + Σ naknade.
    pub fn summary(&self, cat: &Catalog) -> Result<sim_hotel::HotelState, ResolveError>;
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
pub struct DeptLimits { pub seasonal_min: u16, pub seasonal_max: u16, pub price_level_min_pct: u16, pub price_level_max_pct: u16, pub can_close: bool }
pub struct PlanLimits { /* Task 3 polja */ pub seasonal_step_total: u16, pub departments: BTreeMap<DeptCode, DeptLimits> }
pub enum PlanError { /* Task 3 … */ DepartmentUnknown, DepartmentMissing, DepartmentCannotClose, DepartmentSeasonalOutOfRange, ClosedWithSeasonal, PriceLevelOutOfRange }
```

Pravila plana: plan mora imati točno odjele stanja; Smještaj uvijek otvoren i razine cijene 100; `seasonal_target ≤
seasonal_max` odjela; zatvoren odjel ima 0 sezonaca i smije se zatvoriti samo uz `can_close`; razina cijene u
`[price_level_min_pct, price_level_max_pct]`; Σ |cilj − trenutno| po odjelima ≤ `coeffs.seasonal_step`. Zadani plan:
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
maintenance: MaintenanceCode }`; `Model::plan_from_dto` za v2 traži neprazne `departments`.

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
    let sum = s.summary(&c).unwrap();
    assert_eq!((sum.permanent, sum.seasonal), (30, 28));
}

#[test]
fn summary_equipment_is_seventy_thirty_rooms_and_operated_departments() {
    let c = common::catalog_with_departments();
    let mut s = HotelState::from_profile(&c, Profile::C).unwrap();
    for (k, d) in s.departments.iter_mut() {
        d.condition_pts = if k.as_str() == "rooms" { 6_000 } else { 8_000 };
    }
    assert_eq!(s.summary(&c).unwrap().equipment_pts, 6_600);
}

#[test]
fn summary_fixed_is_undistributed_plus_charges() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let undist: i64 = s.undistributed_fixed_cents.values().sum();
    let charges: i64 = s.fixed_charges_cents.values().sum();
    assert_eq!(s.summary(&c).unwrap().fixed_cents, undist + charges);
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
    assert_eq!(plan.validate(&s, &p, &c), Err(PlanError::ClosedWithSeasonal));
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
`profiles.json` prilagođenom testnom katalogu (Σ = v1 testni profil). Ekvivalencijski testovi (Task 2–3) i dalje
koriste `degenerate` i moraju ostati zeleni.

- [ ] **Step 2:** `cargo test -p sim-hotel-v2` → FAIL (novi testovi i prevođenje).
- [ ] **Step 3: Implementiraj** jezgru, pa `crates/game` (DTO i `model.rs`); `cargo test --workspace` mora proći,
  uključujući `game --test equivalence`, `game --test model_v2`, `game --test v1_frozen`.
- [ ] **Step 4: Sve provjere** (i `probe-mutacija`) i commit „Sloj 1/11: odjeli u stanju i planu v2; v1 sažetak izveden".

---

