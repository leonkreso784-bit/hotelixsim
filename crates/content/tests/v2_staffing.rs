// Norma osoblja po klasi hotela i broju soba (Task 23b): Smještaj, Uprava, Prodaja i marketing i Održavanje imaju
// raspon ljudi na 100 soba (u desetinkama) po kategoriji; početni ljudi profila moraju biti u rasponu.
use content::{Snapshot, repo_dir};
use sim_hotel::CatalogError;
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::{Category, Profile, StaffingNorm};

fn v2() -> sim_hotel_v2::Catalog {
    Snapshot::read_v2(&repo_dir())
        .unwrap()
        .build()
        .unwrap()
        .v2
        .unwrap()
}

fn code(s: &str) -> DeptCode {
    DeptCode(s.into())
}

fn norm(min: u16, max: u16) -> StaffingNorm {
    StaffingNorm {
        min_tenths_per_100_rooms: min,
        max_tenths_per_100_rooms: max,
    }
}

fn out_of_range(label: &str) -> Result<(), CatalogError> {
    Err(CatalogError::OutOfRange(label.to_string()))
}

/// Profil A postaje hotel kategorije 5 (v1 dio kategorije 5 je kopija kategorije 4, a A je njen referentni
/// profil, da v1 provjera prolazi).
fn as_five_star(c: &mut sim_hotel_v2::Catalog) {
    let four = c.base.categories[&Category::Four].clone();
    c.base.categories.insert(Category::Five, four);
    c.base.reference_profiles.insert(Category::Five, Profile::A);
    c.base.profiles.get_mut(&Profile::A).unwrap().category = Category::Five;
}

#[test]
fn todays_profiles_are_within_the_four_star_norm() {
    let c = v2();
    assert_eq!(c.validate(), Ok(()));
    for (dept, min, max) in [
        ("rooms", 170, 250),
        ("admin", 50, 70),
        ("sales", 15, 30),
        ("maintenance", 30, 60),
    ] {
        assert_eq!(
            c.dept(&code(dept)).unwrap().staffing.get(&Category::Four),
            Some(&norm(min, max)),
            "{dept}"
        );
    }
    // Hrana i piće, ostali operativni, najam i Energija nemaju normu.
    for dept in ["restaurant", "lobby_bar", "spa", "parking", "energy"] {
        assert!(c.dept(&code(dept)).unwrap().staffing.is_empty(), "{dept}");
    }
}

#[test]
fn hotel_class_changes_the_outcome() {
    // Isti ljudi (Smještaj 24 na 120 soba) prolaze za 4*, a za 5* s višom normom Smještaja (26–35 na 100 soba,
    // brojka samo za test) trebalo bi barem 31,2 → odbijeno.
    let mut c = v2();
    for dept in ["rooms", "admin", "sales", "maintenance"] {
        let d = c.departments.get_mut(&code(dept)).unwrap();
        let row = if dept == "rooms" {
            norm(260, 350)
        } else {
            d.staffing[&Category::Four].clone()
        };
        d.staffing.insert(Category::Five, row);
    }
    assert_eq!(c.validate(), Ok(()));
    as_five_star(&mut c);
    assert_eq!(
        c.validate(),
        out_of_range("v2.profiles.A.departments.rooms.staffing")
    );
}

#[test]
fn hotel_size_changes_the_outcome() {
    // Profil A: Smještaj 24, Uprava 7, Prodaja 3, Održavanje 6. Na 100 soba sve je u normi 4* (Uprava 5–7);
    // na 60 soba Uprava smije imati najviše 4,2 → 7 je previše (prvi odjel s normom po kodu je `admin`).
    let mut c = v2();
    c.base.profiles.get_mut(&Profile::A).unwrap().rooms = 100;
    assert_eq!(c.validate(), Ok(()));
    c.base.profiles.get_mut(&Profile::A).unwrap().rooms = 60;
    assert_eq!(
        c.validate(),
        out_of_range("v2.profiles.A.departments.admin.staffing")
    );
}

#[test]
fn missing_row_for_the_hotel_class_is_a_content_error() {
    let mut c = v2();
    as_five_star(&mut c);
    assert_eq!(c.validate(), out_of_range("v2.departments.admin.staffing"));
}

#[test]
fn norm_must_be_a_positive_range() {
    for bad in [norm(250, 170), norm(0, 250), norm(0, 0)] {
        let mut c = v2();
        c.departments
            .get_mut(&code("rooms"))
            .unwrap()
            .staffing
            .insert(Category::Four, bad.clone());
        assert_eq!(
            c.validate(),
            out_of_range("v2.departments.rooms.staffing"),
            "{bad:?}"
        );
    }
}

#[test]
fn degenerate_catalog_has_no_norm_and_still_validates() {
    let base = Snapshot::read(&repo_dir())
        .unwrap()
        .build()
        .unwrap()
        .catalog;
    let d = sim_hotel_v2::degenerate(&base);
    assert!(d.departments.values().all(|s| s.staffing.is_empty()));
    assert_eq!(d.validate(), Ok(()));
}
