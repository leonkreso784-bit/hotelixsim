use std::collections::BTreeMap;

use sim_hotel::LevelEffect;
use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code};
use sim_hotel_v2::department::*;
use sim_hotel_v2::{Maintenance, Month, MonthTable};

fn restaurant() -> DeptSpec {
    DeptSpec {
        name: "Restoran".into(),
        class: UsaliClass::FoodBeverage,
        revenue: RevenueModel::PerGuest {
            usage_bp: MonthTable([4_500, 4_800, 5_000, 4_500]),
            check_cents: 3_200,
            price_elasticity_bp: -15_000,
            satisfaction_elasticity_bp: 5_000,
        },
        cost_of_sales_bp: 3_200,
        direct_per_use_cents: 0,
        wage_cents: 210_000,
        capacity_tenths_per_staff_day: 50,
        wear_pts_per_month: 300,
        maintenance: [
            LevelEffect {
                cost_cents: 50_000,
                equipment_pts: 0,
            },
            LevelEffect {
                cost_cents: 500_000,
                equipment_pts: 300,
            },
            LevelEffect {
                cost_cents: 1_000_000,
                equipment_pts: 600,
            },
        ],
        fixed_monthly_cents: 0,
        per_occupied_room_cents: 0,
        seasonal_max: 20,
        seasonal_plan: MonthTable([10, 12, 15, 12]),
        can_close: false,
    }
}

fn state() -> DeptState {
    DeptState {
        class: UsaliClass::FoodBeverage,
        open: true,
        permanent: 8,
        seasonal: 10,
        condition_pts: 7_000,
        satisfaction_pts: 7_000,
    }
}

fn plan(seasonal: u16, level: u16) -> DeptPlan {
    DeptPlan {
        open: true,
        seasonal_target: seasonal,
        price_level_pct: level,
        maintenance: Maintenance::Regular,
    }
}

fn june(spec: &DeptSpec, st: &DeptState, pl: &DeptPlan) -> DeptOutcome {
    let p = month_period(Month::Jun);
    dept_step(&DeptInput {
        code: DeptCode("restaurant".into()),
        period: &p,
        spec,
        state: st,
        plan: pl,
        guest_nights: 4_666,
        rooms_revenue_cents: 35_000_000,
        experience_pts: 6_000,
        morale_pts: 5_500,
        wage_pct: 100,
        satisfaction_step_bp: 5_000,
    })
    .unwrap()
}

#[test]
fn restaurant_in_june_by_hand() {
    let o = june(&restaurant(), &state(), &plan(10, 100));
    assert_eq!(
        (o.usage_bp, o.demand_uses, o.capacity_uses, o.served_uses),
        (4_500, 2_100, 2_700, 2_100)
    );
    assert_eq!(
        (o.load_hundredths, o.load_penalty_pts, o.quality_pts),
        (187, 700, 6_840)
    );
    assert_eq!(o.revenue_cents, 6_720_000);
    assert_eq!(o.cost_of_sales_cents, 2_150_400);
    assert_eq!(o.payroll_cents, 3_780_000);
    assert_eq!(o.profit_cents, 789_600);
    assert_eq!(
        (
            o.wear_pts,
            o.load_wear_pts,
            o.maintenance_pts,
            o.condition_after_pts
        ),
        (-300, -100, 300, 6_900)
    );
    assert_eq!(o.maintenance_cents, 500_000);
    assert_eq!(
        (
            o.value_pts,
            o.satisfaction_target_pts,
            o.satisfaction_after_pts
        ),
        (7_000, 6_898, 6_949)
    );
    assert_eq!(o.guest_weight, 2_100);
}

#[test]
fn higher_price_level_raises_check_lowers_usage_and_value() {
    let o = june(&restaurant(), &state(), &plan(10, 110));
    assert_eq!((o.usage_bp, o.demand_uses), (3_825, 1_785));
    assert_eq!(o.revenue_cents, 1_785 * 3_520);
    assert_eq!(o.value_pts, 6_000);
}

#[test]
fn understaffed_restaurant_serves_capacity_and_loses_quality() {
    let o = june(&restaurant(), &state(), &plan(0, 100));
    assert_eq!((o.staff, o.capacity_uses, o.served_uses), (8, 1_200, 1_200));
    assert_eq!(
        (o.load_hundredths, o.load_penalty_pts, o.quality_pts),
        (240, 6_000, 5_250)
    );
}

#[test]
fn closed_department_has_no_revenue_but_pays_permanent_staff() {
    let mut spec = restaurant();
    spec.can_close = true;
    let mut pl = plan(0, 100);
    pl.open = false;
    let o = june(&spec, &state(), &pl);
    assert_eq!((o.served_uses, o.revenue_cents, o.guest_weight), (0, 0, 0));
    assert_eq!(o.payroll_cents, 8 * 2_100 * 100);
    assert_eq!(o.satisfaction_after_pts, 7_000);
    assert_eq!(o.condition_after_pts, 7_000 - 300 + 300);
}

#[test]
fn week_two_pays_seven_thirtieths_of_june_payroll() {
    let p = period_by_code(PeriodKind::Week, "t02").unwrap();
    let o = dept_step(&DeptInput {
        code: DeptCode("restaurant".into()),
        period: &p,
        spec: &restaurant(),
        state: &state(),
        plan: &plan(10, 100),
        guest_nights: 1_089,
        rooms_revenue_cents: 0,
        experience_pts: 6_000,
        morale_pts: 5_500,
        wage_pct: 100,
        satisfaction_step_bp: 1_500,
    })
    .unwrap();
    assert_eq!(o.payroll_cents, 3_780_000 * 14 / 30 - 3_780_000 * 7 / 30);
}

#[test]
fn share_of_rooms_and_fixed_revenue_models() {
    let mut s = restaurant();
    s.revenue = RevenueModel::ShareOfRooms { bp: 3_000 };
    s.class = UsaliClass::Rentals;
    let st = DeptState {
        permanent: 0,
        seasonal: 0,
        ..state()
    };
    assert_eq!(june(&s, &st, &plan(0, 100)).revenue_cents, 10_500_000);
    s.revenue = RevenueModel::Fixed {
        monthly_cents: 600_000,
        per_guest_night_cents: 150,
    };
    let o = june(&s, &st, &plan(0, 100));
    assert_eq!(o.revenue_cents, 600_000 + 4_666 * 150);
    assert_eq!(o.guest_weight, 0);
}

#[test]
fn undistributed_admin_by_hand() {
    let mut spec = restaurant();
    spec.class = UsaliClass::Undistributed(UndistributedKind::AdminGeneral);
    spec.revenue = RevenueModel::None;
    spec.wage_cents = 300_000;
    let st = DeptState {
        class: spec.class,
        open: true,
        permanent: 3,
        seasonal: 0,
        condition_pts: 10_000,
        satisfaction_pts: 0,
    };
    let assigned = AssignedCosts {
        training_cents: 500_000,
        hiring_cents: 1_500_000,
        situation_cents: 5_000,
        ..Default::default()
    };
    let o = undistributed_step(
        &DeptCode("admin".into()),
        &spec,
        &st,
        3_000_000,
        &month_period(Month::Jun),
        2_592,
        100,
        assigned,
    )
    .unwrap();
    assert_eq!(
        (o.payroll_cents, o.fixed_cents, o.per_room_cents),
        (900_000, 3_000_000, 0)
    );
    assert_eq!(
        o.total_cents,
        900_000 + 3_000_000 + 500_000 + 1_500_000 + 5_000
    );
}

fn map(xs: &[(&str, u16)]) -> BTreeMap<DeptCode, u16> {
    xs.iter()
        .map(|(k, v)| (DeptCode((*k).into()), *v))
        .collect()
}

#[test]
fn allocate_seasonal_adds_to_largest_gap_then_smaller_code() {
    let cur = map(&[
        ("rooms", 12),
        ("restaurant", 10),
        ("lobby_bar", 3),
        ("spa", 1),
    ]);
    let plan = map(&[
        ("rooms", 16),
        ("restaurant", 12),
        ("lobby_bar", 4),
        ("spa", 2),
    ]);
    let max = map(&[
        ("rooms", 30),
        ("restaurant", 20),
        ("lobby_bar", 8),
        ("spa", 6),
    ]);
    assert_eq!(
        allocate_seasonal(&cur, &plan, &max, 30),
        map(&[
            ("rooms", 15),
            ("restaurant", 11),
            ("lobby_bar", 3),
            ("spa", 1)
        ])
    );
}

#[test]
fn allocate_seasonal_removes_from_largest_surplus() {
    let cur = map(&[
        ("rooms", 12),
        ("restaurant", 10),
        ("lobby_bar", 3),
        ("spa", 1),
    ]);
    let plan = map(&[
        ("rooms", 10),
        ("restaurant", 10),
        ("lobby_bar", 3),
        ("spa", 1),
    ]);
    let max = map(&[
        ("rooms", 30),
        ("restaurant", 20),
        ("lobby_bar", 8),
        ("spa", 6),
    ]);
    assert_eq!(
        allocate_seasonal(&cur, &plan, &max, 23),
        map(&[
            ("rooms", 10),
            ("restaurant", 10),
            ("lobby_bar", 2),
            ("spa", 1)
        ])
    );
}

#[test]
fn allocate_seasonal_never_exceeds_department_max() {
    let cur = map(&[("rooms", 0), ("spa", 0)]);
    let plan = map(&[("rooms", 50), ("spa", 50)]);
    let max = map(&[("rooms", 3), ("spa", 2)]);
    assert_eq!(
        allocate_seasonal(&cur, &plan, &max, 40),
        map(&[("rooms", 3), ("spa", 2)])
    );
}

#[test]
fn degenerate_catalog_validates_and_has_rooms_and_four_services() {
    let c = sim_hotel_v2::degenerate(&common_base());
    c.validate().unwrap();
    let kinds: Vec<UsaliClass> = c.departments.values().map(|d| d.class).collect();
    assert!(kinds.contains(&UsaliClass::Rooms));
    for k in [
        UndistributedKind::AdminGeneral,
        UndistributedKind::SalesMarketing,
        UndistributedKind::Maintenance,
        UndistributedKind::Utilities,
    ] {
        assert_eq!(
            kinds
                .iter()
                .filter(|c| **c == UsaliClass::Undistributed(k))
                .count(),
            1
        );
    }
}

#[path = "common/mod.rs"]
mod common;
fn common_base() -> sim_hotel::Catalog {
    common::test_catalog()
}

#[test]
fn validate_rejects_staffed_operated_department_without_capacity() {
    use sim_hotel::CatalogError;
    let mut c = sim_hotel_v2::degenerate(&common_base());
    c.departments
        .get_mut(&DeptCode::rooms())
        .unwrap()
        .capacity_tenths_per_staff_day = 0;
    assert!(
        matches!(c.validate(), Err(CatalogError::NotPositive(f)) if f.ends_with("rooms.capacity_tenths_per_staff_day"))
    );
}

#[test]
fn validate_rejects_staff_that_differs_from_v1_profile() {
    use sim_hotel::{CatalogError, Profile};
    let mut c = sim_hotel_v2::degenerate(&common_base());
    c.profiles
        .get_mut(&Profile::C)
        .unwrap()
        .departments
        .get_mut(&DeptCode::rooms())
        .unwrap()
        .permanent += 1;
    assert!(
        matches!(c.validate(), Err(CatalogError::OutOfRange(f)) if f == "v2.profiles.C.departments.staff")
    );
}

#[test]
fn usali_class_codes_round_trip_and_rank_is_account_order() {
    let all = [
        UsaliClass::Rooms,
        UsaliClass::FoodBeverage,
        UsaliClass::OtherOperated,
        UsaliClass::Rentals,
        UsaliClass::Undistributed(UndistributedKind::AdminGeneral),
        UsaliClass::Undistributed(UndistributedKind::SalesMarketing),
        UsaliClass::Undistributed(UndistributedKind::Maintenance),
        UsaliClass::Undistributed(UndistributedKind::Utilities),
    ];
    for (i, k) in all.iter().enumerate() {
        assert_eq!(UsaliClass::from_code(k.code()), Some(*k));
        assert_eq!(usize::from(k.rank()), i);
        assert_eq!(k.is_undistributed(), i >= 4);
    }
}
