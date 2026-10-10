mod common;

use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code, season_periods};
use sim_hotel_v2::*;

fn cat() -> Catalog {
    degenerate(&common::test_catalog())
}

fn refs(c: &Catalog, kind: PeriodKind) -> RefTable {
    RefTable::compute(c, Category::Four, &season_periods(kind)).unwrap()
}

#[test]
fn weekly_season_runs_for_every_profile_and_strategy() {
    let c = cat();
    let r = refs(&c, PeriodKind::Week);
    for p in Profile::ALL {
        for s in Strategy::ALL {
            let out = run_season(
                p,
                s,
                7,
                PeriodKind::Week,
                &c,
                &common::test_situations(),
                &r,
            )
            .unwrap();
            assert_eq!(out.periods.len(), 17, "{p:?} {s:?}");
        }
    }
}

#[test]
fn weekly_season_is_reproducible() {
    let c = cat();
    let r = refs(&c, PeriodKind::Week);
    let a = run_season(
        Profile::C,
        Strategy::Random,
        11,
        PeriodKind::Week,
        &c,
        &common::test_situations(),
        &r,
    )
    .unwrap();
    let b = run_season(
        Profile::C,
        Strategy::Random,
        11,
        PeriodKind::Week,
        &c,
        &common::test_situations(),
        &r,
    )
    .unwrap();
    assert_eq!(a, b);
}

fn sum_lines(out: &SeasonResult, var: Var, cause: Cause) -> i64 {
    out.periods
        .iter()
        .flat_map(|(r, _)| r.summary.lines.iter())
        .filter(|l| l.var == var && l.cause == cause)
        .map(|l| l.amount)
        .sum()
}

fn default_season(kind: PeriodKind) -> SeasonResult {
    let c = cat();
    let r = refs(&c, kind);
    run_season_with(
        Profile::C,
        kind,
        &c,
        &[],
        &r,
        &mut |s, p| Ok(PeriodPlan::default_for(s, p, &c)?),
        &mut |_| Ok(0),
    )
    .unwrap()
}

#[test]
fn weekly_fixed_costs_over_season_equal_monthly() {
    let w = default_season(PeriodKind::Week);
    let m = default_season(PeriodKind::Month);
    let fixed = common::profile_c().fixed_cents;
    assert_eq!(sum_lines(&m, Var::Costs, Cause::FixedCosts), 4 * fixed);
    assert_eq!(sum_lines(&w, Var::Costs, Cause::FixedCosts), 4 * fixed);
}

#[test]
fn wages_of_week_two_are_the_prorated_june_wage() {
    let c = cat();
    let s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let t02 = period_by_code(PeriodKind::Week, "t02").unwrap();
    let plan = PeriodPlan::default_for(&s, &t02, &c).unwrap();
    let r = resolve_period(&s, &t02, &plan, &[], &c).unwrap();
    let staff = i64::from(plan.seasonal_target) + i64::from(s.permanent);
    let monthly = staff
        * (c.base.category(Category::Four).unwrap().wage_cents / 100)
        * i64::from(plan.wage_pct);
    let wage = r
        .summary
        .lines
        .iter()
        .find(|l| l.var == Var::Costs && l.cause == Cause::Wage)
        .unwrap()
        .amount;
    assert_eq!(wage, monthly * 14 / 30 - monthly * 7 / 30);
}

#[test]
fn group_nights_split_across_weeks_sum_to_the_block() {
    let c = cat();
    let r = refs(&c, PeriodKind::Week);
    let out = run_season(
        Profile::C,
        Strategy::Caring,
        7,
        PeriodKind::Week,
        &c,
        &common::test_situations(),
        &r,
    )
    .unwrap();
    let total: u32 = out
        .periods
        .iter()
        .map(|(p, _)| p.summary.market.group_room_nights)
        .sum();
    assert_eq!(total, 100); // T2: 20 soba × 5 noći u kolovozu
    assert!(out.periods.last().unwrap().0.after.groups.is_empty());
}

#[test]
fn carried_price_is_clamped_into_new_period_bounds() {
    let c = cat();
    let s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let aug = month_period(Month::Aug);
    let mut plan = PeriodPlan::default_for(&s, &aug, &c).unwrap();
    let aug_max = PeriodPlan::limits(&s, &aug, &c).unwrap().price_max_cents;
    plan.price_cents = aug_max;
    plan.renovation = true;
    let sep = month_period(Month::Sep);
    let carried = plan.carry_into(&s, &sep, &c).unwrap();
    assert_eq!(
        carried.price_cents,
        PeriodPlan::limits(&s, &sep, &c).unwrap().price_max_cents
    );
    assert!(!carried.renovation);
    assert_eq!(carried.validate(&s, &sep, &c), Ok(()));
}

#[test]
fn default_weekly_price_is_day_weighted_reference_in_whole_euros() {
    let c = cat();
    let s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    let june = c
        .base
        .category(Category::Four)
        .unwrap()
        .price_ref_cents
        .get(Month::Jun);
    let july = c
        .base
        .category(Category::Four)
        .unwrap()
        .price_ref_cents
        .get(Month::Jul);
    let expected_cents = (2 * june + 5 * july + 3) / 7; // div_round
    let expected = (expected_cents + 50) / 100 * 100;
    assert_eq!(
        PeriodPlan::default_for(&s, &t05, &c).unwrap().price_cents,
        expected
    );
}

#[test]
fn situations_attach_to_period_containing_first_of_month() {
    let mut sits = common::test_situations();
    sits[1].month = Month::Jul;
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    let t06 = period_by_code(PeriodKind::Week, "t06").unwrap();
    assert_eq!(
        situations_for(&t05, &sits)
            .iter()
            .map(|t| t.code.as_str())
            .collect::<Vec<_>>(),
        ["T2"]
    );
    assert!(situations_for(&t06, &sits).is_empty());
}

#[test]
fn weekly_and_monthly_default_seasons_end_close() {
    let w = default_season(PeriodKind::Week);
    let m = default_season(PeriodKind::Month);
    let (wi, mi) = (&w.periods.last().unwrap().1, &m.periods.last().unwrap().1);
    for (i, v) in wi.iter() {
        let diff = i32::from(v.hundredths()) - i32::from(mi.get(i).hundredths());
        assert!(
            diff.abs() <= 1_000,
            "{i:?}: tjedno {} mjesečno {}",
            v.hundredths(),
            mi.get(i).hundredths()
        );
    }
}

#[test]
fn weekly_lines_explain_every_period_of_a_default_season() {
    let w = default_season(PeriodKind::Week);
    for (r, _) in &w.periods {
        common::assert_lines_explain(&r.summary);
    }
}
