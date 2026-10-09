#[test]
fn repo_content_loads_and_validates() {
    let c = content::load(&content::repo_dir()).unwrap();
    assert_eq!(c.catalog.validate(), Ok(()));
    assert_eq!(c.catalog.profiles.len(), 3);
    let codes: Vec<&str> = c.situations.iter().map(|s| s.code.as_str()).collect();
    assert_eq!(codes, ["S1", "S3", "S2", "S4"]); // redoslijed: po mjesecu, pa po datoteci
    assert!(c.situations.iter().all(|s| !s.unanswered.is_empty()));
    assert!(c.situations.iter().all(|s| s.answers.len() == 4 && s.answers.iter().all(|a| (1..=5).contains(&a.quality))));
}

#[test]
fn invalid_catalog_is_rejected_with_field_name() {
    let dir = std::env::temp_dir().join(format!("hotelixsim-content-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for f in ["profiles.json", "situations.json"] {
        std::fs::copy(content::repo_dir().join(f), dir.join(f)).unwrap();
    }
    let cat = std::fs::read_to_string(content::repo_dir().join("catalog.json")).unwrap();
    std::fs::write(
        dir.join("catalog.json"),
        cat.replacen("\"wage_cents\": 220000", "\"wage_cents\": 0", 1),
    )
    .unwrap();
    let err = content::load(&dir).unwrap_err();
    assert!(format!("{err:?}").contains("wage_cents"), "{err:?}");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn unknown_field_is_rejected() {
    let dir = std::env::temp_dir().join(format!("hotelixsim-content-u-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for f in ["catalog.json", "situations.json"] {
        std::fs::copy(content::repo_dir().join(f), dir.join(f)).unwrap();
    }
    let p = std::fs::read_to_string(content::repo_dir().join("profiles.json")).unwrap();
    std::fs::write(
        dir.join("profiles.json"),
        p.replacen("\"rooms\"", "\"rooms_typo\": 1, \"rooms\"", 1),
    )
    .unwrap();
    assert!(matches!(
        content::load(&dir),
        Err(content::ContentError::Json(..))
    ));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn group_discount_over_100_percent_is_rejected() {
    let dir = std::env::temp_dir().join(format!("hotelixsim-content-g-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for f in ["catalog.json", "profiles.json"] {
        std::fs::copy(content::repo_dir().join(f), dir.join(f)).unwrap();
    }
    let s = std::fs::read_to_string(content::repo_dir().join("situations.json")).unwrap();
    std::fs::write(
        dir.join("situations.json"),
        s.replacen("\"discount_bp\": 2000", "\"discount_bp\": 10001", 1),
    )
    .unwrap();
    let err = content::load(&dir).unwrap_err();
    assert!(
        matches!(&err, content::ContentError::Invalid(m) if m.contains("S3")),
        "{err:?}"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Privremeni direktorij sadržaja; briše se i kad test padne.
struct TempContent(std::path::PathBuf);

impl TempContent {
    fn new(tag: &str, situations: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("hotelixsim-content-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let guard = TempContent(dir);
        for f in ["catalog.json", "profiles.json"] {
            std::fs::copy(content::repo_dir().join(f), guard.0.join(f)).unwrap();
        }
        std::fs::write(guard.0.join("situations.json"), situations).unwrap();
        guard
    }
}

impl Drop for TempContent {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn situation_without_answers_is_rejected() {
    let t = TempContent::new(
        "ea",
        r#"[{"code": "SX", "month": "jun", "title": "t", "answers": [], "unanswered": []}]"#,
    );
    let err = content::load(&t.0).unwrap_err();
    assert!(
        matches!(&err, content::ContentError::Invalid(m) if m.contains("SX")),
        "{err:?}"
    );
}

#[test]
fn duplicate_situation_code_is_rejected() {
    let s = std::fs::read_to_string(content::repo_dir().join("situations.json")).unwrap();
    let mut list: Vec<serde_json::Value> = serde_json::from_str(&s).unwrap();
    let first = list.first().cloned().unwrap();
    list.push(first);
    let t = TempContent::new("dup", &serde_json::to_string(&list).unwrap());
    let err = content::load(&t.0).unwrap_err();
    assert!(
        matches!(&err, content::ContentError::Invalid(m) if m.contains("S1")),
        "{err:?}"
    );
}

#[test]
fn s1_and_s2_are_free_text_with_rubric_and_character() {
    let c = content::load(&content::repo_dir()).unwrap();
    for code in ["S1", "S2"] {
        let ai = &c.ai[code];
        assert!(ai.free_text, "{code}");
        assert_eq!(ai.rubric.len(), 5, "{code}");
        assert!(ai.character.is_some(), "{code}");
    }
    assert!(!c.ai["S3"].free_text);
}

#[test]
fn offered_answer_quality_matches_criteria_count() {
    // Ponuđeni odgovori i slobodan tekst su na istoj ljestvici (ARHITEKTURA §10.3): q = max(1, broj kriterija).
    let c = content::load(&content::repo_dir()).unwrap();
    for t in &c.situations {
        let ai = &c.ai[&t.code];
        if ai.rubric.is_empty() {
            continue;
        }
        for (a, crit) in t.answers.iter().zip(&ai.criteria) {
            let expected = u8::try_from(crit.len()).unwrap().max(1);
            assert_eq!(a.quality, expected, "{} / {}", t.code, a.label);
        }
    }
}

/// Učita sadržaj iz repoa s promijenjenom situacijom S1 (`edit` mijenja njezin JSON) i vrati grešku.
fn load_with_s1(tag: &str, edit: impl FnOnce(&mut serde_json::Value)) -> content::ContentError {
    let s = std::fs::read_to_string(content::repo_dir().join("situations.json")).unwrap();
    let mut list: Vec<serde_json::Value> = serde_json::from_str(&s).unwrap();
    edit(&mut list[0]);
    let t = TempContent::new(tag, &serde_json::to_string(&list).unwrap());
    content::load(&t.0).unwrap_err()
}

fn assert_invalid_s1(err: content::ContentError) {
    assert!(
        matches!(&err, content::ContentError::Invalid(m) if m.contains("S1")),
        "{err:?}"
    );
}

#[test]
fn free_text_without_character_is_rejected() {
    assert_invalid_s1(load_with_s1("nochar", |s| {
        s.as_object_mut().unwrap().remove("character");
    }));
}

#[test]
fn rubric_with_four_criteria_is_rejected() {
    assert_invalid_s1(load_with_s1("r4", |s| {
        s["rubric"].as_array_mut().unwrap().pop();
    }));
}

#[test]
fn criterion_six_is_rejected() {
    assert_invalid_s1(load_with_s1("c6", |s| {
        s["answers"][2]["criteria"] = serde_json::json!([6]);
    }));
}

#[test]
fn duplicate_criterion_in_answer_is_rejected() {
    assert_invalid_s1(load_with_s1("cdup", |s| {
        s["answers"][1]["criteria"] = serde_json::json!([1, 1, 4]);
    }));
}

#[test]
fn quality_not_matching_criteria_is_rejected() {
    assert_invalid_s1(load_with_s1("qmis", |s| {
        s["answers"][0]["quality"] = serde_json::json!(5);
        s["answers"][0]["criteria"] = serde_json::json!([1, 2]);
    }));
}

#[test]
fn criteria_without_rubric_are_rejected() {
    assert_invalid_s1(load_with_s1("norub", |s| {
        let o = s.as_object_mut().unwrap();
        o.remove("rubric");
        o.remove("free_text");
        o.remove("character");
    }));
}
