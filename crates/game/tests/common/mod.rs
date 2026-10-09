//! Zajednički pomoćnici testova (dijele ih `round.rs` i `resolve.rs`).
#![allow(dead_code)]

use content::{Snapshot, repo_dir};
use game::command::new_game;
use game::*;
use uuid::Uuid;

pub fn hotel_ids() -> [HotelId; 4] {
    [11, 12, 13, 14].map(|n| HotelId(Uuid::from_u128(n)))
}
pub const GUEST: GuestId = GuestId(Uuid::from_u128(99));

/// Nova igra iz zadanog snimka kao zapisani događaji (redni brojevi od 1).
pub fn new_game_events(snap: &Snapshot) -> Vec<Recorded> {
    new_game(GameId(Uuid::from_u128(1)), 42, snap, GUEST, hotel_ids())
        .unwrap()
        .into_iter()
        .zip(1..)
        .map(|((actor, event), seq)| Recorded { seq, actor, event })
        .collect()
}

/// Nova igra kao zapisani događaji (redni brojevi od 1).
pub fn created(seed: u64) -> Vec<Recorded> {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    new_game(GameId(Uuid::from_u128(1)), seed, &snap, GUEST, hotel_ids())
        .unwrap()
        .into_iter()
        .zip(1..)
        .map(|((actor, event), seq)| Recorded { seq, actor, event })
        .collect()
}

/// Igra sa zatvorenim lipnjem (`RoundClosed`), obračun još čeka.
pub fn closed_june_game() -> Game {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    let ev = g
        .close_round(sim_hotel::Month::Jun)
        .unwrap()
        .expect("lipanj se zatvara");
    push(&mut log, Actor::Guest(GUEST), ev);
    Game::fold(&log).unwrap()
}

pub fn push(log: &mut Vec<Recorded>, actor: Actor, event: GameEvent) {
    let seq = log.last().map_or(1, |r| r.seq + 1);
    log.push(Recorded { seq, actor, event });
}
