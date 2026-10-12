//! Mentor stare v1 igre bajt-isto (obnova-v2-spec §8 O-5): ulaz mentora (`DebriefInput`), činjenice koje dobiva AI
//! (`debrief_facts`) i rezerva bez AI-ja (`fallback_debrief`) za zamrznuti dnevnik iz Taska 1 moraju ostati točno
//! kakvi su bili prije obnove v2. Otisak `v1-mentor.json` napisan je JEDNOM, kodom prije O-5, i nikad se ne
//! regenerira (čuva ga i `fixtures_sealed.rs`); ako test padne, kriv je novi kod.

use std::path::PathBuf;

use game::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sim_hotel::Month;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[derive(Deserialize)]
struct StoredEvent {
    seq: i64,
    actor: String,
    kind: String,
    schema_ver: i32,
    payload: Value,
}

/// Mentor jednog hotela u jednom krugu stare igre.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct MentorPrint {
    round: String,
    hotel: String,
    input: Value,
    facts: Value,
    fallback: String,
}

fn old_v1_game() -> Game {
    let text = std::fs::read_to_string(fixtures().join("v1-game-log.json")).unwrap();
    let log: Vec<StoredEvent> = serde_json::from_str(&text).unwrap();
    let recs: Vec<Recorded> = log
        .iter()
        .map(|e| Recorded {
            seq: e.seq,
            actor: Actor::parse(&e.actor).unwrap(),
            event: GameEvent::from_record(&e.kind, e.schema_ver, e.payload.clone()).unwrap(),
        })
        .collect();
    Game::fold(&recs).unwrap()
}

fn mentor_prints(g: &Game) -> Vec<MentorPrint> {
    let mut out = Vec::new();
    for m in [Month::Jun, Month::Jul] {
        for h in &g.hotels {
            let input = g.debrief_input(h.setup.hotel_id, m).unwrap();
            out.push(MentorPrint {
                round: RoundKey::from(m).code(),
                hotel: h.setup.name.clone(),
                input: serde_json::to_value(&input).unwrap(),
                facts: serde_json::to_value(debrief_facts(&input)).unwrap(),
                fallback: fallback_debrief(&input),
            });
        }
    }
    out
}

#[test]
#[ignore = "piše otisak mentora v1; pokrenut JEDNOM u O-5 kodom prije obnove v2"]
#[allow(clippy::disallowed_methods)] // samo generator otiska čita okolinu
fn write_v1_mentor_fixture() {
    assert_eq!(
        std::env::var("HOTELIXSIM_WRITE_V1_MENTOR").as_deref(),
        Ok("1")
    );
    let prints = mentor_prints(&old_v1_game());
    let text = format!("{}\n", serde_json::to_string_pretty(&prints).unwrap());
    std::fs::write(fixtures().join("v1-mentor.json"), text).unwrap();
}

#[test]
fn old_v1_game_mentor_input_facts_and_fallback_are_frozen() {
    let text = std::fs::read_to_string(fixtures().join("v1-mentor.json")).unwrap();
    let frozen: Vec<MentorPrint> = serde_json::from_str(&text).unwrap();
    let now = mentor_prints(&old_v1_game());
    assert_eq!(now.len(), 8, "dva kruga × četiri hotela");
    for (a, b) in now.iter().zip(&frozen) {
        assert_eq!(
            serde_json::to_string(&a.input).unwrap(),
            serde_json::to_string(&b.input).unwrap(),
            "ulaz mentora {} {}",
            a.round,
            a.hotel
        );
        assert_eq!(a, b, "mentor {} {}", a.round, a.hotel);
    }
    assert_eq!(now.len(), frozen.len());
}
