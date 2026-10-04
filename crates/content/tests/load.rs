#[test]
fn repo_content_loads_and_validates() {
    let c = content::load(&content::repo_dir()).unwrap();
    assert_eq!(c.catalog.validate(), Ok(()));
    assert_eq!(c.catalog.profiles.len(), 3);
    let codes: Vec<&str> = c.situations.iter().map(|s| s.code.as_str()).collect();
    assert_eq!(codes, ["S1", "S3", "S2", "S4"]); // redoslijed: po mjesecu, pa po datoteci
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
        r#"[{"code": "SX", "month": "jun", "title": "t", "answers": []}]"#,
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
