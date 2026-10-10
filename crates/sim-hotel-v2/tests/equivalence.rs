mod common;

use sim_hotel_v2::{Profile, Strategy};

#[test]
fn v2_copy_equals_v1_for_every_profile_strategy_and_seed() {
    let base = common::test_catalog();
    let v2 = sim_hotel_v2::degenerate(&base);
    let situations = common::test_situations();
    for p in Profile::ALL {
        let category = base.profile(p).unwrap().category;
        let r1 = sim_hotel::RefTable::compute(&base, category).unwrap();
        let r2 = sim_hotel_v2::RefTable::compute(&v2, category).unwrap();
        for s in Strategy::ALL {
            for seed in [7_u64, 42] {
                for sits in [&[][..], &situations[..]] {
                    let a = sim_hotel::run_season(p, s, seed, &base, sits, &r1).unwrap();
                    let b = sim_hotel_v2::run_season(p, s, seed, &v2, sits, &r2).unwrap();
                    assert_eq!(a.start, b.start, "{p:?} {s:?} {seed}");
                    assert_eq!(a.progress_pts, b.progress_pts, "{p:?} {s:?} {seed}");
                    assert_eq!(a.months.len(), b.periods.len());
                    for ((ra, ia), (rb, ib)) in a.months.iter().zip(&b.periods) {
                        assert_eq!(ra, &rb.summary, "{p:?} {s:?} {seed} {:?}", ra.month);
                        assert_eq!(ia, ib);
                    }
                }
            }
        }
    }
}

/// Plan koji koristi obnovu, puni trening, intenzivno održavanje i gura blagajnu u minus (kamata): pokriva grane
/// koje zadani roboti ne dotiču. Isti plan za v1 i v2, mjesec po mjesec.
#[test]
fn v2_copy_equals_v1_with_renovation_full_training_and_negative_cash() {
    use sim_hotel::{Maintenance, Month, Training};
    let base = common::test_catalog();
    let v2 = sim_hotel_v2::degenerate(&base);
    let mut saw_interest = false;
    let mut saw_capex = false;
    for p in [Profile::A, Profile::B, Profile::C] {
        let category = base.profile(p).unwrap().category;
        let r1 = sim_hotel::RefTable::compute(&base, category).unwrap();
        let r2 = sim_hotel_v2::RefTable::compute(&v2, category).unwrap();
        for sits in [&[][..], &common::test_situations()[..]] {
            let a = sim_hotel::run_season_with(
                p,
                &base,
                sits,
                &r1,
                &mut |s, m| {
                    Ok(sim_hotel::MonthPlan {
                        price_cents: base.category(s.category)?.price_ref_cents.get(m) * 60 / 100,
                        marketing_cents: base.actions.marketing_max_cents,
                        seasonal_target: (s.seasonal + 10).min(base.actions.seasonal_max),
                        wage_pct: base.actions.wage_max_pct,
                        training: Training::Full,
                        maintenance: Maintenance::Intensive,
                        renovation: m == Month::Jun,
                        reply_reviews: true,
                    })
                },
                &mut |_| Ok(0),
            )
            .unwrap();
            let b = sim_hotel_v2::run_season_with(
                p,
                &v2,
                sits,
                &r2,
                &mut |s, m| {
                    Ok(sim_hotel_v2::PeriodPlan {
                        price_cents: v2.base.category(s.category)?.price_ref_cents.get(m) * 60
                            / 100,
                        marketing_cents: v2.base.actions.marketing_max_cents,
                        seasonal_target: (s.seasonal + 10).min(v2.base.actions.seasonal_max),
                        wage_pct: v2.base.actions.wage_max_pct,
                        training: Training::Full,
                        maintenance: Maintenance::Intensive,
                        renovation: m == Month::Jun,
                        reply_reviews: true,
                    })
                },
                &mut |_| Ok(0),
            )
            .unwrap();
            assert_eq!(a.start, b.start, "{p:?}");
            assert_eq!(a.progress_pts, b.progress_pts, "{p:?}");
            assert_eq!(a.months.len(), b.periods.len());
            for ((ra, ia), (rb, ib)) in a.months.iter().zip(&b.periods) {
                assert_eq!(ra, &rb.summary, "{p:?} {:?}", ra.month);
                assert_eq!(ia, ib);
                saw_interest |= ra.money.interest_cents > 0;
                saw_capex |= ra.money.capex_cents > 0;
            }
        }
    }
    assert!(saw_capex, "obnova nije provedena");
    assert!(saw_interest, "blagajna nikad nije otišla u minus");
}
