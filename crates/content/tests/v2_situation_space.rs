//! B1a: `economy.repair_urgent_condition_pts` u sadržaju v2 (ADR-042 t. 1): hitan popravak diže stanje prostora.
use content::{Snapshot, fingerprint_value, repo_dir};
use serde_json::json;

#[test]
fn repair_bonus_round_trips_and_old_snapshot_reads_zero() {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let repo = Snapshot::from_value(&v)
        .unwrap()
        .build()
        .unwrap()
        .v2
        .unwrap();
    assert_eq!(
        repo.economy.repair_urgent_condition_pts, 1_000,
        "repo gradi 1000"
    );

    v["v2"]["economy"]
        .as_object_mut()
        .unwrap()
        .remove("repair_urgent_condition_pts");
    let old = v.clone();
    let none = Snapshot::from_value(&old).unwrap();
    assert_eq!(
        none.build()
            .unwrap()
            .v2
            .unwrap()
            .economy
            .repair_urgent_condition_pts,
        0,
        "stari snimak bez ključa: bonus 0"
    );
    let back = Snapshot::from_value(&old).unwrap();
    assert_eq!(
        fingerprint_value(&back.to_value()),
        fingerprint_value(&old),
        "snimak bez ključa se ne piše s ključem"
    );

    v["v2"]["economy"]["repair_urgent_condition_pts"] = json!(1_000);
    let s = Snapshot::from_value(&v).unwrap();
    assert_eq!(s.to_value(), v);
    s.build().unwrap().v2.unwrap().validate().unwrap();

    v["v2"]["economy"]["repair_urgent_condition_pts"] = json!(20_000);
    assert!(
        Snapshot::from_value(&v).unwrap().build().is_err(),
        "iznad 10 000 bodova: provjera kataloga v2 odbija"
    );
}
