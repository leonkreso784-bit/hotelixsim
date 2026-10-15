//! A1: mjere robota (ispis) odvojene od brana modela (`--check`).
mod common;

use balance::v2::{OptionsV2, failures_v2, render_v2, robot_measures_v2, run_v2};
use common::{content_v2, spoil_robot_measures};
use sim_hotel_v2::{PeriodKind, Strategy};

const ROBOTS: [&str; 5] = [
    "pravednost",
    "tjedno ≈ mjesečno",
    "brižni robot",
    "udio situacija",
    "štediša",
];

fn week_report() -> balance::v2::ReportV2 {
    run_v2(
        &content_v2(),
        &OptionsV2 {
            seeds: 3,
            kind: PeriodKind::Week,
            situations: None,
        },
    )
    .unwrap()
}

#[test]
fn robot_measures_are_listed_and_printed() {
    let mut r = week_report();
    spoil_robot_measures(&mut r);
    let m = robot_measures_v2(&r);
    for robot in ROBOTS {
        assert!(m.iter().any(|x| x.contains(robot)), "{robot}: {m:?}");
    }
    let text = render_v2(&r);
    assert!(text.contains("== Mjere robota (ispis, ne ruše --check) =="));
    for msg in &m {
        assert!(text.contains(msg.as_str()), "{msg}");
    }
}

#[test]
fn model_gates_still_fail_check() {
    let mut r = week_report();
    r.reference_season_gop_margin_bp = 1_000;
    r.reproducible = false;
    let sr = r.situations.as_mut().expect("repo ima predloške");
    sr.group_limit = Some(1);
    sr.draws.groups_max = 2;
    let f = failures_v2(&r);
    let m = robot_measures_v2(&r);
    for gate in ["§11.2 ekonomija", "J-3 grupe", "§11.5 ponovljivost"] {
        assert!(f.iter().any(|x| x.contains(gate)), "{gate}: {f:?}");
        assert!(!m.iter().any(|x| x.contains(gate)), "{gate}: {m:?}");
    }
}

/// Pregled J-3 (minor 1): situacije moraju Brižnom pomagati (Δ > 0); od A1 to je mjera robota, ne brana modela.
#[test]
fn caring_negative_delta_is_a_robot_measure() {
    let mut r = week_report();
    let sr = r.situations.as_mut().expect("repo ima predloške");
    let s = sr
        .shares
        .iter_mut()
        .find(|s| s.strategy == Strategy::Caring)
        .unwrap();
    // Δ = −2,00, bez situacija 6,00 → udio 25 % (u pojasu), ali situacije smanjuju napredak.
    s.delta = -200;
    s.without = 600;
    s.with = 400;
    s.share_bp = 2_500;
    let code = s.profile.code();
    let m = robot_measures_v2(&r);
    assert!(
        m.iter()
            .any(|x| x.contains("Brižnom smanjuju napredak") && x.contains(code)),
        "{m:?}"
    );
    let f = failures_v2(&r);
    assert!(
        !f.iter().any(|x| x.contains("Brižnom smanjuju napredak")),
        "{f:?}"
    );
}

#[test]
fn in_band_robot_measures_print_all_clear() {
    let mut r = week_report();
    for p in &mut r.profile_medians {
        p.1 = 0;
    }
    for w in &mut r.week_vs_month {
        w.3 = w.2;
    }
    for c in &mut r.caring_seasonal_vs_plan_bp {
        c.1 = 10_000;
    }
    let sr = r.situations.as_mut().expect("repo ima predloške");
    for s in &mut sr.shares {
        match s.strategy {
            Strategy::Caring => {
                s.share_bp = 3_000;
                s.delta = 100;
            }
            Strategy::Saver => s.delta = -100,
            _ => {}
        }
    }
    assert_eq!(robot_measures_v2(&r), Vec::<String>::new());
    assert!(render_v2(&r).contains("  sve mjere robota u pojasu"));
}
