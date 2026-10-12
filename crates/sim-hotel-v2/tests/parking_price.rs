// Cijena parkinga (obnova v2, Task O-P; Leonova anketa 13.10. (3)): parking ima prihod po gostu i razinu cijene kao
// restoran. Odjel bez kapaciteta po zaposlenom nema gornje granice usluženih (opterećenje 0), najam (`rentals`) ne ulazi
// u zadovoljstvo hotela, razina cijene postoji samo za odjele s prihodom po gostu, a nasumični robot parking ne dira.
mod common;

use std::collections::BTreeMap;

use engine::{Rng, label};
use sim_hotel::LevelEffect;
use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code};
use sim_hotel_v2::department::*;
use sim_hotel_v2::{
    Catalog, HotelState, Maintenance, Month, MonthTable, PeriodPlan, Profile, Strategy, robot_plan,
};

fn code(s: &str) -> DeptCode {
    DeptCode(s.into())
}

fn per_guest_parking() -> RevenueModel {
    RevenueModel::PerGuest {
        usage_bp: MonthTable([1_200, 1_200, 1_200, 1_200]),
        check_cents: 1_200,
        price_elasticity_bp: -10_000,
        satisfaction_elasticity_bp: 0,
    }
}

fn parking() -> DeptSpec {
    DeptSpec {
        name: "Parking".into(),
        class: UsaliClass::Rentals,
        revenue: per_guest_parking(),
        cost_of_sales_bp: 0,
        direct_per_use_cents: 0,
        wage_cents: 0,
        capacity_tenths_per_staff_day: 0,
        wear_pts_per_month: 50,
        maintenance: [
            LevelEffect {
                cost_cents: 0,
                equipment_pts: 0,
            },
            LevelEffect {
                cost_cents: 50_000,
                equipment_pts: 38,
            },
            LevelEffect {
                cost_cents: 150_000,
                equipment_pts: 300,
            },
        ],
        fixed_monthly_cents: 0,
        per_occupied_room_cents: 0,
        seasonal_max: 0,
        seasonal_plan: MonthTable([0, 0, 0, 0]),
        can_close: false,
        staffing: BTreeMap::new(),
        renewal: Default::default(),
        guest_share_bp: 500,
        rent_condition: None,
    }
}

fn june(level: u16) -> DeptOutcome {
    let p = month_period(Month::Jun);
    let st = DeptState {
        class: UsaliClass::Rentals,
        open: true,
        permanent: 0,
        seasonal: 0,
        condition_pts: 6_500,
        satisfaction_pts: 7_000,
        season_departures: 0,
        start_condition_pts: 6_500,
        book: Default::default(),
    };
    dept_step(&DeptInput {
        code: code("parking"),
        period: &p,
        spec: &parking(),
        state: &st,
        plan: &DeptPlan {
            open: true,
            seasonal_target: 0,
            price_level_pct: level,
            maintenance: Maintenance::Regular,
            renewal_cents: 0,
        },
        guest_nights: 4_666,
        rooms_revenue_cents: 35_000_000,
        experience_pts: 6_000,
        morale_pts: 5_500,
        wage_pct: 100,
        satisfaction_step_bp: 5_000,
        renewal: Default::default(),
        building_pts: 0,
        building_share_bp: 0,
        deferred_wear_bp: 10_000,
    })
    .unwrap()
}

/// Ručno: 4 666 noćenja × 12 % = div_round(5 599 200, 10 000) = 560 auta; bez ljudi i bez kapaciteta po zaposlenom
/// parking nema gornje granice, opterećenje 0; prihod 560 × 12,00 € = 6 720,00 € (≈ 1,44 € po noćenju).
/// Najam ne ulazi u zadovoljstvo hotela (težina 0), njegovo zadovoljstvo stoji.
#[test]
fn parking_without_staff_capacity_serves_every_car_by_hand() {
    let o = june(100);
    assert_eq!(
        (o.usage_bp, o.demand_uses, o.capacity_uses, o.served_uses),
        (1_200, 560, 0, 560)
    );
    assert_eq!((o.load_hundredths, o.load_wear_pts), (0, 0));
    assert_eq!(o.revenue_cents, 672_000);
    assert_eq!(
        (o.payroll_cents, o.cost_of_sales_cents, o.other_direct_cents),
        (0, 0, 0)
    );
    assert_eq!(o.profit_cents, 672_000);
    assert_eq!(o.guest_weight, 0);
    assert_eq!(o.satisfaction_after_pts, 7_000);
}

/// Razina 110 %: udio 12 % × (1 − 10 %) = 10,8 % → div_round(4 666 · 1 080, 10 000) = 504 auta po 13,20 €;
/// razina 90 %: 13,2 % → 616 auta po 10,80 €. Skuplje = manje auta, prihod se mijenja s razinom.
#[test]
fn parking_price_level_changes_revenue_and_cars() {
    let up = june(110);
    assert_eq!((up.usage_bp, up.served_uses), (1_080, 504));
    assert_eq!(up.revenue_cents, 504 * 1_320);
    let down = june(90);
    assert_eq!((down.usage_bp, down.served_uses), (1_320, 616));
    assert_eq!(down.revenue_cents, 616 * 1_080);
    let up20 = june(120);
    assert_eq!((up20.usage_bp, up20.served_uses), (960, 448));
    assert_eq!(up20.revenue_cents, 448 * 1_440);
    assert_ne!(up20.revenue_cents, june(100).revenue_cents);
}

/// Odjel s kapacitetom po zaposlenom i dalje uslužuje najviše kapacitet (bez izmjene): restoran bez ljudi ne uslužuje.
#[test]
fn staffed_department_without_people_still_serves_nothing() {
    let mut spec = parking();
    spec.capacity_tenths_per_staff_day = 50;
    let p = month_period(Month::Jun);
    let st = DeptState {
        class: UsaliClass::Rentals,
        open: true,
        permanent: 0,
        seasonal: 0,
        condition_pts: 6_500,
        satisfaction_pts: 7_000,
        season_departures: 0,
        start_condition_pts: 6_500,
        book: Default::default(),
    };
    let o = dept_step(&DeptInput {
        code: code("parking"),
        period: &p,
        spec: &spec,
        state: &st,
        plan: &DeptPlan {
            open: true,
            seasonal_target: 0,
            price_level_pct: 100,
            maintenance: Maintenance::Regular,
            renewal_cents: 0,
        },
        guest_nights: 4_666,
        rooms_revenue_cents: 0,
        experience_pts: 6_000,
        morale_pts: 5_500,
        wage_pct: 100,
        satisfaction_step_bp: 5_000,
        renewal: Default::default(),
        building_pts: 0,
        building_share_bp: 0,
        deferred_wear_bp: 10_000,
    })
    .unwrap();
    assert_eq!((o.demand_uses, o.served_uses, o.revenue_cents), (560, 0, 0));
    assert_eq!(o.load_hundredths, 0);
}

/// Katalog s odjelima iz testova; parking po gostu ako `priced`.
fn catalog(priced: bool) -> Catalog {
    let mut c = common::catalog_with_departments();
    if priced {
        let d = c.departments.get_mut(&code("parking")).unwrap();
        d.revenue = per_guest_parking();
        d.capacity_tenths_per_staff_day = 0;
    }
    c
}

fn levels(c: &Catalog) -> BTreeMap<String, (u16, u16)> {
    let s = HotelState::from_profile(c, Profile::C).unwrap();
    let p = period_by_code(PeriodKind::Week, "t01").unwrap();
    PeriodPlan::limits(&s, &p, c)
        .unwrap()
        .departments
        .into_iter()
        .map(|(k, l)| (k.0, (l.price_level_min_pct, l.price_level_max_pct)))
        .collect()
}

/// Razina cijene postoji samo za odjele s prihodom po gostu; Smještaj, stalni najam i neraspoređeni imaju 100–100.
#[test]
fn price_level_exists_only_for_per_guest_departments() {
    let c = catalog(true);
    for (k, range) in levels(&c) {
        let per_guest = matches!(
            c.dept(&code(&k)).unwrap().revenue,
            RevenueModel::PerGuest { .. }
        );
        let want = if per_guest { (90, 120) } else { (100, 100) };
        assert_eq!(range, want, "{k}");
    }
    assert_eq!(levels(&c)["parking"], (90, 120));
    assert_eq!(levels(&catalog(false))["parking"], (100, 100));
}

/// Nasumični robot izvlači razinu cijene samo za hranu i piće i ostale operativne odjele: parking s cijenom ostaje na
/// 100 %, a niz slučajnih brojeva (razine ostalih odjela) isti je kao bez cijene parkinga.
#[test]
fn random_robot_leaves_parking_price_level_alone() {
    let (with, without) = (catalog(true), catalog(false));
    let p = period_by_code(PeriodKind::Week, "t01").unwrap();
    for seed in 0..20_u64 {
        let plan = |c: &Catalog| {
            let s = HotelState::from_profile(c, Profile::C).unwrap();
            let mut rng = Rng::derive(seed, &[label("test:parking")]);
            robot_plan(Strategy::Random, &s, &p, c, &mut rng).unwrap()
        };
        let (a, b) = (plan(&with), plan(&without));
        assert_eq!(a.departments[&code("parking")].price_level_pct, 100);
        assert_eq!(a, b, "sjeme {seed}");
    }
}
