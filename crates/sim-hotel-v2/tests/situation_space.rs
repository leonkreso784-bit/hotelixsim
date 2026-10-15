//! B1a (ADR-042 t. 1): situacija mijenja stanje prostora svog odjela. Hitan popravak diže stanje tog odjela za
//! `economy.repair_urgent_condition_pts` bez faktora kruga, privremeni ostaje v1 (−100 puta faktor kruga) u istom
//! odjelu; bez odjela i neraspoređeni odjel idu u Smještaj; nepoznat odjel je greška kataloga.

mod common;

use sim_hotel_v2::calendar::{month_period, period_by_code};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::RoundSituation;
use sim_hotel_v2::*;

fn code(s: &str) -> DeptCode {
    DeptCode(s.into())
}

fn cat(bonus: i32, effect_bp: u16) -> Catalog {
    let mut c = common::catalog_with_departments();
    c.economy.repair_urgent_condition_pts = bonus;
    for k in PeriodKind::ALL {
        c.periods.get_mut(&k).unwrap().situation_effect_bp = effect_bp;
    }
    c
}

fn repair(id: &str, urgent: bool, space: Option<&str>) -> RoundSituation {
    RoundSituation {
        outcome: SituationOutcome {
            code: id.into(),
            mechanics: vec![Mechanic::EquipmentRepair { urgent }],
            quality: 3,
        },
        space: space.map(code),
    }
}

fn run(c: &Catalog, p: &Period, s: &[RoundSituation]) -> Result<PeriodReport, ResolveError> {
    let st = HotelState::from_profile(c, Profile::C).unwrap();
    resolve_period(&st, p, &common::hold_plan(&st, p, c), s, c)
}

/// Stanje odjela nakon kruga sa situacijama minus bez njih.
fn delta(c: &Catalog, p: &Period, s: &[RoundSituation], dept: &str) -> i32 {
    let (a, b) = (run(c, p, s).unwrap(), run(c, p, &[]).unwrap());
    assert!(
        b.after.departments[&code(dept)].condition_pts < 9_000,
        "bez stiskanja na 100"
    );
    a.after.departments[&code(dept)].condition_pts - b.after.departments[&code(dept)].condition_pts
}

#[test]
fn urgent_repair_raises_the_situation_department_without_round_factor() {
    let c = cat(1_000, 350);
    for p in [
        month_period(Month::Jun),
        period_by_code(PeriodKind::Week, "t02").unwrap(),
    ] {
        let s = [repair("P-X.a", true, Some("restaurant"))];
        assert_eq!(delta(&c, &p, &s, "restaurant"), 1_000);
        assert_eq!(
            delta(&c, &p, &s, "rooms"),
            0,
            "Smještaj ne dobiva popravak restorana"
        );
    }
}

#[test]
fn temporary_repair_moves_to_the_situation_department() {
    let c = cat(1_000, 10_000);
    let s = [repair("P-X.a", false, Some("restaurant"))];
    let p = month_period(Month::Jun);
    assert_eq!(
        (delta(&c, &p, &s, "restaurant"), delta(&c, &p, &s, "rooms")),
        (-100, 0)
    );
}

#[test]
fn no_department_and_undistributed_go_to_rooms_unknown_is_an_error() {
    let (c, p) = (cat(1_000, 350), month_period(Month::Jun));
    for space in [None, Some("maintenance")] {
        let s = [repair("P-X.a", true, space)];
        assert_eq!(
            (delta(&c, &p, &s, "rooms"), delta(&c, &p, &s, "restaurant")),
            (1_000, 0),
            "{space:?}"
        );
    }
    assert!(matches!(
        run(&c, &p, &[repair("P-X.a", true, Some("nepostoji"))]),
        Err(ResolveError::Catalog(_))
    ));
}

#[test]
fn lines_explain_department_repairs() {
    let (c, p) = (cat(1_000, 10_000), month_period(Month::Jun));
    let r = run(
        &c,
        &p,
        &[
            repair("P-A.jun", true, Some("restaurant")),
            repair("P-B.jun", false, None),
        ],
    )
    .unwrap();
    common::assert_lines_explain(&r.summary);
    let line = |id: &str| -> i64 {
        r.summary
            .lines
            .iter()
            .filter(|l| l.var == Var::Equipment && l.cause == Cause::Situation(id.into()))
            .map(|l| l.amount)
            .sum()
    };
    assert!(
        line("P-A.jun") > 0,
        "popravak restorana vidi se u sažetku opreme (udio ostalih odjela)"
    );
    assert_eq!(line("P-B.jun"), -100);
}

#[test]
fn situation_effect_is_what_resolve_uses_and_degenerate_has_no_bonus() {
    let c = cat(1_000, 350);
    let cat_c = HotelState::from_profile(&c, Profile::C).unwrap().category;
    let e = situation_effect(
        &repair("P-X.a", true, Some("restaurant")),
        &month_period(Month::Jun),
        &c,
        cat_c,
    )
    .unwrap();
    assert_eq!(
        (e.effects.equipment_pts, e.effects.cost_cents, e.space),
        (1_000, 25_000, code("restaurant"))
    );
    assert_eq!(
        degenerate(&common::test_catalog())
            .economy
            .repair_urgent_condition_pts,
        0
    );
}
