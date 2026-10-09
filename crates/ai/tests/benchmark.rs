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
