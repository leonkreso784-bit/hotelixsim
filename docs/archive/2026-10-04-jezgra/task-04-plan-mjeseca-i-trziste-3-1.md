### Task 4: Plan mjeseca i tržište (§3.1, §6.1)

**Files:**
- Create: `crates/sim-hotel/src/plan.rs`, `crates/sim-hotel/src/market.rs`, `crates/sim-hotel/tests/market.rs`

**Interfaces:**
- Consumes: Task 3 tipovi, `engine::calc`.
- Produces:

```rust
// plan.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Maintenance { Deferred, Regular, Intensive }
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Training { None, Basic, Full }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthPlan {
    pub price_cents: i64, pub marketing_cents: i64, pub seasonal_target: u16, pub wage_pct: u16,
    pub training: Training, pub maintenance: Maintenance, pub renovation: bool, pub reply_reviews: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanError { PriceNotWholeEuro, PriceOutOfRange, MarketingOutOfRange, SeasonalOutOfRange, SeasonalStepTooBig, WageOutOfRange, RenovationTwice }
impl MonthPlan {
    /// Zadano (ADR-013 t. 3): referentna cijena, marketing 0, sezonci po planu kataloga (najviše ±korak od
    /// današnjeg), plaća kao sad, bez obuke, redovno održavanje, bez obnove i odgovaranja.
    pub fn default_for(state: &HotelState, month: Month, cat: &Catalog) -> Result<MonthPlan, CatalogError>;
    pub fn validate(&self, state: &HotelState, month: Month, cat: &Catalog) -> Result<(), PlanError>;
}

// market.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarketOutcome {
    pub rooms_out: u16, pub available: u32, pub f_price_bp: i32, pub f_reputation_bp: i32, pub f_marketing_bp: i32,
    pub base_demand: u32, pub demand: u32, pub staff_capacity: u32, pub group_room_nights: u32,
    pub sold: u32, pub group_sold: u32, pub individual_sold: u32, pub group_revenue_cents: i64,
}
pub fn market(state: &HotelState, month: Month, plan: &MonthPlan, staff: u16, cat: &Catalog) -> engine::calc::Calc<MarketOutcome>;
```
`state` je stanje NAKON promjene osoblja (sezonci postavljeni), a `state.groups` već sadrži nove grupe iz situacija.

- [ ] **Step 1: Formule** (P9–P15; `D = month.days()`, `ref = price_ref_cents(month)`, sve u i128 kroz `calc`):

| polje | formula |
|---|---|
| `rooms_out` | `clamp(div_round(4000 − equipment_pts, 400), 0, rooms)` |
| `available` | `(rooms − rooms_out) × D` |
| `f_price_bp` | `clamp(10000 + div_round(elasticity_bp × (price − ref), ref), 3000, 16000)` |
| `f_reputation_bp` | `max(0, 10000 + div_round(54 × (rating − 840), 10))` |
| `f_marketing_bp` | `10000 + div_round(marketing_max_bp × M, M + marketing_half_cents)` |
| `base_demand` | `div_round(rooms × D × base_occupancy_bp, 10000)` (samo za objašnjenje) |
| `demand` | `max(0, div_round(rooms × D × base_bp × f_price × f_rep × f_mkt, 10^16))` |
| `staff_capacity` | `div_floor(staff × D × 24, 10)` |
| `group_room_nights` | zbroj `room_nights` grupa s `month == month` |
| `sold` | `min(group_room_nights + demand, available, staff_capacity)` |
| `group_sold` | `min(group_room_nights, sold)`; raspodjela po grupama redom kojim su u `state.groups` |
| `individual_sold` | `sold − group_sold` |
| `group_revenue_cents` | `Σ min(rn_i, preostalo) × price_cents_i` |

Uzorak implementacije (isti stil za sve formule u Taskovima 4–7):

```rust
use engine::calc::{self as c, Calc};

fn f_price_bp(price: i64, reference: i64, elasticity_bp: i32) -> Calc<i32> {
    let delta = c::sub(i128::from(price), i128::from(reference))?;
    let shift = c::div_round(c::mul(i128::from(elasticity_bp), delta)?, i128::from(reference))?;
    c::narrow(c::add(10_000, shift)?.clamp(3_000, 16_000))
}
```

- [ ] **Step 2: Testovi** `crates/sim-hotel/tests/market.rs` (zlatne vrijednosti iz `oracle-jezgra.mjs`):

```rust
mod common;
use common::*;
use sim_hotel::*;

fn june_default(s: &HotelState) -> MarketOutcome {
    let cat = test_catalog();
    let plan = MonthPlan::default_for(s, Month::Jun, &cat).unwrap();
    market(s, Month::Jun, &plan, s.staff(), &cat).unwrap()
}

#[test]
fn profile_c_june_default_market() {
    let m = june_default(&state_c());
    assert_eq!((m.rooms_out, m.available), (0, 3_600));
    assert_eq!((m.f_price_bp, m.f_reputation_bp, m.f_marketing_bp), (10_000, 10_000, 10_000));
    assert_eq!((m.base_demand, m.demand, m.staff_capacity), (2_592, 2_592, 3_600));
    assert_eq!((m.sold, m.group_sold, m.individual_sold), (2_592, 0, 2_592));
}

#[test]
fn high_price_and_marketing_june() {
    let cat = test_catalog();
    let mut s = state_c();
    s.seasonal = 30;
    let plan = MonthPlan { price_cents: 15_000, marketing_cents: 1_500_000, seasonal_target: 30, wage_pct: 105,
        training: Training::Basic, maintenance: Maintenance::Intensive, renovation: false, reply_reviews: true };
    let m = market(&s, Month::Jun, &plan, 60, &cat).unwrap();
    assert_eq!((m.f_price_bp, m.f_marketing_bp, m.demand, m.staff_capacity, m.sold), (9_111, 10_750, 2_539, 4_320, 2_539));
}

#[test]
fn low_price_raises_demand() {
    let cat = test_catalog();
    let s = state_c();
    let mut plan = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    plan.price_cents = 8_100; // 60 % reference
    let m = market(&s, Month::Jun, &plan, 50, &cat).unwrap();
    assert_eq!((m.f_price_bp, m.demand, m.sold), (13_200, 3_421, 3_421));
}

#[test]
fn worn_equipment_takes_rooms_out() {
    let mut s = state_c();
    s.equipment_pts = 2_000;
    let m = june_default(&s);
    assert_eq!((m.rooms_out, m.available, m.sold), (5, 3_450, 2_592));
}

#[test]
fn staff_capacity_caps_august() {
    let cat = test_catalog();
    let s = state_c();
    let mut plan = MonthPlan::default_for(&s, Month::Aug, &cat).unwrap();
    plan.seasonal_target = 0;
    let m = market(&s, Month::Aug, &plan, 30, &cat).unwrap();
    assert_eq!((m.demand, m.staff_capacity, m.sold), (3_088, 2_232, 2_232));
}

#[test]
fn zero_staff_sells_nothing() {
    let cat = test_catalog();
    let s = state_c();
    let plan = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    let m = market(&s, Month::Jun, &plan, 0, &cat).unwrap();
    assert_eq!((m.staff_capacity, m.sold), (0, 0));
}

#[test]
fn groups_sell_first_and_are_paid_at_group_price() {
    let cat = test_catalog();
    let mut s = state_c();
    s.groups.push(GroupBlock { month: Month::Jun, room_nights: 175, price_cents: 10_125 });
    s.groups.push(GroupBlock { month: Month::Aug, room_nights: 50, price_cents: 1 });
    let plan = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    let m = market(&s, Month::Jun, &plan, 50, &cat).unwrap();
    assert_eq!((m.group_room_nights, m.sold, m.group_sold, m.individual_sold), (175, 2_767, 175, 2_592));
    assert_eq!(m.group_revenue_cents, 1_771_875);
}

#[test]
fn plan_bounds_reject_out_of_range_inputs() {
    let cat = test_catalog();
    let s = state_c();
    let ok = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    assert_eq!(ok.validate(&s, Month::Jun, &cat), Ok(()));
    let bad = |f: &dyn Fn(&mut MonthPlan)| { let mut p = ok.clone(); f(&mut p); p.validate(&s, Month::Jun, &cat) };
    assert_eq!(bad(&|p| p.price_cents = 13_550), Err(PlanError::PriceNotWholeEuro));
    assert_eq!(bad(&|p| p.price_cents = 8_000), Err(PlanError::PriceOutOfRange));   // < 60 % (8 100)
    assert_eq!(bad(&|p| p.price_cents = 20_300), Err(PlanError::PriceOutOfRange));  // > 150 % (20 250)
    assert_eq!(bad(&|p| p.price_cents = 20_200), Ok(()));
    assert_eq!(bad(&|p| p.marketing_cents = 4_000_001), Err(PlanError::MarketingOutOfRange));
    assert_eq!(bad(&|p| p.marketing_cents = -1), Err(PlanError::MarketingOutOfRange));
    assert_eq!(bad(&|p| p.seasonal_target = 31), Err(PlanError::SeasonalStepTooBig)); // 20 → 31
    assert_eq!(bad(&|p| p.seasonal_target = 41), Err(PlanError::SeasonalOutOfRange));
    assert_eq!(bad(&|p| p.wage_pct = 89), Err(PlanError::WageOutOfRange));
    assert_eq!(bad(&|p| p.wage_pct = 116), Err(PlanError::WageOutOfRange));
    let mut renovated = s.clone();
    renovated.renovated = true;
    let mut p = ok.clone();
    p.renovation = true;
    assert_eq!(p.validate(&renovated, Month::Jun, &cat), Err(PlanError::RenovationTwice));
}

#[test]
fn default_plan_follows_seasonal_plan_within_step() {
    let mut cat = test_catalog();
    cat.categories.get_mut(&Category::Four).unwrap().seasonal_plan.0[0] = 35;
    let p = MonthPlan::default_for(&state_c(), Month::Jun, &cat).unwrap();
    assert_eq!(p.seasonal_target, 30); // 20 + korak 10, ne 35
    assert_eq!((p.price_cents, p.marketing_cents, p.wage_pct), (13_500, 0, 100));
    assert_eq!((p.training, p.maintenance, p.renovation, p.reply_reviews), (Training::None, Maintenance::Regular, false, false));
}
```
Provjera granica: cijena `price·100 ≥ min_pct·ref` i `price·100 ≤ max_pct·ref` (P6), cijeli euro `price % 100 == 0`
(kroz `calc`: `div_floor` i usporedba); redoslijed provjera kao u testu (cijeli euro prije raspona).
`staff_capacity_caps_august`: `market` prima `staff` izričito, a `state` je C s 20 sezonaca, pa `demand` računa
ocjenu 840 → 3 088; ne mijenja se `state.seasonal` (to radi `resolve` u Tasku 5).

- [ ] **Step 3: Pokreni** `cargo test -p sim-hotel --test market; echo EXIT=$?` → ne prevodi se.
- [ ] **Step 4: Implementiraj** `plan.rs` i `market.rs` po tablici. Sve granice čitaj iz `cat.actions`.
- [ ] **Step 5: Pokreni** test, zatim clippy → 0.
- [ ] **Step 6: Commit** — „Jezgra 4/12: plan mjeseca s granicama i tržište (§3.1)".

---

