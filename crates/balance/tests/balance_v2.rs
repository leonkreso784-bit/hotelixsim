use balance::v2::{OptionsV2, failures_v2, render_v2, run_v2};
use content::{Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;

fn content_v2() -> content::Content {
    Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap()
}

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

/// Pregled J-3 (minor 1): udio situacija u pojasu nije dovoljan — situacije Brižnom moraju pomagati (Δ > 0).
#[test]
fn caring_share_in_band_with_negative_delta_fails() {
    let mut r = run_v2(
        &content_v2(),
        &OptionsV2 {
            seeds: 3,
            kind: PeriodKind::Week,
        },
    )
    .unwrap();
    let sr = r.situations.as_mut().expect("repo ima predloške");
    let s = sr
        .shares
        .iter_mut()
        .find(|s| s.strategy == sim_hotel_v2::Strategy::Caring)
        .unwrap();
    // Δ = −2,00, bez situacija 6,00 → udio 25 % (u pojasu), ali situacije smanjuju napredak.
    s.delta = -200;
    s.without = 600;
    s.with = 400;
    s.share_bp = 2_500;
    let code = s.profile.code();
    let f = failures_v2(&r);
    assert!(
        f.iter()
            .any(|x| x.contains("Brižnom smanjuju napredak") && x.contains(code)),
        "{f:?}"
    );
}
