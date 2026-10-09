//! Zajednički pomoćnici testova (dijele ih `round.rs` i `resolve.rs`).
#![allow(dead_code)]

use content::{Snapshot, repo_dir};
use game::command::new_game;
use game::event::AiSource;
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

pub fn guest() -> GuestId {
    GUEST
}

/// Igra s otvorenim lipnjem i hotelom igrača (prvi hotel).
pub fn open_june_game() -> (Game, HotelId) {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let g = Game::fold(&new_game_events(&snap)).unwrap();
    (g, hotel_ids()[0])
}

/// Primijeni događaj na igru sa sljedećim rednim brojem.
pub fn apply_ev(g: &mut Game, actor: Actor, event: GameEvent) {
    let seq = g.last_seq + 1;
    g.apply(Recorded { seq, actor, event }).unwrap();
}

pub fn my_hotel(_g: &Game) -> HotelId {
    hotel_ids()[0]
}

pub fn meta() -> AiMeta {
    AiMeta {
        model: "test-model".into(),
        prompt: "test-prompt".into(),
        input_tokens: 10,
        output_tokens: 5,
    }
}

/// Nacrt presude s točno `met` ispunjenih kriterija (prvih `met`) i najbližim odgovorom `nearest`.
pub fn draft_with(met: usize, nearest: u8) -> VerdictDraft {
    VerdictDraft {
        criteria: (1u8..=5)
            .map(|n| CriterionVerdict {
                n,
                met: usize::from(n) <= met,
                reason: format!("razlog {n}"),
            })
            .collect(),
        nearest_answer: nearest,
        rationale: "obrazloženje".into(),
    }
}

pub fn draft_all_met(nearest: u8) -> VerdictDraft {
    draft_with(5, nearest)
}

const FREE_TEXT: &str = "Ispričat ću se i ponuditi zamjenu sobe.";

/// Lipanj zatvoren, S1 je slobodan odgovor bez presude.
pub fn june_closed_with_pending_response() -> Game {
    let (mut g, h) = open_june_game();
    let ev = g.submit_response(h, "S1", FREE_TEXT).unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    let ev = g.close_round(sim_hotel::Month::Jun).unwrap().unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    g
}

fn run_resolve(g: &mut Game) {
    let ResolveStep::Done(evs) = g.resolve_job(sim_hotel::Month::Jun, false).unwrap() else {
        panic!("obračun je trebao biti gotov");
    };
    for (a, e) in evs {
        apply_ev(g, a, e);
    }
}

/// Lipanj obračunat sa slobodnim odgovorom na S1 i presudom (`q` kriterija, najbliži odgovor `i`).
pub fn june_resolved_with_free_verdict_game(q: usize, i: u8) -> Game {
    let (mut g, h) = open_june_game();
    let ev = g.submit_response(h, "S1", FREE_TEXT).unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    let v = g.validate_verdict("S1", draft_with(q, i)).unwrap();
    let ev = g.record_verdict(h, "S1", v, meta()).unwrap();
    apply_ev(&mut g, Actor::System, ev);
    let ev = g.close_round(sim_hotel::Month::Jun).unwrap().unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    run_resolve(&mut g);
    // Mentor za lipanj (rezervni tekst, kao kad AI ne radi).
    let ev = g
        .record_debrief(
            h,
            sim_hotel::Month::Jun,
            "Mentor: dobar početak.".to_owned(),
            AiSource::Fallback,
            None,
        )
        .unwrap()
        .unwrap();
    apply_ev(&mut g, Actor::System, ev);
    g
}

pub fn june_resolved_with_free_verdict(q: usize, i: u8) -> RoundResolved {
    june_resolved_with_free_verdict_game(q, i).resolved[&sim_hotel::Month::Jun].clone()
}

fn june_resolved_offered_game(i: u8) -> Game {
    let (mut g, h) = open_june_game();
    let ev = g.choose_answer(h, "S1", i).unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    let ev = g.close_round(sim_hotel::Month::Jun).unwrap().unwrap();
    apply_ev(&mut g, Actor::Guest(GUEST), ev);
    run_resolve(&mut g);
    g
}

pub fn june_resolved_with_offered(i: u8) -> RoundResolved {
    june_resolved_offered_game(i).resolved[&sim_hotel::Month::Jun].clone()
}

pub fn june_resolved_game() -> Game {
    june_resolved_offered_game(0)
}

/// Obračun hotela igrača (prvi hotel).
pub fn my_resolution(rr: &RoundResolved) -> &HotelResolution {
    &rr.hotels[0]
}
