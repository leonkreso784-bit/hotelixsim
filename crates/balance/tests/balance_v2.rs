mod common;

use balance::v2::{OptionsV2, failures_v2, render_v2, run_v2};
use common::content_v2;
use content::{Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;

#[test]
fn v2_balance_runs_for_both_kinds_and_is_reproducible() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let r = run_v2(&content_v2(), &OptionsV2 { seeds: 3, kind }).unwrap();
        assert!(r.reproducible);
        assert_eq!(r.starts.len(), 3);
        assert!(render_v2(&r).contains("Marže odjela"));
        assert!(render_v2(&r).contains("Ostali prihodi C / prihod soba"));
        // Sadržaj nudi obnovu v2: probe obnove (obnova-v2-spec §5.2) i F na kraju sezone.
        let text = render_v2(&r);
        for probe in [
            "obnova svega najvećim iznosom u lipnju",
            "obnova Smještaja pola u lipnju, pola u srpnju",
            "obnova zgrade najvećim iznosom u lipnju",
            "obnova Smještaja u zadnjem krugu",
            "F na kraju sezone",
        ] {
            assert!(text.contains(probe), "{probe}");
        }
        assert_eq!(r.end_finance.len(), 12);
        assert!(r.reference_other_revenue_share_bp > 0);
        let _ = failures_v2(&r); // u ovom tasku smiju postojati
    }
}

#[test]
fn degenerate_v2_monthly_balance_matches_v1_numbers() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let c = s.degenerate_v2().build().unwrap();
    let r2 = run_v2(
        &c,
        &OptionsV2 {
            seeds: 5,
            kind: PeriodKind::Month,
        },
    )
    .unwrap();
    let r1 = balance::run(&s.build().unwrap(), &balance::Options { seeds: 5 }).unwrap();
    assert_eq!(r2.starts, r1.starts);
    // Degenerirani katalog nema obnovu v2, pa ni proba obnove.
    assert!(!render_v2(&r2).contains("obnova svega"));
    assert_eq!(r2.profile_medians, r1.profile_medians);
    // Degenerirani katalog: jedan odjel `ostalo` = ShareOfRooms 3 000 bp (v1 `other_revenue_bp`) → omjer 30 % do
    // zaokruživanja po krugu.
    assert!(
        (r2.reference_other_revenue_share_bp - 3_000).abs() <= 1,
        "{}",
        r2.reference_other_revenue_share_bp
    );
}

/// A1: mjere robota ne ruše `--check`, ni kad su sve izvan pojasa.
#[test]
fn robot_measures_do_not_fail_check() {
    let mut r = run_v2(
        &content_v2(),
        &OptionsV2 {
            seeds: 3,
            kind: PeriodKind::Week,
        },
    )
    .unwrap();
    common::spoil_robot_measures(&mut r);
    let f = failures_v2(&r);
    for robot in [
        "pravednost",
        "tjedno ≈ mjesečno",
        "brižni robot",
        "udio situacija",
        "štediša",
    ] {
        assert!(!f.iter().any(|x| x.contains(robot)), "{robot}: {f:?}");
    }
}
