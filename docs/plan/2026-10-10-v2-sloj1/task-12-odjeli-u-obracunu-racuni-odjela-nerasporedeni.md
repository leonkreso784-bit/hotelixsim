### Task 12: Odjeli u obračunu — računi odjela, neraspoređeni troškovi, fiksne naknade, sažetak

**Veličina:** M, 3–4 h · **Ovisi o:** Task 11 · **Model:** sonnet (opus ako se identiteti ne slažu)

**Cilj:** svaki operativni odjel dobiva svoj račun po formulama Taska 9; neraspoređeni odjeli i fiksne naknade ispod
GOP-a; v1 sažetak (prihod, troškovi, GOP, oprema, kvaliteta, zadovoljstvo, opterećenje) slaže se iz odjela; izvješće
razdoblja nosi `accounts: PeriodAccounts`.

**Files:**
- Modify: `crates/sim-hotel-v2/src/{resolve,explain,market}.rs`
- Create: `crates/sim-hotel-v2/tests/accounts.rs`
- Ne dirati: `department.rs` formule (ako je formula kriva — stani, javi, Ruling), `crates/game/**` (osim ako se ne
  prevodi zbog novog polja — tada samo `..` u uzorku), sadržaj.

**Interfaces:**
- Consumes: Task 9 (`dept_step`, `undistributed_step`, `AssignedCosts`, `PeriodAccounts`), Task 11 (stanje, plan).
- Produces: `PeriodReport { period, before, after, summary, pub accounts: PeriodAccounts }`.

Slaganje (redoslijed u `resolve_period`):
1. Osoblje po odjelu (Task 11), tržište i Smještaj kao u Tasku 11 → `sold`, `guest_nights = div_round(sold·guests_per_room_hundredths, 100)`.
2. Ljudi (moral, iskustvo, odlasci) — kazna opterećenja = **ponderirana prema osoblju** kazna odjela s kapacitetom
   (`capacity_tenths_per_staff_day > 0` i osoblje > 0); sažetak `load_hundredths` = isti ponder nad opterećenjima.
   Odlasci po odjelu `div_round(osoblje·turnover·dani, 10 000·D₀)`; trošak zapošljavanja (najam + odlasci) × `hire_cents`.
3. `dept_step` za svaki odjel osim Smještaja i neraspoređenih (u redoslijedu koda); Smještaj dobiva `DeptOutcome` iz
   koraka tržišta (prihod soba, plaće Smještaja + `extra_wage_cents` 🟨 P12, `other_direct = CPOR·sold`, stanje po
   pravilima v1, zadovoljstvo odjelom: cilj po v1 formuli s vrijednošću iz cijene sobe, težina = `guest_nights`).
4. Kvaliteta hotela = Σ(kvaliteta·težina)/Σ težina; cilj zadovoljstva hotela = Σ(cilj odjela·težina)/Σ težina, pa v1
   pravilo pomaka (+ obnova, + situacije). U degeneriranom katalogu ima samo Smještaj → točno v1.
5. Neraspoređeni (`undistributed_step`) s dodjelom troškova odluka (Task 9).
6. Fiksne naknade: `prorate(mjesečni, p)` po kodu.
7. Sažetak: `other_revenue = Σ prihoda odjela osim Smještaja`; `labor = Σ plaća (svi odjeli) + zapošljavanje + obuka`;
   `costs = labor + Σ cost_of_sales + Σ other_direct + Σ održavanje + marketing + odgovaranje + Σ stalni neraspoređeni +
   Σ po noći + situacije`; `gop = prihod − costs`; `cash = cash + gop − Σ naknade − obnova`, pa kamata na negativnu
   blagajnu (Pravila razdoblja).
8. Retci objašnjenja sažetka (samo postojeći `Cause`): `Revenue/OtherRevenue = Σ ostalih prihoda`; `Costs/Baseline =
   Σ cost_of_sales + Σ other_direct + Σ po noći` (svih odjela; CPOR se **ne** dodaje posebno — on je već
   `other_direct` Smještaja = `CPOR·sold`, korak 3, pa bi se inače brojao dvaput); `Costs/Maintenance = Σ održavanje`; `Costs/FixedCosts =
   Σ stalni neraspoređeni`; **novi redak** `Cash/FixedCosts = −Σ naknade`; ostalo kao v1. Oprema u sažetku = 70/30
   (Task 11), a `Equipment` retci objašnjavaju promjenu **sažetka** (Baseline = promjena Smještaja, Load/Maintenance
   po Smještaju; razlika zbog ostalih odjela ide u redak `Equipment/Maintenance` — test provjerava da se retci zbrajaju).

- [ ] **Step 1: Testovi (padaju)** — `tests/accounts.rs`:

```rust
mod common;

use sim_hotel_v2::calendar::{PeriodKind, month_period, season_periods};
use sim_hotel_v2::department::{DeptCode, UsaliClass};
use sim_hotel_v2::*;

fn june_c() -> (Catalog, PeriodReport) {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
    (c, r)
}

fn lines(r: &PeriodReport, var: Var) -> i64 {
    r.summary.lines.iter().filter(|l| l.var == var).map(|l| l.amount).sum()
}

#[test]
fn accounts_add_up_to_summary() {
    let (_, r) = june_c();
    let a = &r.accounts;
    let m = &r.summary.money;
    let rooms = &a.departments[&DeptCode::rooms()];
    let other: i64 = a.departments.values().filter(|d| d.class != UsaliClass::Rooms).map(|d| d.revenue_cents).sum();
    assert_eq!(rooms.revenue_cents, m.rooms_revenue_cents);
    assert_eq!(other, m.other_revenue_cents);
    let dept_costs: i64 = a.departments.values().map(|d| d.payroll_cents + d.cost_of_sales_cents + d.other_direct_cents).sum();
    let undist: i64 = a.undistributed.values().map(|u| u.total_cents).sum();
    assert_eq!(dept_costs + undist, m.costs_cents);
    assert_eq!(m.rooms_revenue_cents + m.other_revenue_cents - m.costs_cents, m.gop_cents);
    let charges: i64 = a.fixed_charges.values().sum();
    assert_eq!(r.after.cash_cents - r.before.cash_cents, m.gop_cents - charges - m.capex_cents - m.interest_cents);
}

#[test]
fn cost_and_cash_lines_explain_the_summary() {
    let (_, r) = june_c();
    assert_eq!(lines(&r, Var::Costs), r.summary.money.costs_cents);
    assert_eq!(lines(&r, Var::Cash), r.after.cash_cents - r.before.cash_cents);
    assert_eq!(lines(&r, Var::Equipment), r.summary.after.equipment_pts - r.summary.before.equipment_pts);
}

#[test]
fn maintenance_is_booked_in_the_maintenance_service() {
    let (_, r) = june_c();
    let spent: i64 = r.accounts.departments.values().map(|d| d.maintenance_cents).sum();
    let pom = r.accounts.undistributed.values().find(|u| u.code.as_str() == "maintenance").unwrap();
    assert_eq!(pom.assigned.maintenance_cents, spent);
}

#[test]
fn closing_the_bar_stops_its_revenue_but_not_permanent_wages() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    // Svi ostali odjeli na trenutnim sezoncima, bar 3 → 0: Σ|Δ| = 3 ≤ korak 10, pa `resolve_period` (koji zove
    // `validate`) prolazi.
    let mut plan = common::hold_plan(&s, &p, &c);
    let bar = plan.departments.get_mut(&DeptCode("lobby_bar".into())).unwrap();
    bar.open = false;
    bar.seasonal_target = 0;
    let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
    let o = &r.accounts.departments[&DeptCode("lobby_bar".into())];
    assert_eq!(o.revenue_cents, 0);
    assert!(o.payroll_cents > 0);
    assert!(!r.after.departments[&DeptCode("lobby_bar".into())].open);
}

#[test]
fn understaffed_restaurant_lowers_its_satisfaction_and_next_week_usage() {
    let c = common::catalog_with_departments();
    let r = RefTable::compute(&c, Category::Four, &season_periods(PeriodKind::Week)).unwrap();
    let rest = DeptCode("restaurant".into());
    let step = c.coeffs(PeriodKind::Week).unwrap().seasonal_step; // 5
    // Oba puta svi odjeli stoje na trenutnim sezoncima; „cut" svaki tjedan reže restoran za najviše korak
    // (10 → 5 → 0 → 0 …), pa svaki plan prolazi `validate` u `resolve_period`.
    let run = |cut: bool| {
        run_season_with(Profile::C, PeriodKind::Week, &c, &[], &r, &mut |s, p| {
            let mut plan = common::hold_plan(s, p, &c);
            if cut {
                let now = s.departments[&rest].seasonal;
                plan.departments.get_mut(&rest).unwrap().seasonal_target = now.saturating_sub(step);
            }
            Ok(plan)
        }, &mut |_| Ok(0)).unwrap()
    };
    let (full, cut) = (run(false), run(true));
    let at = |s: &SeasonResult, i: usize| s.periods[i].0.accounts.departments[&rest].clone();
    assert!(at(&cut, 3).quality_pts < at(&full, 3).quality_pts);
    assert!(at(&cut, 3).satisfaction_after_pts < at(&full, 3).satisfaction_after_pts);
    assert!(at(&cut, 4).usage_bp < at(&full, 4).usage_bp);
}

#[test]
fn weekly_season_with_departments_is_reproducible() {
    let c = common::catalog_with_departments();
    let r = RefTable::compute(&c, Category::Four, &season_periods(PeriodKind::Week)).unwrap();
    let a = run_season(Profile::A, Strategy::Random, 5, PeriodKind::Week, &c, &common::test_situations(), &r).unwrap();
    let b = run_season(Profile::A, Strategy::Random, 5, PeriodKind::Week, &c, &common::test_situations(), &r).unwrap();
    assert_eq!(a, b);
}
```

- [ ] **Step 2:** `cargo test -p sim-hotel-v2 --test accounts` → FAIL.
- [ ] **Step 3: Implementiraj** po redoslijedu gore. Nakon svakog koraka `cargo test -p sim-hotel-v2 --test equivalence`
  mora biti zelen (degenerirani katalog nema operativnih odjela osim `ostalo` s `ShareOfRooms`).
- [ ] **Step 4:** `cargo test --workspace; echo EXIT=$?` → zeleno (uključujući `game --test equivalence`).
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/12: odjeli u obračunu — računi odjela, neraspoređeni troškovi, fiksne naknade".

---

