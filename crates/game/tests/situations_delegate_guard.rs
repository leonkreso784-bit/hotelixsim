//! A-1 čuvari regresije (zeleni i prije i poslije A-1; ne ovise o novom sučelju): roboti nikad ne prepuštaju, a
//! obračun bez prepuštanja ostaje bajt-isti (stari `SituationResult` nema polje `delegated`).

mod common;

use game::*;
use serde_json::Value;

const DELEGATED: &str = "SituationDelegated";

fn play_all(mut g: Game) -> Game {
    while let Some(r) = g.current_round() {
        let ev = g.close_round(r).unwrap().unwrap();
        common::apply_ev(&mut g, Actor::Guest(common::GUEST), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
            panic!("obračun čeka")
        };
        for (a, e) in evs {
            common::apply_ev(&mut g, a, e);
        }
    }
    g
}

fn has_key(v: &Value, key: &str) -> bool {
    match v {
        Value::Object(m) => m.contains_key(key) || m.values().any(|x| has_key(x, key)),
        Value::Array(a) => a.iter().any(|x| has_key(x, key)),
        _ => false,
    }
}

#[test]
fn robots_never_delegate_and_results_without_delegation_have_no_flag() {
    let games = [
        play_all(common::v2_weekly_game_resolved_through("none")),
        play_all(common::v2_monthly_game_resolved_through("none")),
        play_all(Game::fold(&common::created(42)).unwrap()),
    ];
    for g in &games {
        assert!(
            !g.history.iter().any(|r| r.event.kind() == DELEGATED),
            "robot ili sustav je prepustio situaciju"
        );
        for r in &g.history {
            if r.event.kind() == "RoundResolved" {
                assert!(
                    !has_key(&r.event.to_payload(), "delegated"),
                    "obračun bez prepuštanja mora biti bajt-isti (bez polja `delegated`)"
                );
            }
        }
    }
}

#[test]
fn old_situation_result_record_roundtrips_byte_same() {
    for old in [
        r#"{"code":"S1","answer":null,"quality":1}"#,
        r#"{"code":"S2","answer":1,"quality":4}"#,
        r#"{"code":"P-KLIMA.t03","answer":null,"quality":3,"free_text":true}"#,
    ] {
        let back: SituationResult = serde_json::from_str(old).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), old);
    }
}
