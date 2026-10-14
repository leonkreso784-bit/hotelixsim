//! Zapečaćeni otisak v2 (A3): igre nad zamrznutim snimkom pravila daju otisak vezan uz `sim_hotel_v2::CORE_VERSION`.
//! Promjena obračuna bez dizanja verzije ruši test 1; nova verzija traži novi pečat (`seal_v2_digest`), stari ostaju.
mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use content::{Snapshot, fingerprint_value, repo_dir};
use game::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sim_hotel_v2::{CORE_VERSION, PeriodKind};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Human {
    Silent,
    Answers,
}
struct Case {
    name: &'static str,
    seed: u64,
    kind: PeriodKind,
    human: Human,
}
const CASES: [Case; 6] = [
    Case {
        name: "t7-odgovara",
        seed: 7,
        kind: PeriodKind::Week,
        human: Human::Answers,
    },
    Case {
        name: "m7-odgovara",
        seed: 7,
        kind: PeriodKind::Month,
        human: Human::Answers,
    },
    Case {
        name: "t42-sutnja",
        seed: 42,
        kind: PeriodKind::Week,
        human: Human::Silent,
    },
    Case {
        name: "m42-sutnja",
        seed: 42,
        kind: PeriodKind::Month,
        human: Human::Silent,
    },
    Case {
        name: "t2026-odgovara",
        seed: 2026,
        kind: PeriodKind::Week,
        human: Human::Answers,
    },
    Case {
        name: "m2026-sutnja",
        seed: 2026,
        kind: PeriodKind::Month,
        human: Human::Silent,
    },
];
#[derive(Serialize, Deserialize)]
struct Golden {
    snapshot: String,
    digests: BTreeMap<String, BTreeMap<String, String>>,
}

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_v2")
}
fn read(name: &str) -> String {
    std::fs::read_to_string(dir().join(name))
        .unwrap_or_else(|_| panic!("nema tests/fixtures_v2/{name}: v2 još nije zapečaćen"))
}
fn golden() -> Golden {
    serde_json::from_str(&read("v2-golden.json")).unwrap()
}
fn frozen_value() -> Value {
    serde_json::from_str(&read("v2-snapshot.json")).unwrap()
}

/// Cijela sezona: roboti igraju sami; igrač šuti ili odgovara redom (n-ta situacija sezone dobiva odgovor n mod broj
/// odgovora), a svaku drugu situaciju koju smije prepušta voditelju.
fn play(snap: &Snapshot, c: &Case) -> Game {
    let (mut g, me, guest) = (
        common::v2_game_from(snap, c.seed, c.kind),
        common::hotel_ids()[0],
        Actor::Guest(common::GUEST),
    );
    let mut n: usize = 0;
    while let Some(r) = g.current_round() {
        if c.human == Human::Answers {
            let list: Vec<(String, usize)> = g
                .situations_of_hotel(me, r)
                .unwrap()
                .iter()
                .map(|t| (t.code.clone(), t.answers.len()))
                .collect();
            for (code, len) in list {
                let ev = if n % 2 == 1 && !g.situation_mandatory(me, &code).unwrap() {
                    g.delegate_situation(me, &code).unwrap()
                } else {
                    g.choose_answer(me, &code, u8::try_from(n % len).unwrap())
                        .unwrap()
                };
                common::apply_ev(&mut g, guest.clone(), ev);
                n += 1;
            }
        }
        let ev = g.close_round(r).unwrap().unwrap();
        common::apply_ev(&mut g, guest.clone(), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else {
            panic!("obračun čeka")
        };
        for (a, e) in evs {
            common::apply_ev(&mut g, a, e);
        }
    }
    g
}

/// Kanonski zapis: vrsta, autor i JSON svakog događaja (kako ide u bazu); iz `GameCreated` samo krugovi i hoteli.
fn digest(g: &Game) -> String {
    let mut h = Sha256::new();
    for r in &g.history {
        let payload = match &r.event {
            GameEvent::GameCreated(c) => {
                json!({ "months": c.months, "hotels": c.hotels, "rounds": c.rounds })
            }
            e => e.to_payload(),
        };
        let (kind, actor, body) = (
            r.event.kind(),
            r.actor.as_db(),
            serde_json::to_vec(&payload).unwrap(),
        );
        for part in [kind.as_bytes(), actor.as_bytes(), &body[..]] {
            h.update(part);
            h.update(b"\n");
        }
    }
    format!("{:x}", h.finalize())
}
fn digests(snap: &Snapshot) -> BTreeMap<String, String> {
    CASES
        .iter()
        .map(|c| (c.name.to_string(), digest(&play(snap, c))))
        .collect()
}

#[test] // 1
fn v2_digest_is_sealed_for_core_version() {
    let g = golden();
    let want = g.digests.get(CORE_VERSION).unwrap_or_else(|| {
        panic!(
            "jezgra v2 {CORE_VERSION} nema pečat: nova verzija traži novi otisak — \
         HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest"
        )
    });
    let got = digests(&Snapshot::from_value(&frozen_value()).unwrap());
    assert_eq!(
        got.keys().collect::<Vec<_>>(),
        want.keys().collect::<Vec<_>>(),
        "slučajevi pečata"
    );
    for (name, d) in &got {
        assert_eq!(
            d, &want[name],
            "{name}: obračun v2 se promijenio, a CORE_VERSION {CORE_VERSION} nije dignut — \
            digni verziju u crates/sim-hotel-v2/Cargo.toml i zapečati novi otisak"
        );
    }
}

#[test] // 2
fn frozen_snapshot_matches_its_seal() {
    assert_eq!(
        fingerprint_value(&frozen_value()),
        golden().snapshot,
        "zamrznuti snimak se ne mijenja: nova pravila = nova verzija jezgre, ne novi snimak"
    );
}

#[test] // 3: pečat pokriva oba kruga, sva tri profila u svakoj igri i sve putove odgovora igrača
fn digest_cases_cover_rounds_profiles_and_answer_paths() {
    let snap = Snapshot::from_value(&frozen_value()).unwrap();
    let me = common::hotel_ids()[0];
    let (mut answered, mut delegated, mut silent) = (0, 0, 0);
    for c in &CASES {
        let g = play(&snap, c);
        let profiles: BTreeSet<&str> = g.hotels.iter().map(|h| h.setup.profile.as_str()).collect();
        assert_eq!(profiles, BTreeSet::from(["A", "B", "C"]), "{}", c.name);
        for r in &g.history {
            match &r.event {
                GameEvent::AnswerChosen(a) if a.hotel_id == me => answered += 1,
                GameEvent::SituationDelegated(_) => delegated += 1,
                GameEvent::RoundResolved(rr) => {
                    silent += rr
                        .hotels
                        .iter()
                        .filter(|h| h.hotel_id == me)
                        .flat_map(|h| &h.situations)
                        .filter(|s| s.answer.is_none())
                        .count()
                }
                _ => {}
            }
        }
    }
    assert!(
        answered > 0 && delegated > 0 && silent > 0,
        "odgovori {answered}, prepušteno {delegated}, šutnja {silent}"
    );
    assert!(
        CASES.iter().any(|c| c.kind == PeriodKind::Week)
            && CASES.iter().any(|c| c.kind == PeriodKind::Month)
    );
}

#[test] // 4: otisak reagira na pravila (bez pečata; trenutni sadržaj)
fn digest_reacts_to_a_rule_change() {
    let v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let mut changed = v.clone();
    changed["v2"]["periods"]["week"]["situation_effect_bp"] = json!(650);
    let (a, b) = (
        Snapshot::from_value(&v).unwrap(),
        Snapshot::from_value(&changed).unwrap(),
    );
    assert_ne!(digest(&play(&a, &CASES[0])), digest(&play(&b, &CASES[0])));
}

#[test]
#[ignore = "piše pečat v2; pokreće se samo uz dizanje CORE_VERSION"]
#[allow(clippy::disallowed_methods)] // samo generator pečata čita okolinu
fn seal_v2_digest() {
    assert_eq!(std::env::var("HOTELIXSIM_SEAL_V2").as_deref(), Ok("1"));
    std::fs::create_dir_all(dir()).unwrap();
    let pretty = |v: &Value| format!("{}\n", serde_json::to_string_pretty(v).unwrap());
    let path = dir().join("v2-snapshot.json");
    if !path.exists() {
        std::fs::write(
            &path,
            pretty(&Snapshot::read_v2(&repo_dir()).unwrap().to_value()),
        )
        .unwrap();
    }
    let gpath = dir().join("v2-golden.json");
    let mut g = if gpath.exists() {
        golden()
    } else {
        Golden {
            snapshot: fingerprint_value(&frozen_value()),
            digests: BTreeMap::new(),
        }
    };
    assert!(
        !g.digests.contains_key(CORE_VERSION),
        "pečat za {CORE_VERSION} postoji i ne prepisuje se — digni verziju"
    );
    g.digests.insert(
        CORE_VERSION.to_string(),
        digests(&Snapshot::from_value(&frozen_value()).unwrap()),
    );
    std::fs::write(&gpath, pretty(&serde_json::to_value(&g).unwrap())).unwrap();
}
