//! Zapečaćeni otisak v2 (A3): igre nad zamrznutim snimkom pravila daju otisak vezan uz `sim_hotel_v2::CORE_VERSION`.
//! Promjena obračuna bez dizanja verzije ruši test 1; nova verzija traži novi pečat (`seal_v2_digest`), stari ostaju.
//! Od B2a otisak pokriva i broj situacija iz postavki igre (`GameCreated.situation_counts`, I3).
mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use content::{Snapshot, fingerprint_value, repo_dir};
use game::command::GameSettings;
use game::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sim_hotel::Maintenance;
use sim_hotel_v2::situations::SituationCounts;
use sim_hotel_v2::{CORE_VERSION, PeriodKind};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Human {
    Silent,
    Answers,
    /// Slobodni odgovor s presudom (I3): na svaku situaciju s likom šalje slobodan tekst, presuda se zapiše kao u igri
    /// (`submit_response` + `validate_verdict_for` + `record_verdict`); ostale situacije šute.
    FreeAnswer,
    /// Plan čovjeka (I3): prije zatvaranja svakog kruga postavi svoj plan (`set_plan`) umjesto zadanog.
    Plans,
}
struct Case {
    name: &'static str,
    seed: u64,
    kind: PeriodKind,
    human: Human,
    /// Broj situacija iz postavki igre (niski/srednji/visoki); `None` = zadani za vrstu kruga.
    situations: Option<(u8, u8, u8)>,
}
const CASES: [Case; 10] = [
    Case {
        name: "t7-odgovara",
        seed: 7,
        kind: PeriodKind::Week,
        human: Human::Answers,
        situations: None,
    },
    Case {
        name: "m7-odgovara",
        seed: 7,
        kind: PeriodKind::Month,
        human: Human::Answers,
        situations: None,
    },
    Case {
        name: "t42-sutnja",
        seed: 42,
        kind: PeriodKind::Week,
        human: Human::Silent,
        situations: None,
    },
    Case {
        name: "m42-sutnja",
        seed: 42,
        kind: PeriodKind::Month,
        human: Human::Silent,
        situations: None,
    },
    Case {
        name: "t2026-odgovara",
        seed: 2026,
        kind: PeriodKind::Week,
        human: Human::Answers,
        situations: None,
    },
    Case {
        name: "m2026-sutnja",
        seed: 2026,
        kind: PeriodKind::Month,
        human: Human::Silent,
        situations: None,
    },
    Case {
        name: "t7-broj-578",
        seed: 7,
        kind: PeriodKind::Week,
        human: Human::Answers,
        situations: Some((5, 7, 8)),
    },
    Case {
        name: "m42-broj-333",
        seed: 42,
        kind: PeriodKind::Month,
        human: Human::Silent,
        situations: Some((3, 3, 3)),
    },
    Case {
        name: "t42-slobodni",
        seed: 42,
        kind: PeriodKind::Week,
        human: Human::FreeAnswer,
        situations: None,
    },
    Case {
        name: "m42-plan",
        seed: 42,
        kind: PeriodKind::Month,
        human: Human::Plans,
        situations: None,
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

/// Slobodni odgovor na svaku situaciju moga hotela koja ima lika, s presudom (3 od 5 kriterija, najbliži odgovor 0).
fn free_answers(g: &mut Game, r: RoundKey) {
    let (me, guest) = (common::hotel_ids()[0], Actor::Guest(common::GUEST));
    let codes: Vec<String> = g
        .situations_of_hotel(me, r)
        .unwrap()
        .iter()
        .filter(|t| g.ai_entry_for(me, &t.code).is_some_and(|a| a.free_text))
        .map(|t| t.code.clone())
        .collect();
    for code in codes {
        let ev = g
            .submit_response(me, &code, "Ispričat ću se i ponuditi zamjenu sobe.")
            .unwrap();
        common::apply_ev(g, guest.clone(), ev);
        let v = g
            .validate_verdict_for(me, &code, common::draft_with(3, 0))
            .unwrap();
        let ev = g.record_verdict(me, &code, v, common::meta()).unwrap();
        common::apply_ev(g, Actor::System, ev);
    }
}

/// Vlastiti plan čovjeka: intenzivno održavanje u svakom odjelu i odgovaranje na recenzije (ostalo zadano).
fn own_plan(g: &mut Game, r: RoundKey) {
    let (me, guest) = (common::hotel_ids()[0], Actor::Guest(common::GUEST));
    let CorePlan::V2(mut p) = g.effective_plan(me, r).unwrap() else {
        panic!("v2 plan")
    };
    p.reply_reviews = true;
    for d in p.departments.values_mut() {
        d.maintenance = Maintenance::Intensive;
    }
    let ev = g.set_plan(me, r, CorePlan::V2(p)).unwrap();
    common::apply_ev(g, guest, ev);
}

/// Cijela sezona: roboti igraju sami; igrač šuti ili odgovara redom (n-ta situacija sezone dobiva odgovor n mod broj
/// odgovora), a svaku drugu situaciju koju smije prepušta voditelju.
fn play(snap: &Snapshot, c: &Case) -> Game {
    let settings = GameSettings {
        kind: c.kind,
        situations: c.situations.map(|(low, mid, high)| {
            SituationCounts::new(low, mid, high).expect("dopuštena postavka slučaja")
        }),
    };
    let (mut g, me, guest) = (
        common::v2_game_with(snap, c.seed, settings),
        common::hotel_ids()[0],
        Actor::Guest(common::GUEST),
    );
    let mut n: usize = 0;
    while let Some(r) = g.current_round() {
        if c.human == Human::FreeAnswer {
            free_answers(&mut g, r);
        }
        if c.human == Human::Plans {
            own_plan(&mut g, r);
        }
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

/// Kanonski zapis: vrsta, autor i JSON svakog događaja (kako ide u bazu); iz `GameCreated` samo krugovi, hoteli i
/// broj situacija.
fn digest(g: &Game) -> String {
    let mut h = Sha256::new();
    for r in &g.history {
        let payload = match &r.event {
            GameEvent::GameCreated(c) => {
                json!({
                    "months": c.months,
                    "hotels": c.hotels,
                    "rounds": c.rounds,
                    "situation_counts": c.situation_counts,
                })
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
    // Svaka vrsta kruga ima i slučaj sa svojim brojem situacija (I3: otisak zna broj, ne samo vrstu kruga).
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        assert!(
            CASES
                .iter()
                .any(|c| c.kind == kind && c.situations.is_some()),
            "nema slučaja sa svojim brojem situacija za jednu vrstu kruga"
        );
    }
}

#[test] // 3a: pečat pokriva slobodni odgovor s presudom i vlastiti plan čovjeka (I3)
fn digest_cases_cover_free_answer_with_verdict_and_own_plan() {
    let snap = Snapshot::from_value(&frozen_value()).unwrap();
    let me = common::hotel_ids()[0];
    let count = |name: &str, f: &dyn Fn(&GameEvent) -> bool| {
        let c = CASES.iter().find(|c| c.name == name).unwrap();
        play(&snap, c)
            .history
            .iter()
            .filter(|r| f(&r.event))
            .count()
    };
    assert!(
        count(
            "t42-slobodni",
            &|e| matches!(e, GameEvent::VerdictRecorded(v) if v.hotel_id == me)
        ) > 0,
        "t42-slobodni: nijedna presuda na slobodni odgovor"
    );
    assert!(
        count(
            "m42-plan",
            &|e| matches!(e, GameEvent::PlanSet(p) if p.hotel_id == me)
        ) > 0,
        "m42-plan: nijedan plan čovjeka"
    );
}

#[test] // 3b: u svakom slučaju svaki krug ima izvučenu situaciju (šifra s točkom) i najviše `high` situacija
fn every_round_has_a_drawn_situation_and_at_most_high() {
    let snap = Snapshot::from_value(&frozen_value()).unwrap();
    let me = common::hotel_ids()[0];
    for c in &CASES {
        let g = play(&snap, c);
        let high = usize::from(g.situation_counts().levels().2);
        assert!(!g.rounds.is_empty(), "{}", c.name);
        for r in g.rounds.clone() {
            let codes: Vec<String> = g
                .situations_of_hotel(me, r)
                .unwrap()
                .iter()
                .map(|t| t.code.clone())
                .collect();
            assert!(
                codes.iter().any(|code| code.contains('.')),
                "{} {}: nijedna izvučena situacija u {codes:?}",
                c.name,
                r.code()
            );
            assert!(
                codes.len() <= high,
                "{} {}: {} situacija, najviše {high}: {codes:?}",
                c.name,
                r.code(),
                codes.len()
            );
        }
    }
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

#[test] // 5: otisak reagira na broj situacija iz postavki (isti slučaj, drugi broj)
fn digest_reacts_to_situation_counts() {
    let snap = Snapshot::from_value(&frozen_value()).unwrap();
    let case = |situations| Case {
        name: "m42-isti",
        seed: 42,
        kind: PeriodKind::Month,
        human: Human::Silent,
        situations,
    };
    let (own, default) = (case(Some((3, 3, 3))), case(None));
    assert_ne!(
        digest(&play(&snap, &own)),
        digest(&play(&snap, &default)),
        "3/3/3 i zadani broj moraju dati različit otisak"
    );
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
