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
    r.summary
        .lines
        .iter()
        .filter(|l| l.var == var)
        .map(|l| l.amount)
        .sum()
}

#[test]
fn accounts_add_up_to_summary() {
    let (_, r) = june_c();
    let a = &r.accounts;
    let m = &r.summary.money;
    let rooms = &a.departments[&DeptCode::rooms()];
    let other: i64 = a
        .departments
        .values()
        .filter(|d| d.class != UsaliClass::Rooms)
        .map(|d| d.revenue_cents)
        .sum();
    assert_eq!(rooms.revenue_cents, m.rooms_revenue_cents);
    assert_eq!(other, m.other_revenue_cents);
    let dept_costs: i64 = a
        .departments
        .values()
        .map(|d| d.payroll_cents + d.cost_of_sales_cents + d.other_direct_cents)
        .sum();
    let undist: i64 = a.undistributed.values().map(|u| u.total_cents).sum();
    assert_eq!(dept_costs + undist, m.costs_cents);
    assert_eq!(
        m.rooms_revenue_cents + m.other_revenue_cents - m.costs_cents,
        m.gop_cents
    );
    let charges: i64 = a.fixed_charges.values().sum();
    assert_eq!(
        r.after.cash_cents - r.before.cash_cents,
        m.gop_cents - charges - m.capex_cents - m.interest_cents
    );
}

#[test]
fn cost_and_cash_lines_explain_the_summary() {
    let (_, r) = june_c();
    assert_eq!(lines(&r, Var::Costs), r.summary.money.costs_cents);
    assert_eq!(
        lines(&r, Var::Cash),
        r.after.cash_cents - r.before.cash_cents
    );
    assert_eq!(
        lines(&r, Var::Equipment),
        i64::from(r.summary.after.equipment_pts - r.summary.before.equipment_pts)
    );
}

#[test]
fn maintenance_is_booked_in_the_maintenance_service() {
    let (_, r) = june_c();
    let spent: i64 = r
        .accounts
        .departments
        .values()
        .map(|d| d.maintenance_cents)
        .sum();
    let pom = r
        .accounts
        .undistributed
        .values()
        .find(|u| u.code.as_str() == "maintenance")
        .unwrap();
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
    let bar = plan
        .departments
        .get_mut(&DeptCode("lobby_bar".into()))
        .unwrap();
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
        run_season_with(
            Profile::C,
            PeriodKind::Week,
            &c,
            &[],
            &r,
            &mut |s, p| {
                let mut plan = common::hold_plan(s, p, &c);
                if cut {
                    let now = s.departments[&rest].seasonal;
                    plan.departments.get_mut(&rest).unwrap().seasonal_target =
                        now.saturating_sub(step);
                }
                Ok(plan)
            },
            &mut |_| Ok(0),
        )
        .unwrap()
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
    let a = run_season(
        Profile::A,
        Strategy::Random,
        5,
        PeriodKind::Week,
        &c,
        &common::test_situations(),
        &r,
    )
    .unwrap();
    let b = run_season(
        Profile::A,
        Strategy::Random,
        5,
        PeriodKind::Week,
        &c,
        &common::test_situations(),
        &r,
    )
    .unwrap();
    assert_eq!(a, b);
}

#[test]
fn every_week_of_seasons_with_departments_adds_up() {
    // Isti identiteti kao `accounts_add_up_to_summary`, za svaki tjedan sezone sa situacijama i svaku strategiju.
    let c = common::catalog_with_departments();
    let r = RefTable::compute(&c, Category::Four, &season_periods(PeriodKind::Week)).unwrap();
    for strategy in Strategy::ALL {
        let out = run_season(
            Profile::A,
            strategy,
            11,
            PeriodKind::Week,
            &c,
            &common::test_situations(),
            &r,
        )
        .unwrap();
        for (rep, _) in &out.periods {
            let (a, m) = (&rep.accounts, &rep.summary.money);
            let dept_costs: i64 = a
                .departments
                .values()
                .map(|d| d.payroll_cents + d.cost_of_sales_cents + d.other_direct_cents)
                .sum();
            let undist: i64 = a.undistributed.values().map(|u| u.total_cents).sum();
            let charges: i64 = a.fixed_charges.values().sum();
            assert_eq!(
                dept_costs + undist,
                m.costs_cents,
                "{strategy:?} {:?}",
                rep.period
            );
            assert_eq!(
                rep.after.cash_cents - rep.before.cash_cents,
                m.gop_cents - charges - m.capex_cents - m.interest_cents
            );
            assert_eq!(lines(rep, Var::Costs), m.costs_cents);
            assert_eq!(
                lines(rep, Var::Revenue),
                m.rooms_revenue_cents + m.other_revenue_cents
            );
            assert_eq!(
                lines(rep, Var::Cash),
                rep.after.cash_cents - rep.before.cash_cents
            );
        }
    }
}
