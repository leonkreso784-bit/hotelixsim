### Task 1: Zlatni test v1 i zamrznuti dnevnik stare igre

**Veličina:** S–M, 2–3 h · **Ovisi o:** sloj 0 spojen · **Paralelno s:** Task 2 · **Model:** sonnet

**Cilj:** CI pada čim se ijedan v1 broj ili bajt v1 događaja pomakne. Zamrzava se (a) sezona v1 sa sjemenom 7 za sve
profile i strategije nad zamrznutim snimkom i (b) dnevnik jedne stvarne v1 igre (lipanj i srpanj: slobodan odgovor
s presudom, ponuđeni odgovori, razgovor s likom, promjena plana, mentor).

**Files:**
- Create: `crates/game/tests/v1_frozen.rs`, `crates/game/tests/fixtures/v1-snapshot.json`,
  `crates/game/tests/fixtures/v1-game-log.json`, `crates/game/tests/fixtures/v1-golden.json`
- Modify: `crates/game/Cargo.toml` (`[dev-dependencies] sha2.workspace = true`)
- Ne dirati: sve ostalo; posebno `crates/sim-hotel/**` i `crates/game/tests/common/mod.rs`.

**Interfaces:**
- Consumes: `content::{Snapshot, repo_dir, fingerprint_value}`, `game::*` (`Game::fold`, `resolve_round`,
  `resolve_job`, `GameEvent::{from_record, to_payload, kind}`, `Actor::{as_db, parse}`, `dto::{MonthReportDto,
  indices_dto}`), `sim_hotel::{run_season, RefTable, Profile, Strategy, CORE_VERSION}`.
- Produces: tri zamrznute datoteke i test `v1_frozen.rs`; oblik zapisa dnevnika `StoredEvent { seq, actor, kind,
  schema_ver, payload }` (isti kao redak tablice događaja). Ništa u `src`.

- [ ] **Step 1: Napiši test s generatorom** — `crates/game/tests/v1_frozen.rs`:

```rust
//! Zlatni test v1 (MODEL_V2 §6.2): zamrznuti snimak i dnevnik stare igre čitaju se i obračunavaju bajt-isto.
//! Fixturei se pišu JEDNOM (sloj 1, Task 1) i nikad se ne regeneriraju; ako test padne, kriv je novi kod.
mod common;

use std::path::PathBuf;

use content::Snapshot;
use game::dto::{MonthReportDto, indices_dto};
use game::event::AiSource;
use game::talk::ReplyBody;
use game::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sim_hotel::{Maintenance, Month, Profile, RefTable, Strategy, run_season};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[derive(Serialize, Deserialize)]
struct StoredEvent {
    seq: i64,
    actor: String,
    kind: String,
    schema_ver: i32,
    payload: Value,
}

#[derive(Serialize, Deserialize)]
struct Golden {
    season_seed7: String,
}

fn frozen_snapshot() -> Snapshot {
    let text = std::fs::read_to_string(fixtures().join("v1-snapshot.json")).unwrap();
    Snapshot::from_value(&serde_json::from_str(&text).unwrap()).unwrap()
}

fn stored_log() -> Vec<StoredEvent> {
    let text = std::fs::read_to_string(fixtures().join("v1-game-log.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn records(log: &[StoredEvent]) -> Vec<Recorded> {
    log.iter()
        .map(|e| Recorded {
            seq: e.seq,
            actor: Actor::parse(&e.actor).unwrap(),
            event: GameEvent::from_record(&e.kind, e.schema_ver, e.payload.clone()).unwrap(),
        })
        .collect()
}

/// Otisak cijele sezone: A/B/C × četiri strategije, sjeme 7, bez situacija i sa situacijama snimka.
fn season_digest(c: &content::Content) -> String {
    let mut h = Sha256::new();
    for p in Profile::ALL {
        let category = c.catalog.profile(p).unwrap().category;
        let refs = RefTable::compute(&c.catalog, category).unwrap();
        for s in Strategy::ALL {
            for situations in [&[][..], &c.situations[..]] {
                let r = run_season(p, s, 7, &c.catalog, situations, &refs).unwrap();
                h.update(serde_json::to_vec(&indices_dto(&r.start)).unwrap());
                for (rep, idx) in &r.months {
                    h.update(serde_json::to_vec(&MonthReportDto::from(rep)).unwrap());
                    h.update(serde_json::to_vec(&indices_dto(idx)).unwrap());
                }
                h.update(r.progress_pts.to_le_bytes());
            }
        }
    }
    format!("{:x}", h.finalize())
}

fn close_and_resolve(g: &mut Game, m: Month) {
    let ev = g.close_round(m).unwrap().unwrap();
    common::apply_ev(g, Actor::Guest(common::GUEST), ev);
    let ResolveStep::Done(evs) = g.resolve_job(m, false).unwrap() else {
        panic!("obračun je trebao biti gotov");
    };
    for (a, e) in evs {
        common::apply_ev(g, a, e);
    }
}

/// Referentna v1 igra: sve vrste događaja koje stara igra može imati.
fn play_reference_game() -> Game {
    let (mut g, h) = common::open_june_game();
    let me = Actor::Guest(common::GUEST);
    // Lipanj: S1 svojim riječima (presuda 4/5, najbliži odgovor 0), S3 ponuđeni odgovor 1.
    let ev = g.submit_response(h, "S1", "Ispričat ću se i ponuditi zamjenu sobe.").unwrap();
    common::apply_ev(&mut g, me.clone(), ev);
    let v = g.validate_verdict("S1", common::draft_with(4, 0)).unwrap();
    let ev = g.record_verdict(h, "S1", v, common::meta()).unwrap();
    common::apply_ev(&mut g, Actor::System, ev);
    let ev = g.choose_answer(h, "S3", 1).unwrap();
    common::apply_ev(&mut g, me.clone(), ev);
    close_and_resolve(&mut g, Month::Jun);
    let ev = g
        .record_debrief(h, Month::Jun, "Mentor: lipanj.".into(), AiSource::Fallback, None)
        .unwrap()
        .unwrap();
    common::apply_ev(&mut g, Actor::System, ev);
    // Srpanj: plaća 95 %, odgođeno održavanje; S2 razgovor (jedna poruka, odgovor rezervom) pa ponuđeni odgovor 1.
    let mut plan = g.effective_plan(h, Month::Jul).unwrap();
    plan.wage_pct = 95;
    plan.maintenance = Maintenance::Deferred;
    let ev = g.set_plan(h, Month::Jul, plan).unwrap();
    common::apply_ev(&mut g, me.clone(), ev);
    let ev = g.send_message(h, "S2", "Što vas muči u kuhinji?").unwrap();
    common::apply_ev(&mut g, me.clone(), ev);
    let reply = ReplyBody { text: "Premalo nas je.".into(), source: AiSource::Fallback, meta: None };
    let ev = g.record_reply(h, "S2", 1, reply).unwrap().unwrap();
    common::apply_ev(&mut g, Actor::System, ev);
    let ev = g.choose_answer(h, "S2", 1).unwrap();
    common::apply_ev(&mut g, me, ev);
    close_and_resolve(&mut g, Month::Jul);
    let ev = g
        .record_debrief(h, Month::Jul, "Mentor: srpanj.".into(), AiSource::Fallback, None)
        .unwrap()
        .unwrap();
    common::apply_ev(&mut g, Actor::System, ev);
    g
}

#[test]
#[ignore = "piše fixturee; pokreće se SAMO u Tasku 1 sloja 1"]
fn write_v1_fixtures() {
    assert_eq!(std::env::var("HOTELIXSIM_WRITE_V1_FIXTURES").as_deref(), Ok("1"));
    let snap = Snapshot::read(&content::repo_dir()).unwrap();
    let dir = fixtures();
    std::fs::create_dir_all(&dir).unwrap();
    let pretty = |v: &Value| format!("{}\n", serde_json::to_string_pretty(v).unwrap());
    std::fs::write(dir.join("v1-snapshot.json"), pretty(&snap.to_value())).unwrap();
    let g = play_reference_game();
    let log: Vec<StoredEvent> = g
        .history
        .iter()
        .map(|r| StoredEvent {
            seq: r.seq,
            actor: r.actor.as_db(),
            kind: r.event.kind().to_owned(),
            schema_ver: SCHEMA_VER,
            payload: r.event.to_payload(),
        })
        .collect();
    std::fs::write(dir.join("v1-game-log.json"), pretty(&serde_json::to_value(&log).unwrap())).unwrap();
    let golden = Golden { season_seed7: season_digest(&snap.build().unwrap()) };
    std::fs::write(dir.join("v1-golden.json"), pretty(&serde_json::to_value(&golden).unwrap())).unwrap();
}

#[test]
fn core_version_is_frozen() {
    assert_eq!(sim_hotel::CORE_VERSION, "0.1.0");
}

#[test]
fn v1_season_digest_is_frozen() {
    let text = std::fs::read_to_string(fixtures().join("v1-golden.json")).unwrap();
    let golden: Golden = serde_json::from_str(&text).unwrap();
    assert_eq!(season_digest(&frozen_snapshot().build().unwrap()), golden.season_seed7);
}

#[test]
fn old_v1_game_events_reserialize_byte_identically() {
    for e in stored_log() {
        let ev = GameEvent::from_record(&e.kind, e.schema_ver, e.payload.clone()).unwrap();
        assert_eq!(ev.kind(), e.kind);
        assert_eq!(
            serde_json::to_string(&ev.to_payload()).unwrap(),
            serde_json::to_string(&e.payload).unwrap(),
            "događaj {} ({})",
            e.seq,
            e.kind
        );
    }
}

#[test]
fn old_v1_game_folds_with_both_rounds_resolved() {
    let g = Game::fold(&records(&stored_log())).unwrap();
    assert_eq!(g.hotels.len(), 4);
    assert_eq!(g.status(Month::Jun).unwrap(), RoundStatus::Resolved);
    assert_eq!(g.status(Month::Jul).unwrap(), RoundStatus::Resolved);
    let GameEvent::GameCreated(c) = &g.history[0].event else { panic!("prvi je GameCreated") };
    assert_eq!(c.ruleset_id, content::fingerprint_value(&c.snapshot));
}

#[test]
fn old_v1_game_rounds_resolve_to_the_stored_numbers() {
    let recs = records(&stored_log());
    let mut checked = 0;
    for (i, r) in recs.iter().enumerate() {
        let GameEvent::RoundResolved(stored) = &r.event else { continue };
        let before = Game::fold(&recs[..i]).unwrap();
        let again = before.resolve_round(stored.month.into()).unwrap();
        assert_eq!(&again, stored, "obračun {:?}", stored.month);
        checked += 1;
    }
    assert_eq!(checked, 2);
}

#[test]
fn old_v1_game_resolve_job_replays_the_stored_system_events() {
    let recs = records(&stored_log());
    for (i, r) in recs.iter().enumerate() {
        let GameEvent::RoundClosed(c) = &r.event else { continue };
        let g = Game::fold(&recs[..=i]).unwrap();
        let ResolveStep::Done(evs) = g.resolve_job(c.month.into(), false).unwrap() else {
            panic!("presude su već u dnevniku, obračun ne čeka");
        };
        let stored: Vec<(Actor, GameEvent)> = recs[i + 1..i + 1 + evs.len()]
            .iter()
            .map(|r| (r.actor.clone(), r.event.clone()))
            .collect();
        assert_eq!(evs, stored, "sistemski događaji nakon zatvaranja {:?}", c.month);
    }
}
```

Napomena: imena (`common::GUEST`, `draft_with`, `meta`, `apply_ev`, `open_june_game`, `record_reply`, `record_debrief`,
`ReplyBody`) su iz postojećeg `crates/game/tests/common/mod.rs` i `crates/game/src/talk.rs`. Ako situacija S2 u
trenutnom sadržaju nema lika, zamijeni razgovor razgovorom na S1 **prije** slobodnog odgovora (poruka pa
`record_reply`), da dnevnik ipak ima `ChatMessageSent` i `CharacterReplied`.

- [ ] **Step 2: Pokreni testove — moraju pasti (nema fixtureâ)**

Run: `cd /c/Users/leonk/Documents/hotelixsim && cargo test -p game --test v1_frozen; echo EXIT=$?`
Expected: FAIL u `v1_season_digest_is_frozen` i `old_v1_*` („No such file"); `core_version_is_frozen` prolazi.

- [ ] **Step 3: Napiši fixturee (jedini put u cijelom sloju)**

Run: `cd /c/Users/leonk/Documents/hotelixsim && HOTELIXSIM_WRITE_V1_FIXTURES=1 cargo test -p game --test v1_frozen -- --ignored write_v1_fixtures; echo EXIT=$?`
Expected: `test result: ok. 1 passed`; tri datoteke u `crates/game/tests/fixtures/`. Otvori `v1-game-log.json` i
provjeri da sadrži barem: `GameCreated`, `PlanSet` (robot i igrač), `AnswerChosen`, `ResponseSubmitted`,
`VerdictRecorded`, `ChatMessageSent`, `CharacterReplied`, `RoundClosed`, `RoundResolved` (2), `DebriefRecorded` (2).

- [ ] **Step 4: Pokreni testove — prolaze**

Run: `cd /c/Users/leonk/Documents/hotelixsim && cargo test -p game --test v1_frozen; echo EXIT=$?`
Expected: `test result: ok. 6 passed; 0 failed; 1 ignored`.

- [ ] **Step 5: Dokaz da brana grize (bez commita)** — u `crates/sim-hotel/src/resolve.rs` privremeno promijeni
`exp_mix, 300,` u `exp_mix, 301,`, pokreni isti test (očekuj FAIL u `v1_season_digest_is_frozen` i
`old_v1_game_rounds_resolve_to_the_stored_numbers`), pa vrati promjenu **ručno istim alatom Edit** (ne git) i provjeri
`git diff --stat crates/sim-hotel` — prazno.

- [ ] **Step 6: Sve provjere** (Global Constraints, Rust dio) i **commit** „Sloj 1/1: zlatni test v1 i zamrznuti dnevnik
stare igre" s izričitim putanjama četiriju novih datoteka i `crates/game/Cargo.toml`, `Cargo.lock`.

**Prihvaćanje:** `cargo test -p game --test v1_frozen` → 6 passed; `git diff main --stat -- crates/sim-hotel` prazno.

---

