use content::{Snapshot, repo_dir};
use sim_hotel_v2::Profile;
use sim_hotel_v2::department::{DeptCode, UsaliClass};

fn v2() -> sim_hotel_v2::Catalog {
    Snapshot::read_v2(&repo_dir())
        .unwrap()
        .build()
        .unwrap()
        .v2
        .unwrap()
}

#[test]
fn content_v2_loads_and_validates() {
    v2().validate().unwrap();
}

#[test]
fn profiles_have_the_departments_from_model_v2() {
    let c = v2();
    let codes = |p: Profile| -> Vec<String> {
        c.profile_v2(p)
            .unwrap()
            .departments
            .keys()
            .map(|k| k.as_str().to_owned())
            .filter(|k| !["rooms", "admin", "sales", "maintenance", "energy"].contains(&k.as_str()))
            .collect()
    };
    assert_eq!(
        codes(Profile::A),
        ["lobby_bar", "parking", "restaurant", "rooftop_bar", "spa"]
    );
    assert_eq!(
        codes(Profile::B),
        ["parking", "pool_bar", "restaurant", "retail_lease"]
    );
    assert_eq!(
        codes(Profile::C),
        ["lobby_bar", "parking", "restaurant", "spa"]
    );
    assert_eq!(
        c.dept_name(Profile::C, &DeptCode("spa".into())).unwrap(),
        "Mali wellness"
    );
    assert_eq!(
        c.dept_name(Profile::A, &DeptCode("spa".into())).unwrap(),
        "Wellness i spa"
    );
}

#[test]
fn staff_per_profile_matches_v1_profile() {
    let c = v2();
    for p in Profile::ALL {
        let v = c.profile_v2(p).unwrap();
        let perm: u16 = v.departments.values().map(|d| d.permanent).sum();
        let seas: u16 = v.departments.values().map(|d| d.seasonal).sum();
        let base = c.base.profile(p).unwrap();
        assert_eq!((perm, seas), (base.permanent, base.seasonal), "{p:?}");
    }
}

#[test]
fn fixed_charges_are_below_gop_items_with_names() {
    let c = v2();
    let names: Vec<&str> = c
        .economy
        .fixed_charges
        .values()
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(
        names,
        ["Osiguranje", "Najam zgrade i opreme", "Porez na imovinu"]
    );
    assert!(c.departments.values().any(|d| d.class == UsaliClass::Rooms));
}

#[test]
fn unknown_department_in_profile_is_rejected() {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    v["v2"]["profiles"]["C"]["departments"]["casino"] =
        v["v2"]["profiles"]["C"]["departments"]["parking"].clone();
    assert!(Snapshot::from_value(&v).unwrap().build().is_err());
}

#[test]
fn degenerate_v2_still_matches_core_degenerate() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let c = s.degenerate_v2().build().unwrap();
    assert_eq!(c.v2.unwrap(), sim_hotel_v2::degenerate(&c.catalog));
}

#[test]
fn closable_departments_have_only_demand_driven_revenue() {
    // Odluka kontrolora (pregled Taska 9): zatvaranje gasi samo potražnju, pa Fixed i ShareOfRooms ne smiju biti zatvorivi.
    use sim_hotel_v2::department::RevenueModel;
    for (code, d) in &v2().departments {
        if matches!(
            d.revenue,
            RevenueModel::Fixed { .. } | RevenueModel::ShareOfRooms { .. }
        ) {
            assert!(!d.can_close, "{}", code.as_str());
        }
    }
}
