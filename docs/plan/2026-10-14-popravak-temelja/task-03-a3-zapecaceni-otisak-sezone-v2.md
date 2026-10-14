### Task A3: Zapečaćeni otisak sezone v2 vezan uz `CORE_VERSION`

**Veličina:** S, 2–3 h · **Ovisi o:** A1 (pečat se piše nad kodom spojenim u `sloj1`; kod čitan na b1bf446) ·
**Paralelno s:** A2 (druge datoteke; najviše 2 agenta koji prevode Rust) · **Model:** sonnet · **Prevodi Rust:** da

**Cilj:** Svaka promjena koda koja bi igri u tijeku promijenila brojke (obračun, izvlačenje situacija, primjerak predloška,
potezi robota, početno stanje) a ne digne verziju jezgre v2 ruši test u CI-ju. Dizanje verzije traži novi pečat — svjestan,
vidljiv potez u pregledu. Tako se pravila igre koja traje više ne mogu tiho promijeniti (danas se mogu, i već jesu).

**Zašto (nalaz revizije):** REVIZIJA_TEMELJA §4 #7; 01 nalaz 4. Verzija se diže ručno: `crates/sim-hotel-v2/src/lib.rs:50-52`
(`CORE_VERSION = env!("CARGO_PKG_VERSION")`), `crates/sim-hotel-v2/Cargo.toml:3` = `2.1.0` od f9c635c (O-1), a J-3, G-1 i L1-11
su poslije mijenjali obračun. v1 ima pečat (`crates/game/tests/v1_frozen.rs:193`, `fixtures_sealed.rs:9`), v2 nema.
Jedina čuvarica je `Model::check_version` (`crates/game/src/model.rs:140-149`), a ona vjeruje broju.

**Odluka: kanonski zapis i mjesto otisaka.**
- **Zapis = JSON događaja svježe v2 igre** (`GameEvent::to_payload`, `event.rs:352`), redom dnevnika, s vrstom i autorom.
  To je točno ono što ide u bazu i što stara igra čita: `RoundResolved` nosi plan, ishode situacija, izvješće, indekse,
  napredak i cijelo v2 stanje nakon kruga (`detail`, `event.rs:103-122`); `SituationsDrawn` nosi izvlačenje. Novo polje
  događaja ima `skip_serializing_if`, pa dodavanje polja ne mijenja otisak — mijenja ga samo promjena brojki. `sim-hotel-v2`
  nema serde, a `Debug` ispis se mijenja sa svakim novim poljem (lažni padovi), pa otisak nad jezgrom izravno nije bolji.
  Iz `GameCreated` ulaze samo `months`, `hotels`, `rounds` (snimak ima svoj otisak, a `core_version` je ključ pečata).
- **Igra, ne `run_season`:** stare igre računa petlja igre (`open_round` → `robot_moves` → `resolve_job`), ne petlja
  balansa (`sim-hotel-v2/src/season.rs:142`; razlika je #8, posebna kartica). Petlja igre pokriva i `content::instance`,
  `game::drawn` i prepuštanje voditelju — sve što određuje brojke igre u tijeku.
- **Zamrznuti snimak pravila**, ne trenutni sadržaj: izmjena sadržaja ne smije tražiti novu verziju jezgre (igra nosi svoj
  snimak). Snimak se piše jednom (generator), u **novu** mapu `crates/game/tests/fixtures_v2/` — mapu `tests/fixtures/`
  čuva `frozen_v1_fixtures_match_their_sealed_hash` (`fixtures_sealed.rs:47-56`), koji pada na svaku novu datoteku.
- **Otisci po verziji** u `fixtures_v2/v2-golden.json`: `{"snapshot": "<otisak snimka>", "digests": {"2.2.0": {"<slučaj>":
  "<sha256>", …}}}`. Provjerava se samo ključ jednak `CORE_VERSION`; stari ključevi ostaju kao povijest; generator nikad ne
  prepisuje postojeći ključ.
- **Balans v2 zlatni izlaz (03 nalaz 12) je DRUGI test**, ne ovaj: on mjeri trenutni sadržaj i robote na 1000 sjemena i
  legitimno se mijenja sa svakom kalibracijom sadržaja (bez nove verzije jezgre); da je u ovom pečatu, svaka izmjena sadržaja
  tražila bi lažno dizanje verzije ili bi se pečat navikao prepisivati. Preporuka: staviti ga u karticu „igra i balans daju
  isto" (#8), jer obje čuvaju petlju balansa.

**Files:**
- Create: `crates/game/tests/v2_sealed.rs`; generatorom `crates/game/tests/fixtures_v2/v2-snapshot.json` i `v2-golden.json`.
- Modify: `crates/sim-hotel-v2/Cargo.toml:3` (`2.1.0` → `2.2.0`, 🟨 dolje); `Cargo.lock` (samo redak verzije paketa
  `sim-hotel-v2`, koji cargo sam osvježi); `crates/sim-hotel-v2/src/lib.rs:50` (komentar: „raste sa svakom promjenom koja
  igri u tijeku mijenja brojke; čuva je pečat `crates/game/tests/v2_sealed.rs`"); `crates/game/tests/renewal_v2.rs:667` i
  `:693` (literal `"2.1.0"` → `sim_hotel_v2::CORE_VERSION`; r. 667 `assert_eq!` na literal se briše).
- Ne dirati: `crates/game/tests/fixtures/**`, `fixtures_sealed.rs`, `v1_frozen.rs`, `crates/sim-hotel/**`, sav `src/` osim
  komentara iz Modify, sadržaj, `docs/`. Zajednička pravila: `.superpowers/sdd/2026-10-10-v2-sloj1/global-constraints.md`
  i `implementer-common.md`.

**Interfaces:**
- Consumes: `common::{v2_game_from, hotel_ids, apply_ev, GUEST}` (`crates/game/tests/common/mod.rs:10-70, 227`),
  `Game::{current_round, close_round, resolve_job, situations_of_hotel, situation_mandatory, delegate_situation,
  choose_answer, history, hotels}`, `ResolveStep::Done`, `GameEvent::{kind, to_payload}`, `Actor::as_db`,
  `content::{Snapshot, fingerprint_value, repo_dir}`, `sim_hotel_v2::{CORE_VERSION, PeriodKind}`, `sha2` (dev-dep već postoji).
- Produces: samo test i dvije datoteke pečata (oblik gore). Nijedan javni potpis se ne mijenja.

**Testovi čuvara** — `crates/game/tests/v2_sealed.rs`. Danas crveni 1, 2 i 3 s porukom `nema tests/fixtures_v2/…: v2 još
nije zapečaćen` (datoteke ne postoje). Test 4 je zelen odmah: dokazuje da otisak reagira na pravila (bez njega bi pečat
mogao biti „prazan").
```rust
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
enum Human { Silent, Answers }
struct Case { name: &'static str, seed: u64, kind: PeriodKind, human: Human }
const CASES: [Case; 6] = [
    Case { name: "t7-odgovara", seed: 7, kind: PeriodKind::Week, human: Human::Answers },
    Case { name: "m7-odgovara", seed: 7, kind: PeriodKind::Month, human: Human::Answers },
    Case { name: "t42-sutnja", seed: 42, kind: PeriodKind::Week, human: Human::Silent },
    Case { name: "m42-sutnja", seed: 42, kind: PeriodKind::Month, human: Human::Silent },
    Case { name: "t2026-odgovara", seed: 2026, kind: PeriodKind::Week, human: Human::Answers },
    Case { name: "m2026-sutnja", seed: 2026, kind: PeriodKind::Month, human: Human::Silent },
];
#[derive(Serialize, Deserialize)]
struct Golden { snapshot: String, digests: BTreeMap<String, BTreeMap<String, String>> }

fn dir() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures_v2") }
fn read(name: &str) -> String {
    std::fs::read_to_string(dir().join(name)).unwrap_or_else(|_| panic!("nema tests/fixtures_v2/{name}: v2 još nije zapečaćen"))
}
fn golden() -> Golden { serde_json::from_str(&read("v2-golden.json")).unwrap() }
fn frozen_value() -> Value { serde_json::from_str(&read("v2-snapshot.json")).unwrap() }

/// Cijela sezona: roboti igraju sami; igrač šuti ili odgovara redom (n-ta situacija sezone dobiva odgovor n mod broj
/// odgovora), a svaku drugu situaciju koju smije prepušta voditelju.
fn play(snap: &Snapshot, c: &Case) -> Game {
    let (mut g, me, guest) = (common::v2_game_from(snap, c.seed, c.kind), common::hotel_ids()[0], Actor::Guest(common::GUEST));
    let mut n: usize = 0;
    while let Some(r) = g.current_round() {
        if c.human == Human::Answers {
            let list: Vec<(String, usize)> =
                g.situations_of_hotel(me, r).unwrap().iter().map(|t| (t.code.clone(), t.answers.len())).collect();
            for (code, len) in list {
                let ev = if n % 2 == 1 && !g.situation_mandatory(me, &code).unwrap() {
                    g.delegate_situation(me, &code).unwrap()
                } else {
                    g.choose_answer(me, &code, u8::try_from(n % len).unwrap()).unwrap()
                };
                common::apply_ev(&mut g, guest.clone(), ev);
                n += 1;
            }
        }
        let ev = g.close_round(r).unwrap().unwrap();
        common::apply_ev(&mut g, guest.clone(), ev);
        let ResolveStep::Done(evs) = g.resolve_job(r, false).unwrap() else { panic!("obračun čeka") };
        for (a, e) in evs { common::apply_ev(&mut g, a, e); }
    }
    g
}

/// Kanonski zapis: vrsta, autor i JSON svakog događaja (kako ide u bazu); iz `GameCreated` samo krugovi i hoteli.
fn digest(g: &Game) -> String {
    let mut h = Sha256::new();
    for r in &g.history {
        let payload = match &r.event {
            GameEvent::GameCreated(c) => json!({ "months": c.months, "hotels": c.hotels, "rounds": c.rounds }),
            e => e.to_payload(),
        };
        let (kind, actor, body) = (r.event.kind(), r.actor.as_db(), serde_json::to_vec(&payload).unwrap());
        for part in [kind.as_bytes(), actor.as_bytes(), &body[..]] { h.update(part); h.update(b"\n"); }
    }
    format!("{:x}", h.finalize())
}
fn digests(snap: &Snapshot) -> BTreeMap<String, String> {
    CASES.iter().map(|c| (c.name.to_string(), digest(&play(snap, c)))).collect()
}

#[test] // 1
fn v2_digest_is_sealed_for_core_version() {
    let g = golden();
    let want = g.digests.get(CORE_VERSION).unwrap_or_else(|| panic!(
        "jezgra v2 {CORE_VERSION} nema pečat: nova verzija traži novi otisak — \
         HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest"));
    let got = digests(&Snapshot::from_value(&frozen_value()).unwrap());
    assert_eq!(got.keys().collect::<Vec<_>>(), want.keys().collect::<Vec<_>>(), "slučajevi pečata");
    for (name, d) in &got {
        assert_eq!(d, &want[name], "{name}: obračun v2 se promijenio, a CORE_VERSION {CORE_VERSION} nije dignut — \
            digni verziju u crates/sim-hotel-v2/Cargo.toml i zapečati novi otisak");
    }
}

#[test] // 2
fn frozen_snapshot_matches_its_seal() {
    assert_eq!(fingerprint_value(&frozen_value()), golden().snapshot,
        "zamrznuti snimak se ne mijenja: nova pravila = nova verzija jezgre, ne novi snimak");
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
                GameEvent::RoundResolved(rr) => silent += rr.hotels.iter().filter(|h| h.hotel_id == me)
                    .flat_map(|h| &h.situations).filter(|s| s.answer.is_none()).count(),
                _ => {}
            }
        }
    }
    assert!(answered > 0 && delegated > 0 && silent > 0, "odgovori {answered}, prepušteno {delegated}, šutnja {silent}");
    assert!(CASES.iter().any(|c| c.kind == PeriodKind::Week) && CASES.iter().any(|c| c.kind == PeriodKind::Month));
}

#[test] // 4: otisak reagira na pravila (bez pečata; trenutni sadržaj)
fn digest_reacts_to_a_rule_change() {
    let v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let mut changed = v.clone();
    changed["v2"]["periods"]["week"]["situation_effect_bp"] = json!(326);
    let (a, b) = (Snapshot::from_value(&v).unwrap(), Snapshot::from_value(&changed).unwrap());
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
        std::fs::write(&path, pretty(&Snapshot::read_v2(&repo_dir()).unwrap().to_value())).unwrap();
    }
    let gpath = dir().join("v2-golden.json");
    let mut g = if gpath.exists() { golden() } else {
        Golden { snapshot: fingerprint_value(&frozen_value()), digests: BTreeMap::new() }
    };
    assert!(!g.digests.contains_key(CORE_VERSION), "pečat za {CORE_VERSION} postoji i ne prepisuje se — digni verziju");
    g.digests.insert(CORE_VERSION.to_string(), digests(&Snapshot::from_value(&frozen_value()).unwrap()));
    std::fs::write(&gpath, pretty(&serde_json::to_value(&g).unwrap())).unwrap();
}
```

**Koraci graditelja:**
1. `cargo test -p game --test v2_sealed` nad čuvarovim testom → 1–3 padaju s „nema tests/fixtures_v2/…", 4 prolazi (RED).
   Test se ne mijenja osim da se prevede. **Ne pečatiti 2.1.0** (Rizik).
2. `crates/sim-hotel-v2/Cargo.toml:3` → `version = "2.2.0"`; komentar `lib.rs:50`; `renewal_v2.rs:667` (makni
   `assert_eq!(sim_hotel_v2::CORE_VERSION, "2.1.0");`) i `:693` (`current: sim_hotel_v2::CORE_VERSION.into()`).
3. `HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest; echo EXIT=$?` → EXIT=0; nastaju
   `fixtures_v2/v2-snapshot.json` (~250 KB) i `v2-golden.json` s ključem `"2.2.0"` i 6 otisaka.
4. Dokaz da pečat čuva: (a) drugo pokretanje generatora pada („postoji i ne prepisuje se"); (b) privremeno u
   `crates/sim-hotel-v2/src/market.rs:23` promijeni `840` u `841` → test 1 pada s „obračun v2 se promijenio" → vrati
   (`git diff crates/sim-hotel-v2/src` prazan). Izlaze (a) i (b) u izvještaj. Commit: `Cargo.lock` samo redak verzije.

**Provjere (iz korijena stabla, svaka zasebno, `; echo EXIT=$?`):**
- `cargo test -p game --test v2_sealed` → `ok. 4 passed; 0 failed; 1 ignored`; `--test renewal_v2`, `--test v1_frozen`, `--test fixtures_sealed` → ok.
- `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `cargo test --workspace` ·
  `node scripts/check-core-deps.mjs` · `node scripts/probe-mutacija.mjs` (dira paket jezgre) · `corepack pnpm check:kod`.
- Balans bajt-isti prije/poslije (verzija ne ulazi u obračun): `cargo run -q -p balance --release -- --seeds 100 --check`,
  `-- --model v2 --round week --seeds 1000 --check` i `--round month` → izlaz u scratchpad prije koraka 2 i poslije; `diff`
  prazan, isti EXIT. `git diff --stat` dira samo datoteke iz Files.

**Rizik i odluke:**
- 🟨 **Dizanje na 2.2.0 u ovoj kartici.** 2.1.0 je postavljen kod O-1; J-3, G-1 i L1-11 su poslije promijenili brojke, pa
  „2.1.0" danas znači dvije različite igre; pečat na 2.1.0 naknadno bi ozakonio tihu promjenu. Posljedica: Leonove lokalne
  v2 igre dobiju 409 `game_outdated` (web još nema poruku — #6, Faza C); v2 nije na `main`, pa tuđih igara nema. Ako Leon
  kaže drukčije: preskoči korak 2 i pečatiraj 2.1.0.
- **Svaka buduća promjena brojki = nova verzija = stare v2 igre 409** dok Faza C (#6) ne postoji. Namjerno: bolje 409 nego
  tiho drukčija igra; Leon nakon takve promjene počinje novu igru.
- **Pečat je širi od jezgre** (pokriva i `content::instance`, `game::drawn`, robote u `game`): i promjena u tim paketima
  koja mijenja brojke traži dizanje `sim_hotel_v2::CORE_VERSION`, jer je to jedini broj koji `check_version` gleda. Graditelji
  budućih kartica to moraju znati — kontrolor neka doda redak u `global-constraints.md` popravka.
- **Provjeravatelj neka gleda:** da generator ne prepisuje postojeći ključ; da se `fixtures_v2/` ne preklapa s
  `fixtures/`; da test 3 stvarno broji prepuštanje (nije 0); da je `Cargo.lock` promijenjen samo u verziji.

**Leonov dio:** samo obavijest (ne odluka o igri): nakon ove kartice njegove postojeće lokalne v2 igre više se ne otvaraju
(409); otvara novu igru. Ako želi zadržati staru igru za pregled, javlja prije koraka 2 (tada 🟨 alternativa gore).

## Review Focus kandidati
1. **Konverzacijske situacije sa slobodnim odgovorom i presudom AI-ja** nisu u pečatu (igrač bira samo ponuđene odgovore,
   bez `record_verdict`); promjena puta presude → kvaliteta ne bi pala ovdje.
2. **Igra iz stvarnog starog dnevnika v2** (fold zapisanog loga, ne svježa igra) nije pokrivena; fold početno stanje
   računa trenutnim kodom (01 nalaz 3), pa bi pomak koji ne mijenja svježu igru, a mijenja čitanje stare, prošao.
3. **Promjena plana igrača** (cijene, sezonci, obnova) nije u slučajevima: igrač vozi prenesen zadani plan; putove plana
   pokrivaju samo roboti (tri strategije).
