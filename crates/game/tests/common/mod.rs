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

pub fn push(log: &mut Vec<Recorded>, actor: Actor, event: GameEvent) {
    let seq = log.last().map_or(1, |r| r.seq + 1);
    log.push(Recorded { seq, actor, event });
}
