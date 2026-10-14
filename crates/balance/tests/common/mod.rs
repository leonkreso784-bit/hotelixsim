//! Zajednički pomoćnici testova balansa v2 (A1).
#![allow(dead_code)]
use balance::v2::ReportV2;
use content::{Snapshot, repo_dir};
use sim_hotel_v2::Strategy;

pub fn content_v2() -> content::Content {
    Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap()
}

/// Svih šest mjera robota izvan pojasa; brane modela netaknute.
pub fn spoil_robot_measures(r: &mut ReportV2) {
    r.profile_medians[0].1 += 10_000; // §11.3: jedan profil daleko ispred
    r.week_vs_month[0].3 = r.week_vs_month[0].2 + 1_000; // tjedno − mjesečno = 10 bodova
    r.caring_seasonal_vs_plan_bp[0].1 = 0; // Brižni ne drži sezonce
    let sr = r.situations.as_mut().expect("repo ima predloške");
    for s in &mut sr.shares {
        match s.strategy {
            Strategy::Caring => s.share_bp = 0, // udio izvan pojasa
            Strategy::Saver => s.delta = 100,   // situacije Štediši pomažu
            _ => {}
        }
    }
}
