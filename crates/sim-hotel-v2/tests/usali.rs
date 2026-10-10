use sim_hotel_v2::department::PeriodAccounts;
use sim_hotel_v2::usali::*;

// tests/common/usali.rs: june_accounts() — ručno složeni račun lipnja; podmapa se ne prevodi kao zaseban test.
#[path = "common/usali.rs"]
mod fixtures;

#[test]
fn statement_identities_hold() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    assert_eq!(s.total_revenue_cents, 35_000_000 + 6_720_000 + 699_900);
    let expenses: i64 = s.departments.iter().map(|d| d.expenses_cents).sum();
    assert_eq!(
        s.departmental_profit_cents,
        s.total_revenue_cents - expenses
    );
    assert_eq!(
        s.gop_cents,
        s.departmental_profit_cents - s.total_undistributed_cents
    );
    assert_eq!(s.total_fixed_charges_cents, 5_500_000);
    assert_eq!(s.ebitda_cents, s.gop_cents - 5_500_000);
    assert_eq!(s.room_nights, 3_600);
    assert_eq!(s.goppar_cents, Some((s.gop_cents * 2 + 3_600) / 7_200));
}

#[test]
fn statement_matches_hand_computed_figures() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    assert_eq!(s.departmental_profit_cents, 25_587_100);
    assert_eq!(s.total_undistributed_cents, 15_260_200);
    assert_eq!(s.gop_cents, 10_326_900);
    assert_eq!(s.ebitda_cents, 4_826_900);
    assert_eq!(s.goppar_cents, Some(2_869));
    assert_eq!((s.interest_cents, s.capex_cents), (300_000, 1_000_000));
}

#[test]
fn rooms_come_first_and_rentals_have_no_departmental_expense() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    let codes: Vec<&str> = s.departments.iter().map(|d| d.code.as_str()).collect();
    assert_eq!(codes, ["rooms", "restaurant", "parking"]);
    assert_eq!(s.departments[2].expenses_cents, 0);
    assert_eq!(s.departments[1].margin_bp, Some(1_175));
}

#[test]
fn maintenance_spend_sits_in_the_maintenance_service_not_in_departments() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    let pom = s
        .undistributed
        .iter()
        .find(|u| u.code.as_str() == "maintenance")
        .unwrap();
    assert_eq!(pom.other_cents, 1_500_000 + 2_300_000);
}

#[test]
fn variance_signs_follow_line_kind() {
    let a = statement(&fixtures::june_accounts()).unwrap();
    let mut worse: PeriodAccounts = fixtures::june_accounts();
    worse
        .departments
        .get_mut(&sim_hotel_v2::department::DeptCode::rooms())
        .unwrap()
        .revenue_cents += 100_000;
    let b = statement(&worse).unwrap(); // budžet s većim prihodom soba
    let v = budget_variance(&a, &b);
    let rooms_rev = v
        .iter()
        .find(|l| l.section == Section::Revenue && l.code == "rooms")
        .unwrap();
    assert_eq!(
        (rooms_rev.variance_cents, rooms_rev.favorable),
        (-100_000, false)
    );
    let rooms_exp = v
        .iter()
        .find(|l| l.section == Section::DepartmentalExpenses && l.code == "rooms")
        .unwrap();
    assert_eq!((rooms_exp.variance_cents, rooms_exp.favorable), (0, true));
}

#[test]
fn variance_lists_departments_missing_from_the_budget_with_zero() {
    let a = statement(&fixtures::june_accounts()).unwrap();
    let mut no_parking = fixtures::june_accounts();
    no_parking
        .departments
        .remove(&sim_hotel_v2::department::DeptCode("parking".into()));
    let b = statement(&no_parking).unwrap();
    let line = budget_variance(&a, &b)
        .into_iter()
        .find(|l| l.section == Section::Revenue && l.code == "parking")
        .unwrap();
    assert_eq!(
        (line.actual_cents, line.budget_cents, line.favorable),
        (699_900, 0, true)
    );
}

#[test]
fn four_weeks_sum_like_one_statement() {
    let s = statement(&fixtures::june_accounts()).unwrap();
    let sum = sum_statements(&[s.clone(), s.clone()]).unwrap();
    assert_eq!(sum.gop_cents, 2 * s.gop_cents);
    assert_eq!(sum.room_nights, 7_200);
    assert_eq!(sum.gop_margin_bp, s.gop_margin_bp);
}

#[test]
fn margins_are_none_without_revenue() {
    let mut a = fixtures::june_accounts();
    for d in a.departments.values_mut() {
        d.revenue_cents = 0;
    }
    assert_eq!(statement(&a).unwrap().gop_margin_bp, None);
}

#[test]
fn variance_has_totals_and_single_lines_for_gop_and_ebitda() {
    let a = statement(&fixtures::june_accounts()).unwrap();
    let v = budget_variance(&a, &a);
    assert!(v.iter().all(|l| l.variance_cents == 0 && l.favorable));
    let sections: Vec<Section> = v.iter().map(|l| l.section).collect();
    assert!(sections.windows(2).all(|w| w[0] <= w[1]));
    for s in [
        Section::Revenue,
        Section::DepartmentalExpenses,
        Section::DepartmentalProfit,
        Section::Undistributed,
        Section::FixedCharges,
    ] {
        assert_eq!(
            v.iter()
                .filter(|l| l.section == s && l.is_total && l.code == "total")
                .count(),
            1
        );
    }
    let codes = |s: Section| {
        v.iter()
            .filter(|l| l.section == s)
            .map(|l| l.code.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(codes(Section::Gop), ["gop"]);
    assert_eq!(codes(Section::Ebitda), ["ebitda"]);
    let tot = v
        .iter()
        .find(|l| l.section == Section::Revenue && l.is_total)
        .unwrap();
    assert_eq!(tot.actual_cents, a.total_revenue_cents);
}
