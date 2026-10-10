mod common;
use game::*;

fn crit(met: [bool; 5]) -> Vec<CriterionVerdict> {
    met.iter()
        .zip(1u8..)
        .map(|(&m, n)| CriterionVerdict {
            n,
            met: m,
            reason: format!("razlog {n}"),
        })
        .collect()
}

#[test]
fn quality_is_computed_not_trusted() {
    assert_eq!(quality_from(&crit([false; 5])), 1);
    assert_eq!(quality_from(&crit([true, false, false, false, false])), 1);
    assert_eq!(quality_from(&crit([true, true, false, true, false])), 3);
    assert_eq!(quality_from(&crit([true; 5])), 5);
}

#[test]
fn verdict_shape_is_checked() {
    let (g, _) = common::open_june_game();
    let ok = VerdictDraft {
        criteria: crit([true; 5]),
        nearest_answer: 0,
        rationale: "dobro".into(),
    };
    assert_eq!(g.validate_verdict("S1", ok.clone()).unwrap().quality, 5);
    let mut four = ok.clone();
    four.criteria.pop();
    assert_eq!(
        g.validate_verdict("S1", four),
        Err(VerdictError::CriteriaCount(4))
    );
    let mut dup = ok.clone();
    dup.criteria[1].n = 1;
    assert_eq!(
        g.validate_verdict("S1", dup),
        Err(VerdictError::DuplicateCriterion(1))
    );
    let mut six = ok.clone();
    six.criteria[4].n = 6;
    assert_eq!(
        g.validate_verdict("S1", six),
        Err(VerdictError::CriterionNumber(6))
    );
    let mut far = ok.clone();
    far.nearest_answer = 4; // S1 ima 4 odgovora: 0..=3
    assert_eq!(
        g.validate_verdict("S1", far),
        Err(VerdictError::NearestAnswer(4))
    );
    let mut blank = ok.clone();
    blank.criteria[2].reason = " ".into();
    assert_eq!(
        g.validate_verdict("S1", blank),
        Err(VerdictError::EmptyReason(3))
    );
    let mut norat = ok.clone();
    norat.rationale = String::new();
    assert_eq!(
        g.validate_verdict("S1", norat),
        Err(VerdictError::EmptyRationale)
    );
    assert_eq!(g.validate_verdict("S3", ok), Err(VerdictError::NotFreeText));
}

/// Student ne vidi redne brojeve ponuđenih odgovora ni kriterija (ADR-025): presuda koja ih spominje se odbija
/// i ponavlja, kao svaki drugi krivi oblik.
#[test]
fn verdict_text_never_names_internal_numbers() {
    let (g, _) = common::open_june_game();
    let ok = VerdictDraft {
        criteria: crit([true; 5]),
        nearest_answer: 0,
        rationale: "dobro".into(),
    };
    for leak in [
        "Odgovor odgovara ponuđenom odgovoru 0.",
        "Najbliži je odgovor [2], uz manju nadoknadu.",
        "Ispunjavaš kriterije 1 i 3, ali ne i ostale.",
        "Tvoj prijedlog je kao ponuđeni odgovor br. 1.",
        "Sličan je odgovoru (3).",
        "Ispunio si 2. kriterij, ali ne i ostale.",
    ] {
        let mut rat = ok.clone();
        rat.rationale = leak.into();
        assert_eq!(
            g.validate_verdict("S1", rat),
            Err(VerdictError::InternalNumber),
            "{leak}"
        );
    }
    let mut reason = ok.clone();
    reason.criteria[1].reason = "Kao u kriteriju 2, nudiš popust.".into();
    assert_eq!(
        g.validate_verdict("S1", reason),
        Err(VerdictError::InternalNumber)
    );
    for fine in [
        "Nudiš 20 % popusta i besplatan doručak za 2 osobe.",
        "Tvoj je odgovor najbliži ponudi da hotel plati drugu noć; ispričao si se gošći.",
        "Odgovor ima pet dijelova, a nedostaje rok od 24 sata.",
        "Preuzimaš odgovornost 5 minuta nakon poziva i odgovorio si 2 puta.",
    ] {
        let mut rat = ok.clone();
        rat.rationale = fine.into();
        assert!(g.validate_verdict("S1", rat).is_ok(), "{fine}");
    }
}
