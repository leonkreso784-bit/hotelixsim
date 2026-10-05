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
