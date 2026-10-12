// Obnova v2 (O-2): brojke obnove u sadržaju `content/hotel/v2/*.json` (obnova-v2-spec §0, O1–O15).
use content::{Snapshot, repo_dir};
use sim_hotel_v2::Maintenance;
use sim_hotel_v2::department::DeptCode;

fn v2() -> sim_hotel_v2::Catalog {
    Snapshot::read_v2(&repo_dir())
        .unwrap()
        .build()
        .unwrap()
        .v2
        .unwrap()
}

/// `engine::calc::div_round` za pozitivne brojeve (pola ide gore).
fn div_round(a: i64, b: i64) -> i64 {
    (2 * a + b) / (2 * b)
}

fn code(s: &str) -> DeptCode {
    DeptCode(s.to_string())
}

#[test]
fn content_v2_with_renewal_numbers_validates() {
    v2().validate().unwrap();
}

/// O7, O8, O11: redovno održavanje svakog odjela koji se troši pokriva 75 % trošenja (`div_round(trošenje·7 500,
/// 10 000)`), odgođeno ne donosi bodove.
#[test]
fn regular_maintenance_covers_three_quarters_of_wear() {
    let c = v2();
    let mut worn = 0;
    for (k, d) in &c.departments {
        if d.wear_pts_per_month == 0 {
            continue;
        }
        worn += 1;
        let regular = d.maintenance_level(Maintenance::Regular).equipment_pts;
        let want = div_round(i64::from(d.wear_pts_per_month) * 7_500, 10_000);
        assert_eq!(i64::from(regular), want, "{}", k.as_str());
        assert_eq!(
            d.maintenance_level(Maintenance::Deferred).equipment_pts,
            0,
            "{}",
            k.as_str()
        );
    }
    // Smještaj, Restoran, Lobby bar, Rooftop bar, Bar uz bazen, Wellness, Parking, Lokal.
    assert_eq!(worn, 8);
}

/// O7, O8, O11 doslovno: redovno nakon spuštanja na 75 % i najmovi.
#[test]
fn maintenance_numbers_match_the_table() {
    let c = v2();
    let level = |k: &str, m: Maintenance| {
        let l = c.dept(&code(k)).unwrap().maintenance_level(m);
        (l.cost_cents, l.equipment_pts)
    };
    let regular = [
        ("rooms", 1_500_000, 300),
        ("restaurant", 400_000, 225),
        ("lobby_bar", 150_000, 188),
        ("rooftop_bar", 200_000, 263),
        ("pool_bar", 150_000, 338),
        ("spa", 250_000, 188),
        ("parking", 50_000, 38),
        ("retail_lease", 30_000, 38),
    ];
    for (k, cost, pts) in regular {
        assert_eq!(level(k, Maintenance::Regular), (cost, pts), "{k}");
    }
    // Najmovi (O8): trošenje 0,5 boda mjesečno; pojačano parking 1.500 €/mj = +3, lokal 800 €/mj = +2.
    for k in ["parking", "retail_lease"] {
        assert_eq!(c.dept(&code(k)).unwrap().wear_pts_per_month, 50, "{k}");
        assert_eq!(level(k, Maintenance::Deferred), (0, 0), "{k}");
    }
    assert_eq!(level("parking", Maintenance::Intensive), (150_000, 300));
    assert_eq!(level("retail_lease", Maintenance::Intensive), (80_000, 200));
    // Bar uz bazen (O7) troši 4,5 boda mjesečno; pojačano ostaje kao prije (O11).
    assert_eq!(c.dept(&code("pool_bar")).unwrap().wear_pts_per_month, 450);
    assert_eq!(level("pool_bar", Maintenance::Intensive), (300_000, 600));
}
