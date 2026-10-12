use content::{Model, Snapshot, fingerprint_value, repo_dir};
use sim_hotel::CatalogError;
use sim_hotel_v2::{PeriodKind, degenerate};

fn v1_fixture() -> serde_json::Value {
    let p = repo_dir().join("../../crates/game/tests/fixtures/v1-snapshot.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn base() -> sim_hotel::Catalog {
    Snapshot::read(&repo_dir())
        .unwrap()
        .build()
        .unwrap()
        .catalog
}

#[test]
fn old_v1_snapshot_round_trips_without_new_keys() {
    let v = v1_fixture();
    let s = Snapshot::from_value(&v).unwrap();
    assert_eq!(s.model(), Model::V1);
    let again = s.to_value();
    assert!(again.get("model").is_none() && again.get("v2").is_none());
    assert_eq!(fingerprint_value(&again), fingerprint_value(&v));
}

#[test]
fn current_v1_snapshot_has_no_model_key() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    assert_eq!(s.model(), Model::V1);
    assert!(s.to_value().get("model").is_none());
    assert!(s.build().unwrap().v2.is_none());
}

#[test]
fn v2_snapshot_builds_with_period_coefficients() {
    let s = Snapshot::read_v2(&repo_dir()).unwrap();
    assert_eq!(s.model(), Model::V2);
    assert_eq!(s.to_value()["model"], "v2");
    let c = s.build().unwrap();
    let v2 = c.v2.unwrap();
    assert_eq!(v2.base, c.catalog);
    let m = v2.coeffs(PeriodKind::Month).unwrap();
    assert_eq!(
        (m.morale_step_bp, m.satisfaction_step_bp, m.rating_step_bp),
        (3_000, 5_000, 2_500)
    );
    let w = v2.coeffs(PeriodKind::Week).unwrap();
    assert!(w.morale_step_bp < m.morale_step_bp && w.satisfaction_step_bp < m.satisfaction_step_bp);
}

#[test]
fn v2_snapshot_round_trip_keeps_fingerprint() {
    let v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let again = Snapshot::from_value(&v).unwrap().to_value();
    assert_eq!(fingerprint_value(&again), fingerprint_value(&v));
}

#[test]
fn model_and_section_must_agree() {
    let mut only_model = Snapshot::read(&repo_dir()).unwrap().to_value();
    only_model["model"] = "v2".into();
    assert!(Snapshot::from_value(&only_model).unwrap().build().is_err());
    let mut only_section = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    only_section.as_object_mut().unwrap().remove("model");
    assert!(
        Snapshot::from_value(&only_section)
            .unwrap()
            .build()
            .is_err()
    );
}

#[test]
fn degenerate_v2_matches_core_degenerate() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let c = s.degenerate_v2().build().unwrap();
    assert_eq!(c.v2.unwrap(), sim_hotel_v2::degenerate(&c.catalog));
}

#[test]
fn unknown_period_kind_is_invalid() {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    v["v2"]["periods"]["fortnight"] = v["v2"]["periods"]["week"].clone();
    assert!(Snapshot::from_value(&v).unwrap().build().is_err());
}

// Katalog v2: `validate` i `degenerate` (nalaz pregleda Taska 2).

#[test]
fn degenerate_validates_and_month_equals_v1_constants() {
    let b = base();
    let c = degenerate(&b);
    assert_eq!(c.validate(), Ok(()));
    let m = c.coeffs(PeriodKind::Month).unwrap();
    assert_eq!(
        (m.morale_step_bp, m.satisfaction_step_bp, m.rating_step_bp),
        (3_000, 5_000, 2_500)
    );
    assert_eq!(m.seasonal_step, b.actions.seasonal_step);
}

fn out_of_range(label: &str) -> Result<(), CatalogError> {
    Err(CatalogError::OutOfRange(label.to_string()))
}

#[test]
fn validate_rejects_each_coefficient_out_of_range() {
    let b = base();
    type Set = fn(&mut sim_hotel_v2::PeriodCoefficients, u16);
    let setters: [(&str, Set); 3] = [
        ("morale_step_bp", |p, v| p.morale_step_bp = v),
        ("satisfaction_step_bp", |p, v| p.satisfaction_step_bp = v),
        ("rating_step_bp", |p, v| p.rating_step_bp = v),
    ];
    for (name, set) in setters {
        for bad in [0, 10_001] {
            let mut c = degenerate(&b);
            set(c.periods.get_mut(&PeriodKind::Week).unwrap(), bad);
            assert_eq!(
                c.validate(),
                out_of_range(&format!("v2.periods.week.{name}"))
            );
        }
        let mut c = degenerate(&b);
        set(c.periods.get_mut(&PeriodKind::Month).unwrap(), 10_000);
        assert_eq!(c.validate(), Ok(()), "{name}: gornja granica je dopuštena");
    }
}

#[test]
fn validate_rejects_zero_seasonal_step() {
    let mut c = degenerate(&base());
    c.periods.get_mut(&PeriodKind::Month).unwrap().seasonal_step = 0;
    assert_eq!(c.validate(), out_of_range("v2.periods.month.seasonal_step"));
}

#[test]
fn validate_reports_missing_kind() {
    let mut c = degenerate(&base());
    c.periods.remove(&PeriodKind::Week);
    assert_eq!(
        c.validate(),
        Err(CatalogError::Missing("v2.periods.week".to_string()))
    );
}

/// J-3: ključevi učinka situacija i uvjeta grupe. Sadržaj iz repoa nosi Leonove brojke (13.10.); snimak bez
/// tih ključeva (v2 igra prije J-3) gradi pravila kao prije (puni učinak, bez granica, grupa bez uvjeta) i
/// zapisuje se bez njih, s istim otiskom.
#[test]
fn j3_keys_are_read_from_content_and_absent_keys_keep_old_rules() {
    let s = Snapshot::read_v2(&repo_dir()).unwrap();
    let c = s.build().unwrap();
    let v2 = c.v2.unwrap();
    for (k, effect) in [(PeriodKind::Week, 325), (PeriodKind::Month, 350)] {
        let p = v2.coeffs(k).unwrap();
        assert_eq!(
            (p.situation_effect_bp, p.situation_round_bound_pts),
            (effect, Some(800))
        );
    }
    let g = &v2.economy.groups;
    assert_eq!(
        (
            g.commission_bp,
            g.attrition_bp,
            g.displacement_bp,
            g.load_satisfaction_pts,
            g.load_morale_pts
        ),
        (1_500, 1_500, 3_000, 4_000, 4_000)
    );
    let set = c.situation_templates.unwrap().to_core();
    assert_eq!((set.group_limit, set.groups_per_month), (Some(3), Some(1)));

    let mut old = s.to_value();
    for k in ["week", "month"] {
        let p = old["v2"]["periods"][k].as_object_mut().unwrap();
        assert!(p.remove("situation_effect_bp").is_some());
        assert!(p.remove("situation_round_bound_pts").is_some());
    }
    assert!(
        old["v2"]["economy"]
            .as_object_mut()
            .unwrap()
            .remove("groups")
            .is_some()
    );
    let t = old["v2"]["situation_templates"].as_object_mut().unwrap();
    assert!(t.remove("groups_per_season").is_some());
    assert!(t.remove("groups_per_month").is_some());
    let back = Snapshot::from_value(&old).unwrap();
    assert_eq!(fingerprint_value(&back.to_value()), fingerprint_value(&old));
    let c = back.build().unwrap();
    let v2 = c.v2.unwrap();
    for k in PeriodKind::ALL {
        let p = v2.coeffs(k).unwrap();
        assert_eq!(
            (p.situation_effect_bp, p.situation_round_bound_pts),
            (10_000, None)
        );
    }
    assert_eq!(v2.economy.groups, sim_hotel_v2::GroupTerms::default());
    let set = c.situation_templates.unwrap().to_core();
    assert_eq!((set.group_limit, set.groups_per_month), (None, None));
}
