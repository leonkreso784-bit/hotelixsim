//! L1-13 dio 1 (A1): igra primjenjuje kvar prostora iz izvučenog predloška (`DrawnInstance.space_fault`,
//! `Game::fault`). Isto sjeme, dvije igre: s ključevima kvara u snimku (−1 / +2) i bez njih (repo, isključeno). Do
//! prvog kruga s kvarom prostora moga hotela stanja su ista, pa razlika stanja nakon tog kruga je samo pravilo kvara.
mod common;

use common::{GUEST, apply_ev, hotel_ids};
use content::{Snapshot, repo_dir};
use game::*;
use serde_json::json;
use sim_hotel::Mechanic;
use sim_hotel_v2::PeriodKind;
use sim_hotel_v2::department::DeptCode;

fn snap(keys: bool) -> Snapshot {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    // L1-13-1b: test mjeri samo kvar i popravak; klik (`click_gain_bp`) ovdje je puna dobit kao prije.
    v["v2"]["economy"]
        .as_object_mut()
        .unwrap()
        .remove("click_gain_bp");
    if keys {
        v["v2"]["economy"]["fault_condition_pts"] = json!(100);
        v["v2"]["economy"]["fault_fixed_condition_pts"] = json!(200);
    }
    Snapshot::from_value(&v).unwrap()
}

fn close_and_resolve(g: &mut Game, r: RoundKey) {
    let ev = g.close_round(r).unwrap().unwrap();
    apply_ev(g, Actor::Guest(GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
        panic!("obračun je trebao biti gotov")
    };
    for (a, e) in evs {
        apply_ev(g, a, e);
    }
}

fn conditions(g: &Game) -> Vec<(DeptCode, i32)> {
    let CoreState::V2(s) = g.latest_state(hotel_ids()[0]).unwrap() else {
        panic!("v2 stanje")
    };
    s.departments
        .iter()
        .map(|(d, st)| (d.clone(), st.condition_pts))
        .collect()
}

/// Šifre primjeraka kvara prostora mojeg hotela u krugu `r`.
fn fault_codes(g: &Game, r: RoundKey) -> Vec<String> {
    let me = hotel_ids()[0];
    let h = g.hotels.iter().find(|h| h.setup.hotel_id == me).unwrap();
    h.drawn
        .get(&r)
        .into_iter()
        .flatten()
        .filter(|i| i.space_fault)
        .map(|i| i.situation.code.clone())
        .collect()
}

/// Prvi krug s TOČNO jednim kvarom prostora mojeg hotela, uz nepromijenjena stanja prije njega: igra `with` (ključevi)
/// i `without` (repo) pred zatvaranjem tog kruga, plus šifra primjerka.
fn first_fault_round(seed: u64) -> Option<(Game, Game, RoundKey, String)> {
    let mut with = common::v2_game_from(&snap(true), seed, PeriodKind::Week);
    let mut without = common::v2_game_from(&snap(false), seed, PeriodKind::Week);
    while let Some(r) = without.current_round() {
        let faults = fault_codes(&without, r);
        if !faults.is_empty() {
            if faults.len() != 1 || fault_codes(&with, r) != faults {
                return None;
            }
            return Some((with, without, r, faults[0].clone()));
        }
        assert!(fault_codes(&with, r).is_empty(), "isti izvlačeni predlošci");
        for g in [&mut with, &mut without] {
            close_and_resolve(g, r);
        }
    }
    None
}

/// Razlika stanja po odjelima (`with` − `without`) osim nula.
fn diff(with: &Game, without: &Game) -> Vec<(DeptCode, i32)> {
    conditions(with)
        .into_iter()
        .zip(conditions(without))
        .filter_map(|((d, a), (_, b))| (a != b).then_some((d, a - b)))
        .collect()
}

fn find() -> (u64, Game, Game, RoundKey, String) {
    (1..=60)
        .find_map(|s| first_fault_round(s).map(|(a, b, r, c)| (s, a, b, r, c)))
        .expect(
            "nijedno sjeme 1..=60 nema prvi krug s točno jednim kvarom prostora: proširi raspon",
        )
}

#[test] // 10
fn game_applies_fault_from_drawn_template() {
    let (seed, mut with, mut without, r, code) = find();
    let before = conditions(&without);
    assert_eq!(conditions(&with), before, "do tog kruga stanja su ista");
    for g in [&mut with, &mut without] {
        close_and_resolve(g, r); // šutnja
    }
    let d = diff(&with, &without);
    assert_eq!(
        d.len(),
        1,
        "sjeme {seed}, krug {r:?}, {code}: kvar spušta stanje točno jednog prostora: {d:?}"
    );
    let (dept, delta) = &d[0];
    assert_eq!(*delta, -100, "sjeme {seed}, {code}, {dept:?}: 100 bp niže");
    // Dovoljno daleko od donje granice da stiskanje na 0 ne skriva razliku.
    let was = before.iter().find(|(x, _)| x == dept).unwrap().1;
    assert!(
        was >= 200,
        "stanje {dept:?} prije kruga {was}: odaberi drugo sjeme"
    );
}

#[test] // 10 (odgovor: hitan popravak uz ocjenu >= 4 diže +200, ispod 4 ostaje bez bonusa)
fn game_applies_fix_bonus_only_for_good_urgent_answers() {
    let me = hotel_ids()[0];
    let mut checked = (false, false);
    for seed in 1..=60_u64 {
        let Some((with, without, r, code)) = first_fault_round(seed) else {
            continue;
        };
        let sit = with
            .situations_of_hotel(me, r)
            .unwrap()
            .into_iter()
            .find(|t| t.code == code)
            .unwrap();
        let urgent = |q_ok: bool| {
            sit.answers.iter().position(|a| {
                (a.quality >= 4) == q_ok
                    && a.mechanics
                        .iter()
                        .any(|m| matches!(m, Mechanic::EquipmentRepair { urgent: true }))
            })
        };
        for good in [true, false] {
            let Some(i) = urgent(good) else { continue };
            let (mut a, mut b) = (with.clone(), without.clone());
            for g in [&mut a, &mut b] {
                let ev = g
                    .choose_answer(me, &code, u8::try_from(i).unwrap())
                    .unwrap();
                apply_ev(g, Actor::Guest(GUEST), ev);
                close_and_resolve(g, r);
            }
            let d = diff(&a, &b);
            let expected = if good { 100 } else { -100 };
            assert_eq!(
                d.iter().map(|(_, x)| *x).sum::<i32>(),
                expected,
                "sjeme {seed}, {code}, odgovor {i} (dobar: {good}): −100 uz bonus +200 → +100, bez njega −100: {d:?}"
            );
            if good {
                checked.0 = true;
            } else {
                checked.1 = true;
            }
        }
        if checked == (true, true) {
            break;
        }
    }
    assert!(
        checked.0,
        "nijedno sjeme 1..=60 nema kvar s hitnim odgovorom ocjene >= 4"
    );
}
