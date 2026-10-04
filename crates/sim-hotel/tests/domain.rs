mod common;
use common::*;
use sim_hotel::*;

#[test]
fn month_days_and_codes_are_stable() {
    let days: Vec<u8> = Month::SEASON.iter().map(|m| m.days()).collect();
    assert_eq!(days, [30, 31, 31, 30]);
    let codes: Vec<&str> = Month::SEASON.iter().map(|m| m.code()).collect();
    assert_eq!(codes, ["jun", "jul", "aug", "sep"]);
    assert_eq!(Month::from_code("aug"), Some(Month::Aug));
    assert_eq!(Month::from_code("Aug"), None);
}

#[test]
fn month_position_matches_season_order() {
    for (i, m) in Month::SEASON.iter().enumerate() {
        assert_eq!(m.position(), i);
    }
}

#[test]
fn test_catalog_is_valid() {
    assert_eq!(test_catalog().validate(), Ok(()));
}

#[test]
fn catalog_rejects_zero_reference_price() {
    let mut cat = test_catalog();
    cat.categories
        .get_mut(&Category::Four)
        .unwrap()
        .price_ref_cents
        .0[0] = 0;
    assert!(matches!(cat.validate(), Err(CatalogError::NotPositive(_))));
}

#[test]
fn catalog_rejects_profile_without_category_tables() {
    let mut cat = test_catalog();
    cat.profiles.get_mut(&Profile::A).unwrap().category = Category::Five;
    assert!(matches!(cat.validate(), Err(CatalogError::Missing(_))));
}

#[test]
fn catalog_rejects_missing_reference_profile() {
    let mut cat = test_catalog();
    cat.reference_profiles.clear();
    assert!(matches!(cat.validate(), Err(CatalogError::Missing(_))));
}

#[test]
fn state_from_profile_c_derives_turnover() {
    let s = HotelState::from_profile(&profile_c()).unwrap();
    assert_eq!(s.turnover_bp, 375); // 3 % + 0,15 % × (60 − 55) = 3,75 %
    assert_eq!(s.staff(), 50);
    assert!(s.groups.is_empty());
}

#[test]
fn turnover_rounds_half_away_from_zero() {
    assert_eq!(sim_hotel::turnover_bp(6_000), Ok(300));
    assert_eq!(sim_hotel::turnover_bp(4_200), Ok(570));
    assert_eq!(sim_hotel::turnover_bp(0), Ok(1_200));
}

#[test]
fn index_set_from_named_keeps_meaning() {
    let s = IndexSet::from_named(NamedIndices {
        guests: 6_000,
        reviews: 7_500,
        staff: 3_000,
        operations: 5_000,
        finance: 3_500,
    })
    .unwrap();
    assert_eq!(s.get(Index::Reviews).hundredths(), 7_500);
    assert_eq!(s.get(Index::Finance).hundredths(), 3_500);
    assert!(
        IndexSet::from_named(NamedIndices {
            guests: 10_001,
            reviews: 0,
            staff: 0,
            operations: 0,
            finance: 0
        })
        .is_none()
    );
}

#[test]
fn index_set_rejects_invalid_value_in_every_position() {
    for i in 0_usize..5 {
        let v: [u16; 5] = std::array::from_fn(|j| if j == i { 10_001 } else { 5_000 });
        assert!(IndexSet::from_hundredths(v).is_none(), "mjesto {i}");
    }
}

#[test]
fn profile_targets_match_demo_section_5_fully() {
    let full = |p: Profile| {
        p.starting_indices()
            .iter()
            .map(|(_, v)| v.hundredths())
            .collect::<Vec<_>>()
    };
    assert_eq!(full(Profile::A), [6_000, 7_500, 3_000, 5_000, 3_500]);
    assert_eq!(full(Profile::B), [4_500, 3_500, 5_500, 4_000, 7_500]);
    assert_eq!(full(Profile::C), [5_000; 5]);
}
