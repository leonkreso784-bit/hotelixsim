//! B2b: balans prima broj situacija kao ulaz (`--situations L,M,H`), a brana J-3 mjeri prema njemu.
mod common;

use balance::v2::{OptionsV2, failures_v2, parse_situations, run_v2};
use common::content_v2;
use sim_hotel_v2::PeriodKind;
use sim_hotel_v2::situations::SituationCounts;

#[test]
fn parse_situations_accepts_only_valid_counts() {
    assert_eq!(parse_situations("4,5,6"), SituationCounts::new(4, 5, 6));
    assert!(parse_situations("4,5,6").is_some());
    for bad in ["2,4,5", "3,5,4", "4,5", "a,b,c"] {
        assert_eq!(parse_situations(bad), None, "{bad}");
    }
}

#[test]
fn count_band_follows_the_setting() {
    let three = SituationCounts::new(3, 3, 3);
    let mut r = run_v2(
        &content_v2(),
        &OptionsV2 {
            seeds: 3,
            kind: PeriodKind::Month,
            situations: three,
        },
    )
    .unwrap();
    let sr = r.situations.as_mut().expect("sadržaj ima predloške");
    assert_eq!(sr.counts.levels(), (3, 3, 3));
    assert!(sr.draws.max <= 3, "najviše {}", sr.draws.max);
    // Krug izvan pojasa 3–3 ruši branu J-3, a poruka nosi postavljeni pojas.
    sr.draws.max = 4;
    let f = failures_v2(&r);
    assert!(
        f.iter()
            .any(|m| m.contains("J-3 broj situacija") && m.contains("dopušteno 3–3")),
        "{f:?}"
    );
}
