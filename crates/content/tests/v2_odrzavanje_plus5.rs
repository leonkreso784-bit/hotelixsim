// L1-13b dio 4 (ADR-042 t. 2, Leonove odluke 19.10.): pojačano održavanje podiže stanje neto +5 bodova mjesečno
// (prije dodatnog trošenja zbog gužve), uz ISTU cijenu; parking i lokal ostaju kakvi jesu.
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

fn code(s: &str) -> DeptCode {
    DeptCode(s.to_string())
}

/// Šest operativnih odjela koji dobivaju +5 neto: ključ, trošenje (stotinke boda/mj) i cijena pojačanog (nepromijenjena).
const SIX: [(&str, i32, i64); 6] = [
    ("rooms", 400, 3_000_000),
    ("restaurant", 300, 800_000),
    ("lobby_bar", 250, 300_000),
    ("rooftop_bar", 350, 400_000),
    ("pool_bar", 450, 300_000),
    ("spa", 250, 500_000),
];

/// `intensive.equipment_pts` = trošenje + 500 (neto +5 bodova mjesečno) za svih šest odjela.
#[test]
fn intensive_nets_plus_five_points_per_month() {
    let c = v2();
    for (k, wear, _) in SIX {
        let d = c.dept(&code(k)).unwrap();
        assert_eq!(d.wear_pts_per_month, wear, "{k}: trošenje se ne mijenja");
        let pts = d.maintenance_level(Maintenance::Intensive).equipment_pts;
        assert_eq!(pts - d.wear_pts_per_month, 500, "{k}: neto pojačano");
    }
}

/// Ciljne brojke doslovno (trošenje + 500): 900, 800, 750, 850, 950, 750.
#[test]
fn intensive_target_points_per_department() {
    let c = v2();
    let want = [
        ("rooms", 900),
        ("restaurant", 800),
        ("lobby_bar", 750),
        ("rooftop_bar", 850),
        ("pool_bar", 950),
        ("spa", 750),
    ];
    for (k, pts) in want {
        let l = c
            .dept(&code(k))
            .unwrap()
            .maintenance_level(Maintenance::Intensive);
        assert_eq!(l.equipment_pts, pts, "{k}");
    }
}

/// Cijena pojačanog ostaje ista (Leon 19.10.: bodova je više za isti novac).
#[test]
fn intensive_price_unchanged() {
    let c = v2();
    for (k, _, cost) in SIX {
        let l = c
            .dept(&code(k))
            .unwrap()
            .maintenance_level(Maintenance::Intensive);
        assert_eq!(l.cost_cents, cost, "{k}");
    }
}

/// Parking i lokal ostaju kakvi jesu (pojačano +3 / +2 boda, trošenje 0,5): +5 bi ih bez razloga dizalo.
#[test]
fn parking_and_lease_intensive_unchanged() {
    let c = v2();
    for (k, cost, pts) in [("parking", 150_000, 300), ("retail_lease", 80_000, 200)] {
        let l = c
            .dept(&code(k))
            .unwrap()
            .maintenance_level(Maintenance::Intensive);
        assert_eq!((l.cost_cents, l.equipment_pts), (cost, pts), "{k}");
    }
}

/// Opis Smještaja (web `dept.rooms.about`) tvrdi: odgođeno ubrzava trošenje, redovno ga samo usporava, pojačano
/// podiže stanje, a hitan popravak vrati više nego što je kvar uzeo. Ovaj test drži sadržaj istinitim prema tekstu.
#[test]
fn rooms_description_stays_true() {
    let c = v2();
    let d = c.dept(&code("rooms")).unwrap();
    let wear = i64::from(d.wear_pts_per_month);
    let f = i64::from(c.economy.deferred_wear_bp);
    let pts = |m| i64::from(d.maintenance_level(m).equipment_pts);
    let deferred = pts(Maintenance::Deferred) - (wear * f + 5_000) / 10_000;
    let regular = pts(Maintenance::Regular) - wear;
    let intensive = pts(Maintenance::Intensive) - wear;
    assert!(
        deferred < regular,
        "odgođeno ubrzava trošenje ({deferred} < {regular})"
    );
    assert!(regular < 0, "redovno ga samo usporava ({regular} < 0)");
    assert!(intensive > 0, "pojačano podiže stanje ({intensive} > 0)");
    assert!(
        c.economy.fault_fixed_condition_pts > c.economy.fault_condition_pts,
        "hitan popravak vrati više nego što je kvar uzeo"
    );
}
