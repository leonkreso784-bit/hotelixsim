// Cijena parkinga u sadržaju (obnova v2, Task O-P; Leonova anketa 13.10. (3)): parking ima prihod po gostu — 12 %
// noćenja dolazi autom, 12 € po danu, skuplje = manje auta (−10 000 bp), zadovoljstvo ne utječe (0) — i razinu cijene
// kao restoran. Lokal ostaje na stalnoj najamnini bez cijene.
use content::{Snapshot, repo_dir};
use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code};
use sim_hotel_v2::department::{DeptCode, RevenueModel, UsaliClass};
use sim_hotel_v2::{
    Catalog, HotelState, Month, MonthTable, Period, PeriodPlan, Profile, resolve_period,
};

fn v2() -> Catalog {
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

#[test]
fn parking_has_per_guest_revenue_in_content() {
    let c = v2();
    c.validate().unwrap();
    let p = c.dept(&code("parking")).unwrap();
    assert_eq!(p.class, UsaliClass::Rentals);
    assert_eq!(
        p.revenue,
        RevenueModel::PerGuest {
            usage_bp: MonthTable([1_200, 1_200, 1_200, 1_200]),
            check_cents: 1_200,
            price_elasticity_bp: -10_000,
            satisfaction_elasticity_bp: 0,
        }
    );
    assert_eq!(p.capacity_tenths_per_staff_day, 0);
    assert!(matches!(
        c.dept(&code("retail_lease")).unwrap().revenue,
        RevenueModel::Fixed { .. }
    ));
}

/// (parking, noćenja, auta, prihod) prvog kruga profila uz zadani plan i razinu cijene parkinga `level`.
fn first_round(c: &Catalog, profile: Profile, p: &Period, level: u16) -> (u32, u32, i64) {
    let s = HotelState::from_profile(c, profile).unwrap();
    let mut plan = PeriodPlan::default_for(&s, p, c).unwrap();
    plan.departments
        .get_mut(&code("parking"))
        .unwrap()
        .price_level_pct = level;
    let r = resolve_period(&s, p, &plan, &[], c).unwrap();
    let nights = r.accounts.departments[&DeptCode::rooms()].guest_weight;
    let o = &r.accounts.departments[&code("parking")];
    assert_eq!(o.guest_weight, 0, "najam ne ulazi u zadovoljstvo hotela");
    (nights, o.served_uses, o.revenue_cents)
}

/// Početni prihod parkinga je 12 % noćenja × 12,00 € ≈ 1,44 € po noćenju (do zaokruživanja broja auta), za svaki
/// profil, tjedno i mjesečno.
#[test]
fn starting_parking_revenue_is_about_one_euro_forty_four_per_night() {
    let c = v2();
    let periods = [
        period_by_code(PeriodKind::Week, "t01").unwrap(),
        month_period(Month::Jun),
    ];
    for profile in [Profile::A, Profile::B, Profile::C] {
        for p in &periods {
            let (nights, cars, revenue) = first_round(&c, profile, p, 100);
            assert!(nights > 0);
            assert_eq!(cars, (nights * 1_200 + 5_000) / 10_000);
            assert_eq!(revenue, i64::from(cars) * 1_200);
            let gap = (revenue - i64::from(nights) * 144).abs();
            assert!(gap <= 600, "{profile:?} {p:?}: {revenue} za {nights}");
        }
    }
}

/// Razina cijene parkinga mijenja broj auta i prihod; granice razine parkinga su kao restoranove, lokal i odjeli bez
/// prihoda po gostu imaju 100–100.
#[test]
fn parking_price_level_changes_cars_and_revenue() {
    let c = v2();
    let p = period_by_code(PeriodKind::Week, "t01").unwrap();
    let base = first_round(&c, Profile::C, &p, 100);
    let dear = first_round(&c, Profile::C, &p, 120);
    let cheap = first_round(&c, Profile::C, &p, 90);
    assert_eq!(dear.0, base.0, "noćenja ne ovise o cijeni parkinga");
    assert!(dear.1 < base.1 && base.1 < cheap.1);
    assert_eq!(dear.1, (base.0 * 960 + 5_000) / 10_000);
    assert_eq!(dear.2, i64::from(dear.1) * 1_440);
    assert_eq!(cheap.2, i64::from(cheap.1) * 1_080);
    assert_ne!(dear.2, base.2);

    // Profil B ima i lokal.
    let s = HotelState::from_profile(&c, Profile::B).unwrap();
    let l = PeriodPlan::limits(&s, &p, &c).unwrap();
    let range = |k: &str| {
        let d = l.departments[&code(k)];
        (d.price_level_min_pct, d.price_level_max_pct)
    };
    assert_eq!(range("parking"), range("restaurant"));
    assert_eq!(range("parking"), (90, 120));
    for k in [
        "rooms",
        "retail_lease",
        "admin",
        "sales",
        "maintenance",
        "energy",
    ] {
        assert_eq!(range(k), (100, 100), "{k}");
    }
}
