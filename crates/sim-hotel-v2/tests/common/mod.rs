// Kopija `crates/sim-hotel/tests/common/mod.rs` (testni v1 katalog) uz situacije za ekvivalenciju v1 ↔ v2.
#![allow(dead_code)]

use sim_hotel::*;
use std::collections::BTreeMap;

pub fn profile_c() -> ProfileSpec {
    ProfileSpec {
        category: Category::Four,
        rooms: 120,
        permanent: 30,
        seasonal: 20,
        wage_pct: 100,
        morale_pts: 5_500,
        experience_pts: 6_000,
        equipment_pts: 6_000,
        satisfaction_pts: 7_000,
        rating: 840,
        cash_cents: 30_000_000,
        fixed_cents: 12_000_000,
        load_hundredths: 173,
    }
}

pub fn profile_a() -> ProfileSpec {
    ProfileSpec {
        rating: 910,
        satisfaction_pts: 7_800,
        morale_pts: 4_200,
        load_hundredths: 195,
        cash_cents: 12_000_000,
        fixed_cents: 15_000_000,
        ..profile_c()
    }
}

pub fn profile_b() -> ProfileSpec {
    ProfileSpec {
        rating: 800,
        equipment_pts: 4_000,
        experience_pts: 5_500,
        cash_cents: 60_000_000,
        wage_pct: 97,
        morale_pts: 5_800,
        fixed_cents: 9_000_000,
        ..profile_c()
    }
}

pub fn test_catalog() -> Catalog {
    let four = CategorySpec {
        price_ref_cents: MonthTable([13_500, 16_500, 17_500, 13_500]),
        base_occupancy_bp: MonthTable([7_200, 7_800, 8_300, 7_300]),
        seasonal_plan: MonthTable([20, 20, 20, 20]),
        elasticity_bp: -8_000,
        wage_cents: 220_000,
        cpor_cents: 2_200,
    };
    Catalog {
        categories: BTreeMap::from([(Category::Four, four)]),
        economy: Economy {
            other_revenue_bp: 3_000,
            hire_cents: 150_000,
            interest_bp: 100,
            marketing_half_cents: 1_500_000,
            marketing_max_bp: 1_500,
            goppar_floor_cents: 1_000,
        },
        actions: ActionSpec {
            maintenance_deferred: LevelEffect {
                cost_cents: 300_000,
                equipment_pts: 0,
            },
            maintenance_regular: LevelEffect {
                cost_cents: 1_500_000,
                equipment_pts: 400,
            },
            maintenance_intensive: LevelEffect {
                cost_cents: 3_000_000,
                equipment_pts: 800,
            },
            training_basic: TrainingEffect {
                cost_cents: 500_000,
                experience_pts: 400,
                morale_pts: 200,
            },
            training_full: TrainingEffect {
                cost_cents: 1_000_000,
                experience_pts: 700,
                morale_pts: 200,
            },
            renovation_cents: 15_000_000,
            renovation_equipment_pts: 2_500,
            renovation_satisfaction_pts: 300,
            reply_cents: 150_000,
            reply_rating: 5,
            price_min_pct: 60,
            price_max_pct: 150,
            marketing_max_cents: 4_000_000,
            seasonal_max: 40,
            seasonal_step: 10,
            wage_min_pct: 90,
            wage_max_pct: 115,
            release_morale_pts: -300,
        },
        mechanics: MechanicSpec {
            recovery_pts_per_q: 200,
            retention_pts_per_q: 300,
            crisis_pts_per_q: 300,
            repair_urgent_cents: 25_000,
            repair_temporary_cents: 8_000,
            repair_temporary_equipment_pts: -100,
            satisfaction_bound_pts: 800,
            morale_bound_pts: 800,
            rating_bound: 15,
            group_good_quality_cut_bp: 500,
        },
        profiles: BTreeMap::from([
            (Profile::A, profile_a()),
            (Profile::B, profile_b()),
            (Profile::C, profile_c()),
        ]),
        reference_profiles: BTreeMap::from([(Category::Four, Profile::C)]),
    }
}

pub fn state_c() -> HotelState {
    HotelState::from_profile(&profile_c()).expect("profil C je valjan")
}

/// Retci objašnjenja moraju stvarno objasniti promjenu (nema odsječenja u ovim scenarijima).
pub fn assert_lines_explain(r: &MonthReport) {
    let total = |v: Var| {
        r.lines
            .iter()
            .filter(|l| l.var == v)
            .map(|l| l.amount)
            .sum::<i64>()
    };
    let (b, a) = (&r.before, &r.after);
    assert_eq!(total(Var::Costs), r.money.costs_cents);
    assert_eq!(
        total(Var::Revenue),
        r.money.rooms_revenue_cents + r.money.other_revenue_cents
    );
    assert_eq!(total(Var::Cash), a.cash_cents - b.cash_cents);
    assert_eq!(
        total(Var::Experience),
        i64::from(a.experience_pts - b.experience_pts)
    );
    assert_eq!(
        total(Var::Equipment),
        i64::from(a.equipment_pts - b.equipment_pts)
    );
    assert_eq!(
        total(Var::Satisfaction),
        i64::from(a.satisfaction_pts - b.satisfaction_pts)
    );
    assert_eq!(
        total(Var::Rating),
        i64::from(a.rating) - i64::from(b.rating)
    );
    let morale_baseline: i64 = r
        .lines
        .iter()
        .filter(|l| l.var == Var::Morale && l.cause == Cause::Baseline)
        .map(|l| l.amount)
        .sum();
    assert_eq!(morale_baseline, i64::from(a.morale_pts - b.morale_pts));
    let staff_change =
        i64::from(r.people.hires) - i64::from(r.people.released) + i64::from(r.people.departures);
    assert_eq!(total(Var::Staff), staff_change);
}

/// Dvije situacije za ekvivalenciju: oporavak gosta u lipnju i grupa za kolovoz prihvaćena u lipnju.
pub fn test_situations() -> Vec<SituationTemplate> {
    vec![
        SituationTemplate {
            code: "T1".into(),
            month: Month::Jun,
            title: "Pritužba".into(),
            answers: vec![
                OfferedAnswer {
                    label: "Isprika".into(),
                    mechanics: vec![Mechanic::ServiceRecovery {
                        compensation_cents: 5_000,
                    }],
                    quality: 4,
                },
                OfferedAnswer {
                    label: "Ništa".into(),
                    mechanics: vec![Mechanic::ServiceRecovery {
                        compensation_cents: 0,
                    }],
                    quality: 1,
                },
            ],
            unanswered: vec![Mechanic::ServiceRecovery {
                compensation_cents: 0,
            }],
        },
        SituationTemplate {
            code: "T2".into(),
            month: Month::Jun,
            title: "Grupa".into(),
            answers: vec![
                OfferedAnswer {
                    label: "Da".into(),
                    mechanics: vec![Mechanic::GroupBooking {
                        accept: true,
                        rooms: 20,
                        nights: 5,
                        discount_bp: 1_500,
                        month: Month::Aug,
                    }],
                    quality: 3,
                },
                OfferedAnswer {
                    label: "Ne".into(),
                    mechanics: vec![Mechanic::GroupBooking {
                        accept: false,
                        rooms: 20,
                        nights: 5,
                        discount_bp: 1_500,
                        month: Month::Aug,
                    }],
                    quality: 3,
                },
            ],
            unanswered: vec![Mechanic::GroupBooking {
                accept: false,
                rooms: 20,
                nights: 5,
                discount_bp: 1_500,
                month: Month::Aug,
            }],
        },
    ]
}

// ---- Task 11: katalog s odjelima (brojke iz content/hotel/v2/departments.json i profiles.json, prepisane u kod —
// ovaj paket ne ovisi o `content`; raspodjela osoblja prilagođena testnom v1 katalogu).

use sim_hotel_v2::department::{DeptCode, DeptSpec, RevenueModel, UndistributedKind, UsaliClass};

fn levels(costs: [i64; 3], pts: [i32; 3]) -> [LevelEffect; 3] {
    [0, 1, 2].map(|i| LevelEffect {
        cost_cents: costs[i],
        equipment_pts: pts[i],
    })
}

/// Odjel bez stalnog mjesečnog troška i troška po prodanoj sobi.
#[allow(clippy::too_many_arguments)]
fn operated(
    name: &str,
    class: UsaliClass,
    revenue: RevenueModel,
    cost_of_sales_bp: u16,
    direct_per_use_cents: i64,
    wage_cents: i64,
    capacity: u16,
    wear: i32,
    maintenance: [LevelEffect; 3],
    seasonal_max: u16,
    seasonal_plan: [u16; 4],
    can_close: bool,
) -> DeptSpec {
    DeptSpec {
        name: name.into(),
        class,
        revenue,
        cost_of_sales_bp,
        direct_per_use_cents,
        wage_cents,
        capacity_tenths_per_staff_day: capacity,
        wear_pts_per_month: wear,
        maintenance,
        fixed_monthly_cents: 0,
        per_occupied_room_cents: 0,
        seasonal_max,
        seasonal_plan: MonthTable(seasonal_plan),
        can_close,
        staffing: BTreeMap::new(),
    }
}

fn per_guest(usage: [u16; 4], check: i64, price_el: i32, sat_el: i32) -> RevenueModel {
    RevenueModel::PerGuest {
        usage_bp: MonthTable(usage),
        check_cents: check,
        price_elasticity_bp: price_el,
        satisfaction_elasticity_bp: sat_el,
    }
}

fn undistributed_spec(name: &str, kind: UndistributedKind, wage: i64, per_room: i64) -> DeptSpec {
    DeptSpec {
        per_occupied_room_cents: per_room,
        ..operated(
            name,
            UsaliClass::Undistributed(kind),
            RevenueModel::None,
            0,
            0,
            wage,
            0,
            0,
            levels([0; 3], [0; 3]),
            0,
            [0; 4],
            false,
        )
    }
}

fn dept_start(
    permanent: u16,
    seasonal: u16,
    condition: i32,
    satisfaction: i32,
) -> sim_hotel_v2::DeptStart {
    sim_hotel_v2::DeptStart {
        name: None,
        open: true,
        permanent,
        seasonal,
        condition_pts: condition,
        satisfaction_pts: satisfaction,
        fixed_monthly_cents: None,
    }
}

fn undistributed_start(permanent: u16, fixed: i64) -> sim_hotel_v2::DeptStart {
    sim_hotel_v2::DeptStart {
        fixed_monthly_cents: Some(fixed),
        ..dept_start(permanent, 0, 10_000, 0)
    }
}

fn dept_code(s: &str) -> DeptCode {
    DeptCode(s.into())
}

/// `degenerate(&test_catalog())` s odjelima iz sadržaja: restoran, lobby bar, spa, parking, rooftop bar i četiri
/// neraspoređena (bez degeneriranog `ostalo`). Profili C i A: raspodjela osoblja iz `profiles.json`, razliku do v1
/// testnog profila (30 stalnih, 20 sezonaca) nosi Smještaj; profil B ima sve osoblje u Smještaju.
pub fn catalog_with_departments() -> sim_hotel_v2::Catalog {
    let mut c = sim_hotel_v2::degenerate(&test_catalog());
    for k in ["ostalo", "admin", "sales", "maintenance", "utilities"] {
        c.departments.remove(&dept_code(k));
    }
    let fb = UsaliClass::FoodBeverage;
    c.departments.extend([
        (
            dept_code("restaurant"),
            operated(
                "Restoran",
                fb,
                per_guest([4_500, 4_800, 5_000, 4_500], 3_200, -15_000, 5_000),
                3_200,
                0,
                210_000,
                50,
                300,
                levels([50_000, 400_000, 800_000], [0, 300, 600]),
                20,
                [10, 12, 15, 12],
                false,
            ),
        ),
        (
            dept_code("lobby_bar"),
            operated(
                "Lobby bar",
                fb,
                per_guest([4_000, 4_200, 4_500, 4_000], 1_200, -12_000, 4_000),
                2_500,
                0,
                190_000,
                120,
                250,
                levels([20_000, 150_000, 300_000], [0, 250, 500]),
                8,
                [3, 4, 5, 4],
                true,
            ),
        ),
        (
            dept_code("rooftop_bar"),
            operated(
                "Rooftop bar",
                fb,
                per_guest([1_500, 2_500, 3_000, 1_500], 2_200, -15_000, 6_000),
                2_800,
                0,
                200_000,
                100,
                350,
                levels([30_000, 200_000, 400_000], [0, 350, 700]),
                6,
                [2, 3, 4, 2],
                true,
            ),
        ),
        (
            dept_code("spa"),
            operated(
                "Wellness i spa",
                UsaliClass::OtherOperated,
                per_guest([800, 900, 1_000, 900], 4_500, -10_000, 6_000),
                1_200,
                300,
                230_000,
                60,
                250,
                levels([30_000, 250_000, 500_000], [0, 250, 500]),
                6,
                [1, 2, 2, 1],
                true,
            ),
        ),
        (
            dept_code("parking"),
            operated(
                "Parking",
                UsaliClass::Rentals,
                RevenueModel::Fixed {
                    monthly_cents: 0,
                    per_guest_night_cents: 150,
                },
                0,
                0,
                0,
                0,
                100,
                levels([0, 50_000, 100_000], [0, 100, 200]),
                0,
                [0; 4],
                false,
            ),
        ),
        (
            dept_code("admin"),
            undistributed_spec("Uprava", UndistributedKind::AdminGeneral, 300_000, 0),
        ),
        (
            dept_code("sales"),
            undistributed_spec(
                "Prodaja i marketing",
                UndistributedKind::SalesMarketing,
                280_000,
                0,
            ),
        ),
        (
            dept_code("maintenance"),
            undistributed_spec("Održavanje", UndistributedKind::Maintenance, 240_000, 0),
        ),
        (
            dept_code("energy"),
            undistributed_spec("Energija i voda", UndistributedKind::Utilities, 0, 600),
        ),
    ]);
    let charges = |i: i64, l: i64, t: i64| {
        BTreeMap::from([
            ("insurance".to_string(), i),
            ("lease".to_string(), l),
            ("property_tax".to_string(), t),
        ])
    };
    let base = test_catalog();
    let rooms = |p: Profile, permanent: u16, seasonal: u16| {
        let v1 = base.profile(p).unwrap();
        dept_start(permanent, seasonal, v1.equipment_pts, v1.satisfaction_pts)
    };
    let profile_c = sim_hotel_v2::ProfileV2 {
        departments: BTreeMap::from([
            (dept_code("rooms"), rooms(Profile::C, 12, 6)),
            (dept_code("restaurant"), dept_start(8, 10, 6_500, 7_000)),
            (dept_code("lobby_bar"), dept_start(2, 3, 6_500, 7_000)),
            (
                dept_code("spa"),
                sim_hotel_v2::DeptStart {
                    name: Some("Mali wellness".into()),
                    ..dept_start(2, 1, 6_500, 7_000)
                },
            ),
            (dept_code("parking"), dept_start(0, 0, 6_500, 7_000)),
            (dept_code("admin"), undistributed_start(2, 3_000_000)),
            (dept_code("sales"), undistributed_start(1, 1_000_000)),
            (dept_code("maintenance"), undistributed_start(3, 1_500_000)),
            (dept_code("energy"), undistributed_start(0, 2_500_000)),
        ]),
        fixed_charges_cents: charges(800_000, 4_000_000, 700_000),
    };
    let profile_a = sim_hotel_v2::ProfileV2 {
        departments: BTreeMap::from([
            (dept_code("rooms"), rooms(Profile::A, 13, 5)),
            (dept_code("restaurant"), dept_start(7, 9, 7_800, 8_200)),
            (dept_code("lobby_bar"), dept_start(2, 2, 7_600, 8_000)),
            (dept_code("spa"), dept_start(3, 2, 8_000, 8_400)),
            (dept_code("rooftop_bar"), dept_start(0, 2, 8_200, 8_500)),
            (dept_code("parking"), dept_start(0, 0, 7_000, 7_000)),
            (dept_code("admin"), undistributed_start(2, 3_300_000)),
            (dept_code("sales"), undistributed_start(1, 1_200_000)),
            (dept_code("maintenance"), undistributed_start(2, 1_800_000)),
            (dept_code("energy"), undistributed_start(0, 2_800_000)),
        ]),
        fixed_charges_cents: charges(900_000, 4_500_000, 800_000),
    };
    let profile_b = sim_hotel_v2::ProfileV2 {
        departments: BTreeMap::from([
            (dept_code("rooms"), rooms(Profile::B, 30, 20)),
            (dept_code("admin"), undistributed_start(0, 2_900_000)),
            (dept_code("sales"), undistributed_start(0, 800_000)),
            (dept_code("maintenance"), undistributed_start(0, 1_400_000)),
            (dept_code("energy"), undistributed_start(0, 2_400_000)),
        ]),
        fixed_charges_cents: charges(800_000, 4_000_000, 700_000),
    };
    c.profiles = BTreeMap::from([
        (Profile::A, profile_a),
        (Profile::B, profile_b),
        (Profile::C, profile_c),
    ]);
    c.economy = sim_hotel_v2::EconomyV2 {
        groups: Default::default(),
        guests_per_room_hundredths: 180,
        price_level_min_pct: 90,
        price_level_max_pct: 120,
        fixed_charges: [
            ("insurance", "Osiguranje"),
            ("lease", "Najam zgrade i opreme"),
            ("property_tax", "Porez na imovinu"),
        ]
        .into_iter()
        .map(|(k, n)| {
            (
                k.to_string(),
                sim_hotel_v2::FixedChargeSpec { name: n.into() },
            )
        })
        .collect(),
    };
    c
}

/// Zadani plan u kojem svaki odjel zadržava današnje sezonce (Σ|Δ| = 0): plan koji uvijek prolazi korak.
pub fn hold_plan(
    s: &sim_hotel_v2::HotelState,
    p: &sim_hotel_v2::Period,
    c: &sim_hotel_v2::Catalog,
) -> sim_hotel_v2::PeriodPlan {
    let mut plan = sim_hotel_v2::PeriodPlan::default_for(s, p, c).unwrap();
    for (k, d) in plan.departments.iter_mut() {
        d.seasonal_target = s.departments[k].seasonal;
    }
    plan
}
