// Mijenja se samo zajedno s `scripts/oracle-jezgra.mjs` — zlatne vrijednosti ovise o tome.
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
