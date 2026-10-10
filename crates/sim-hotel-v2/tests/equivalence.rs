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
