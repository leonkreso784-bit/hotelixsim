//! Mjerilo AI presuda: brojanje slaganja i oblik isporučenih nacrta (bez mreže).

#[test]
fn agreement_counts_within_one() {
    use ai::benchmark::*;
    let o = |a, l| Outcome {
        ai_quality: a,
        leon_quality: l,
    };
    assert_eq!(
        agreement_permille(&[o(5, 5), o(4, 5), o(2, 4), o(1, 1)]),
        750
    );
    assert_eq!(agreement_permille(&[]), 0);
}

#[test]
fn shipped_cases_parse_and_have_twenty_each() {
    for code in ["S1", "S2"] {
        let path = content::repo_dir().join(format!("mjerilo/{code}.json"));
        let cases: Vec<ai::benchmark::Case> =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(cases.len(), 20, "{code}");
        assert!(
            cases
                .iter()
                .all(|c| c.leon_quality.is_none_or(|q| (1..=5).contains(&q)))
        );
    }
}

#[test]
fn exit_code_threshold_and_errors() {
    use ai::benchmark::*;
    let o = |a, l| Outcome {
        ai_quality: a,
        leon_quality: l,
    };
    // 4 od 5 unutar ±1 = 800 promila: zadovoljava; 3 od 5 = 600: ne
    let at = [o(5, 5), o(4, 5), o(3, 4), o(2, 3), o(1, 5)];
    assert_eq!(agreement_permille(&at), 800);
    assert_eq!(verdict_exit(0, &at), 0);
    let below: Vec<_> = (0..1000)
        .map(|i| if i < 799 { o(3, 3) } else { o(1, 5) })
        .collect();
    assert_eq!(agreement_permille(&below), 799);
    assert_eq!(verdict_exit(0, &below), 1);
    // greška poziva čini mjerenje nemogućim čak i uz savršeno slaganje
    assert_eq!(verdict_exit(1, &[o(3, 3)]), 2);
}

#[test]
fn leon_quality_out_of_range_is_rejected() {
    use ai::benchmark::*;
    let case = |q| Case {
        text: "x".into(),
        leon_quality: q,
        note: String::new(),
    };
    assert!(validate_cases("S1", &[case(Some(5)), case(None)]).is_ok());
    let e = validate_cases("S1", &[case(Some(3)), case(Some(6))]).unwrap_err();
    assert!(e.contains("slučaj 2") && e.contains('6'), "{e}");
    assert!(validate_cases("S1", &[case(Some(0))]).is_err());
}
