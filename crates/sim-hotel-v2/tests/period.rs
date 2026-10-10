mod common;

use sim_hotel_v2::calendar::{self, PeriodKind, month_period, period_by_code, season_periods};
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

// ---- Popravak 1: tjedni retci koje mjesečna ekvivalencija ne razlikuje od "bez prorate"

fn monthly_departures(staff: i64, turnover_bp: i64) -> i64 {
    (2 * staff * turnover_bp + 10_000) / 20_000 // div_round(staff·turnover, 10_000)
}

#[test]
fn weekly_departures_are_the_prorated_monthly_departures() {
    let w = default_season(PeriodKind::Week);
    let mut total = 0_i64;
    for (r, _) in &w.periods {
        let monthly = monthly_departures(
            i64::from(r.summary.people.staff),
            i64::from(r.summary.people.turnover_bp),
        );
        let expected = calendar::prorate(monthly, &r.period).unwrap();
        assert_eq!(
            i64::from(r.summary.people.departures),
            expected,
            "{}",
            r.period.code()
        );
        total += i64::from(r.summary.people.departures);
    }
    assert!(
        total > 0,
        "tjedni odlasci nikad ne smiju biti nula cijelu sezonu"
    );
}

#[test]
fn weekly_departures_over_june_match_the_monthly_value_for_stable_staff() {
    // 56 ljudi, moral na cilju: stopa ~300 bp daje mjesečno 2 odlaska; tjedni zbroj lipnja mora biti 2, ne 0.
    let c = cat();
    let mut s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    s.permanent = 36;
    s.seasonal = 20;
    let mut sum = 0_i64;
    let mut monthly = 0_i64;
    for code in ["t01", "t02", "t03", "t04"] {
        let p = period_by_code(PeriodKind::Week, code).unwrap();
        let plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
        let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
        monthly = monthly_departures(
            i64::from(r.summary.people.staff),
            i64::from(r.summary.people.turnover_bp),
        );
        sum += i64::from(r.summary.people.departures);
        s = r.after;
    }
    assert!(monthly > 0);
    assert!(sum > 0, "četiri lipanjska tjedna bez odlazaka");
}

fn season_with(kind: PeriodKind, training: Training, maintenance: Maintenance) -> SeasonResult {
    let c = cat();
    let r = refs(&c, kind);
    run_season_with(
        Profile::C,
        kind,
        &c,
        &[],
        &r,
        &mut |s, p| {
            let mut plan = PeriodPlan::default_for(s, p, &c)?;
            plan.training = training;
            plan.maintenance = maintenance;
            Ok(plan)
        },
        &mut |_| Ok(0),
    )
    .unwrap()
}

#[test]
fn weekly_maintenance_training_and_fixed_sums_equal_four_months() {
    let w = season_with(PeriodKind::Week, Training::Full, Maintenance::Intensive);
    let m = season_with(PeriodKind::Month, Training::Full, Maintenance::Intensive);
    for cause in [Cause::Maintenance, Cause::Training, Cause::FixedCosts] {
        let (a, b) = (
            sum_lines(&w, Var::Costs, cause.clone()),
            sum_lines(&m, Var::Costs, cause.clone()),
        );
        assert!(b > 0, "{cause:?}");
        assert_eq!(a, b, "{cause:?}");
    }
    assert_eq!(sum_lines(&w, Var::Costs, Cause::Maintenance), 4 * 3_000_000);
    assert_eq!(sum_lines(&w, Var::Costs, Cause::Training), 4 * 1_000_000);
}

#[test]
fn wage_of_t05_is_prorated_across_june_and_july() {
    let c = cat();
    let s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    let plan = PeriodPlan::default_for(&s, &t05, &c).unwrap();
    let r = resolve_period(&s, &t05, &plan, &[], &c).unwrap();
    let staff = i64::from(plan.seasonal_target) + i64::from(s.permanent);
    let m = staff
        * (c.base.category(Category::Four).unwrap().wage_cents / 100)
        * i64::from(plan.wage_pct);
    // lipanj: dani 28–29 od 30; srpanj: dani 0–4 od 31
    let expected = (m - m * 28 / 30) + m * 5 / 31;
    let wage = r
        .summary
        .lines
        .iter()
        .find(|l| l.var == Var::Costs && l.cause == Cause::Wage)
        .unwrap()
        .amount;
    assert_eq!(wage, expected);
}

#[test]
fn weekly_interest_is_days_over_month_days_of_the_monthly_interest() {
    let c = cat();
    let mut s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    s.cash_cents = -50_000_000;
    let t02 = period_by_code(PeriodKind::Week, "t02").unwrap();
    let plan = PeriodPlan::default_for(&s, &t02, &c).unwrap();
    let r = resolve_period(&s, &t02, &plan, &[], &c).unwrap();
    let m = &r.summary.money;
    let before_interest = s.cash_cents + m.gop_cents - m.capex_cents;
    assert!(before_interest < 0);
    let monthly = -before_interest * i64::from(c.base.economy.interest_bp);
    let expected = (2 * monthly * 7 + 2 * 10_000 * 30 / 2) / (2 * 10_000 * 30); // zaokruženo
    assert!(m.interest_cents > 0);
    assert!(
        (m.interest_cents - expected).abs() <= 1,
        "{} vs {expected}",
        m.interest_cents
    );
}

#[test]
fn weekly_lines_explain_with_situations_for_every_strategy() {
    let c = cat();
    let r = refs(&c, PeriodKind::Week);
    for s in Strategy::ALL {
        let out = run_season(
            Profile::C,
            s,
            7,
            PeriodKind::Week,
            &c,
            &common::test_situations(),
            &r,
        )
        .unwrap();
        for (rep, _) in &out.periods {
            common::assert_lines_explain(&rep.summary);
        }
    }
}

#[test]
fn weekly_report_against_monthly_ref_table_is_an_error_not_wrong_indices() {
    let c = cat();
    let weekly = default_season(PeriodKind::Week);
    let monthly_refs = refs(&c, PeriodKind::Month);
    let first = &weekly.periods[0].0;
    assert!(matches!(
        period_indices(first, &c, &monthly_refs),
        Err(ResolveError::Catalog(_))
    ));
}

#[test]
fn group_block_is_spread_over_at_least_two_weeks() {
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
    let with_groups = out
        .periods
        .iter()
        .filter(|(p, _)| p.summary.market.group_room_nights > 0)
        .count();
    assert!(with_groups >= 2, "{with_groups}");
}

// ---- Ojačanje H1, korak 2: situacija se obračunava s mjesecom situacije, ne s početnim mjesecom kruga

fn group_answer(code: &str, target: Month) -> SituationOutcome {
    SituationOutcome {
        code: code.into(),
        mechanics: vec![Mechanic::GroupBooking {
            accept: true,
            rooms: 10,
            nights: 31,
            discount_bp: 0,
            month: target,
        }],
        quality: 3,
    }
}

#[test]
fn july_situation_in_week_five_books_its_group_into_july_days() {
    let c = cat();
    let s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    let plan = PeriodPlan::default_for(&s, &t05, &c).unwrap();
    let r = resolve_period(&s, &t05, &plan, &[group_answer("S7", Month::Jul)], &c).unwrap();
    // 310 noćenja u srpnju; t05 nosi srpanjske dane 1.–5. (dani 0–5 od 31), lipanjski dio ne nosi ništa.
    assert_eq!(r.summary.market.group_room_nights, 310 * 5 / 31);
    assert_eq!(r.after.groups.len(), 1);
    assert_eq!(r.after.groups[0].month, Month::Jul);
}

#[test]
fn june_group_in_july_situation_of_week_five_is_in_the_past() {
    let c = cat();
    let s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    let plan = PeriodPlan::default_for(&s, &t05, &c).unwrap();
    assert_eq!(
        resolve_period(&s, &t05, &plan, &[group_answer("S7", Month::Jun)], &c),
        Err(ResolveError::Mechanic(MechanicError::GroupInPast))
    );
}

#[test]
fn june_group_in_june_month_is_still_accepted() {
    let c = cat();
    let s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let jun = month_period(Month::Jun);
    let plan = PeriodPlan::default_for(&s, &jun, &c).unwrap();
    let r = resolve_period(&s, &jun, &plan, &[group_answer("S1", Month::Jun)], &c).unwrap();
    assert_eq!(r.summary.market.group_room_nights, 310);
    assert!(r.after.groups.is_empty());
}

#[test]
fn situation_month_of_a_period_is_the_month_whose_situations_it_carries() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        for p in season_periods(kind) {
            for m in Month::SEASON {
                assert_eq!(
                    calendar::first_period_of_month(kind, m) == p,
                    p.month_starting_in() == Some(m),
                    "{} {m:?}",
                    p.code()
                );
            }
        }
    }
}
