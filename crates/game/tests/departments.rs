mod common;

use game::*;

#[test]
fn departments_of_profile_after_first_week() {
    let g = common::v2_weekly_game_resolved_through("t01");
    let h = common::hotel_ids()[0];
    let v = g.departments(h).unwrap();
    assert_eq!(v.round, Some(RoundKey::Week(1)));
    assert!(
        v.departments
            .iter()
            .any(|d| d.code == "rooms" && d.last.is_some())
    );
    let admin = v.departments.iter().find(|d| d.code == "admin").unwrap();
    assert_eq!(admin.guest_satisfaction_pts, None);
    assert!(admin.undistributed_last.is_some());
}

#[test]
fn department_history_has_one_entry_per_resolved_round() {
    let g = common::v2_weekly_game_resolved_through("t03");
    let d = g.department(common::hotel_ids()[0], "rooms").unwrap();
    assert_eq!(
        d.history.iter().map(|(r, _)| r.code()).collect::<Vec<_>>(),
        ["t01", "t02", "t03"]
    );
    assert!(matches!(
        g.department(common::hotel_ids()[0], "casino"),
        Err(GameError::UnknownDepartment)
    ));
}

#[test]
fn pl_lines_align_actual_and_budget() {
    let g = common::v2_weekly_game_resolved_through("t02");
    let pl = g
        .profit_and_loss(common::hotel_ids()[0], None, PlScope::Round)
        .unwrap();
    assert_eq!(pl.round, RoundKey::Week(2));
    assert_eq!(
        pl.actual.ebitda_cents,
        pl.actual.gop_cents - pl.actual.total_fixed_charges_cents
    );
    // Obnova v2 (O-3, svjesna izmjena): iza EBITDA dolaze sekcije Amortizacija (oprema, zgrada, ukupno) i EBIT.
    // Njihovi retci nemaju ime iz sadržaja (`names`), nego stalno ime u API-ju, kao GOP i EBITDA.
    use sim_hotel_v2::usali::Section;
    for l in &pl.lines {
        assert_eq!(l.variance_cents, l.actual_cents - l.budget_cents);
        assert!(
            pl.names.contains_key(&l.code)
                || l.is_total
                || ["gop", "ebitda", "ebit"].contains(&l.code.as_str())
                || (l.section == Section::Depreciation
                    && ["equipment", "building"].contains(&l.code.as_str())),
            "{}",
            l.code
        );
    }
    let tail: Vec<(Section, &str, i64)> = pl
        .lines
        .iter()
        .skip_while(|l| l.section != Section::Ebitda)
        .map(|l| (l.section, l.code.as_str(), l.budget_cents))
        .collect();
    assert_eq!(
        tail,
        [
            (Section::Ebitda, "ebitda", pl.budget.ebitda_cents),
            (Section::Depreciation, "equipment", 0),
            (Section::Depreciation, "building", 0),
            (Section::Depreciation, "total", 0),
            (Section::Ebit, "ebit", pl.budget.ebitda_cents),
        ]
    );
    assert_eq!(pl.actual.ebit_cents, pl.actual.ebitda_cents); // sadržaj bez obnove: amortizacija 0
    let gop = pl.lines.iter().find(|l| l.code == "gop").unwrap();
    assert_eq!(
        gop.actual_cents,
        g.resolved[&RoundKey::Week(2)].hotels[0]
            .report
            .money
            .gop_cents
    );
}

#[test]
fn season_scope_sums_rounds() {
    let g = common::v2_weekly_game_resolved_through("t02");
    let h = common::hotel_ids()[0];
    let one = g
        .profit_and_loss(h, Some(RoundKey::Week(1)), PlScope::Round)
        .unwrap();
    let two = g
        .profit_and_loss(h, Some(RoundKey::Week(2)), PlScope::Round)
        .unwrap();
    let season = g
        .profit_and_loss(h, Some(RoundKey::Week(2)), PlScope::Season)
        .unwrap();
    assert_eq!(
        season.actual.gop_cents,
        one.actual.gop_cents + two.actual.gop_cents
    );
}

#[test]
fn v1_game_has_no_departments_and_no_pl() {
    let (g, h) = common::open_june_game();
    assert_eq!(g.departments(h).unwrap_err(), GameError::WrongModel);
    assert_eq!(
        g.profit_and_loss(h, None, PlScope::Round).unwrap_err(),
        GameError::WrongModel
    );
}

#[test]
fn pl_before_first_resolution_is_nothing_resolved() {
    let g = common::v2_weekly_game_resolved_through("none");
    assert_eq!(
        g.profit_and_loss(common::hotel_ids()[0], None, PlScope::Round)
            .unwrap_err(),
        GameError::NothingResolved
    );
}

#[test]
fn plan_limits_carry_department_limits_only_for_v2() {
    let g = common::v2_weekly_game_resolved_through("none");
    let l = g
        .private_hotel(common::hotel_ids()[0])
        .unwrap()
        .plan
        .unwrap()
        .limits;
    assert!(l.seasonal_step_total.is_some_and(|s| s > 0)); // `periods.json` week.seasonal_step (Task 17 ga smije mijenjati)
    assert!(!l.departments["restaurant"].can_close); // restoran ima svaki profil (Task 10)
    assert!(l.departments.contains_key("rooms"));
    let (v1, h) = common::open_june_game();
    let l1 = v1.private_hotel(h).unwrap().plan.unwrap().limits;
    assert_eq!(l1.seasonal_step_total, None);
    assert!(l1.departments.is_empty());
}

#[test]
fn weekly_kpi_uses_seven_days() {
    let g = common::v2_weekly_game_resolved_through("t01");
    let k = g.hotel_kpi(common::hotel_ids()[0]).unwrap();
    assert_eq!(k[0].kpi.room_nights, 120 * 7);
}
