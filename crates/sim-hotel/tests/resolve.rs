mod common;
use common::*;
use sim_hotel::*;

fn run(s: &HotelState, m: Month, plan: &MonthPlan) -> MonthReport {
    resolve_month(s, m, plan, &[], &test_catalog()).unwrap()
}
fn default(s: &HotelState, m: Month) -> MonthPlan {
    MonthPlan::default_for(s, m, &test_catalog()).unwrap()
}

#[test]
fn profile_c_june_default() {
    let s = state_c();
    let r = run(&s, Month::Jun, &default(&s, Month::Jun));
    let p = &r.people;
    assert_eq!(
        (p.hires, p.staff, p.load_hundredths, p.load_penalty_pts),
        (0, 50, 173, 0)
    );
    assert_eq!(
        (
            p.morale_target_pts,
            p.turnover_bp,
            p.departures,
            p.experience_mix_pts
        ),
        (5_500, 375, 2, 5_840)
    );
    let m = &r.money;
    assert_eq!(
        (m.rooms_revenue_cents, m.other_revenue_cents, m.labor_cents),
        (34_992_000, 10_497_600, 11_300_000)
    );
    assert_eq!(
        (m.costs_cents, m.gop_cents, m.goppar_cents, m.interest_cents),
        (30_502_400, 14_987_200, 4_163, 0)
    );
    let g = &r.guests;
    assert_eq!(
        (
            g.quality_pts,
            g.value_pts,
            g.satisfaction_target_pts,
            g.rating_target
        ),
        (7_106, 7_000, 6_748, 835)
    );
    let a = &r.after;
    assert_eq!(
        (
            a.morale_pts,
            a.experience_pts,
            a.equipment_pts,
            a.satisfaction_pts,
            a.rating
        ),
        (5_500, 6_140, 6_000, 6_874, 839)
    );
    assert_eq!(
        (a.cash_cents, a.load_hundredths, a.turnover_bp),
        (44_987_200, 173, 375)
    );
}

#[test]
fn june_variant_price_marketing_staff_wage_training() {
    let s = state_c();
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
    let r = run(&s, Month::Jun, &plan);
    assert_eq!(
        (r.people.hires, r.people.staff, r.people.load_hundredths),
        (10, 60, 141)
    );
    assert_eq!(
        (
            r.people.morale_target_pts,
            r.people.turnover_bp,
            r.people.departures,
            r.people.experience_mix_pts
        ),
        (6_500, 330, 2, 5_200)
    );
    assert_eq!(
        (
            r.money.rooms_revenue_cents,
            r.money.other_revenue_cents,
            r.money.labor_cents
        ),
        (38_085_000, 11_425_500, 16_160_000)
    );
    assert_eq!(
        (r.money.costs_cents, r.money.gop_cents, r.money.goppar_cents),
        (38_395_800, 11_114_700, 3_087)
    );
    assert_eq!(
        (
            r.guests.quality_pts,
            r.guests.value_pts,
            r.guests.satisfaction_target_pts,
            r.guests.rating_target
        ),
        (6_940, 5_889, 6_515, 830)
    );
    let a = &r.after;
    assert_eq!(
        (
            a.morale_pts,
            a.experience_pts,
            a.equipment_pts,
            a.satisfaction_pts,
            a.rating,
            a.cash_cents
        ),
        (5_800, 5_500, 6_400, 6_757, 842, 41_114_700)
    );
    assert_eq!(
        a.pending,
        Pending {
            experience_pts: 400,
            morale_pts: 200,
            renovation: false
        }
    );
}

#[test]
fn zero_staff_month() {
    let mut s = state_c();
    s.permanent = 0;
    s.seasonal = 0;
    let mut plan = default(&s, Month::Jun);
    plan.seasonal_target = 0;
    let r = run(&s, Month::Jun, &plan);
    assert_eq!(
        (
            r.market.sold,
            r.people.load_hundredths,
            r.people.load_penalty_pts
        ),
        (0, 300, 6_000)
    );
    assert_eq!(
        (
            r.people.morale_target_pts,
            r.people.turnover_bp,
            r.people.departures,
            r.people.experience_mix_pts
        ),
        (2_500, 510, 0, 6_000)
    );
    assert_eq!(
        (r.money.costs_cents, r.money.gop_cents, r.money.goppar_cents),
        (13_500_000, -13_500_000, -3_750)
    );
    assert_eq!((r.after.experience_pts, r.after.morale_pts), (6_300, 4_600));
}

#[test]
fn negative_cash_pays_interest() {
    let mut s = state_c();
    s.cash_cents = -500_000;
    s.fixed_cents = 50_000_000;
    let r = run(&s, Month::Jun, &default(&s, Month::Jun));
    assert_eq!(
        (
            r.money.costs_cents,
            r.money.gop_cents,
            r.money.interest_cents
        ),
        (68_502_400, -23_012_800, 235_128)
    );
    assert_eq!(
        (r.after.cash_cents, r.money.goppar_cents),
        (-23_747_928, -6_392)
    );
}

#[test]
fn training_and_renovation_act_next_month() {
    let s = state_c();
    let mut plan = default(&s, Month::Jun);
    plan.training = Training::Basic;
    plan.renovation = true;
    let june = run(&s, Month::Jun, &plan);
    assert_eq!(
        (
            june.money.labor_cents,
            june.money.gop_cents,
            june.money.capex_cents
        ),
        (11_800_000, 14_487_200, 15_000_000)
    );
    assert_eq!(
        (
            june.after.cash_cents,
            june.after.equipment_pts,
            june.after.experience_pts
        ),
        (29_487_200, 6_000, 6_140)
    );
    assert!(june.after.renovated);
    let july = run(&june.after, Month::Jul, &default(&june.after, Month::Jul));
    assert_eq!(
        (
            july.people.load_hundredths,
            july.people.load_penalty_pts,
            july.people.morale_target_pts
        ),
        (187, 700, 5_350)
    );
    let a = &july.after;
    assert_eq!(
        (
            a.morale_pts,
            a.experience_pts,
            a.equipment_pts,
            a.satisfaction_pts,
            a.rating,
            a.cash_cents
        ),
        (5_455, 6_674, 8_400, 7_469, 844, 60_512_200)
    );
    assert_eq!(a.pending, Pending::default());
}

#[test]
fn worn_equipment_month() {
    let mut s = state_c();
    s.equipment_pts = 2_000;
    let r = run(&s, Month::Jun, &default(&s, Month::Jun));
    assert_eq!(
        (
            r.market.rooms_out,
            r.after.equipment_pts,
            r.after.satisfaction_pts
        ),
        (5, 2_000, 6_274)
    );
}

#[test]
fn cheap_price_deferred_maintenance_and_replies() {
    let s = state_c();
    let mut plan = default(&s, Month::Jun);
    plan.price_cents = 8_100;
    plan.reply_reviews = true;
    plan.maintenance = Maintenance::Deferred;
    let r = run(&s, Month::Jun, &plan);
    assert_eq!(
        (
            r.money.rooms_revenue_cents,
            r.money.other_revenue_cents,
            r.money.costs_cents,
            r.money.gop_cents
        ),
        (27_710_100, 8_313_030, 31_276_200, 4_746_930)
    );
    assert_eq!(
        (
            r.guests.quality_pts,
            r.guests.value_pts,
            r.guests.satisfaction_target_pts,
            r.guests.rating_target
        ),
        (5_450, 10_000, 6_603, 832)
    );
    let a = &r.after;
    assert_eq!(
        (
            a.morale_pts,
            a.equipment_pts,
            a.satisfaction_pts,
            a.rating,
            a.load_hundredths,
            a.turnover_bp
        ),
        (4_780, 5_500, 6_801, 843, 228, 483)
    );
}

#[test]
fn releasing_seasonals_in_august_overloads_staff() {
    let mut s = state_c();
    s.seasonal = 10;
    let mut plan = default(&s, Month::Aug);
    plan.seasonal_target = 0;
    let r = run(&s, Month::Aug, &plan);
    assert_eq!(
        (
            r.people.released,
            r.people.staff,
            r.people.load_hundredths,
            r.people.morale_target_pts
        ),
        (10, 30, 240, 2_200)
    );
    assert_eq!(
        (
            r.people.turnover_bp,
            r.people.departures,
            r.people.experience_mix_pts
        ),
        (524, 2, 5_733)
    );
    let a = &r.after;
    assert_eq!(
        (
            a.morale_pts,
            a.experience_pts,
            a.equipment_pts,
            a.satisfaction_pts,
            a.rating,
            a.cash_cents
        ),
        (4_510, 6_033, 5_900, 6_377, 834, 55_467_600)
    );
}

#[test]
fn every_changed_variable_has_a_cause_and_money_lines_add_up() {
    let s = state_c();
    let mut plan = default(&s, Month::Jun);
    plan.price_cents = 15_000;
    plan.renovation = true;
    plan.training = Training::Full;
    let r = run(&s, Month::Jun, &plan);
    let total = |v: Var| {
        r.lines
            .iter()
            .filter(|l| l.var == v)
            .map(|l| l.amount)
            .sum::<i64>()
    };
    assert_eq!(total(Var::Costs), r.money.costs_cents);
    assert_eq!(
        total(Var::Revenue),
        r.money.rooms_revenue_cents + r.money.other_revenue_cents
    );
    assert_eq!(total(Var::Cash), r.after.cash_cents - r.before.cash_cents);
    let changed = [
        (Var::Morale, r.before.morale_pts != r.after.morale_pts),
        (
            Var::Experience,
            r.before.experience_pts != r.after.experience_pts,
        ),
        (
            Var::Equipment,
            r.before.equipment_pts != r.after.equipment_pts,
        ),
        (
            Var::Satisfaction,
            r.before.satisfaction_pts != r.after.satisfaction_pts,
        ),
        (Var::Rating, r.before.rating != r.after.rating),
        (Var::Cash, r.before.cash_cents != r.after.cash_cents),
    ];
    for (v, did) in changed {
        if did {
            assert!(
                r.lines.iter().any(|l| l.var == v),
                "{v:?} se promijenio bez uzroka"
            );
        }
    }
}

#[test]
fn invalid_plan_is_rejected_before_any_calculation() {
    let s = state_c();
    let mut plan = default(&s, Month::Jun);
    plan.wage_pct = 200;
    assert_eq!(
        resolve_month(&s, Month::Jun, &plan, &[], &test_catalog()),
        Err(ResolveError::Plan(PlanError::WageOutOfRange))
    );
}
