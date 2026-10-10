use sim_hotel_v2::calendar::*;
use sim_hotel_v2::{Month, MonthTable};

#[test]
fn weekly_season_is_sixteen_weeks_and_ten_days() {
    let w = season_periods(PeriodKind::Week);
    assert_eq!(w.len(), 17);
    assert!(w[..16].iter().all(|p| p.days == 7));
    assert_eq!(w[16].days, 10);
    assert_eq!(w[16].start_day, 112);
    assert_eq!(w[16].first_date(), (21, Month::Sep));
    assert_eq!(w[16].last_date(), (30, Month::Sep));
    assert_eq!(
        w.iter().map(|p| u16::from(p.days)).sum::<u16>(),
        SEASON_DAYS
    );
    let codes: Vec<String> = w.iter().map(Period::code).collect();
    assert_eq!(codes.first().map(String::as_str), Some("t01"));
    assert_eq!(codes.last().map(String::as_str), Some("t17"));
}

#[test]
fn monthly_periods_match_month_days() {
    let m = season_periods(PeriodKind::Month);
    let days: Vec<u8> = m.iter().map(|p| p.days).collect();
    assert_eq!(days, [30, 31, 31, 30]);
    let codes: Vec<String> = m.iter().map(Period::code).collect();
    assert_eq!(codes, ["jun", "jul", "aug", "sep"]);
    assert!(m.iter().all(Period::ends_month));
    assert_eq!(month_period(Month::Aug), m[2]);
}

#[test]
fn week_five_spans_june_and_july() {
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    assert_eq!(t05.first_date(), (29, Month::Jun));
    assert_eq!(t05.last_date(), (5, Month::Jul));
    assert_eq!(
        t05.parts(),
        vec![
            MonthPart {
                month: Month::Jun,
                from_day: 28,
                days: 2
            },
            MonthPart {
                month: Month::Jul,
                from_day: 0,
                days: 5
            },
        ]
    );
    assert_eq!(t05.month_days_at_start(), 30);
    assert!(!t05.ends_month());
}

#[test]
fn prorate_is_exact_within_each_month() {
    for kind in PeriodKind::ALL {
        for monthly in [15_000_000_i64, 220_000, 300, -400, -100, 7, 1, 0] {
            for m in Month::SEASON {
                let total: i64 = season_periods(kind)
                    .iter()
                    .flat_map(Period::parts)
                    .filter(|part| part.month == m)
                    .map(|part| prorate_part(monthly, &part).unwrap())
                    .sum();
                assert_eq!(total, monthly, "{kind:?} {m:?} {monthly}");
            }
        }
    }
}

#[test]
fn prorate_of_full_month_is_the_monthly_amount() {
    for m in Month::SEASON {
        assert_eq!(prorate(1_234_567, &month_period(m)).unwrap(), 1_234_567);
    }
}

#[test]
fn average_is_day_weighted_and_exact_for_full_month() {
    let t = MonthTable([13_500_i64, 16_500, 17_500, 13_500]);
    assert_eq!(avg_i64(&t, &month_period(Month::Jul)).unwrap(), 16_500);
    let t05 = period_by_code(PeriodKind::Week, "t05").unwrap();
    // (2·13 500 + 5·16 500) / 7 = 15 642,86 → 15 643
    assert_eq!(avg_i64(&t, &t05).unwrap(), 15_643);
    let bp = MonthTable([7_200_u16, 7_800, 10_600, 7_300]);
    // (2·7 200 + 5·7 800) / 7 = 7 628,57 → 7 629
    assert_eq!(avg_u16(&bp, &t05).unwrap(), 7_629);
}

#[test]
fn first_period_of_each_month_contains_its_first_day() {
    let codes: Vec<String> = Month::SEASON
        .iter()
        .map(|&m| first_period_of_month(PeriodKind::Week, m).code())
        .collect();
    assert_eq!(codes, ["t01", "t05", "t09", "t14"]);
    assert_eq!(
        first_period_of_month(PeriodKind::Month, Month::Aug).code(),
        "aug"
    );
}

#[test]
fn month_ends_fall_in_t05_t09_t14_and_t17() {
    let ends: Vec<(String, Month)> = season_periods(PeriodKind::Week)
        .iter()
        .filter_map(|p| p.month_ending_in().map(|m| (p.code(), m)))
        .collect();
    assert_eq!(
        ends,
        [
            ("t05".into(), Month::Jun),
            ("t09".into(), Month::Jul),
            ("t14".into(), Month::Aug),
            ("t17".into(), Month::Sep)
        ]
    );
    assert!(
        season_periods(PeriodKind::Month)
            .iter()
            .all(|p| p.month_ending_in().is_some())
    );
}

#[test]
fn codes_parse_back_and_unknown_codes_do_not() {
    for kind in PeriodKind::ALL {
        for p in season_periods(kind) {
            assert_eq!(period_by_code(kind, &p.code()), Some(p));
        }
    }
    assert_eq!(period_by_code(PeriodKind::Week, "t18"), None);
    assert_eq!(period_by_code(PeriodKind::Week, "jun"), None);
    assert_eq!(period_by_code(PeriodKind::Month, "t01"), None);
}

#[test]
fn per_month_equivalent_scales_by_start_month_days() {
    let t02 = period_by_code(PeriodKind::Week, "t02").unwrap();
    assert_eq!(per_month_equiv(700, &t02).unwrap(), 3_000); // 700 · 30 / 7
    assert_eq!(
        per_month_equiv(700, &month_period(Month::Jul)).unwrap(),
        700
    );
}

#[test]
fn only_monthly_periods_are_whole_months() {
    assert!(
        season_periods(PeriodKind::Month)
            .iter()
            .all(Period::is_whole_month)
    );
    assert!(
        !season_periods(PeriodKind::Week)
            .iter()
            .any(Period::is_whole_month)
    );
}
