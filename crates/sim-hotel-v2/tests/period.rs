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
    default_season_of(Profile::C, kind)
}

fn default_season_of(profile: Profile, kind: PeriodKind) -> SeasonResult {
    let c = cat();
    let r = refs(&c, kind);
    run_season_with(
        profile,
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

// Profil C je referentni hotel tablice indeksa, pa su njegovi indeksi u zadanoj sezoni po građi 50,00 i u tjednoj i
// u mjesečnoj igri, kakva god bila pravila razdoblja. Čuvari zato gledaju profile A i B.

/// Indeksi na kraju svakog mjeseca: početni, pa poslije kruga koji sadrži zadnji dan lipnja, srpnja, kolovoza i rujna
/// (u tjednoj igri to je tjedan koji prelazi u idući mjesec, npr. t05 za lipanj).
fn month_end_indices(out: &SeasonResult) -> Vec<IndexSet> {
    let mut v = vec![out.start.clone()];
    v.extend(
        out.periods
            .iter()
            .filter(|(r, _)| r.period.month_ending_in().is_some())
            .map(|(_, i)| i.clone()),
    );
    assert_eq!(v.len(), 5);
    v
}

fn hundredths(set: &IndexSet, i: Index) -> i32 {
    i32::from(set.get(i).hundredths())
}

/// Najveća dopuštena razlika tjednog i mjesečnog indeksa na kraju mjeseca (stotinke boda). Izvedeno iz stvarnih
/// razlika profila A i B (najveće: gosti 0,71, recenzije 3,57, osoblje 0,63, poslovanje 1,50, financije 3,12 boda) uz
/// rezervu ~30 %; razlika dolazi od tjedna koji na kraju mjeseca već nosi dane idućeg mjeseca.
fn month_end_tolerance(i: Index) -> i32 {
    match i {
        Index::Guests | Index::Staff => 100,
        Index::Operations => 200,
        Index::Reviews | Index::Finance => 400,
    }
}

#[test]
fn weekly_and_monthly_default_seasons_stay_close_at_every_month_end() {
    for p in [Profile::A, Profile::B] {
        let w = month_end_indices(&default_season_of(p, PeriodKind::Week));
        let m = month_end_indices(&default_season_of(p, PeriodKind::Month));
        for k in 1..5 {
            for i in Index::ALL {
                let (a, b) = (hundredths(&w[k], i), hundredths(&m[k], i));
                assert!(
                    (a - b).abs() <= month_end_tolerance(i),
                    "{p:?} kraj mjeseca {k} {i:?}: tjedno {a} mjesečno {b}"
                );
            }
        }
    }
}

/// Promjena indeksa preko mjeseca je „jasna" kad je u mjesečnoj igri barem 2 boda; manje od toga može biti samo
/// pomak od tjedna koji na kraju mjeseca nosi dane idućeg mjeseca.
const CLEAR_MONTHLY_CHANGE: i32 = 200;

#[test]
fn weekly_and_monthly_default_seasons_move_indices_the_same_way() {
    for p in [Profile::A, Profile::B] {
        let w = month_end_indices(&default_season_of(p, PeriodKind::Week));
        let m = month_end_indices(&default_season_of(p, PeriodKind::Month));
        for i in Index::ALL {
            // Cijela sezona: isti smjer za svaki indeks.
            let (dw, dm) = (
                hundredths(&w[4], i) - hundredths(&w[0], i),
                hundredths(&m[4], i) - hundredths(&m[0], i),
            );
            assert_eq!(
                dw.signum(),
                dm.signum(),
                "{p:?} {i:?} sezona: tjedno {dw} mjesečno {dm}"
            );
            // Svaki mjesec s jasnom mjesečnom promjenom: isti smjer.
            for k in 1..5 {
                let dm = hundredths(&m[k], i) - hundredths(&m[k - 1], i);
                let dw = hundredths(&w[k], i) - hundredths(&w[k - 1], i);
                if dm.abs() >= CLEAR_MONTHLY_CHANGE {
                    assert_eq!(
                        dw.signum(),
                        dm.signum(),
                        "{p:?} {i:?} mjesec {k}: tjedno {dw} mjesečno {dm}"
                    );
                }
            }
        }
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
    // 56 ljudi, isti mjesečni broj odlazaka M svaki tjedan: t01–t04 su lipanjski dani 0–28, pa se razmjerni
    // dijelovi teleskopski zbrajaju u ⌊M·28/30⌋. Staro pravilo (svaki tjedan zasebno zaokružen) daje 0, a
    // pokvaren `prorate` (pun M po tjednu) daje 4·M.
    let c = cat();
    let mut s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    s.permanent = 36;
    s.seasonal = 20;
    let mut sum = 0_i64;
    let mut monthly = Vec::new();
    for code in ["t01", "t02", "t03", "t04"] {
        let p = period_by_code(PeriodKind::Week, code).unwrap();
        let plan = PeriodPlan::default_for(&s, &p, &c).unwrap();
        let r = resolve_period(&s, &p, &plan, &[], &c).unwrap();
        assert_eq!(r.summary.people.staff, 56, "{code}");
        monthly.push(monthly_departures(
            i64::from(r.summary.people.staff),
            i64::from(r.summary.people.turnover_bp),
        ));
        sum += i64::from(r.summary.people.departures);
        s = r.after;
    }
    let m = monthly[0];
    assert!(monthly.iter().all(|&x| x == m), "{monthly:?}");
    assert_eq!(m, 2);
    assert_eq!(sum, m * 28 / 30);
    assert_eq!(sum, 1);
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

// ---- Ojačanje H1, korak 4: u krugu koji nije cijeli mjesec mali razmak do cilja ne smije zapeti na 0

struct Gauge {
    name: &'static str,
    get: fn(&HotelState) -> i64,
    set: fn(&mut HotelState, i64),
    target: fn(&PeriodReport) -> i64,
}

const GAUGES: [Gauge; 3] = [
    Gauge {
        name: "moral",
        get: |s| i64::from(s.morale_pts),
        set: |s, v| s.morale_pts = i32::try_from(v).unwrap(),
        target: |r| i64::from(r.summary.people.morale_target_pts),
    },
    Gauge {
        name: "zadovoljstvo",
        get: |s| i64::from(s.satisfaction_pts),
        set: |s, v| s.satisfaction_pts = i32::try_from(v).unwrap(),
        target: |r| i64::from(r.summary.guests.satisfaction_target_pts),
    },
    Gauge {
        name: "ocjena",
        get: |s| i64::from(s.rating),
        set: |s, v| s.rating = u16::try_from(v).unwrap(),
        target: |r| i64::from(r.summary.guests.rating_target),
    },
];

fn resolve_default(c: &Catalog, s: &HotelState, p: &Period) -> PeriodReport {
    let plan = PeriodPlan::default_for(s, p, c).unwrap();
    resolve_period(s, p, &plan, &[], c).unwrap()
}

#[test]
fn weekly_small_gap_closes_one_unit_per_week_without_overshooting() {
    let c = cat();
    let base = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let t02 = period_by_code(PeriodKind::Week, "t02").unwrap();
    // Tjedni koraci 800 / 1 500 / 700 bp: razmak 3 (moral, zadovoljstvo) i 5 (ocjena) zaokruži se na 0.
    for (g, gap) in GAUGES.iter().zip([3_i64, 3, 5]) {
        let target = (g.target)(&resolve_default(&c, &base, &t02));
        for start in [target - gap, target + gap] {
            let mut s = base.clone();
            (g.set)(&mut s, start);
            for week in 0..gap {
                let r = resolve_default(&c, &s, &t02);
                assert_eq!((g.target)(&r), target, "{} cilj se pomaknuo", g.name);
                let (before, after) = ((g.get)(&s), (g.get)(&r.after));
                assert_eq!(
                    (target - after).abs(),
                    (target - before).abs() - 1,
                    "{} od {start}, tjedan {week}: {before} → {after}, cilj {target}",
                    g.name
                );
                (g.set)(&mut s, after);
            }
            assert_eq!((g.get)(&s), target, "{} od {start}", g.name);
            // Na cilju korak je 0 (ne titra oko cilja).
            let r = resolve_default(&c, &s, &t02);
            assert_eq!((g.get)(&r.after), target, "{} na cilju", g.name);
        }
    }
}

#[test]
fn monthly_small_gap_still_rounds_to_zero_as_in_v1() {
    let c = cat();
    let base = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let jun = month_period(Month::Jun);
    // Mjesečni koraci 3 000 (moral) i 2 500 (ocjena) bp: razmak 1 daje 0 kao u v1.
    for g in [&GAUGES[0], &GAUGES[2]] {
        let target = (g.target)(&resolve_default(&c, &base, &jun));
        let mut s = base.clone();
        (g.set)(&mut s, target - 1);
        let r = resolve_default(&c, &s, &jun);
        assert_eq!((g.target)(&r), target, "{}", g.name);
        assert_eq!((g.get)(&r.after), target - 1, "{}", g.name);
    }
}

#[test]
fn equipment_wear_and_load_of_t05_are_prorated_across_june_and_july() {
    let c = cat();
    let s = HotelState::from_profile(c.base.profile(Profile::C).unwrap()).unwrap();
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    let r = resolve_default(&c, &s, &t05);
    assert!(r.summary.people.load_hundredths > 180);
    // lipanj: dani 28–29 od 30; srpanj: dani 0–4 od 31; ⌊·⌋ kao `prorate_part` (i za negativne iznose).
    let split = |m: i64| (m - (m * 28).div_euclid(30)) + (m * 5).div_euclid(31);
    let line = |cause: Cause| {
        r.summary
            .lines
            .iter()
            .find(|l| l.var == Var::Equipment && l.cause == cause)
            .unwrap()
            .amount
    };
    assert_eq!(line(Cause::Baseline), split(-400));
    assert_eq!(line(Cause::Load), split(-100));
    assert_eq!(split(-100), -23);
}
