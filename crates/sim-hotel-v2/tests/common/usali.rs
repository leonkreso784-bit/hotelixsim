// Ručno složeni računi razdoblja (lipanj, 30 dana, 120 soba) za testove računa dobiti po USALI-ju.
#![allow(dead_code)]

use sim_hotel_v2::department::*;
use std::collections::BTreeMap;

fn dept(
    code: &str,
    class: UsaliClass,
    revenue: i64,
    payroll: i64,
    cost_of_sales: i64,
    other: i64,
) -> (DeptCode, DeptOutcome) {
    let code = DeptCode(code.to_string());
    let outcome = DeptOutcome {
        code: code.clone(),
        class,
        open: true,
        staff: 0,
        capacity_uses: 0,
        demand_uses: 0,
        served_uses: 0,
        usage_bp: 0,
        load_hundredths: 0,
        load_penalty_pts: 0,
        quality_pts: 0,
        price_level_pct: 100,
        revenue_cents: revenue,
        payroll_cents: payroll,
        cost_of_sales_cents: cost_of_sales,
        other_direct_cents: other,
        profit_cents: revenue - payroll - cost_of_sales - other,
        condition_before_pts: 0,
        wear_pts: 0,
        load_wear_pts: 0,
        maintenance_pts: 0,
        condition_after_pts: 0,
        maintenance_cents: 0,
        value_pts: 0,
        satisfaction_before_pts: 0,
        satisfaction_target_pts: 0,
        satisfaction_after_pts: 0,
        guest_weight: 0,
        departures: 0,
        renewal_cents: 0,
        renewal_pts: 0,
        felt_condition_pts: 0,
    };
    (code, outcome)
}

fn undistributed(
    code: &str,
    kind: UndistributedKind,
    payroll: i64,
    fixed: i64,
    per_room: i64,
    assigned: AssignedCosts,
) -> (DeptCode, UndistributedOutcome) {
    let code = DeptCode(code.to_string());
    let total = payroll + fixed + per_room + assigned.marketing_cents + assigned.maintenance_cents;
    let outcome = UndistributedOutcome {
        code: code.clone(),
        kind,
        staff: 0,
        payroll_cents: payroll,
        fixed_cents: fixed,
        per_room_cents: per_room,
        assigned,
        total_cents: total,
        departures: 0,
    };
    (code, outcome)
}

pub fn june_accounts() -> PeriodAccounts {
    let departments: BTreeMap<DeptCode, DeptOutcome> = [
        dept(
            "rooms",
            UsaliClass::Rooms,
            35_000_000,
            5_200_000,
            0,
            5_702_400,
        ),
        dept(
            "restaurant",
            UsaliClass::FoodBeverage,
            6_720_000,
            3_780_000,
            2_150_400,
            0,
        ),
        dept("parking", UsaliClass::Rentals, 699_900, 0, 0, 0),
    ]
    .into_iter()
    .collect();
    let none = AssignedCosts::default;
    let undistributed: BTreeMap<DeptCode, UndistributedOutcome> = [
        undistributed(
            "admin",
            UndistributedKind::AdminGeneral,
            2_000_000,
            3_905_000,
            0,
            none(),
        ),
        undistributed(
            "sales",
            UndistributedKind::SalesMarketing,
            0,
            1_000_000,
            0,
            AssignedCosts {
                marketing_cents: 500_000,
                ..none()
            },
        ),
        undistributed(
            "maintenance",
            UndistributedKind::Maintenance,
            0,
            1_500_000,
            0,
            AssignedCosts {
                maintenance_cents: 2_300_000,
                ..none()
            },
        ),
        undistributed(
            "energy",
            UndistributedKind::Utilities,
            0,
            2_500_000,
            2_592 * 600,
            none(),
        ),
    ]
    .into_iter()
    .collect();
    PeriodAccounts {
        days: 30,
        rooms: 120,
        departments,
        undistributed,
        fixed_charges: [
            ("insurance".to_string(), 800_000),
            ("lease".to_string(), 4_000_000),
            ("property_tax".to_string(), 700_000),
        ]
        .into_iter()
        .collect(),
        interest_cents: 300_000,
        capex_cents: 1_000_000,
        building: BuildingOutcome::default(),
        depreciation_equipment_cents: 0,
        depreciation_building_cents: 0,
    }
}
