use content::{Snapshot, repo_dir};
use serde_json::json;

fn snap() -> Snapshot {
    Snapshot::read(&repo_dir()).unwrap()
}

#[test]
fn snapshot_json_roundtrip_builds_same_content_and_fingerprint() {
    let s = snap();
    let text = serde_json::to_string(&s).unwrap();
    let back: Snapshot = serde_json::from_str(&text).unwrap();
    assert_eq!(back.fingerprint(), s.fingerprint());
    let built = back.build().unwrap();
    let loaded = content::load(&repo_dir()).unwrap();
    assert_eq!(built.catalog, loaded.catalog);
    assert_eq!(built.situations, loaded.situations);
}

#[test]
fn snapshot_survives_json_roundtrip() {
    let s = snap();
    let back: Snapshot = serde_json::from_value(serde_json::to_value(&s).unwrap()).unwrap();
    assert_eq!(back, s);
    assert_eq!(back.fingerprint(), s.fingerprint());
}

#[test]
fn fingerprint_has_fixed_shape_and_reacts_to_change() {
    let s = snap();
    let f = s.fingerprint();
    assert!(f.starts_with("sha256:"), "{f}");
    assert_eq!(f.len(), 7 + 64);
    let mut v = serde_json::to_value(&s).unwrap();
    v["catalog"]["economy"]["hire_cents"] = json!(150_001);
    let changed: Snapshot = serde_json::from_value(v).unwrap();
    assert_ne!(changed.fingerprint(), f);
}

#[test]
fn fingerprint_ignores_key_order() {
    // JSONB u Postgresu mijenja redoslijed ključeva; otisak se računa iz tipiziranog snimka, ne iz teksta.
    let s = snap();
    let text = serde_json::to_string(&s).unwrap();
    let reordered: serde_json::Value = serde_json::from_str(&text).unwrap(); // Value sortira ključeve
    let back: Snapshot = serde_json::from_value(reordered).unwrap();
    assert_eq!(back.fingerprint(), s.fingerprint());
}

#[test]
fn unknown_field_in_snapshot_is_rejected() {
    let mut v = serde_json::to_value(snap()).unwrap();
    v["catalog"]["economy"]["typo"] = json!(1);
    assert!(serde_json::from_value::<Snapshot>(v).is_err());
}

#[test]
fn fingerprint_value_ignores_key_order() {
    // Brana: serde_json mora ostati bez značajke `preserve_order`, inače ovaj test pada.
    let a = serde_json::json!({"b": 1, "a": {"y": 2, "x": 3}});
    let b = serde_json::json!({"a": {"x": 3, "y": 2}, "b": 1});
    assert_eq!(
        content::fingerprint_value(&a),
        content::fingerprint_value(&b)
    );
    assert!(content::fingerprint_value(&a).starts_with("sha256:"));
}

#[test]
fn fingerprint_is_over_value_and_survives_roundtrip() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let v = s.to_value();
    assert_eq!(s.fingerprint(), content::fingerprint_value(&v));
    let back = Snapshot::from_value(&v).unwrap();
    assert_eq!(back, s);
    assert_eq!(back.fingerprint(), s.fingerprint());
}

#[test]
fn legacy_fingerprint_differs_from_value_fingerprint() {
    // Stari otisak je nad tipiziranim zapisom (redoslijed polja iz deklaracije), novi nad poredanim ključevima.
    let s = Snapshot::read(&repo_dir()).unwrap();
    assert_ne!(s.legacy_fingerprint(), s.fingerprint());
}

#[test]
fn from_value_rejects_unknown_field() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let mut v = s.to_value();
    v.as_object_mut()
        .unwrap()
        .insert("nepoznato".into(), serde_json::json!(1));
    assert!(Snapshot::from_value(&v).is_err());
}

#[test]
fn legacy_fingerprint_of_step3_snapshot_is_frozen() {
    // Literal je otisak snimka iz koraka 3 (izračunan prije izmjene DTO-a u koraku 4) i NIKAD se ne smije mijenjati:
    // dokazuje da `skip_serializing_if` daje bajt-isti stari zapis pa stvarne igre iz koraka 3 i dalje prolaze.
    // Ulaz je zamrznuta kopija tog snimka, ne živi sadržaj: igre nose vlastiti snimak, pa se `content/` smije
    // mijenjati, a test i dalje mjeri samo stari zapis.
    // Kopija je snimak iz `content/` na 796618f bez polja koraka 4 (rubric, character, free_text, criteria).
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/step3-snapshot.json"
    );
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let old = Snapshot::from_value(&v).unwrap();
    assert_eq!(
        old.legacy_fingerprint(),
        "sha256:5ef1b8541e85d0f2311a6c6ba1af2411ee043e1f79473823452bcbfbbc88973e"
    );
}
