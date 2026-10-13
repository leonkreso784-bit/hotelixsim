### Task 5: Obračun mjeseca — osoblje, ljudi, novac, oprema i gosti, objašnjenje

**Files:**
- Create: `crates/sim-hotel/src/resolve.rs`, `crates/sim-hotel/src/explain.rs`, `crates/sim-hotel/tests/resolve.rs`

**Interfaces:**
- Consumes: `market()`, `MonthPlan`, `HotelState`, `Catalog`.
- Produces:

```rust
// explain.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Var { Sold, Revenue, Costs, Cash, Staff, Morale, Experience, Equipment, Satisfaction, Rating }
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cause {
    Baseline, Price, Reputation, Marketing, Groups, Capacity, OtherRevenue,
    Wage, Load, Training, Hires, Release, Turnover, Maintenance, Renovation, ReplyReviews,
    FixedCosts, Interest, Situation(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit { RoomNights, Cents, People, Points, RatingHundredths, Bp }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line { pub var: Var, pub cause: Cause, pub amount: i64, pub unit: Unit }

// resolve.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeopleOutcome {
    pub hires: u16, pub released: u16, pub staff: u16, pub load_hundredths: u16, pub load_penalty_pts: i32,
    pub morale_target_pts: i32, pub turnover_bp: u16, pub departures: u16, pub experience_mix_pts: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoneyOutcome {
    pub rooms_revenue_cents: i64, pub other_revenue_cents: i64, pub labor_cents: i64, pub costs_cents: i64,
    pub gop_cents: i64, pub capex_cents: i64, pub interest_cents: i64, pub goppar_cents: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuestOutcome { pub quality_pts: i32, pub value_pts: i32, pub satisfaction_target_pts: i32, pub rating_target: u16 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthReport {
    pub month: Month, pub before: HotelState, pub after: HotelState, pub market: MarketOutcome,
    pub people: PeopleOutcome, pub money: MoneyOutcome, pub guests: GuestOutcome, pub lines: Vec<Line>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolveError { Calc(engine::calc::CalcError), Plan(PlanError), Catalog(CatalogError), Mechanic(MechanicError) }
// From<CalcError>, From<PlanError>, From<CatalogError>, From<MechanicError> za ResolveError
pub fn resolve_month(state: &HotelState, month: Month, plan: &MonthPlan, situations: &[SituationOutcome], cat: &Catalog)
    -> Result<MonthReport, ResolveError>;
```
U ovom tasku napravi i `crates/sim-hotel/src/mechanic.rs` s PUNIM deklaracijama tipova iz Taska 6 (`Mechanic`,
`SituationOutcome`, `OfferedAnswer`, `SituationTemplate`, `Effects`, `MechanicError`) bez logike, uz dodatnu
varijantu `MechanicError::NotYetSupported`: `resolve_month` je vraća ako `situations` nije prazan. Task 6 dodaje
logiku i briše tu varijantu i granu.

- [ ] **Step 1: Redoslijed i formule** (ADR-013 t. 4; `D` dani, `pend = state.pending`, sve kroz `calc`):

1. `plan.validate(state, month, cat)?`.
2. **Osoblje:** `hires = max(0, target − seasonal)`, `released = max(0, seasonal − target)`; `seasonal = target`;
   `wage_pct = plan.wage_pct`; `staff = permanent + seasonal`. `pending` se isprazni (učinci se troše ovaj mjesec).
3. **Tržište:** `market(&s, month, plan, staff, cat)?`; nakon toga iz `s.groups` makni grupe ovog mjeseca.
4. **Ljudi** (P16–P22):
   `load = staff == 0 ? 300 : clamp(div_round(sold·100, staff·D), 0, 300)` ·
   `penalty = clamp(100·(load − 180), 0, 6000)` ·
   `morale_target = clamp(5500 + 200·(wage − 100) − div_round(penalty, 2) + pend.morale + release_morale(ako released > 0) + Σ situacije, 0, 10000)` ·
   `morale' = clamp(morale + div_round(3000·(target − morale), 10000), 0, 10000)` ·
   `turnover = turnover_bp(morale')` · `departures = clamp(div_round(staff·turnover, 10000), 0, staff)` ·
   `n = min(departures + hires, staff)` ·
   `exp_mix = staff == 0 ? experience : div_round(experience·(staff − n) + 2000·n, staff)` ·
   `experience' = clamp(exp_mix + 300 + pend.experience, 0, 10000)`.
5. **Novac** (P25–P28): `rooms_revenue = individual_sold·price + group_revenue` ·
   `other = div_round(rooms_revenue·other_revenue_bp, 10000)` ·
   `labor = staff·(wage_cents/100)·wage_pct + extra_wage + (hires + departures)·hire_cents + training_cost` ·
   `costs = labor + cpor·sold + maintenance_cost + marketing + fixed + reply_cost + Σ situacije` ·
   `gop = rooms_revenue + other − costs` · `capex = renovation ? renovation_cents : 0` ·
   `cash' = cash + gop − capex`; ako `cash' < 0`: `interest = div_round(−cash', 100)`, `cash' −= interest` ·
   `goppar = div_round(gop, rooms·D)` (P28: nazivnik sve sobe). `renovated |= plan.renovation`.
6. **Oprema i gosti** (P19–P23):
   `equipment' = clamp(equipment − 400 − (load > 180 ? 100 : 0) + maintenance_eff + (pend.renovation ? renovation_equipment : 0) + Σ situacije, 0, 10000)` ·
   `quality = div_round(4000·experience' + 3000·morale' + 3000·(10000 − penalty), 10000)` ·
   `value = clamp(7000 − div_round(10000·(price − ref), ref), 0, 10000)` ·
   `sat_target = div_round(4500·quality + 3000·equipment' + 2500·value, 10000)` ·
   `satisfaction' = clamp(sat + div_round(5000·(sat_target − sat), 10000) + Σ situacije + (pend.renovation ? renovation_satisfaction : 0), 0, 10000)` ·
   `rating_target = div_round(14000 + satisfaction', 25)` ·
   `rating' = clamp(rating + div_round(2500·(rating_target − rating), 10000) + (reply ? reply_rating : 0) + Σ situacije, 100, 1000)`.
7. Spremi `load_hundredths = load`, `turnover_bp = turnover`; novi `pending = { training exp, training morale, plan.renovation }`.

**Objašnjenje (`lines`, §11.6)** — redak se piše kad je iznos ≠ 0, osim `Baseline`, koji se piše uvijek:

| var (jedinica) | uzroci i iznos |
|---|---|
| `Sold` (RoomNights; faktori u Bp) | `Baseline` = base_demand · `Price`/`Reputation`/`Marketing` = faktor − 10 000 (Bp) · `Groups` = group_sold · `Capacity` = sold − (group_room_nights + demand) kad je negativno |
| `Revenue` (Cents) | `Baseline` = individual_sold·price · `Groups` = group_revenue · `OtherRevenue` = other |
| `Costs` (Cents) | `Wage` = staff·(wage_cents/100)·wage_pct + extra_wage · `Hires` = (hires + departures)·hire_cents · `Training` · `Baseline` = cpor·sold · `Maintenance` · `Marketing` · `FixedCosts` · `ReplyReviews` · `Situation(code)` |
| `Cash` (Cents) | `Baseline` = gop · `Renovation` = −capex · `Interest` = −interest |
| `Staff` (People) | `Hires` = hires · `Release` = −released · `Turnover` = departures (zamijenjeni) |
| `Morale` (Points) | `Baseline` = morale' − morale · uz to doprinosi CILJU: `Wage` = 200·(wage − 100) · `Load` = −div_round(penalty, 2) · `Training` = pend.morale · `Release` · `Situation(code)` |
| `Experience` (Points) | `Baseline` = 300 · `Turnover` = exp_mix − experience · `Training` = pend.experience |
| `Equipment` (Points) | `Baseline` = −400 · `Load` = −100 ako opterećenje > 180 · `Maintenance` = učinak · `Renovation` · `Situation(code)` |
| `Satisfaction` (Points) | `Baseline` = div_round(5000·(target − sat), 10000) · `Renovation` · `Situation(code)` |
| `Rating` (RatingHundredths) | `Baseline` = div_round(2500·(target − rating), 10000) · `ReplyReviews` · `Situation(code)` |

- [ ] **Step 2: Testovi** `crates/sim-hotel/tests/resolve.rs` (zlatne vrijednosti iz `oracle-jezgra.mjs`):

```rust
mod common;
use common::*;
use sim_hotel::*;

fn run(s: &HotelState, m: Month, plan: &MonthPlan) -> MonthReport {
    resolve_month(s, m, plan, &[], &test_catalog()).unwrap()
}
fn default(s: &HotelState, m: Month) -> MonthPlan { MonthPlan::default_for(s, m, &test_catalog()).unwrap() }

#[test]
fn profile_c_june_default() {
    let s = state_c();
    let r = run(&s, Month::Jun, &default(&s, Month::Jun));
    let p = &r.people;
    assert_eq!((p.hires, p.staff, p.load_hundredths, p.load_penalty_pts), (0, 50, 173, 0));
    assert_eq!((p.morale_target_pts, p.turnover_bp, p.departures, p.experience_mix_pts), (5_500, 375, 2, 5_840));
    let m = &r.money;
    assert_eq!((m.rooms_revenue_cents, m.other_revenue_cents, m.labor_cents), (34_992_000, 10_497_600, 11_300_000));
    assert_eq!((m.costs_cents, m.gop_cents, m.goppar_cents, m.interest_cents), (30_502_400, 14_987_200, 4_163, 0));
    let g = &r.guests;
    assert_eq!((g.quality_pts, g.value_pts, g.satisfaction_target_pts, g.rating_target), (7_106, 7_000, 6_748, 835));
    let a = &r.after;
    assert_eq!((a.morale_pts, a.experience_pts, a.equipment_pts, a.satisfaction_pts, a.rating), (5_500, 6_140, 6_000, 6_874, 839));
    assert_eq!((a.cash_cents, a.load_hundredths, a.turnover_bp), (44_987_200, 173, 375));
}

#[test]
fn june_variant_price_marketing_staff_wage_training() {
    let s = state_c();
    let plan = MonthPlan { price_cents: 15_000, marketing_cents: 1_500_000, seasonal_target: 30, wage_pct: 105,
        training: Training::Basic, maintenance: Maintenance::Intensive, renovation: false, reply_reviews: true };
    let r = run(&s, Month::Jun, &plan);
    assert_eq!((r.people.hires, r.people.staff, r.people.load_hundredths), (10, 60, 141));
    assert_eq!((r.people.morale_target_pts, r.people.turnover_bp, r.people.departures, r.people.experience_mix_pts), (6_500, 330, 2, 5_200));
    assert_eq!((r.money.rooms_revenue_cents, r.money.other_revenue_cents, r.money.labor_cents), (38_085_000, 11_425_500, 16_160_000));
    assert_eq!((r.money.costs_cents, r.money.gop_cents, r.money.goppar_cents), (38_395_800, 11_114_700, 3_087));
    assert_eq!((r.guests.quality_pts, r.guests.value_pts, r.guests.satisfaction_target_pts, r.guests.rating_target), (6_940, 5_889, 6_515, 830));
    let a = &r.after;
    assert_eq!((a.morale_pts, a.experience_pts, a.equipment_pts, a.satisfaction_pts, a.rating, a.cash_cents), (5_800, 5_500, 6_400, 6_757, 842, 41_114_700));
    assert_eq!(a.pending, Pending { experience_pts: 400, morale_pts: 200, renovation: false });
}

#[test]
fn zero_staff_month() {
    let mut s = state_c();
    s.permanent = 0;
    s.seasonal = 0;
    let mut plan = default(&s, Month::Jun);
    plan.seasonal_target = 0;
    let r = run(&s, Month::Jun, &plan);
    assert_eq!((r.market.sold, r.people.load_hundredths, r.people.load_penalty_pts), (0, 300, 6_000));
    assert_eq!((r.people.morale_target_pts, r.people.turnover_bp, r.people.departures, r.people.experience_mix_pts), (2_500, 510, 0, 6_000));
    assert_eq!((r.money.costs_cents, r.money.gop_cents, r.money.goppar_cents), (13_500_000, -13_500_000, -3_750));
    assert_eq!((r.after.experience_pts, r.after.morale_pts), (6_300, 4_600));
}

#[test]
fn negative_cash_pays_interest() {
    let mut s = state_c();
    s.cash_cents = -500_000;
    s.fixed_cents = 50_000_000;
    let r = run(&s, Month::Jun, &default(&s, Month::Jun));
    assert_eq!((r.money.costs_cents, r.money.gop_cents, r.money.interest_cents), (68_502_400, -23_012_800, 235_128));
    assert_eq!((r.after.cash_cents, r.money.goppar_cents), (-23_747_928, -6_392));
}

#[test]
fn training_and_renovation_act_next_month() {
    let s = state_c();
    let mut plan = default(&s, Month::Jun);
    plan.training = Training::Basic;
    plan.renovation = true;
    let june = run(&s, Month::Jun, &plan);
    assert_eq!((june.money.labor_cents, june.money.gop_cents, june.money.capex_cents), (11_800_000, 14_487_200, 15_000_000));
    assert_eq!((june.after.cash_cents, june.after.equipment_pts, june.after.experience_pts), (29_487_200, 6_000, 6_140));
    assert!(june.after.renovated);
    let july = run(&june.after, Month::Jul, &default(&june.after, Month::Jul));
    assert_eq!((july.people.load_hundredths, july.people.load_penalty_pts, july.people.morale_target_pts), (187, 700, 5_350));
    let a = &july.after;
    assert_eq!((a.morale_pts, a.experience_pts, a.equipment_pts, a.satisfaction_pts, a.rating, a.cash_cents), (5_455, 6_674, 8_400, 7_469, 844, 60_512_200));
    assert_eq!(a.pending, Pending::default());
}

#[test]
fn worn_equipment_month() {
    let mut s = state_c();
    s.equipment_pts = 2_000;
    let r = run(&s, Month::Jun, &default(&s, Month::Jun));
    assert_eq!((r.market.rooms_out, r.after.equipment_pts, r.after.satisfaction_pts), (5, 2_000, 6_274));
}

#[test]
fn cheap_price_deferred_maintenance_and_replies() {
    let s = state_c();
    let mut plan = default(&s, Month::Jun);
    plan.price_cents = 8_100;
    plan.reply_reviews = true;
    plan.maintenance = Maintenance::Deferred;
    let r = run(&s, Month::Jun, &plan);
    assert_eq!((r.money.rooms_revenue_cents, r.money.other_revenue_cents, r.money.costs_cents, r.money.gop_cents), (27_710_100, 8_313_030, 31_276_200, 4_746_930));
    assert_eq!((r.guests.quality_pts, r.guests.value_pts, r.guests.satisfaction_target_pts, r.guests.rating_target), (5_450, 10_000, 6_603, 832));
    let a = &r.after;
    assert_eq!((a.morale_pts, a.equipment_pts, a.satisfaction_pts, a.rating, a.load_hundredths, a.turnover_bp), (4_780, 5_500, 6_801, 843, 228, 483));
}

#[test]
fn releasing_seasonals_in_august_overloads_staff() {
    let mut s = state_c();
    s.seasonal = 10;
    let mut plan = default(&s, Month::Aug);
    plan.seasonal_target = 0;
    let r = run(&s, Month::Aug, &plan);
    assert_eq!((r.people.released, r.people.staff, r.people.load_hundredths, r.people.morale_target_pts), (10, 30, 240, 2_200));
    assert_eq!((r.people.turnover_bp, r.people.departures, r.people.experience_mix_pts), (524, 2, 5_733));
    let a = &r.after;
    assert_eq!((a.morale_pts, a.experience_pts, a.equipment_pts, a.satisfaction_pts, a.rating, a.cash_cents), (4_510, 6_033, 5_900, 6_377, 834, 55_467_600));
}

#[test]
fn every_changed_variable_has_a_cause_and_money_lines_add_up() {
    let s = state_c();
    let mut plan = default(&s, Month::Jun);
    plan.price_cents = 15_000;
    plan.renovation = true;
    plan.training = Training::Full;
    let r = run(&s, Month::Jun, &plan);
    let total = |v: Var| r.lines.iter().filter(|l| l.var == v).map(|l| l.amount).sum::<i64>();
    assert_eq!(total(Var::Costs), r.money.costs_cents);
    assert_eq!(total(Var::Revenue), r.money.rooms_revenue_cents + r.money.other_revenue_cents);
    assert_eq!(total(Var::Cash), r.after.cash_cents - r.before.cash_cents);
    let changed = [
        (Var::Morale, r.before.morale_pts != r.after.morale_pts),
        (Var::Experience, r.before.experience_pts != r.after.experience_pts),
        (Var::Equipment, r.before.equipment_pts != r.after.equipment_pts),
        (Var::Satisfaction, r.before.satisfaction_pts != r.after.satisfaction_pts),
        (Var::Rating, r.before.rating != r.after.rating),
        (Var::Cash, r.before.cash_cents != r.after.cash_cents),
    ];
    for (v, did) in changed {
        if did { assert!(r.lines.iter().any(|l| l.var == v), "{v:?} se promijenio bez uzroka"); }
    }
}

#[test]
fn invalid_plan_is_rejected_before_any_calculation() {
    let s = state_c();
    let mut plan = default(&s, Month::Jun);
    plan.wage_pct = 200;
    assert_eq!(resolve_month(&s, Month::Jun, &plan, &[], &test_catalog()), Err(ResolveError::Plan(PlanError::WageOutOfRange)));
}
```
(Testovi u ovoj datoteci smiju koristiti `+ - ==` jer `forbid` iz `lib.rs` ne veže `tests/`.)

- [ ] **Step 3: Pokreni** `cargo test -p sim-hotel --test resolve; echo EXIT=$?` → ne prevodi se.
- [ ] **Step 4: Implementiraj** `resolve.rs` i `explain.rs` po koraku 1. Svaka faza je privatna funkcija s vlastitim
  ulazom i izlazom (`staffing`, `people`, `money`, `guests`); `resolve_month` ih samo slaže i puni `lines`.
- [ ] **Step 5: Pokreni** `cargo test -p sim-hotel; echo EXIT=$?` → svi prolaze. Ako zlatna vrijednost ne štima,
  usporedi međuvrijednosti s `node scripts/oracle-jezgra.mjs` (ispisuje lipanj C) — popravlja se Rust, ne broj u testu;
  ako je model kriv, popravi `oracle-jezgra.mjs` I test, i zapiši zašto u commit poruci.
- [ ] **Step 6: Commit** — „Jezgra 5/12: obračun mjeseca u fazama i objašnjenje promjena (§3, §11.6)".

---

