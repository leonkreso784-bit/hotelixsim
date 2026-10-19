//! L1-13 dio 1 (A1): igra primjenjuje kvar prostora iz izvučenog predloška (`DrawnInstance.space_fault`,
//! `Game::fault`). Isto sjeme, dvije igre: s ključevima kvara u snimku i bez njih (isključeno). Do prvog kruga s
//! kvarom prostora moga hotela stanja su ista, pa razlika stanja nakon tog kruga je samo pravilo kvara. Runda 2 (Leon
//! 18.10., K1 i L1): učinak je ključ × faktor kruga × faktor razine primjerka (jedno zaokruživanje), a bonus popravka
//! dobiva hitni odgovor ocjene >= 3.
mod common;

use common::{GUEST, apply_ev, hotel_ids};
use content::{Snapshot, repo_dir};
use game::*;
use serde_json::json;
use sim_hotel::Mechanic;
use sim_hotel_v2::PeriodKind;
use sim_hotel_v2::department::DeptCode;
use sim_hotel_v2::situations::Significance;

fn snap(keys: bool) -> Snapshot {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    // L1-13-1b: test mjeri samo kvar i popravak; klik (`click_gain_bp`) ovdje je puna dobit kao prije.
    v["v2"]["economy"]
        .as_object_mut()
        .unwrap()
        .remove("click_gain_bp");
    if keys {
        // Ključevi u bodovima situacije (K1), veliki da učinak uz tjedni faktor bude vidljiv.
        v["v2"]["economy"]["fault_condition_pts"] = json!(FAULT_KEY);
        v["v2"]["economy"]["fault_fixed_condition_pts"] = json!(FIXED_KEY);
    } else {
        // Od L1-13 dijela 4 repo nosi ključeve kvara: igra „bez" ih briše (stari snimak).
        let e = v["v2"]["economy"].as_object_mut().unwrap();
        e.remove("fault_condition_pts");
        e.remove("fault_fixed_condition_pts");
    }
    Snapshot::from_value(&v).unwrap()
}

const FAULT_KEY: i64 = 5_000;
const FIXED_KEY: i64 = 10_000;

/// Najbliži cijeli broj `key × f × sig / 10^8`, polovina od nule (kao `c::div_round`): jedno zaokruživanje.
fn scaled(key: i64, f: i64, sig: i64) -> i32 {
    let n = key * f * sig;
    let d = 100_000_000_i64;
    let q = (2 * n.abs() + d) / (2 * d);
    i32::try_from(if n < 0 { -q } else { q }).unwrap()
}

/// (kvar, bonus) u bp za primjerak `code` kruga `r`: ključ × tjedni faktor kruga × faktor razine iz `weight` primjerka.
fn fault_and_fix(g: &Game, r: RoundKey, code: &str) -> (i32, i32) {
    let me = hotel_ids()[0];
    let h = g.hotels.iter().find(|h| h.setup.hotel_id == me).unwrap();
    let i = h.drawn[&r]
        .iter()
        .find(|i| i.situation.code == code)
        .unwrap();
    let cat = g.content.v2.as_ref().unwrap();
    let f = i64::from(cat.coeffs(PeriodKind::Week).unwrap().situation_effect_bp);
    let s = &cat.economy.significance_bp;
    let sig = i64::from(match Significance::from_weight(i.drawn.weight) {
        Significance::Key => s.key,
        Significance::Important => s.important,
        Significance::Everyday => s.everyday,
    });
    (scaled(FAULT_KEY, f, sig), scaled(FIXED_KEY, f, sig))
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
    let (lose, _) = fault_and_fix(&with, r, &code);
    assert!(lose > 0, "sjeme {seed}, {code}: kvar je vidljiv");
    assert_eq!(
        *delta, -lose,
        "sjeme {seed}, {code}, {dept:?}: ključ × faktor kruga × razina niže"
    );
    // Dovoljno daleko od donje granice da stiskanje na 0 ne skriva razliku.
    let was = before.iter().find(|(x, _)| x == dept).unwrap().1;
    assert!(
        was >= 200,
        "stanje {dept:?} prije kruga {was}: odaberi drugo sjeme"
    );
}

#[test] // 10 (odgovor: hitan popravak uz ocjenu >= 3 (L1) dobiva bonus, ispod 3 ostaje bez bonusa)
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
                (a.quality >= 3) == q_ok
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
            let (lose, gain) = fault_and_fix(&with, r, &code);
            let expected = if good { gain - lose } else { -lose };
            assert_eq!(
                d.iter().map(|(_, x)| *x).sum::<i32>(),
                expected,
                "sjeme {seed}, {code}, odgovor {i} (dobar: {good}): −kvar uz bonus, bez njega −kvar: {d:?}"
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
        "nijedno sjeme 1..=60 nema kvar s hitnim odgovorom ocjene >= 3"
    );
}
