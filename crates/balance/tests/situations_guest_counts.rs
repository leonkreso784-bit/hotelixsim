//! L1-13 dio 3 (A7): ispis balansa v2 dobiva JEDAN novi redak u bloku J-3, prosjek gostinjskih i ostalih
//! (izvučenih) situacija po krugu za svaku razinu očekivane popunjenosti:
//! `  po popunjenosti (prosjek po krugu): niska 1.20 gostinjskih + 1.80 ostalih; srednja ...; visoka ...`
//! Redak se ispisuje i dok je ključ isključen (u repou do dijela 4), pa se prije/poslije vidi u istom obliku;
//! ostali retci ispisa ostaju bajt-isti (to provjerava `cmp` balansa graditelja).
mod common;

use balance::v2::{OptionsV2, render_v2, run_v2};
use common::content_v2;
use sim_hotel_v2::PeriodKind;

const HEADER: &str = "== Situacije iz predložaka (J-3) ==";

/// Prvi broj ispred riječi `word` u odjeljku (`1.20 gostinjskih`).
fn before(segment: &str, word: &str) -> f64 {
    let tokens: Vec<&str> = segment.split_whitespace().collect();
    let at = tokens
        .iter()
        .position(|t| t.trim_end_matches([';', '.']) == word)
        .unwrap_or_else(|| panic!("riječ {word} u odjeljku: {segment}"));
    tokens[at - 1]
        .parse()
        .unwrap_or_else(|_| panic!("broj ispred {word} u odjeljku: {segment}"))
}

#[test]
fn report_prints_guest_and_other_counts() {
    let r = run_v2(
        &content_v2(),
        &OptionsV2 {
            seeds: 5,
            kind: PeriodKind::Week,
            situations: None,
        },
    )
    .unwrap();
    let text = render_v2(&r);
    let block: Vec<&str> = text
        .lines()
        .skip_while(|l| *l != HEADER)
        .skip(1)
        .take_while(|l| !l.starts_with("== "))
        .collect();
    assert!(!block.is_empty(), "blok J-3 postoji");
    let lines: Vec<&&str> = block
        .iter()
        .filter(|l| l.contains("gostinjskih") && l.contains("ostalih"))
        .collect();
    assert_eq!(
        lines.len(),
        1,
        "točno jedan novi redak u bloku J-3: {block:#?}"
    );
    let line = lines[0];
    assert!(line.contains("po popunjenosti"), "{line}");
    let (_, rest) = line.split_once(':').expect("dvotočka iza uvoda");
    let segments: Vec<&str> = rest.split(';').map(str::trim).collect();
    assert_eq!(segments.len(), 3, "niska; srednja; visoka: {line}");
    let mut totals = Vec::new();
    for (segment, name) in segments.iter().zip(["niska", "srednja", "visoka"]) {
        assert!(segment.starts_with(name), "{name} u {segment}");
        let (guest, other) = (before(segment, "gostinjskih"), before(segment, "ostalih"));
        assert!(guest >= 0.0 && other >= 0.0, "{segment}");
        totals.push(guest + other);
    }
    // Izvučenih je najviše koliko krug ima mjesta (3 / 4 / 5), a visoka popunjenost ih ima više od niske.
    assert!(
        totals[0] <= 3.0 + 1e-9 && totals[1] <= 4.0 + 1e-9 && totals[2] <= 5.0 + 1e-9,
        "{totals:?}"
    );
    assert!(
        totals[2] > totals[0],
        "visoka popunjenost ima više situacija od niske: {totals:?}"
    );
}
