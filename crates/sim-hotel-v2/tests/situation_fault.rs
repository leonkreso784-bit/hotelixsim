//! L1-13 dio 1 (A1, Leon 17.10. i 18.10.): kvar prostora spušta stanje prostora za `economy.fault_condition_pts`
//! u krugu u kojem se pojavio (i uz šutnju); hitan popravak uz ocjenu >= `FAULT_FIXED_MIN_QUALITY` (4) diže
//! `economy.fault_fixed_condition_pts`, jednom po situaciji. Oba učinka su fizička (bez faktora kruga), a bez ključeva
//! (0) pravilo je isključeno i obračun je bajt-isti današnjem. Kvar prostora je `RoundSituation.fault`.

mod common;

use sim_hotel_v2::calendar::{month_period, period_by_code};
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::{FAULT_FIXED_MIN_QUALITY, RoundSituation};
use sim_hotel_v2::*;

fn code(s: &str) -> DeptCode {
    DeptCode(s.into())
}

/// Katalog s ključevima kvara (`fault`, `fixed`), starim ključem hitnog popravka i faktorom učinka situacija.
fn cat(fault: i32, fixed: i32, old_urgent: i32, effect_bp: u16) -> Catalog {
    let mut c = common::catalog_with_departments();
    c.economy.fault_condition_pts = fault;
    c.economy.fault_fixed_condition_pts = fixed;
    c.economy.repair_urgent_condition_pts = old_urgent;
    for k in PeriodKind::ALL {
        c.periods.get_mut(&k).unwrap().situation_effect_bp = effect_bp;
    }
    c
}

/// Leonovi ključevi: kvar −1, popravak +2 (stari ključ 0, kao poslije dijela 4); faktor kruga 3,5 %.
fn leon() -> Catalog {
    cat(100, 200, 0, 350)
}

fn sit(fault: bool, space: Option<&str>, mechanics: Vec<Mechanic>, q: u8) -> RoundSituation {
    RoundSituation {
        outcome: SituationOutcome {
            code: "P-KVAR.a".into(),
            mechanics,
            quality: q,
        },
        space: space.map(code),
        fault,
        click: false,
    }
}

fn urgent() -> Mechanic {
    Mechanic::EquipmentRepair { urgent: true }
}

fn temporary() -> Mechanic {
    Mechanic::EquipmentRepair { urgent: false }
}

fn periods() -> [Period; 3] {
    [
        month_period(Month::Jun),
        month_period(Month::Aug),
        period_by_code(PeriodKind::Week, "t02").unwrap(),
    ]
}

fn category(c: &Catalog) -> Category {
    HotelState::from_profile(c, Profile::C).unwrap().category
}

fn effect(c: &Catalog, p: &Period, s: &RoundSituation) -> SituationEffect {
    situation_effect(s, p, c, category(c)).unwrap()
}

fn equipment(c: &Catalog, p: &Period, s: &RoundSituation) -> i32 {
    effect(c, p, s).effects.equipment_pts
}

fn run(c: &Catalog, p: &Period, s: &[RoundSituation]) -> PeriodReport {
    let st = HotelState::from_profile(c, Profile::C).unwrap();
    resolve_period(&st, p, &common::hold_plan(&st, p, c), s, c).unwrap()
}

/// Stanje odjela nakon kruga sa situacijama minus bez njih.
fn delta(c: &Catalog, p: &Period, s: &[RoundSituation], dept: &str) -> i32 {
    let (a, b) = (run(c, p, s), run(c, p, &[]));
    assert!(
        b.after.departments[&code(dept)].condition_pts < 9_000,
        "bez stiskanja na 100"
    );
    a.after.departments[&code(dept)].condition_pts - b.after.departments[&code(dept)].condition_pts
}

#[test] // 1
fn fault_lowers_space_condition_by_fault_pts() {
    let c = leon();
    for p in periods() {
        let silent = sit(true, None, vec![], 1);
        assert_eq!(equipment(&c, &p, &silent), -100, "{p:?}");
        assert_eq!(delta(&c, &p, &[silent], "rooms"), -100, "{p:?}");
    }
}

#[test] // 1 (prostor je odjel primjerka)
fn fault_hits_the_situation_department_not_rooms() {
    let c = leon();
    let p = month_period(Month::Jun);
    let s = [sit(true, Some("restaurant"), vec![], 1)];
    assert_eq!(delta(&c, &p, &s, "restaurant"), -100);
    assert_eq!(delta(&c, &p, &s, "rooms"), 0);
    assert_eq!(effect(&c, &p, &s[0]).space, code("restaurant"));
    // Neraspoređeni odjel i situacija bez odjela: Smještaj (postojeće pravilo).
    for space in [None, Some("maintenance")] {
        let s = [sit(true, space, vec![], 1)];
        assert_eq!(delta(&c, &p, &s, "rooms"), -100, "{space:?}");
        assert_eq!(effect(&c, &p, &s[0]).space, DeptCode::rooms());
    }
}

#[test] // 1 (kvar uz privremeni popravak: oba gubitka se zbrajaju)
fn fault_stacks_with_the_temporary_repair_loss() {
    // Privremeni popravak: −100 (katalog) puta faktor kruga; kvar: −100 bez faktora.
    let p = month_period(Month::Jun);
    for (bp, temp) in [(10_000_u16, -100), (5_000, -50)] {
        let c = cat(100, 200, 0, bp);
        let s = sit(true, None, vec![temporary()], 1);
        assert_eq!(equipment(&c, &p, &s), temp - 100, "faktor {bp}");
    }
}

#[test] // 2
fn urgent_repair_with_good_answer_nets_plus_one() {
    let c = leon();
    for q in [5, 4] {
        for p in periods() {
            let s = sit(true, None, vec![urgent()], q);
            assert_eq!(equipment(&c, &p, &s), 100, "q {q} {p:?}: −100 + 200");
            assert_eq!(delta(&c, &p, &[s], "rooms"), 100, "q {q} {p:?}");
        }
    }
    assert_eq!(FAULT_FIXED_MIN_QUALITY, 4);
}

#[test] // 3
fn urgent_repair_with_weak_answer_keeps_minus_one() {
    let c = leon();
    for q in [3, 1] {
        for p in periods() {
            let s = sit(true, None, vec![urgent()], q);
            assert_eq!(equipment(&c, &p, &s), -100, "q {q} {p:?}");
            assert_eq!(delta(&c, &p, &[s], "rooms"), -100, "q {q} {p:?}");
        }
    }
}

#[test] // 2 (rizik 2: jednom po situaciji, ma koliko hitnih popravaka)
fn fix_bonus_applies_once_per_situation() {
    let c = leon();
    let p = month_period(Month::Jun);
    let s = sit(true, None, vec![urgent(), urgent()], 5);
    assert_eq!(equipment(&c, &p, &s), 100, "−100 + 200, ne −100 + 400");
}

#[test] // 2 (bonus traži HITNI popravak u odgovoru)
fn fix_bonus_needs_an_urgent_repair_in_the_answer() {
    let c = leon();
    let p = month_period(Month::Jun);
    for mechanics in [vec![], vec![temporary()]] {
        let s = sit(true, None, mechanics, 5);
        assert!(equipment(&c, &p, &s) <= -100, "bez hitnog popravka nema +2");
    }
}

#[test] // 4 (karakterizacija; sav obračun za sve predloške je u content/v2_situation_fault.rs)
fn fault_rule_is_off_without_keys() {
    for old in [0, 100, 1_000] {
        let c = cat(0, 0, old, 350);
        for p in periods() {
            for q in 1..=5 {
                for mechanics in [vec![], vec![urgent()], vec![temporary()]] {
                    let (off, on) = (
                        sit(false, None, mechanics.clone(), q),
                        sit(true, None, mechanics, q),
                    );
                    assert_eq!(
                        effect(&c, &p, &on),
                        effect(&c, &p, &off),
                        "stari {old}, q {q}, {p:?}: ključevi 0 = današnji račun"
                    );
                }
            }
        }
    }
    // Današnji račun: hitan popravak = stari ključ, bez faktora kruga.
    let c = cat(0, 0, 1_000, 350);
    let s = sit(true, None, vec![urgent()], 5);
    assert_eq!(equipment(&c, &month_period(Month::Jun), &s), 1_000);
}

#[test] // 5
fn repair_without_fault_gets_no_fix_bonus() {
    let c = cat(100, 200, 100, 350);
    for p in periods() {
        let s = sit(false, None, vec![urgent()], 5);
        assert_eq!(
            equipment(&c, &p, &s),
            100,
            "samo stari ključ, bez +200 i bez −100"
        );
        let silent = sit(false, None, vec![], 1);
        assert_eq!(equipment(&c, &p, &silent), 0, "bez kvara nema −1");
    }
}

#[test] // 5 (stari ključ i novi se zbrajaju kad su oba uključena)
fn old_urgent_key_and_fault_keys_add_up() {
    let c = cat(100, 200, 100, 350);
    let s = sit(true, None, vec![urgent()], 5);
    assert_eq!(equipment(&c, &month_period(Month::Jun), &s), 200);
}

#[test] // 6
fn fault_effect_has_no_round_factor() {
    let p_month = month_period(Month::Jun);
    let p_week = period_by_code(PeriodKind::Week, "t02").unwrap();
    for bp in [350_u16, 325, 1, 10_000] {
        let c = cat(100, 200, 0, bp);
        for p in [&p_month, &p_week] {
            assert_eq!(equipment(&c, p, &sit(true, None, vec![], 1)), -100, "{bp}");
            assert_eq!(
                equipment(&c, p, &sit(true, None, vec![urgent()], 5)),
                100,
                "{bp}"
            );
        }
    }
}

#[test] // 7
fn fault_keys_out_of_range_are_rejected() {
    let ok = |f: i32, x: i32| cat(f, x, 0, 10_000);
    for (f, x) in [(0, 0), (100, 200), (10_000, 10_000)] {
        ok(f, x).validate().unwrap();
    }
    for bad in [-1, 10_001] {
        match ok(bad, 0).validate() {
            Err(CatalogError::OutOfRange(k)) => assert_eq!(k, "v2.economy.fault_condition_pts"),
            other => panic!("{bad}: {other:?}"),
        }
        match ok(0, bad).validate() {
            Err(CatalogError::OutOfRange(k)) => {
                assert_eq!(k, "v2.economy.fault_fixed_condition_pts")
            }
            other => panic!("{bad}: {other:?}"),
        }
    }
}

#[test] // 7 (degenerirani katalog: pravilo isključeno)
fn degenerate_catalog_has_the_fault_rule_off() {
    let d = degenerate(&common::test_catalog());
    assert_eq!(
        (
            d.economy.fault_condition_pts,
            d.economy.fault_fixed_condition_pts
        ),
        (0, 0)
    );
}
