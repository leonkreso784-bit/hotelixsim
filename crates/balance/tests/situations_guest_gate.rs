//! L1-13 dio 4 (pregled dijela 3, presuda 1): brana „J-3 gosti" na STVARNIM sezonama (roboti, povijest, veće i
//! kolektivne situacije), tjedno i mjesečno. Uz uključen ključ `guest_extra_slots` u svakom krugu najviše
//! `low − (sidra + kolektivna + veća)` (ne manje od 0) običnih izvučenih situacija smije biti bez gostiju; ostala
//! mjesta pune samo situacije s gostima (`SituationSpec::guest_related`). Točna brana, bez praga.
//!
//! Produces (balans, `balance::v2`): `DrawStats.non_guest_extra: u64` = zbroj, po svim krugovima sezona balansa, običnih
//! (ne sidro, ne kolektivna `slot` 0, ne veća `major`) izvučenih situacija bez gostiju iznad
//! `max(0, low − (sidra + kolektivna + veća))` u svom krugu; broji se UVIJEK (i uz isključen ključ, kao mjera);
//! `SituationReport.guest_extra_slots: bool` (ključ sadržaja sezone); brana u `failures_v2` samo uz uključen ključ:
//! `J-3 gosti: {n} negostinjskih situacija iznad low − (sidra + kolektivna + veća) (cilj 0)`.
//! Uz to (pregled dijela 3, M2): razinu popunjenosti balans uzima iz jezgre (`SituationCounts::level`).

use balance::v2::{OptionsV2, ReportV2, failures_v2, robot_measures_v2, run_v2};
use content::{Content, Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;

fn content_with_flag(on: bool) -> Content {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    v["v2"]["situation_templates"]["guest_extra_slots"] = on.into();
    Snapshot::from_value(&v).unwrap().build().unwrap()
}

fn report(c: &Content, kind: PeriodKind) -> ReportV2 {
    run_v2(
        c,
        &OptionsV2 {
            seeds: 3,
            kind,
            situations: None,
        },
    )
    .unwrap()
}

fn guest_gate(r: &ReportV2) -> Vec<String> {
    failures_v2(r)
        .into_iter()
        .filter(|x| x.starts_with("J-3 gosti"))
        .collect()
}

#[test]
fn real_seasons_fill_extra_slots_only_with_guests() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let on = report(&content_with_flag(true), kind);
        let sr = on.situations.as_ref().unwrap();
        assert!(sr.guest_extra_slots, "{kind:?}");
        assert_eq!(
            sr.draws.non_guest_extra, 0,
            "{kind:?}: uz ključ nijedna situacija bez gostiju iznad prvih mjesta"
        );
        assert_eq!(guest_gate(&on), Vec::<String>::new(), "{kind:?}");
        // Brojač nije tautologija: bez ključa dodatna mjesta pune i situacije bez gostiju, a brane tada nema.
        let off = report(&content_with_flag(false), kind);
        let sr = off.situations.as_ref().unwrap();
        assert!(!sr.guest_extra_slots, "{kind:?}");
        assert!(
            sr.draws.non_guest_extra > 0,
            "{kind:?}: bez ključa brojač mora vidjeti situacije bez gostiju na dodatnim mjestima"
        );
        assert_eq!(
            guest_gate(&off),
            Vec::<String>::new(),
            "{kind:?}: bez ključa nema brane"
        );
    }
}

#[test]
fn guest_gate_fails_check_only_with_flag() {
    let mut r = report(&content_with_flag(true), PeriodKind::Week);
    let sr = r.situations.as_mut().unwrap();
    sr.draws.non_guest_extra = 2;
    assert_eq!(
        guest_gate(&r),
        vec![
            "J-3 gosti: 2 negostinjskih situacija iznad low − (sidra + kolektivna + veća) (cilj 0)"
                .to_string()
        ]
    );
    assert!(
        !robot_measures_v2(&r)
            .iter()
            .any(|x| x.contains("J-3 gosti")),
        "brana modela, ne mjera robota"
    );
    r.situations.as_mut().unwrap().guest_extra_slots = false;
    assert_eq!(
        guest_gate(&r),
        Vec::<String>::new(),
        "isključen ključ: nema brane"
    );
}

/// Pregled dijela 3 (M2): razina popunjenosti u balansu dolazi iz jezgre (`SituationCounts::level`), a ne iz
/// `WEEK.for_occupancy(bp) − MIN` (vrijedi samo dok je tjedni broj 3 / 4 / 5).
#[test]
fn balance_takes_occupancy_level_from_core() {
    let src = include_str!("../src/v2_situations.rs");
    assert!(
        src.contains("SituationCounts::level("),
        "v2_situations.rs: razina iz SituationCounts::level"
    );
    assert!(
        !src.contains(".for_occupancy("),
        "v2_situations.rs: razina se ne izvodi iz broja situacija"
    );
}
