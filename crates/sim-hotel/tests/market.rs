mod common;
use common::*;
use sim_hotel::*;

fn june_default(s: &HotelState) -> MarketOutcome {
    let cat = test_catalog();
    let plan = MonthPlan::default_for(s, Month::Jun, &cat).unwrap();
    market(s, Month::Jun, &plan, s.staff(), &cat).unwrap()
}

#[test]
fn profile_c_june_default_market() {
    let m = june_default(&state_c());
    assert_eq!((m.rooms_out, m.available), (0, 3_600));
    assert_eq!(
        (m.f_price_bp, m.f_reputation_bp, m.f_marketing_bp),
        (10_000, 10_000, 10_000)
    );
    assert_eq!(
        (m.base_demand, m.demand, m.staff_capacity),
        (2_592, 2_592, 3_600)
    );
    assert_eq!((m.sold, m.group_sold, m.individual_sold), (2_592, 0, 2_592));
}

#[test]
fn high_price_and_marketing_june() {
    let cat = test_catalog();
    let mut s = state_c();
    s.seasonal = 30;
    let plan = MonthPlan {
        price_cents: 15_000,
        marketing_cents: 1_500_000,
        seasonal_target: 30,
        wage_pct: 105,
        training: Training::Basic,
        maintenance: Maintenance::Intensive,
        renovation: false,
        reply_reviews: true,
    };
    let m = market(&s, Month::Jun, &plan, 60, &cat).unwrap();
    assert_eq!(
        (
            m.f_price_bp,
            m.f_marketing_bp,
            m.demand,
            m.staff_capacity,
            m.sold
        ),
        (9_111, 10_750, 2_539, 4_320, 2_539)
    );
}

#[test]
fn low_price_raises_demand() {
    let cat = test_catalog();
    let s = state_c();
    let mut plan = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    plan.price_cents = 8_100; // 60 % reference
    let m = market(&s, Month::Jun, &plan, 50, &cat).unwrap();
    assert_eq!((m.f_price_bp, m.demand, m.sold), (13_200, 3_421, 3_421));
}

#[test]
fn worn_equipment_takes_rooms_out() {
    let mut s = state_c();
    s.equipment_pts = 2_000;
    let m = june_default(&s);
    assert_eq!((m.rooms_out, m.available, m.sold), (5, 3_450, 2_592));
}

#[test]
fn staff_capacity_caps_august() {
    let cat = test_catalog();
    let s = state_c();
    let mut plan = MonthPlan::default_for(&s, Month::Aug, &cat).unwrap();
    plan.seasonal_target = 0;
    let m = market(&s, Month::Aug, &plan, 30, &cat).unwrap();
    assert_eq!((m.demand, m.staff_capacity, m.sold), (3_088, 2_232, 2_232));
}

#[test]
fn zero_staff_sells_nothing() {
    let cat = test_catalog();
    let s = state_c();
    let plan = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    let m = market(&s, Month::Jun, &plan, 0, &cat).unwrap();
    assert_eq!((m.staff_capacity, m.sold), (0, 0));
}

#[test]
fn groups_sell_first_and_are_paid_at_group_price() {
    let cat = test_catalog();
    let mut s = state_c();
    s.groups.push(GroupBlock {
        month: Month::Jun,
        room_nights: 175,
        price_cents: 10_125,
    });
    s.groups.push(GroupBlock {
        month: Month::Aug,
        room_nights: 50,
        price_cents: 1,
    });
    let plan = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    let m = market(&s, Month::Jun, &plan, 50, &cat).unwrap();
    assert_eq!(
        (m.group_room_nights, m.sold, m.group_sold, m.individual_sold),
        (175, 2_767, 175, 2_592)
    );
    assert_eq!(m.group_revenue_cents, 1_771_875);
}

#[test]
fn plan_bounds_reject_out_of_range_inputs() {
    let cat = test_catalog();
    let s = state_c();
    let ok = MonthPlan::default_for(&s, Month::Jun, &cat).unwrap();
    assert_eq!(ok.validate(&s, Month::Jun, &cat), Ok(()));
    let bad = |f: &dyn Fn(&mut MonthPlan)| {
        let mut p = ok.clone();
        f(&mut p);
        p.validate(&s, Month::Jun, &cat)
    };
    assert_eq!(
        bad(&|p| p.price_cents = 13_550),
        Err(PlanError::PriceNotWholeEuro)
    );
    assert_eq!(
        bad(&|p| p.price_cents = 8_000),
        Err(PlanError::PriceOutOfRange)
    ); // < 60 % (8 100)
    assert_eq!(
        bad(&|p| p.price_cents = 20_300),
        Err(PlanError::PriceOutOfRange)
    ); // > 150 % (20 250)
    assert_eq!(bad(&|p| p.price_cents = 20_200), Ok(()));
    assert_eq!(
        bad(&|p| p.marketing_cents = 4_000_001),
        Err(PlanError::MarketingOutOfRange)
    );
    assert_eq!(
        bad(&|p| p.marketing_cents = -1),
        Err(PlanError::MarketingOutOfRange)
    );
    assert_eq!(
        bad(&|p| p.seasonal_target = 31),
        Err(PlanError::SeasonalStepTooBig)
    ); // 20 → 31
    assert_eq!(
        bad(&|p| p.seasonal_target = 41),
        Err(PlanError::SeasonalOutOfRange)
    );
    assert_eq!(bad(&|p| p.wage_pct = 89), Err(PlanError::WageOutOfRange));
    assert_eq!(bad(&|p| p.wage_pct = 116), Err(PlanError::WageOutOfRange));
    let mut renovated = s.clone();
    renovated.renovated = true;
    let mut p = ok.clone();
    p.renovation = true;
    assert_eq!(
        p.validate(&renovated, Month::Jun, &cat),
        Err(PlanError::RenovationTwice)
    );
}

#[test]
fn default_plan_follows_seasonal_plan_within_step() {
    let mut cat = test_catalog();
    cat.categories
        .get_mut(&Category::Four)
        .unwrap()
        .seasonal_plan
        .0[0] = 35;
    let p = MonthPlan::default_for(&state_c(), Month::Jun, &cat).unwrap();
    assert_eq!(p.seasonal_target, 30); // 20 + korak 10, ne 35
    assert_eq!(
        (p.price_cents, p.marketing_cents, p.wage_pct),
        (13_500, 0, 100)
    );
    assert_eq!(
        (p.training, p.maintenance, p.renovation, p.reply_reviews),
        (Training::None, Maintenance::Regular, false, false)
    );
}
