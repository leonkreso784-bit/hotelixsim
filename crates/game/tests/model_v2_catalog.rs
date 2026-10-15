//! v2 igra ne čita v1 katalog: referentna cijena, robotski izbor i pokazatelji dolaze iz kataloga v2 (`base`).
//! Danas su oba kataloga ista pa greška ne bi bila vidljiva; test ih namjerno razdvoji.
mod common;

use content::{Snapshot, repo_dir};
use game::command::{GameSettings, new_game_with};
use game::*;
use sim_hotel::Catalog;
use sim_hotel_v2::PeriodKind;
use uuid::Uuid;

const BUMP: i64 = 777;

fn v2_game() -> Game {
    let snap = Snapshot::read_v2(&repo_dir()).unwrap();
    let ev: Vec<Recorded> = new_game_with(
        GameId(Uuid::from_u128(1)),
        42,
        &snap,
        common::GUEST,
        common::hotel_ids(),
        GameSettings {
            kind: PeriodKind::Week,
            situations: None,
        },
    )
    .unwrap()
    .into_iter()
    .zip(1..)
    .map(|((actor, event), seq)| Recorded { seq, actor, event })
    .collect();
    let mut g = Game::fold(&ev).unwrap();
    for _ in 0..2 {
        let r = g.current_round().unwrap();
        let ev = g.close_round(r).unwrap().unwrap();
        common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
            panic!("čeka")
        };
        for (a, e) in evs {
            common::apply_ev(&mut g, a, e);
        }
    }
    g
}

/// Namjerno kriva ekonomija: drugačiji CPOR, referentne cijene i plaća.
fn poison(cat: &mut Catalog) {
    for spec in cat.categories.values_mut() {
        spec.cpor_cents += BUMP;
        spec.wage_cents += BUMP * 10;
        for p in &mut spec.price_ref_cents.0 {
            *p += BUMP;
        }
    }
}

struct Reads {
    cpor: Vec<Option<i64>>,
    price_ref: i64,
    moves: Vec<(Actor, GameEvent)>,
}

fn reads(g: &Game) -> Reads {
    let id = common::hotel_ids()[0];
    let first = g.rounds[0];
    Reads {
        cpor: g
            .hotel_kpi(id)
            .unwrap()
            .iter()
            .map(|k| k.kpi.cpor_cents)
            .collect(),
        price_ref: g.debrief_input(id, first).unwrap().price_ref_cents,
        moves: g.robot_moves(g.rounds[2]).unwrap(),
    }
}

#[test]
fn v2_game_ignores_the_v1_catalog() {
    let mut g = v2_game();
    let before = reads(&g);
    poison(&mut g.content.catalog);
    let after = reads(&g);
    assert_eq!(
        before.cpor, after.cpor,
        "CPOR u pokazateljima čita v1 katalog"
    );
    assert_eq!(
        before.price_ref, after.price_ref,
        "referentna cijena čita v1 katalog"
    );
    assert_eq!(before.moves, after.moves, "robot čita v1 katalog");
}

#[test]
fn v2_game_reads_the_v2_base_catalog() {
    let mut g = v2_game();
    let before = reads(&g);
    let v2 = g.content.v2.as_mut().unwrap();
    poison(&mut v2.base);
    // Od Taska 14 izravni trošak po noći v2 igre je trošak po korištenju odjela Smještaj, ne CPOR kategorije.
    v2.departments
        .get_mut(&sim_hotel_v2::department::DeptCode::rooms())
        .unwrap()
        .direct_per_use_cents += BUMP;
    let after = reads(&g);
    let shifted: Vec<Option<i64>> = before.cpor.iter().map(|c| c.map(|v| v + BUMP)).collect();
    assert_eq!(
        after.cpor, shifted,
        "pokazatelji moraju koristiti izravni trošak odjela Smještaj kataloga v2"
    );
    assert_ne!(
        before.price_ref, after.price_ref,
        "referentna cijena mora doći iz v2"
    );
}
