mod common;

use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::*;

fn code(s: &str) -> DeptCode {
    DeptCode(s.into())
}

#[test]
fn catalog_with_departments_is_valid() {
    assert_eq!(common::catalog_with_departments().validate(), Ok(()));
}

#[test]
fn summary_counts_staff_from_departments() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let sum = s.summary();
    let base = c.base.profile(Profile::C).unwrap(); // testni profil C (30, 20), ne sadržaj (30, 28)
    assert_eq!(
        (sum.permanent, sum.seasonal),
        (base.permanent, base.seasonal)
    );
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
    let moved: u16 = plan
        .departments
        .iter()
        .map(|(k, d)| d.seasonal_target.abs_diff(s.departments[k].seasonal))
        .sum();
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
    plan.departments
        .get_mut(&code("restaurant"))
        .unwrap()
        .seasonal_target = 0;
    // Uz pomake zadanog plana ovo prelazi i korak 10 — pravilo odjela se provjerava prije koraka.
    assert_eq!(
        plan.validate(&s, &p, &c),
        Err(PlanError::DepartmentCannotClose)
    );
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
    assert_eq!(
        plan.validate(&s, &p, &c),
        Err(PlanError::ClosedWithSeasonal)
    );
}

#[test]
fn hold_plan_is_valid_and_moves_nobody() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = period_by_code(PeriodKind::Week, "t01").unwrap();
    let plan = common::hold_plan(&s, &p, &c);
    assert!(
        plan.departments
            .iter()
            .all(|(k, d)| d.seasonal_target == s.departments[k].seasonal)
    );
    assert_eq!(plan.validate(&s, &p, &c), Ok(()));
}

#[test]
fn carried_plan_respects_department_limits() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let mut plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
    plan.departments
        .get_mut(&code("spa"))
        .unwrap()
        .price_level_pct = 120;
    let carried = plan.carry_into(&s, &month_period(Month::Jul), &c).unwrap();
    assert_eq!(carried.departments[&code("spa")].price_level_pct, 120);
    assert_eq!(carried.validate(&s, &month_period(Month::Jul), &c), Ok(()));
}

#[test]
fn robots_produce_valid_department_plans_every_week() {
    let c = common::catalog_with_departments();
    let r = RefTable::compute(
        &c,
        Category::Four,
        &calendar::season_periods(PeriodKind::Week),
    )
    .unwrap();
    for s in Strategy::ALL {
        run_season(Profile::A, s, 3, PeriodKind::Week, &c, &[], &r).unwrap();
    }
}

// ---- Obračun Taska 11: samo Smještaj, ostali odjeli se nose

fn june_with_hold_plan() -> (Catalog, HotelState, PeriodPlan, PeriodReport) {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let plan = common::hold_plan(&s, &p, &c);
    let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
    (c, s, plan, r)
}

fn line(r: &PeriodReport, var: Var, cause: Cause) -> i64 {
    r.summary
        .lines
        .iter()
        .filter(|l| l.var == var && l.cause == cause)
        .map(|l| l.amount)
        .sum()
}

#[test]
fn market_capacity_and_load_come_from_rooms_staff_only() {
    let (_, s, _, r) = june_with_hold_plan();
    let rooms = &s.departments[&DeptCode::rooms()];
    let rooms_staff = u32::from(rooms.permanent + rooms.seasonal); // 12 + 6
    assert_eq!(r.summary.market.staff_capacity, rooms_staff * 30 * 24 / 10);
    assert_eq!(r.summary.people.staff, s.staff()); // odlasci i iskustvo: cijeli hotel
    let load = (r.summary.market.sold * 100 + rooms_staff * 30 / 2) / (rooms_staff * 30);
    assert_eq!(u32::from(r.summary.people.load_hundredths), load.min(300));
}

#[test]
fn wages_are_the_sum_of_department_payrolls() {
    let (c, s, plan, r) = june_with_hold_plan();
    let monthly: i64 = s
        .departments
        .iter()
        .map(|(k, d)| {
            // Smještaj plaća plaćom kategorije hotela, ostali svojom.
            let wage = if *k == DeptCode::rooms() {
                c.base.category(s.category).unwrap().wage_cents
            } else {
                c.dept(k).unwrap().wage_cents
            };
            i64::from(d.permanent + d.seasonal) * (wage / 100) * i64::from(plan.wage_pct)
        })
        .sum();
    assert!(monthly > 18 * 2_200 * 100); // više od samog Smještaja
    assert_eq!(line(&r, Var::Costs, Cause::Wage), monthly);
}

#[test]
fn fixed_costs_are_the_undistributed_departments() {
    let (_, s, _, r) = june_with_hold_plan();
    let undistributed: i64 = s.undistributed_fixed_cents.values().sum();
    assert_eq!(undistributed, 8_000_000);
    assert_eq!(line(&r, Var::Costs, Cause::FixedCosts), undistributed);
    // Bez udjela prihoda soba (nema odjela `ShareOfRooms`): ostali prihod dolazi u Tasku 12.
    assert_eq!(r.summary.money.other_revenue_cents, 0);
}

#[test]
fn rooms_condition_is_the_equipment_and_other_departments_are_carried() {
    let (_, s, _, r) = june_with_hold_plan();
    let rooms = DeptCode::rooms();
    // Retci opreme u objašnjenju opisuju promjenu stanja prostora Smještaja (v1 pravila opreme).
    let line_sum = line(&r, Var::Equipment, Cause::Baseline)
        + line(&r, Var::Equipment, Cause::Load)
        + line(&r, Var::Equipment, Cause::Maintenance);
    assert_eq!(
        i64::from(r.after.departments[&rooms].condition_pts),
        i64::from(s.departments[&rooms].condition_pts) + line_sum
    );
    for (k, d) in &s.departments {
        if *k != rooms {
            assert_eq!(r.after.departments[k], *d, "{}", k.as_str());
        }
    }
}

#[test]
fn closing_a_department_is_copied_into_the_state() {
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let mut plan = common::hold_plan(&s, &p, &c);
    let bar = plan.departments.get_mut(&code("lobby_bar")).unwrap();
    bar.open = false;
    bar.seasonal_target = 0; // 3 puštena, unutar koraka 10
    let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
    let after = &r.after.departments[&code("lobby_bar")];
    assert!(!after.open);
    assert_eq!(after.seasonal, 0);
    assert_eq!(r.summary.people.released, 3);
    let next = PeriodPlan::default_for(&r.after, &month_period(Month::Jul), &c).unwrap();
    assert!(!next.departments[&code("lobby_bar")].open);
    assert_eq!(next.departments[&code("lobby_bar")].seasonal_target, 0);
}

#[test]
fn category_seasonal_plan_is_the_whole_hotel_plan() {
    // Testna kategorija planira 20 sezonaca; restoran 10, lobby bar 3 i spa 1 ostavljaju Smještaju 6 — točno
    // današnja raspodjela profila C, pa zadani plan nikoga ne pomiče.
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let plan = PeriodPlan::default_for(&s, &month_period(Month::Jun), &c).unwrap();
    for (k, d) in &plan.departments {
        assert_eq!(
            d.seasonal_target,
            s.departments[k].seasonal,
            "{}",
            k.as_str()
        );
    }
    assert_eq!(plan.seasonal_total(), 20);
}
