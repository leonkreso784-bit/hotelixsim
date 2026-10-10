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
            raw: raw(6_753, 4_893, 5_282, 6_862, 8_394),
            goppar_cents: 8_340
        }
    );
    assert_eq!(
        t.months.get(Month::Aug),
        RefPoint {
            raw: raw(6_591, 4_786, 4_939, 6_556, 10_876),
            goppar_cents: 10_364
        }
    );
    assert_eq!(
        t.months.get(Month::Sep),
        RefPoint {
            raw: raw(6_658, 4_714, 5_084, 7_068, 11_868),
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
        five([5_000, 3_571, 5_405, 3_950, 8_195])
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

#[test]
fn liquidity_has_no_upper_bound_in_raw_f() {
    let cat = test_catalog();
    let mut s = state_c();
    s.cash_cents = 300_000_000;
    let raw = raw_indices(&s, 4_163, 11_000_000, 4_163, &cat).unwrap();
    assert_eq!(raw.f, 23_065); // likvidnost 65 217, ADR-014
}

fn raw_of(r: &RawIndices, i: Index) -> i32 {
    match i {
        Index::Guests => r.g,
        Index::Reviews => r.r,
        Index::Staff => r.z,
        Index::Operations => r.o,
        Index::Finance => r.f,
    }
}

/// Čuvar `index_drivers`: pomak svake varijable mijenja točno one indekse kojima je upisana kao pokretač.
#[test]
fn index_drivers_match_raw_formula() {
    let cat = test_catalog();
    let base = state_c();
    let (goppar, labor) = (4_163, 11_000_000);
    let start = raw_indices(&base, goppar, labor, goppar, &cat).unwrap();
    let all = [
        Var::Sold,
        Var::Revenue,
        Var::Costs,
        Var::Cash,
        Var::Staff,
        Var::Morale,
        Var::Experience,
        Var::Equipment,
        Var::Satisfaction,
        Var::Rating,
    ];
    for v in all {
        let mut s = base.clone();
        let (mut g, mut l) = (goppar, labor);
        match v {
            // Prodaja, prihod i troškovi ulaze kroz GOPPAR; troškovi i kroz plaće u likvidnosti.
            Var::Sold | Var::Revenue => g += 500,
            Var::Costs => {
                g -= 500;
                l += 1_000_000;
            }
            Var::Cash => s.cash_cents += 1_000_000,
            Var::Staff => s.turnover_bp += 400,
            Var::Morale => s.morale_pts += 500,
            Var::Experience => s.experience_pts += 500,
            Var::Equipment => s.equipment_pts += 500,
            Var::Satisfaction => s.satisfaction_pts += 500,
            Var::Rating => s.rating += 20,
        }
        let moved = raw_indices(&s, g, l, goppar, &cat).unwrap();
        for i in Index::ALL {
            let changed = raw_of(&moved, i) != raw_of(&start, i);
            assert_eq!(
                changed,
                index_drivers(i).contains(&v),
                "{v:?} → {i:?}: formula i index_drivers se ne slažu"
            );
        }
    }
    // Svaka varijabla pokreće barem jedan indeks.
    for v in all {
        assert!(
            Index::ALL.iter().any(|&i| index_drivers(i).contains(&v)),
            "{v:?}"
        );
    }
}
