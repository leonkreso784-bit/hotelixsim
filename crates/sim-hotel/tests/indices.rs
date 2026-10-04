mod common;
use common::*;
use sim_hotel::*;

fn raw(g: i32, r: i32, z: i32, o: i32, f: i32) -> RawIndices {
    RawIndices { g, r, z, o, f }
}
fn five(v: [u16; 5]) -> IndexSet {
    IndexSet::from_hundredths(v).unwrap()
}

#[test]
fn reference_table_for_test_catalog() {
    let t = RefTable::compute(&test_catalog(), Category::Four).unwrap();
    assert_eq!(
        t.start,
        RefPoint {
            raw: raw(7_000, 5_000, 5_425, 7_000, 5_457),
            goppar_cents: 4_163
        }
    );
    assert_eq!(
        t.months.get(Month::Jun),
        RefPoint {
            raw: raw(6_874, 4_964, 5_425, 7_042, 6_396),
            goppar_cents: 4_163
        }
    );
    assert_eq!(
        t.months.get(Month::Jul),
        RefPoint {
            raw: raw(6_753, 4_893, 5_282, 6_862, 6_500),
            goppar_cents: 8_340
        }
    );
    assert_eq!(
        t.months.get(Month::Aug),
        RefPoint {
            raw: raw(6_591, 4_786, 4_939, 6_556, 6_500),
            goppar_cents: 10_364
        }
    );
    assert_eq!(
        t.months.get(Month::Sep),
        RefPoint {
            raw: raw(6_658, 4_714, 5_084, 7_068, 6_500),
            goppar_cents: 4_283
        }
    );
}

#[test]
fn starting_indices_of_profiles() {
    let cat = test_catalog();
    let refs = RefTable::compute(&cat, Category::Four).unwrap();
    let start = |p: ProfileSpec| {
        start_indices(&HotelState::from_profile(&p).unwrap(), &cat, &refs).unwrap()
    };
    assert_eq!(start(profile_c()), five([5_000; 5]));
    assert_eq!(
        start(profile_a()),
        five([5_800, 7_500, 3_245, 4_625, 3_351])
    );
    assert_eq!(
        start(profile_b()),
        five([5_000, 3_571, 5_405, 3_950, 6_620])
    );
}

#[test]
fn passive_reference_hotel_stays_at_fifty_all_season() {
    let cat = test_catalog();
    let refs = RefTable::compute(&cat, Category::Four).unwrap();
    let mut s = state_c();
    for m in Month::SEASON {
        let r = resolve_month(
            &s,
            m,
            &MonthPlan::default_for(&s, m, &cat).unwrap(),
            &[],
            &cat,
        )
        .unwrap();
        assert_eq!(
            month_indices(&r, &cat, &refs).unwrap(),
            five([5_000; 5]),
            "{m:?}"
        );
        s = r.after;
    }
    assert_eq!(s.cash_cents, 129_984_850);
}

#[test]
fn june_variant_indices_and_progress() {
    let cat = test_catalog();
    let refs = RefTable::compute(&cat, Category::Four).unwrap();
    let s = state_c();
    let plan = MonthPlan {
        price_cents: 15_000,
        marketing_cents: 1_500_000,
        seasonal_target: 30,
        wage_pct: 105,
        training: Training::Basic,
        maintenance: Maintenance::Intensive,
        renovation: false,
        reply_reviews: true,
    };
    let r = resolve_month(&s, Month::Jun, &plan, &[], &cat).unwrap();
    let idx = month_indices(&r, &cat, &refs).unwrap();
    assert_eq!(idx, five([4_883, 5_107, 5_405, 4_988, 3_390]));
    assert_eq!(progress_pts(&five([5_000; 5]), &idx), Ok(-491));
}

#[test]
fn gain_follows_room_for_growth() {
    assert_eq!(gain_bp(5_000, 6_000), Ok(2_000));
    assert_eq!(gain_bp(5_000, 4_000), Ok(-2_000));
    assert_eq!(gain_bp(10_000, 10_000), Ok(0));
    assert_eq!(gain_bp(0, 100), Ok(100));
    assert_eq!(gain_bp(7_500, 8_000), Ok(2_000));
    assert_eq!(gain_bp(3_500, 4_000), Ok(769));
    assert_eq!(gain_bp(0, 0), Ok(0));
    assert_eq!(
        progress_pts(
            &five([5_000; 5]),
            &five([6_000, 4_000, 5_000, 5_500, 5_100])
        ),
        Ok(240)
    );
}

#[test]
fn liquidity_is_zero_when_cash_negative() {
    let cat = test_catalog();
    let mut s = state_c();
    s.cash_cents = -1;
    let raw = raw_indices(&s, 4_163, 11_000_000, 4_163, &cat).unwrap();
    assert_eq!(raw.f, 3_500); // 0,7 × 50 + 0,3 × 0
}
