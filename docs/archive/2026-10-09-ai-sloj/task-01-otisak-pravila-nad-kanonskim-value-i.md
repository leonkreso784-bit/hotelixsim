### Task 1: Otisak pravila nad kanonskim `Value` i provjera verzije jezgre

Popravak iz BACKLOG-a `[korak 4]` (ADR-015 t. 3): danas se otisak računa iz **tipiziranog** snimka pri svakom foldu,
pa svako novo polje content DTO-a čini sve stare igre `Corrupt`. Poslije ovog taska `GameCreated.snapshot` je sirovi
`serde_json::Value` kakav je spremljen, a otisak se računa nad njim. Stare igre (otisak iz koraka 3) i dalje prolaze
preko `legacy_fingerprint`. Uz to `core_version` se počinje provjeravati pri obračunu (BACKLOG M3).

**Files:**
- Modify: `crates/content/src/lib.rs` (otisak, `to_value`, `from_value`)
- Modify: `crates/game/src/event.rs` (`GameCreated.snapshot: Box<serde_json::Value>`)
- Modify: `crates/game/src/game.rs` (`Game::created`, polje `core_version`)
- Modify: `crates/game/src/command.rs` (`new_game`)
- Modify: `crates/game/src/resolve.rs` (`resolve_job` odbija drugu verziju jezgre)
- Modify: `crates/game/src/error.rs` (`GameError::CoreMismatch`), `crates/app/src/error.rs` (mapiranje → `Internal`)
- Modify: svi testovi koji grade `GameCreated` ručno (`crates/game/tests/common/mod.rs` i dr. — `grep -rn "snapshot: Box::new" crates`)
- Test: `crates/content/tests/snapshot.rs`, `crates/game/tests/events.rs`, `crates/game/tests/round.rs`

**Interfaces:**
- Produces: `content::fingerprint_value(&serde_json::Value) -> String` · `Snapshot::to_value(&self) -> serde_json::Value` ·
  `Snapshot::from_value(&serde_json::Value) -> Result<Snapshot, ContentError>` · `Snapshot::fingerprint(&self)` (sad nad
  `Value`) · `Snapshot::legacy_fingerprint(&self)` · `GameCreated.snapshot: Box<serde_json::Value>` · `Game.core_version: String` ·
  `GameError::CoreMismatch(String)`.

- [ ] **Step 1: Napiši testove koji padaju (content)** — dodaj u `crates/content/tests/snapshot.rs`:

```rust
use content::{Snapshot, fingerprint_value, repo_dir};
use serde_json::json;

#[test]
fn fingerprint_ignores_key_order() {
    // Brana: serde_json mora ostati bez značajke `preserve_order`, inače ovaj test pada.
    let a = json!({"b": 1, "a": {"y": 2, "x": 3}});
    let b = json!({"a": {"x": 3, "y": 2}, "b": 1});
    assert_eq!(fingerprint_value(&a), fingerprint_value(&b));
    assert!(fingerprint_value(&a).starts_with("sha256:"));
}

#[test]
fn fingerprint_is_over_value_and_survives_roundtrip() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let v = s.to_value();
    assert_eq!(s.fingerprint(), fingerprint_value(&v));
    let back = Snapshot::from_value(&v).unwrap();
    assert_eq!(back, s);
    assert_eq!(back.fingerprint(), s.fingerprint());
}

#[test]
fn legacy_fingerprint_differs_from_value_fingerprint() {
    // Stari otisak je nad tipiziranim zapisom (redoslijed polja iz deklaracije), novi nad poredanim ključevima.
    let s = Snapshot::read(&repo_dir()).unwrap();
    assert_ne!(s.legacy_fingerprint(), s.fingerprint());
}

#[test]
fn from_value_rejects_unknown_field() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let mut v = s.to_value();
    v.as_object_mut().unwrap().insert("nepoznato".into(), json!(1));
    assert!(Snapshot::from_value(&v).is_err());
}
```

- [ ] **Step 2: Pokreni i provjeri da pada** — `cargo test -p content --test snapshot; echo EXIT=$?` → ne prevodi se
  (`fingerprint_value`, `to_value`, `from_value`, `legacy_fingerprint` ne postoje).

- [ ] **Step 3: Implementiraj u `crates/content/src/lib.rs`** — zamijeni postojeći `fingerprint` i dodaj:

```rust
/// Otisak proizvoljnog JSON-a: SHA-256 kanonskog zapisa. `serde_json::Value` drži objekte u `BTreeMap` (bez značajke
/// `preserve_order`), pa su ključevi poredani i otisak ne ovisi o redoslijedu iz datoteke ni o JSONB-u.
pub fn fingerprint_value(v: &serde_json::Value) -> String {
    let bytes = serde_json::to_vec(v).expect("Value se uvijek serijalizira");
    format!("sha256:{:x}", Sha256::digest(&bytes))
}

impl Snapshot {
    // … postojeći `read` i `build` ostaju …

    pub fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("snimak nema mapu s ne-tekstualnim ključem pa se uvijek serijalizira")
    }

    /// Snimak iz spremljenog `GameCreated`. Nova polja DTO-a imaju `#[serde(default)]`, pa se stari snimci čitaju.
    pub fn from_value(v: &serde_json::Value) -> Result<Snapshot, ContentError> {
        serde_json::from_value(v.clone())
            .map_err(|e| ContentError::Json(PathBuf::from("<snimak iz dnevnika>"), e.to_string()))
    }

    /// Otisak pravila (`RulesetId`) nad kanonskim `Value` snimka (ADR-015 t. 3, ispravak iz koraka 4).
    pub fn fingerprint(&self) -> String {
        fingerprint_value(&self.to_value())
    }

    /// Otisak iz koraka 3: nad tipiziranim zapisom, redoslijedom polja iz deklaracije. Vrijedi samo za igre stvorene
    /// prije koraka 4; nova polja DTO-a zato imaju `skip_serializing_if`, da stari snimak da isti stari zapis.
    pub fn legacy_fingerprint(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("snimak se uvijek serijalizira");
        format!("sha256:{:x}", Sha256::digest(&bytes))
    }
}
```

- [ ] **Step 4: Pokreni** — `cargo test -p content; echo EXIT=$?` → PASS.

- [ ] **Step 5: Napiši testove koji padaju (game)** — u `crates/game/tests/events.rs` (pomoćnik `created_from` u
  `tests/common/mod.rs` neka gradi `GameCreated` iz `Snapshot` preko `new_game`; koristi postojeće pomoćnike):

```rust
#[test]
fn legacy_game_still_folds() {
    // Igra iz koraka 3: ruleset_id je stari otisak tipiziranog snimka.
    let snap = content::Snapshot::read(&content::repo_dir()).unwrap();
    let mut events = common::new_game_events(&snap); // Vec<Recorded> iz new_game, redni brojevi od 1
    let GameEvent::GameCreated(c) = &mut events[0].event else { panic!() };
    c.ruleset_id = snap.legacy_fingerprint();
    assert!(Game::fold(&events).is_ok());
}

#[test]
fn shuffled_jsonb_keys_still_fold() {
    // JSONB vraća ključeve drugim redom; otisak nad Value to ne smije primijetiti.
    let snap = content::Snapshot::read(&content::repo_dir()).unwrap();
    let events = common::new_game_events(&snap);
    let payload = events[0].event.to_payload();
    let text = serde_json::to_string(&payload).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&text).unwrap();
    let ev = GameEvent::from_record("GameCreated", 1, reparsed).unwrap();
    let mut again = events.clone();
    again[0].event = ev;
    assert!(Game::fold(&again).is_ok());
}

#[test]
fn tampered_snapshot_is_corrupt() {
    let snap = content::Snapshot::read(&content::repo_dir()).unwrap();
    let mut events = common::new_game_events(&snap);
    let GameEvent::GameCreated(c) = &mut events[0].event else { panic!() };
    c.snapshot["catalog"]["economy"]["hire_cents"] = serde_json::json!(1);
    assert!(matches!(Game::fold(&events), Err(GameError::Corrupt(_))));
}
```

  i u `crates/game/tests/round.rs`:

```rust
#[test]
fn resolve_refuses_other_core_version() {
    let mut g = common::closed_june_game(); // igra sa zatvorenim lipnjem (postojeći pomoćnici)
    g.core_version = "0.0.0-drugi".into();
    assert!(matches!(g.resolve_job(Month::Jun, false), Err(GameError::CoreMismatch(_))));
}
```

  Ako `new_game_events` / `closed_june_game` ne postoje u `tests/common/mod.rs`, dodaj ih (vraćaju `Vec<Recorded>` iz
  `game::command::new_game` sa sjemenom 42, odnosno `Game` nakon `RoundClosed` za lipanj).
  `resolve_job` u ovom tasku još ima stari potpis `resolve_job(m)`; drugi argument dolazi u Tasku 4 — u ovom tasku
  napiši test kao `g.resolve_job(Month::Jun)`, a Task 4 ga ažurira.

- [ ] **Step 6: Pokreni i provjeri da pada** — `cargo test -p game; echo EXIT=$?` → ne prevodi se.

- [ ] **Step 7: Implementiraj** —
  - `event.rs`: `pub snapshot: Box<serde_json::Value>` (komentar: „sirovi snimak kakav je spremljen; otisak se računa nad
    njim"). `serde_json::Value` ima `Eq`, pa izvodi ostaju.
  - `command.rs` `new_game`: `let value = snapshot.to_value();` → `ruleset_id: content::fingerprint_value(&value)`,
    `snapshot: Box::new(value)`.
  - `game.rs` `Game::created`:

```rust
let snap = content::Snapshot::from_value(&c.snapshot).map_err(|e| corrupt(format!("snimak: {e:?}")))?;
let ok = content::fingerprint_value(&c.snapshot) == c.ruleset_id || snap.legacy_fingerprint() == c.ruleset_id;
if !ok {
    return Err(corrupt("otisak pravila se ne slaže sa snimkom"));
}
let content = snap.build().map_err(|e| corrupt(format!("snimak: {e:?}")))?;
```

    i novo polje `pub core_version: String` u `Game` (iz `c.core_version`).
  - `error.rs`: `CoreMismatch(String)` s doc-komentarom „igra je stvorena drugom verzijom jezgre; obračun bi dao druge
    brojke (BACKLOG M3)". `app/src/error.rs`: `GameError::CoreMismatch(_) => C::Internal` (500, zabilježeno).
  - `resolve.rs` `resolve_job` na početku:

```rust
if self.core_version != sim_hotel::CORE_VERSION {
    return Err(GameError::CoreMismatch(format!(
        "igra je stvorena jezgrom {}, a radi {}",
        self.core_version,
        sim_hotel::CORE_VERSION
    )));
}
```

  - `server/src/main.rs` već logira `snapshot.fingerprint()` — ništa ne mijenjaj osim ako se ne prevodi.

- [ ] **Step 8: Pokreni sve provjere** (Global Constraints) — sve `EXIT=0`. Napomena: igre u lokalnoj bazi iz koraka 3
  moraju i dalje raditi; e2e test u `crates/api/tests/e2e.rs` to pokriva za nove igre, `legacy_game_still_folds` za stare.

- [ ] **Step 9: Ukloni stavku iz BACKLOG-a** — u `docs/records/BACKLOG.md` obriši dvije stavke `[korak 4]` (otisak
  nad kanonskim `Value`; `core_version` se ne provjerava). Stavka „Svako čitanje radi puni fold" ostaje.

- [ ] **Step 10: Commit** — „AI sloj 1/11: otisak pravila nad kanonskim Value i provjera verzije jezgre".

---

