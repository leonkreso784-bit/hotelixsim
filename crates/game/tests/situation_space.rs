//! B1a (ADR-042 t. 1): u igri hitan popravak odgovora diže stanje odjela izvučene situacije (prostor primjerka), a ne
//! Smještaja. Igrač B šuti, igrač A bira odgovor s hitnim popravkom; razlika stanja odjela je ≥ +10 bodova.
mod common;

use common::{GUEST, apply_ev, hotel_ids};
use content::{Snapshot, repo_dir};
use game::*;
use serde_json::json;
use sim_hotel::Mechanic;
use sim_hotel_v2::PeriodKind;
use sim_hotel_v2::department::DeptCode;

/// Pogodak: igra pred zatvaranjem kruga `round`, šifra situacije mog hotela i indeks odgovora s hitnim popravkom.
struct Find {
    game: Game,
    round: RoundKey,
    situation: String,
    answer: u8,
    dept: DeptCode,
}

fn urgent_answer(g: &Game, r: RoundKey, code: &str) -> Option<u8> {
    let me = hotel_ids()[0];
    let t = g
        .situations_of_hotel(me, r)
        .unwrap()
        .into_iter()
        .find(|t| t.code == code)?;
    t.answers
        .iter()
        .position(|a| {
            a.mechanics
                .iter()
                .any(|m| matches!(m, Mechanic::EquipmentRepair { urgent: true }))
        })
        .map(|i| u8::try_from(i).unwrap())
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

fn condition(g: &Game, d: &DeptCode) -> i32 {
    let CoreState::V2(s) = g.latest_state(hotel_ids()[0]).unwrap() else {
        panic!("v2 stanje")
    };
    s.departments[d].condition_pts
}

/// Prvi krug (tjedna igra, šuti se) u kojem moj hotel ima primjerak s odjelom `d` (nije Smještaj ni neraspoređen),
/// stanjem `d` ≤ 8 500 i odgovorom s hitnim popravkom.
fn find(snap: &Snapshot, seed: u64) -> Option<Find> {
    find_if(snap, seed, &|g, _, d| {
        let class = g.content.v2.as_ref().unwrap().dept(d).unwrap().class;
        d != &DeptCode::rooms() && !class.is_undistributed() && condition(g, d) <= 8_500
    })
}

/// Kao `find`, ali primjerak bira `accept(igra, šifra situacije, odjel primjerka)`.
fn find_if(
    snap: &Snapshot,
    seed: u64,
    accept: &dyn Fn(&Game, &str, &DeptCode) -> bool,
) -> Option<Find> {
    let me = hotel_ids()[0];
    let mut g = common::v2_game_from(snap, seed, PeriodKind::Week);
    while let Some(r) = g.current_round() {
        let h = g.hotels.iter().find(|h| h.setup.hotel_id == me).unwrap();
        for i in h.drawn.get(&r).into_iter().flatten() {
            let Some(d) = i.drawn.department.clone() else {
                continue;
            };
            if !accept(&g, &i.situation.code, &d) {
                continue;
            }
            let Some(answer) = urgent_answer(&g, r, &i.situation.code) else {
                continue;
            };
            return Some(Find {
                situation: i.situation.code.clone(),
                round: r,
                answer,
                dept: d,
                game: g.clone(),
            });
        }
        close_and_resolve(&mut g, r);
    }
    None
}

const SEEDS: u64 = 100;

#[test]
fn drawn_department_gets_the_urgent_repair() {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let me = hotel_ids()[0];
    let (seed, f) = (1..=40)
        .find_map(|seed| find(&snap, seed).map(|f| (seed, f)))
        .expect("nijedno sjeme 1..=40 nema primjerak s odjelom, hitnim odgovorom i stanjem <= 8 500: proširi raspon");
    let (mut a, mut b) = (f.game.clone(), f.game);
    let ev = a.choose_answer(me, &f.situation, f.answer).unwrap();
    apply_ev(&mut a, Actor::Guest(GUEST), ev);
    for g in [&mut a, &mut b] {
        close_and_resolve(g, f.round);
    }
    let bonus = a
        .content
        .v2
        .as_ref()
        .unwrap()
        .economy
        .repair_urgent_condition_pts;
    assert!(
        bonus > 0,
        "bonus hitnog popravka iz sadržaja mora biti pozitivan"
    );
    assert!(
        condition(&a, &f.dept) - condition(&b, &f.dept) >= bonus,
        "sjeme {seed}, krug {:?}: hitan popravak diže stanje odjela {:?}: A {} / B {}",
        f.round,
        f.dept,
        condition(&a, &f.dept),
        condition(&b, &f.dept)
    );
    assert_eq!(
        condition(&a, &DeptCode::rooms()),
        condition(&b, &DeptCode::rooms()),
        "Smještaj ne dobiva popravak tuđeg odjela"
    );
}

/// B1b (ADR-042 t. 7): `condition_of` premješta hitan popravak u taj prostor. P-INSPEKCIJA je na Restoranu (troši se),
/// bez uvjeta stanja i s hitnim odgovorom 0; snimak joj daje `condition_of: "rooms"`, pa popravak ide u Smještaj, ne u
/// Restoran. Izvorno je `major: low` i u igri se ne izvuče ni u 300 sjemena, pa joj snimak briše `major`
/// (P-BRAVE i P-CURENJE-VODE na Održavanju ne bi razlikovali stari i novi put: popravak neraspoređenog odjela ionako
/// ne ide u Održavanje).
#[test]
fn condition_of_moves_the_repair_to_that_space() {
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let i = v["v2"]["situation_templates"]["templates"]
        .as_array()
        .unwrap()
        .iter()
        .position(|t| t["code"] == "P-INSPEKCIJA")
        .unwrap();
    let t = v["v2"]["situation_templates"]["templates"][i]
        .as_object_mut()
        .unwrap();
    t.remove("major");
    t.insert("condition_of".into(), json!("rooms"));
    let snap = Snapshot::from_value(&v).unwrap();
    let me = hotel_ids()[0];
    let (seed, f) = (1..=SEEDS)
        .find_map(|seed| {
            find_if(&snap, seed, &|g, code, d| {
                code.starts_with("P-INSPEKCIJA.")
                    && d.as_str() == "restaurant"
                    && condition(g, &DeptCode::rooms()) <= 9_000
                    && condition(g, d) <= 9_000
            })
            .map(|f| (seed, f))
        })
        .expect(
            "nijedno sjeme nema P-INSPEKCIJA na Restoranu sa stanjima <= 9 000: proširi raspon",
        );
    let (mut a, mut b) = (f.game.clone(), f.game.clone());
    let ev = a.choose_answer(me, &f.situation, f.answer).unwrap();
    apply_ev(&mut a, Actor::Guest(GUEST), ev);
    for g in [&mut a, &mut b] {
        close_and_resolve(g, f.round);
    }
    let bonus = a
        .content
        .v2
        .as_ref()
        .unwrap()
        .economy
        .repair_urgent_condition_pts;
    assert!(bonus > 0);
    let rooms = DeptCode::rooms();
    assert!(
        condition(&a, &rooms) - condition(&b, &rooms) >= bonus,
        "sjeme {seed}, krug {:?}: popravak ide u Smještaj (condition_of): A {} / B {}",
        f.round,
        condition(&a, &rooms),
        condition(&b, &rooms)
    );
    assert_eq!(
        condition(&a, &f.dept),
        condition(&b, &f.dept),
        "Restoran ne dobiva popravak kad je condition_of Smještaj"
    );
}
