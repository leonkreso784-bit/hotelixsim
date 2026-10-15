//! Broj situacija u krugu (J-1, plan situacija §3 i Leonove odluke 13.10.; ADR-042 t. 13): tjedno 3 / 4 / 5, mjesečno
//! 4 / 5 / 6 po očekivanoj popunjenosti kruga (ispod 75 % / 75–89 % / od 90 %).

use content::{Snapshot, repo_dir};
use sim_hotel_v2::situations::{SituationCounts, expected_occupancy_bp, situation_count};
use sim_hotel_v2::{Catalog, HotelState, PeriodKind, Profile, season_periods};

/// Katalog iz repoa, ne zamrznuti (m9): pragovi i razmjer po danima su pravilo i ne ovise o katalogu, a tablice po
/// profilu su alarm — opisuju broj situacija koji student stvarno vidi, pa promjena kataloga mora ovdje pasti.
fn catalog() -> Catalog {
    Snapshot::read_v2(&repo_dir())
        .unwrap()
        .build()
        .unwrap()
        .v2
        .expect("v2 snimak ima katalog v2")
}

fn counts(cat: &Catalog, profile: Profile, kind: PeriodKind) -> Vec<u8> {
    let state = HotelState::from_profile(cat, profile).unwrap();
    season_periods(kind)
        .iter()
        .map(|p| situation_count(&state, p, cat).unwrap())
        .collect()
}

#[test]
fn weekly_thresholds_are_75_and_90_percent() {
    let cases = [
        (0, 3),
        (7_499, 3),
        (7_500, 4),
        (8_999, 4),
        (9_000, 5),
        (10_000, 5),
    ];
    for (occ, n) in cases {
        assert_eq!(
            SituationCounts::WEEK.for_occupancy(occ),
            n,
            "popunjenost {occ} bp"
        );
    }
}

/// Tablica §3 („Tipični tjedni"): početna ocjena profila, krugovi t01–t17; zbroj 68 / 64 / 65.
#[test]
fn weekly_table_for_profiles_a_b_c() {
    let cat = catalog();
    #[rustfmt::skip]
    let expected: [(Profile, [u8; 17], u32); 3] = [
        (Profile::A, [3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 4, 4, 4, 4], 68),
        (Profile::B, [3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 4, 3, 3, 3], 64),
        (Profile::C, [3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 4, 3, 3, 3], 65),
    ];
    for (profile, weeks, total) in expected {
        let got = counts(&cat, profile, PeriodKind::Week);
        assert_eq!(got, weeks, "profil {}", profile.code());
        assert_eq!(got.iter().map(|&n| u32::from(n)).sum::<u32>(), total);
    }
}

/// Mjesečni krug: 4 / 5 / 6 po istim pragovima kao tjedni, bez obzira na broj dana mjeseca (ADR-042 t. 13).
#[test]
fn monthly_count_is_four_five_six() {
    for (occ, n) in [
        (0, 4),
        (7_000, 4),
        (7_499, 4),
        (7_500, 5),
        (8_000, 5),
        (8_999, 5),
        (9_000, 6),
        (9_500, 6),
    ] {
        assert_eq!(
            SituationCounts::MONTH.for_occupancy(occ),
            n,
            "popunjenost {occ} bp"
        );
    }
    // Isti mjeseci kao prije (jun 13, jul-kol 18/22, sep 17 ili 13): razina popunjenosti je ista, mijenja se samo broj.
    let cat = catalog();
    for (profile, months) in [
        (Profile::A, [4, 5, 6, 5]),
        (Profile::B, [4, 5, 6, 4]),
        (Profile::C, [4, 5, 6, 4]),
    ] {
        assert_eq!(
            counts(&cat, profile, PeriodKind::Month),
            months,
            "profil {}",
            profile.code()
        );
    }
}

/// Broj gleda samo popunjenost: ni najbolja ni najgora ocjena ne izbacuje tjedni krug iz 3..=5.
#[test]
fn weekly_count_stays_between_3_and_5_for_any_rating() {
    let cat = catalog();
    for profile in [Profile::A, Profile::B, Profile::C] {
        let mut state = HotelState::from_profile(&cat, profile).unwrap();
        for rating in [0, 500, 840, 1_000] {
            state.rating = rating;
            for p in season_periods(PeriodKind::Week) {
                let occ = expected_occupancy_bp(&state, &p, &cat).unwrap();
                assert!(occ <= 10_000, "popunjenost najviše 100 %");
                let n = situation_count(&state, &p, &cat).unwrap();
                assert!((3..=5).contains(&n), "{} ocjena {rating}: {n}", p.code());
            }
        }
    }
}

/// Stanje hotela (moral, blagajna, stanje prostora) ne mijenja broj, samo popunjenost.
#[test]
fn count_does_not_depend_on_hotel_condition() {
    let cat = catalog();
    let state = HotelState::from_profile(&cat, Profile::C).unwrap();
    let mut worse = state.clone();
    worse.morale_pts = 0;
    worse.cash_cents = -50_000_000;
    worse.satisfaction_pts = 0;
    for d in worse.departments.values_mut() {
        d.condition_pts = 0;
    }
    for p in season_periods(PeriodKind::Week) {
        assert_eq!(
            situation_count(&state, &p, &cat).unwrap(),
            situation_count(&worse, &p, &cat).unwrap()
        );
    }
}
