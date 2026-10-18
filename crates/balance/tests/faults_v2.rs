//! L1-13 dio 4 (A2, test čuvara 5): blok balansa „Kvarovi prostora" za Leonovu tablicu. Zadani plan, put „najbolji"
//! (vlastiti odgovor), a održavanje SVIH odjela postavljeno kao u probi `MaintenanceAll`: pojačano / zadano (plan
//! kakav jest) / odgođeno. Po profilu medijan po sjemenu: broj kvarova prostora u sezoni (izvučeni primjerci
//! predložaka `TemplateSet::space_fault`), stanje Smještaja na kraju sezone i napredak na kraju sezone.
//!
//! Produces (balans, `balance::v2`): `SituationReport.faults: FaultReport`;
//! `FaultReport { kind, rows: Vec<FaultRow> }`, redom profili A, B, C;
//! `FaultRow { profile, faults: [i32; 3], rooms_condition: [i32; 3], progress: [i32; 3] }`, svaki niz redom
//! pojačano / zadano / odgođeno; `faults` broj u sezoni, `rooms_condition` stanje Smještaja (0..=10 000, 100 = 1 bod),
//! `progress` napredak (stotinke boda). Ispis (`render_v2`) unutar odjeljka situacija, prije „== Ponovljivost":
//! ```text
//! == Kvarovi prostora (A2): zadani plan, najbolji odgovor; održavanje svih odjela pojačano / zadano / odgođeno ==
//!   kvarova po sezoni, medijan: A 2 / 4 / 9   B … C …
//!   stanje Smještaja na kraju, medijan: A 71,0 / 64,5 / 48,2 …
//!   napredak na kraju, medijan: A 12.34 / … / …
//! ```
//! (broj kvarova cijeli, stanje s jednom decimalom i zarezom, napredak u stotinkama s točkom kao ostatak ispisa).
mod common;

use balance::v2::{FaultReport, FaultRow, OptionsV2, ReportV2, render_v2, run_v2};
use common::content_v2;
use sim_hotel_v2::{PeriodKind, Profile};

const HEADER: &str = "== Kvarovi prostora (A2): zadani plan, najbolji odgovor; održavanje svih odjela pojačano / zadano / odgođeno ==";

fn report(kind: PeriodKind) -> ReportV2 {
    run_v2(
        &content_v2(),
        &OptionsV2 {
            seeds: 3,
            kind,
            situations: None,
        },
    )
    .unwrap()
}

fn faults(r: &ReportV2) -> &FaultReport {
    &r.situations.as_ref().expect("repo ima predloške").faults
}

/// Retci ispisa „<uvod>: A a / b / c   B … C …" → tri trojke brojeva (zarez kao decimalna točka).
fn triples(line: &str, intro: &str) -> Vec<(String, [f64; 3])> {
    let rest = line
        .strip_prefix(&format!("  {intro}:"))
        .unwrap_or_else(|| panic!("redak počinje s „  {intro}:\u{201d}: {line}"));
    let t: Vec<&str> = rest.split_whitespace().collect();
    assert_eq!(t.len(), 18, "tri profila × (oznaka, a, /, b, /, c): {line}");
    t.chunks(6)
        .map(|c| {
            assert_eq!((c[2], c[4]), ("/", "/"), "{line}");
            let num = |s: &str| -> f64 {
                s.replace(',', ".")
                    .parse()
                    .unwrap_or_else(|_| panic!("broj {s} u {line}"))
            };
            (c[0].to_string(), [num(c[1]), num(c[3]), num(c[5])])
        })
        .collect()
}

#[test]
fn fault_report_prints_three_maintenance_levels() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let r = report(kind);
        let f = faults(&r);
        assert_eq!(f.kind, kind);
        let profiles: Vec<Profile> = f.rows.iter().map(|x| x.profile).collect();
        assert_eq!(profiles, Profile::ALL.to_vec(), "{kind:?}");
        let text = render_v2(&r);
        let lines: Vec<&str> = text.lines().collect();
        let at = lines
            .iter()
            .position(|l| *l == HEADER)
            .unwrap_or_else(|| panic!("{kind:?}: nema bloka A2: {text}"));
        let b3 = lines
            .iter()
            .position(|l| l.starts_with("== Situacije na zadanom planu (B3)"))
            .unwrap();
        let repro = lines
            .iter()
            .position(|l| l.starts_with("== Ponovljivost"))
            .unwrap();
        assert!(
            b3 < at && at < repro,
            "{kind:?}: blok A2 je iza B3, prije ponovljivosti"
        );
        let want = |row: &FaultRow, k: usize| -> [f64; 3] {
            match k {
                0 => row.faults.map(f64::from),
                1 => row.rooms_condition.map(|x| f64::from(x) / 100.0),
                _ => row.progress.map(|x| f64::from(x) / 100.0),
            }
        };
        for (k, (intro, tol)) in [
            ("kvarova po sezoni, medijan", 0.0),
            ("stanje Smještaja na kraju, medijan", 0.05),
            ("napredak na kraju, medijan", 0.005),
        ]
        .into_iter()
        .enumerate()
        {
            let got = triples(lines[at + 1 + k], intro);
            for ((code, vals), row) in got.iter().zip(&f.rows) {
                assert_eq!(code, row.profile.code(), "{kind:?} {intro}");
                for (g, w) in vals.iter().zip(want(row, k)) {
                    assert!(
                        (g - w).abs() <= tol + 1e-9,
                        "{kind:?} {intro} {code}: ispis {g}, izvještaj {w}"
                    );
                }
            }
        }
        // Kvarovi su prirodni brojevi, stanje je u 0..=100 bodova.
        for row in &f.rows {
            assert!(row.faults.iter().all(|x| *x >= 0), "{kind:?} {row:?}");
            assert!(
                row.rooms_condition.iter().all(|x| (0..=10_000).contains(x)),
                "{kind:?} {row:?}"
            );
        }
    }
}

/// Dobro održan hotel ima manje kvarova (A2 „pravedno"): pojačano ≤ zadano ≤ odgođeno po medijanu na profilima A i C,
/// tjedno i mjesečno (3 sjemena), i u zbroju sva tri profila (B14); održavanje stvarno mijenja stanje Smještaja (isti
/// poredak, obrnut) na svakom profilu, a odgođeno ukupno ima strogo više kvarova od pojačanog (inače blok ne mjeri ništa).
/// Runda 2: hotel B je od početka zapušten i cijelu sezonu ispod pragova pojave (dijagnoza 18.10., t. 1b), pa mu broj
/// kvarova određuje samo izvlačenje (izmjereno uz K1: 14 / 13 / 13) — poredak po hotelu za B nije zahtjev dok Leon ne
/// odluči o pragovima ili početku B.
#[test]
fn well_kept_hotel_has_fewer_faults() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let r = report(kind);
        let f = faults(&r);
        for row in &f.rows {
            let [up, mid, down] = row.faults;
            assert!(
                row.profile == Profile::B || (up <= mid && mid <= down),
                "{kind:?} {}: kvarova pojačano {up} / zadano {mid} / odgođeno {down}",
                row.profile.code()
            );
            let [cu, cm, cd] = row.rooms_condition;
            assert!(
                cu >= cm && cm >= cd,
                "{kind:?} {}: stanje Smještaja pojačano {cu} / zadano {cm} / odgođeno {cd}",
                row.profile.code()
            );
        }
        let sum = |i: usize| f.rows.iter().map(|x| x.faults[i]).sum::<i32>();
        assert!(
            sum(0) <= sum(1) && sum(1) <= sum(2),
            "{kind:?}: zbroj kvarova pojačano {} / zadano {} / odgođeno {}",
            sum(0),
            sum(1),
            sum(2)
        );
        assert!(
            sum(0) < sum(2),
            "{kind:?}: odgođeno održavanje mora imati više kvarova od pojačanog ({} prema {})",
            sum(2),
            sum(0)
        );
    }
}
