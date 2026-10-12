// Obnova v2 (O-1): nova polja sadržaja (krivulja obnove odjela, zgrada, pričuva, osnova indeksa F, stanje zgrade
// profila) — round-trip sa i bez njih, zadane vrijednosti i provjera kataloga.
use content::{ContentError, Snapshot, fingerprint_value, repo_dir};
use serde_json::{Value, json};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::{FinanceBasis, Profile};

fn snapshot() -> Value {
    Snapshot::read_v2(&repo_dir()).unwrap().to_value()
}

fn build(v: &Value) -> Result<sim_hotel_v2::Catalog, ContentError> {
    Ok(Snapshot::from_value(v)?.build()?.v2.unwrap())
}

fn renewal(half: i64, max: i64, step: i64, life: u16) -> Value {
    json!({ "gain_pts": 5000, "half_cents": half, "max_monthly_cents": max, "step_cents": step, "life_months": life })
}

/// Snimak s obnovom restorana, zgradom, pričuvom 4 %, indeksom F iz EBIT-a i stanjem zgrade profila A.
fn with_renewal() -> Value {
    let mut v = snapshot();
    v["v2"]["departments"]["restaurant"]["renewal"] = renewal(5_500_000, 5_000_000, 500_000, 84);
    v["v2"]["economy"]["building"] = json!({
        "name": "Zgrada",
        "wear_pts_per_month": 50,
        "satisfaction_share_bp": 2000,
        "renewal": renewal(15_000_000, 20_000_000, 1_000_000, 480),
    });
    v["v2"]["economy"]["ffe_reserve_bp"] = json!(400);
    v["v2"]["economy"]["finance_index"] = json!("ebit");
    v["v2"]["profiles"]["A"]["building_pts"] = json!(8000);
    v
}

fn invalid(v: &Value, label: &str) {
    match build(v) {
        Err(ContentError::Invalid(m)) => assert!(m.contains(label), "{m}"),
        other => panic!("očekivana greška {label}, dobiveno {other:?}"),
    }
}

#[test]
fn content_without_new_fields_has_no_renewal() {
    let v = snapshot();
    assert!(
        v["v2"]["departments"]
            .as_object()
            .unwrap()
            .values()
            .all(|d| d.get("renewal").is_none())
    );
    assert!(
        v["v2"]["profiles"]
            .as_object()
            .unwrap()
            .values()
            .all(|p| p.get("building_pts").is_none())
    );
    for key in ["building", "ffe_reserve_bp", "finance_index"] {
        assert!(v["v2"]["economy"].get(key).is_none(), "{key}");
    }
    let c = build(&v).unwrap();
    assert!(!c.renewal_enabled());
    assert!(
        c.departments
            .values()
            .all(|d| d.renewal == Default::default())
    );
    assert_eq!(c.economy.building, sim_hotel_v2::BuildingSpec::none());
    assert_eq!(c.economy.ffe_reserve_bp, 0);
    assert_eq!(c.economy.finance_index, FinanceBasis::Gop);
    assert!(c.profiles.values().all(|p| p.building_pts == 10_000));
    // bez novih polja snimak ostaje bajt-isti
    let again = Snapshot::from_value(&v).unwrap().to_value();
    assert_eq!(again, v);
}

#[test]
fn new_fields_round_trip_and_build() {
    let v = with_renewal();
    let c = build(&v).unwrap();
    assert!(c.renewal_enabled());
    let r = &c.dept(&DeptCode("restaurant".into())).unwrap().renewal;
    assert_eq!(
        (
            r.gain_pts,
            r.half_cents,
            r.max_monthly_cents,
            r.step_cents,
            r.life_months
        ),
        (5_000, 5_500_000, 5_000_000, 500_000, 84)
    );
    let b = &c.economy.building;
    assert_eq!(
        (
            b.name.as_str(),
            b.wear_pts_per_month,
            b.satisfaction_share_bp
        ),
        ("Zgrada", 50, 2_000)
    );
    assert_eq!(b.renewal.max_monthly_cents, 20_000_000);
    assert_eq!(c.economy.ffe_reserve_bp, 400);
    assert_eq!(c.economy.finance_index, FinanceBasis::Ebit);
    assert_eq!(c.profiles[&Profile::A].building_pts, 8_000);
    assert_eq!(c.profiles[&Profile::B].building_pts, 10_000);
    let again = Snapshot::from_value(&v).unwrap().to_value();
    assert_eq!(fingerprint_value(&again), fingerprint_value(&v));
    assert_eq!(again, v);
}

#[test]
fn unknown_finance_basis_is_invalid() {
    let mut v = snapshot();
    v["v2"]["economy"]["finance_index"] = json!("ebitda");
    assert!(matches!(build(&v), Err(ContentError::Invalid(m)) if m.contains("finance_index")));
}

#[test]
fn validate_rejects_bad_renewal_building_and_profile() {
    let dept = "v2.departments.restaurant.renewal";
    let bad_dept = |f: &dyn Fn(&mut Value)| {
        let mut v = with_renewal();
        f(&mut v["v2"]["departments"]["restaurant"]["renewal"]);
        v
    };
    invalid(&bad_dept(&|r| r["gain_pts"] = json!(0)), dept);
    invalid(&bad_dept(&|r| r["gain_pts"] = json!(10_001)), dept);
    invalid(&bad_dept(&|r| r["half_cents"] = json!(0)), dept);
    invalid(&bad_dept(&|r| r["step_cents"] = json!(0)), dept);
    invalid(&bad_dept(&|r| r["step_cents"] = json!(300_000)), dept);
    invalid(&bad_dept(&|r| r["life_months"] = json!(0)), dept);
    invalid(
        &bad_dept(&|r| r["max_monthly_cents"] = json!(-500_000)),
        dept,
    );
    // neraspoređeni odjel ne smije imati obnovu
    let mut v = with_renewal();
    v["v2"]["departments"]["admin"]["renewal"] = renewal(100, 100, 100, 1);
    invalid(&v, "v2.departments.admin.renewal");
    // zgrada
    let building = "v2.economy.building";
    for (k, x) in [
        ("wear_pts_per_month", json!(-1)),
        ("wear_pts_per_month", json!(10_001)),
        ("satisfaction_share_bp", json!(10_001)),
    ] {
        let mut v = with_renewal();
        v["v2"]["economy"]["building"][k] = x;
        invalid(&v, building);
    }
    let mut v = with_renewal();
    v["v2"]["economy"]["building"]["renewal"]["life_months"] = json!(0);
    invalid(&v, building);
    // pričuva i stanje zgrade profila
    let mut v = with_renewal();
    v["v2"]["economy"]["ffe_reserve_bp"] = json!(10_001);
    invalid(&v, "v2.economy.ffe_reserve_bp");
    for x in [-1, 10_001] {
        let mut v = with_renewal();
        v["v2"]["profiles"]["B"]["building_pts"] = json!(x);
        invalid(&v, "v2.profiles.B.building_pts");
    }
}

#[test]
fn degenerate_snapshot_still_matches_core_degenerate() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let c = s.degenerate_v2().build().unwrap();
    let v2 = c.v2.unwrap();
    assert_eq!(v2, sim_hotel_v2::degenerate(&c.catalog));
    assert!(!v2.renewal_enabled());
}
