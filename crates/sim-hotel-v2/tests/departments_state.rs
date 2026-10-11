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

// ---- Obračun s odjelima (Task 11: osoblje i tržište; Task 12: računi odjela)

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
fn market_capacity_comes_from_rooms_staff_and_hotel_load_is_staff_weighted() {
    let (c, s, _, r) = june_with_hold_plan();
    let rooms = &s.departments[&DeptCode::rooms()];
    let rooms_staff = u32::from(rooms.permanent + rooms.seasonal); // 12 + 6
    assert_eq!(r.summary.market.staff_capacity, rooms_staff * 30 * 24 / 10);
    assert_eq!(r.summary.people.staff, s.staff()); // odlasci i iskustvo: cijeli hotel
    // Opterećenje Smještaja (v1 formula) je u njegovom računu.
    let load = (r.summary.market.sold * 100 + rooms_staff * 30 / 2) / (rooms_staff * 30);
    let rooms_acc = &r.accounts.departments[&DeptCode::rooms()];
    assert_eq!(u32::from(rooms_acc.load_hundredths), load.min(300));
    // Task 12: opterećenje hotela je prosjek odjela s kapacitetom i osobljem, ponderiran osobljem.
    let (num, den) = r
        .accounts
        .departments
        .iter()
        .filter(|(k, o)| c.dept(k).unwrap().capacity_tenths_per_staff_day > 0 && o.staff > 0)
        .fold((0_u32, 0_u32), |(n, d), (_, o)| {
            (
                n + u32::from(o.load_hundredths) * u32::from(o.staff),
                d + u32::from(o.staff),
            )
        });
    assert_eq!(
        u32::from(r.summary.people.load_hundredths),
        (num + den / 2) / den
    );
    assert!(r.summary.people.load_hundredths < rooms_acc.load_hundredths);
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
    // Task 12: ostali prihod je zbroj prihoda odjela osim Smještaja; fiksne naknade su ispod GOP-a (redak
    // blagajne, ne trošak).
    let other: i64 = r
        .accounts
        .departments
        .iter()
        .filter(|(k, _)| **k != DeptCode::rooms())
        .map(|(_, o)| o.revenue_cents)
        .sum();
    assert!(other > 0);
    assert_eq!(r.summary.money.other_revenue_cents, other);
    let charges: i64 = s.fixed_charges_cents.values().sum();
    assert_eq!(line(&r, Var::Cash, Cause::FixedCosts), -charges);
}

#[test]
fn department_conditions_follow_their_accounts_and_services_are_carried() {
    let (_, s, _, r) = june_with_hold_plan();
    // Task 12: svaki odjel s računom prenosi stanje prostora i zadovoljstvo iz svog računa; neraspoređeni odjeli
    // (bez računa odjela) se nose.
    for (k, d) in &s.departments {
        let after = &r.after.departments[k];
        match r.accounts.departments.get(k) {
            Some(o) => {
                assert_eq!(o.condition_before_pts, d.condition_pts, "{}", k.as_str());
                assert_eq!(after.condition_pts, o.condition_after_pts, "{}", k.as_str());
                assert_eq!(
                    after.satisfaction_pts,
                    o.satisfaction_after_pts,
                    "{}",
                    k.as_str()
                );
            }
            None => assert_eq!(after, d, "{}", k.as_str()),
        }
    }
    // Smještaj: v1 pravila opreme (trošenje, opterećenje, održavanje; bez obnove i situacija).
    let rooms = &r.accounts.departments[&DeptCode::rooms()];
    assert_eq!(
        rooms.condition_after_pts - rooms.condition_before_pts,
        rooms.wear_pts + rooms.load_wear_pts + rooms.maintenance_pts
    );
    assert_eq!(
        i64::from(rooms.wear_pts),
        line(&r, Var::Equipment, Cause::Baseline)
    );
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

#[test]
fn closed_department_does_not_dilute_hotel_load() {
    // Zatvoreni bar zadržava stalne ljude (opterećenje 0, kazna 0); u prosjek opterećenja hotela ulaze samo otvoreni
    // odjeli, inače bi zatvaranje spustilo kaznu i podiglo cilj morala.
    let c = common::catalog_with_departments();
    let s = HotelState::from_profile(&c, Profile::C).unwrap();
    let p = month_period(Month::Jun);
    let mut plan = common::hold_plan(&s, &p, &c);
    let bar = plan.departments.get_mut(&code("lobby_bar")).unwrap();
    bar.open = false;
    bar.seasonal_target = 0;
    let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
    let bar = &r.accounts.departments[&code("lobby_bar")];
    assert!(!bar.open);
    assert!(bar.staff > 0, "bar bez stalnih ljudi ne provjerava ništa");
    let (num, den) = r
        .accounts
        .departments
        .iter()
        .filter(|(k, o)| {
            o.open && c.dept(k).unwrap().capacity_tenths_per_staff_day > 0 && o.staff > 0
        })
        .fold((0_u32, 0_u32), |(n, d), (_, o)| {
            (
                n + u32::from(o.load_hundredths) * u32::from(o.staff),
                d + u32::from(o.staff),
            )
        });
    assert_eq!(
        u32::from(r.summary.people.load_hundredths),
        (num + den / 2) / den
    );
}
