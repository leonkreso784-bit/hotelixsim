//! J-3: učinak situacija po vrsti kruga (`situation_effect_bp`) i granica zbroja situacija kruga
//! (`situation_round_bound_pts`). Bodovi (zadovoljstvo, moral, oprema) se skaliraju, troškovi ostaju puni; retci
//! objašnjenja i dalje zbrojem daju promjenu stanja.

mod common;

use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code};
use sim_hotel_v2::*;

fn cat(effect_bp: u16, bound: Option<u16>) -> Catalog {
    let mut c = degenerate(&common::test_catalog());
    for k in PeriodKind::ALL {
        let p = c.periods.get_mut(&k).unwrap();
        p.situation_effect_bp = effect_bp;
        p.situation_round_bound_pts = bound;
    }
    c
}

/// Oporavak gosta kvalitete `q` (±200 bodova zadovoljstva po stupnju) s kompenzacijom od 120 €.
fn recovery(code: &str, q: u8) -> SituationOutcome {
    SituationOutcome {
        code: code.into(),
        mechanics: vec![
            Mechanic::ServiceRecovery {
                compensation_cents: 12_000,
            },
            Mechanic::EquipmentRepair { urgent: false },
        ],
        quality: q,
    }
}

/// Zadržavanje radnika kvalitete `q` (±300 bodova morala po stupnju) s bonusom 500 €.
fn retention(code: &str, q: u8) -> SituationOutcome {
    SituationOutcome {
        code: code.into(),
        mechanics: vec![Mechanic::StaffRetention {
            raise_one_pct: 5,
            bonus_cents: 50_000,
        }],
        quality: q,
    }
}

fn run(c: &Catalog, p: &Period, outcomes: &[SituationOutcome]) -> PeriodReport {
    let s = HotelState::from_profile(c, Profile::C).unwrap();
    let plan = PeriodPlan::default_for(&s, p, c).unwrap();
    let outcomes: Vec<RoundSituation> =
        outcomes.iter().cloned().map(RoundSituation::from).collect();
    resolve_period(&s, p, &plan, &outcomes, c).unwrap()
}

/// Zbroj redaka situacije `code` za varijablu `var`.
fn sit_lines(r: &PeriodReport, code: &str, var: Var) -> i64 {
    r.summary
        .lines
        .iter()
        .filter(|l| l.var == var && l.cause == Cause::Situation(code.to_string()))
        .map(|l| l.amount)
        .sum()
}

fn all_sit_lines(r: &PeriodReport, var: Var) -> i64 {
    r.summary
        .lines
        .iter()
        .filter(|l| l.var == var && matches!(l.cause, Cause::Situation(_)))
        .map(|l| l.amount)
        .sum()
}

#[test]
fn degenerate_catalog_has_full_effect_and_no_round_bound() {
    let c = degenerate(&common::test_catalog());
    for k in PeriodKind::ALL {
        let p = c.coeffs(k).unwrap();
        assert_eq!(p.situation_effect_bp, 10_000);
        assert_eq!(p.situation_round_bound_pts, None);
    }
}

#[test]
fn factor_scales_points_but_keeps_costs_full() {
    let t03 = period_by_code(PeriodKind::Week, "t03").unwrap();
    let outcomes = [recovery("A", 5), retention("B", 1)];
    let full = run(&cat(10_000, None), &t03, &outcomes);
    let quarter = run(&cat(2_500, None), &t03, &outcomes);
    // Puni učinak: +400 zadovoljstva, −100 opreme (privremeni popravak), −600 morala.
    assert_eq!(sit_lines(&full, "A", Var::Satisfaction), 400);
    assert_eq!(sit_lines(&full, "A", Var::Equipment), -100);
    assert_eq!(sit_lines(&full, "B", Var::Morale), -600);
    assert_eq!(sit_lines(&quarter, "A", Var::Satisfaction), 100);
    assert_eq!(sit_lines(&quarter, "A", Var::Equipment), -25);
    assert_eq!(sit_lines(&quarter, "B", Var::Morale), -150);
    // Troškovi (kompenzacija, popravak, bonus, povišica) jednaki.
    for code in ["A", "B"] {
        assert_eq!(
            sit_lines(&quarter, code, Var::Costs),
            sit_lines(&full, code, Var::Costs),
            "{code}"
        );
    }
    assert_eq!(
        quarter.summary.money.costs_cents,
        full.summary.money.costs_cents
    );
    common::assert_lines_explain(&quarter.summary);
}

#[test]
fn round_bound_caps_the_sum_of_a_round_and_lines_still_explain() {
    let t03 = period_by_code(PeriodKind::Week, "t03").unwrap();
    let c = cat(10_000, Some(800));
    let up = run(
        &c,
        &t03,
        &[recovery("A", 5), recovery("B", 5), recovery("C", 5)],
    );
    assert_eq!(all_sit_lines(&up, Var::Satisfaction), 800);
    // Višak se skida od zadnje situacije prema prvoj.
    assert_eq!(sit_lines(&up, "A", Var::Satisfaction), 400);
    assert_eq!(sit_lines(&up, "B", Var::Satisfaction), 400);
    assert_eq!(sit_lines(&up, "C", Var::Satisfaction), 0);
    common::assert_lines_explain(&up.summary);

    let down = run(
        &c,
        &t03,
        &[retention("A", 1), retention("B", 1), retention("C", 2)],
    );
    // −600 −600 −300 = −1500 → −800.
    assert_eq!(all_sit_lines(&down, Var::Morale), -800);
    assert_eq!(sit_lines(&down, "A", Var::Morale), -600);
    assert_eq!(sit_lines(&down, "B", Var::Morale), -200);
    assert_eq!(sit_lines(&down, "C", Var::Morale), 0);
    common::assert_lines_explain(&down.summary);

    // Bez granice isti krug nosi puni zbroj.
    let free = run(
        &cat(10_000, None),
        &t03,
        &[recovery("A", 5), recovery("B", 5), recovery("C", 5)],
    );
    assert_eq!(all_sit_lines(&free, Var::Satisfaction), 1_200);
}

#[test]
fn round_bound_leaves_a_sum_within_bounds_alone() {
    let t03 = period_by_code(PeriodKind::Week, "t03").unwrap();
    let c = cat(10_000, Some(800));
    let r = run(
        &c,
        &t03,
        &[
            recovery("A", 5),
            recovery("B", 1),
            recovery("C", 5),
            recovery("D", 5),
        ],
    );
    // +400 −400 +400 +400 = +800: granica ne dira nijednu situaciju.
    assert_eq!(sit_lines(&r, "B", Var::Satisfaction), -400);
    assert_eq!(sit_lines(&r, "D", Var::Satisfaction), 400);
    assert_eq!(all_sit_lines(&r, Var::Satisfaction), 800);
}

#[test]
fn factor_applies_before_the_round_bound() {
    let june = month_period(Month::Jun);
    let c = cat(2_500, Some(800));
    let outcomes: Vec<SituationOutcome> = (0..10).map(|i| recovery(&format!("S{i}"), 5)).collect();
    let r = run(&c, &june, &outcomes);
    // 10 × 100 = 1000 → 800.
    assert_eq!(all_sit_lines(&r, Var::Satisfaction), 800);
    common::assert_lines_explain(&r.summary);
}

#[test]
fn catalog_rejects_zero_factor_and_zero_round_bound() {
    let mut c = cat(10_000, None);
    c.periods
        .get_mut(&PeriodKind::Week)
        .unwrap()
        .situation_effect_bp = 0;
    assert!(c.validate().is_err());
    let mut c = cat(10_000, Some(0));
    assert!(c.validate().is_err());
    c.periods
        .get_mut(&PeriodKind::Month)
        .unwrap()
        .situation_round_bound_pts = Some(800);
    c.periods
        .get_mut(&PeriodKind::Week)
        .unwrap()
        .situation_round_bound_pts = Some(800);
    assert!(c.validate().is_ok());
}
