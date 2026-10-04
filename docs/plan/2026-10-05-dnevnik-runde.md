# Dnevnik, runde i red poslova — plan izgradnje

**Status:** 🟩 AKTIVAN · **Otvoren:** 2026-10-05 · **Grana:** `gradnja/dnevnik-runde` · **Rok demoa:** ~14.10. ([ADR-012](../records/DECISIONS.md#adr-012))

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Igra (igrač + tri robota) živi kao dnevnik događaja u Postgresu: gost preko poveznice dobije hotel, mijenja
plan mjeseca i bira ponuđene odgovore dok je runda otvorena, „Zatvori mjesec" upiše zatvaranje i posao u red, a radnik
obračuna sve hotele jezgrom — za lipanj i srpanj, s ljestvicom i kronološkim feedom hotela preko API-ja (bez ekrana).

**Architecture:** Novi čisti crate `game` (iznad jezgre, bez baze i mreže) drži događaje, serde DTO-e, projekciju
(fold), pravila runde, poteze robota i obračun runde iz **snimka pravila** zapisanog u `GameCreated`. `app` dobiva
portove (`EventLog`, `Guests`, `JobQueue`, `Ids`) i slučajeve upotrebe s pravom pristupa po ulozi; svaka naredba je
jedna transakcija „zaključaj igru → učitaj tok → odluči (čista funkcija) → upiši". `store` izvodi portove u Postgresu
(okidač nepromjenjivosti, tablice `guests/games/hotels/jobs`, `FOR UPDATE SKIP LOCKED` sa zakupom), `api` samo zove
`app`, a `server` učitava sadržaj pri startu i vrti radnika kao tokio zadatak.

**Tech Stack:** Rust 1.98.1 (edition 2024) · Axum 0.8 + utoipa 6 · sqlx 0.9 (Postgres, `uuid`, `json`, offline `.sqlx/`) ·
`uuid` 1 · `sha2` 0.10 · serde/serde_json · Postgres 16 (lokalno 5433) · Node 24 skripte brana · openapi-typescript klijent.

**Spec:** [DEMO.md](../product/DEMO.md) §1 (demo način, gost, roboti, 15 minuta), §6.1, §7 (raspored), §10 ·
[ARHITEKTURA.md](../architecture/ARHITEKTURA.md) §6.2, §7.2, §8.1–8.4, §9.1–9.4, §11, §12 · pretpostavke
[ADR-015](../records/DECISIONS.md#adr-015) · stavke `[dnevnik]` u [BACKLOG.md](../records/BACKLOG.md) ·
oblik plana po uzoru na [plan jezgre](../archive/2026-10-04-jezgra.md) · tok rada [AGENTI.md](../workflow/AGENTI.md).

**Leonove proizvodne odluke (05.10.)** — vrijede kao spec: (1) gost preko poveznice sad samo kao pozadina
(`POST /v1/demo`, kolačić `httpOnly`, nova igra s tri robota); limit 3 igre po IP-u i brisanje nakon 7 dana idu u
drugi val. (2) „Zatvori mjesec" s neodgovorenom situacijom je dopušten; neodgovorena dobiva ocjenu 1, a API kaže koliko
ih je neodgovoreno. (3) Mjesec smije zatvoriti svatko tko je u igri. (4) „Kao BitLife": „Zatvori mjesec" je „Age +";
plan se mijenja bilo kad dok je runda otvorena (svaka promjena je događaj, zadnja vrijedi, nedirnuto = zadano iz
DEMO §6.1); API daje kronološki feed hotela; svaka promjena nosi uzrok. Opseg: samo lipanj (S1, S3) i srpanj (S2);
samo ponuđeni odgovori (AI, slobodan tekst, likovi i mentor su korak 4 — **ne izmišljaj** `VerdictRecorded`).

## Global Constraints

- **Jezgra** (`engine`, `sim-hotel`) i dalje ima **nula** vanjskih ovisnosti i sve brane ADR-009; jedine promjene jezgre
  su u Tasku 1 (polje `unanswered`, `SituationOutcome::unanswered`, `CORE_VERSION`). Jezgra nema serde.
- **`game`** smije ovisiti samo o `engine`, `sim-hotel`, `content`, `serde`, `serde_json`, `uuid` (bez značajke `v4`).
  Nikad `tokio`, `sqlx*`, `axum`, `hyper`, `reqwest`, `mio`, `socket2` — čuva `scripts/check-core-deps.mjs` (Task 2).
  Bez `f32`/`f64`, bez sata, bez `HashMap`; slučajnost samo `engine::Rng::derive(seed, …)`.
- **Obračun koristi snimak iz `GameCreated`, nikad trenutne datoteke** (`content::Snapshot`); otisak pravila
  `ruleset_id = "sha256:<hex>"` kanonskog JSON-a snimka.
- Događaji: `GameCreated`, `PlanSet`, `AnswerChosen`, `RoundClosed`, `RoundResolved`; `schema_ver = 1`; stupac `type`
  nosi ime, `payload` samo tijelo. Tok: `game:<uuid>`. `actor`: `guest:<uuid>` · `robot:<kod strategije>` · `system`.
- Jedinice kao u jezgri: novac `i64` `_cents` · bodovi `_pts` (stotinke, 0..=10 000) · `_bp` · indeksi `u16` stotinke ·
  napredak `i32` stotinke boda (`progress_hundredths`, 10 000 = +100).
- **Vidljivost (ARHITEKTURA §12):** tuđi hotel = samo ukupna ocjena i pet indeksa; vlasnik vidi stvarne brojke, plan,
  situacije i feed. Kvaliteta i mehanike ponuđenih odgovora **nikad** ne izlaze kroz API.
- Greške API-ja: `application/problem+json`, `code` je **enum** u OpenAPI-ju (`ProblemCode`); nikad prazan odgovor.
- SQL: novi `sqlx::query!` → `cargo sqlx prepare --workspace` i commit `.sqlx/`; u **testovima** samo funkcije
  `sqlx::query(...)`/`query_scalar(...)` (ne makroi), da offline cache ne ovisi o testovima. Testovi baze: `#[sqlx::test]`
  (baza mora raditi: `bash scripts/baza.sh start`).
- Hrvatski u komentarima, porukama i naslovima grešaka; nazivi u kodu engleski.
- Commit poruke na hrvatskom, `git commit -F <datoteka u scratchpadu>`, izričite putanje, oblik „Dnevnik N/12: …";
  **push samo uz Leonov OK**.
- Provjere prije commita svakog taska (svaka zasebno, s `; echo EXIT=$?`, bez `| tail`):
  `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `node scripts/check-core-deps.mjs` ·
  `cargo test --workspace` · `cargo run -q -p balance --release -- --seeds 100 --check`. Taskovi koji mijenjaju API
  dodaju `corepack pnpm gen:api`, `corepack pnpm typecheck` i `corepack pnpm test`.

## Review Focus

1. **Dvostruki klik ili dvije kartice na „Zatvori mjesec"**: točno jedan `RoundClosed`, jedan posao i jedan
   `RoundResolved`; drugi klik vraća trenutno stanje (Task 6 `double_close_is_idempotent`, Task 7
   `decide_enqueues_job_once_per_key`, Task 10 `close_twice_is_202_then_200`).
2. **Zastario ekran**: plan ili odgovor za mjesec koji je upravo zatvoren mora dati 409 `round_not_open`, nikad tihi upis
   u srpanj (Task 3 `stale_commands_are_rejected`, Task 6 `plan_for_closed_month_rejected`).
3. **Radnik padne usred posla ili isti posao stigne dvaput**: istekao zakup vraća posao u red, a ponovni obračun ne
   upisuje drugi `RoundResolved` (Task 4 `resolve_job_twice_is_noop`, Task 6 `crashed_worker_job_is_reclaimed_and_duplicate_job_is_noop`, Task 8
   `expired_lease_is_reclaimed`).
4. **Tuđi hotel, tuđa igra, bez kolačića ili s pokvarenim kolačićem**: nikad privatne brojke; 401/403 kao problem+json
   (Task 5 `visibility_by_role`, Task 9 `foreign_and_anonymous_requests`).
5. **Sadržaj se promijeni nakon stvaranja igre ili JSONB promijeni redoslijed ključeva**: igra računa po svom snimku,
   ponovni obračun je bajt-isti nakon tipiziranog čitanja (Task 1 `fingerprint_ignores_key_order`, Task 4
   `resolution_uses_snapshot_not_files`, Task 12 `replay_from_database_matches`).

Dodatno (ne-5, ali pokriveno): neispravno JSON tijelo ili nepoznata vrijednost enuma u planu → 422 `invalid_body`
(Task 10 `bad_plan_body_is_422_problem`).

---

## Pregled datoteka

```text
Cargo.toml                                   + member game; deps uuid, sha2; sqlx +uuid +json; tokio +time
crates/sim-hotel/src/mechanic.rs             SituationTemplate.unanswered, SituationOutcome::unanswered
crates/sim-hotel/src/lib.rs                  CORE_VERSION
crates/content/src/lib.rs                    DTO-i Serialize; Snapshot (read/build/fingerprint); polje unanswered
content/hotel/situations.json                + "unanswered" za S1–S4 (nacrt, Leonova provjera)
crates/game/                                 NOVO  ids, actor, dto, event, game (fold), command, resolve, feed, view
scripts/check-core-deps.mjs                  + pravilo čistoće za `game`
crates/app/src/{lib,ports,error,demo,views,commands,jobs,memory}.rs   portovi, slučajevi upotrebe, lažna pohrana
crates/store/migrations/0002_events_append_only.sql   NOVO  indeks, okidač, CHECK na actor
crates/store/migrations/0003_platform.sql    NOVO  guests, games, hotels, jobs
crates/store/src/{lib,log,guests,jobs}.rs    izvedbe portova
crates/api/src/{lib,problem,auth,schema,health,demo,hotels,games}.rs  rute, problem kodovi, kolačić
crates/server/src/main.rs                    Deps, sadržaj pri startu, radnik
scripts/check-clean.mjs                      NOVO  check:api pada i na nepraćenu generiranu datoteku
openapi.json, packages/api-client/src/schema.d.ts   GENERIRANO
.sqlx/                                       GENERIRANO
```

**Paralelizacija:** taskovi su redom ovisni o sučeljima. Jedini sigurni par za paralelne izvođače je **Task 6 ‖ Task 7**
(nakon Taska 5; Task 6 piše samo `crates/app/src/{commands,jobs}.rs` i `crates/app/tests/rounds.rs`, Task 7 samo
`crates/store/**` i `.sqlx/`). Sve ostalo ide redom.

---

### Task 1: Snimak pravila u `content` i ishod neodgovorene situacije

**Files:**
- Modify: `crates/sim-hotel/src/mechanic.rs` (`SituationTemplate`, `SituationOutcome`)
- Modify: `crates/sim-hotel/src/lib.rs` (`CORE_VERSION`)
- Modify: `crates/sim-hotel/tests/robots.rs:133` (literal `SituationTemplate` dobiva `unanswered: vec![]`)
- Test: `crates/sim-hotel/tests/mechanics.rs` (novi test na kraju)
- Modify: `crates/content/src/lib.rs`, `crates/content/Cargo.toml`, `Cargo.toml` (workspace `sha2`)
- Modify: `content/hotel/situations.json`
- Test: `crates/content/tests/snapshot.rs` (NOVO), `crates/content/tests/load.rs`

**Interfaces:**
- Produces: `sim_hotel::SituationTemplate { code, month, title, answers, unanswered: Vec<Mechanic> }`;
  `sim_hotel::SituationOutcome::unanswered(t: &SituationTemplate) -> SituationOutcome` (kvaliteta 1);
  `sim_hotel::CORE_VERSION: &str`;
  `content::Snapshot` (`Clone, Debug, PartialEq, Eq, Serialize, Deserialize`) s
  `Snapshot::read(dir: &Path) -> Result<Snapshot, ContentError>`, `Snapshot::build(&self) -> Result<Content, ContentError>`,
  `Snapshot::fingerprint(&self) -> String` (`"sha256:" + 64 hex`); `content::Content` dobiva `PartialEq, Eq`.

- [ ] **Step 1: Test jezgre (crveni).** Na kraj `crates/sim-hotel/tests/mechanics.rs`:

```rust
#[test]
fn unanswered_is_quality_one_with_template_mechanics() {
    let cat = test_catalog();
    let t = SituationTemplate {
        code: "S1".into(),
        month: Month::Jun,
        title: "Klima".into(),
        answers: vec![],
        unanswered: vec![
            Mechanic::ServiceRecovery { compensation_cents: 0 },
            Mechanic::EquipmentRepair { urgent: false },
        ],
    };
    let o = SituationOutcome::unanswered(&t);
    assert_eq!(o.code, "S1");
    assert_eq!(o.quality, 1);
    assert_eq!(o.mechanics, t.unanswered);
    let e = effects(&o, Month::Jun, &cat, Category::Four).unwrap();
    // DEMO §6.2: zadovoljstvo Δ = 2 × (q − 3) bodova = −4,00 → −400 stotinki
    assert_eq!(e.satisfaction_pts, cat.mechanics.recovery_pts_per_q * -2);
    assert_eq!(e.satisfaction_pts, -400);
}
```

- [ ] **Step 2: Pokreni** `cargo test -p sim-hotel --test mechanics unanswered; echo EXIT=$?` → ne prevodi se
  (`no field unanswered`, `no function unanswered`), `EXIT=101`.

- [ ] **Step 3: Jezgra.** U `mechanic.rs` struktura i metoda:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationTemplate {
    pub code: String,
    pub month: Month,
    pub title: String,
    pub answers: Vec<OfferedAnswer>,
    /// Što se dogodi kad do zatvaranja runde nitko ne odgovori (DEMO §1.2): ove mehanike uz kvalitetu 1.
    /// Ignoriranje je odluka; bez mehanika kvaliteta 1 ne bi imala učinka pa bi šutnja bila bolja od lošeg odgovora.
    pub unanswered: Vec<Mechanic>,
}

impl SituationOutcome {
    /// Ishod neodgovorene situacije: mehanike `unanswered` iz predloška i kvaliteta 1 (najgori ishod rubrike).
    pub fn unanswered(t: &SituationTemplate) -> SituationOutcome {
        SituationOutcome {
            code: t.code.clone(),
            mechanics: t.unanswered.clone(),
            quality: 1,
        }
    }
}
```
  U `lib.rs` iza `pub use state::…`: `/// Verzija jezgre koja ulazi u GameCreated (ARHITEKTURA §7.2).`
  `pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");` (makro `env!` čita u prevođenju, nije `std::env::var`).
  U `tests/robots.rs` literalu dodaj `unanswered: vec![],`. **Ne diraj** retke `pub mod index;` u `lib.rs` — na njima
  sjede mutacijske probe (`scripts/probe-mutacija.mjs` traži točno jedan pogodak).

- [ ] **Step 4: Pokreni** `cargo test -p sim-hotel; echo EXIT=$?` → prolazi, `EXIT=0`.

- [ ] **Step 5: Testovi snimka (crveni).** `crates/content/tests/snapshot.rs`:

```rust
use content::{Snapshot, repo_dir};
use serde_json::json;

fn snap() -> Snapshot {
    Snapshot::read(&repo_dir()).unwrap()
}

#[test]
fn snapshot_builds_same_content_as_load() {
    assert_eq!(snap().build().unwrap(), content::load(&repo_dir()).unwrap());
}

#[test]
fn snapshot_survives_json_roundtrip() {
    let s = snap();
    let back: Snapshot = serde_json::from_value(serde_json::to_value(&s).unwrap()).unwrap();
    assert_eq!(back, s);
    assert_eq!(back.fingerprint(), s.fingerprint());
}

#[test]
fn fingerprint_has_fixed_shape_and_reacts_to_change() {
    let s = snap();
    let f = s.fingerprint();
    assert!(f.starts_with("sha256:"), "{f}");
    assert_eq!(f.len(), 7 + 64);
    let mut v = serde_json::to_value(&s).unwrap();
    v["catalog"]["economy"]["hire_cents"] = json!(150_001);
    let changed: Snapshot = serde_json::from_value(v).unwrap();
    assert_ne!(changed.fingerprint(), f);
}

#[test]
fn fingerprint_ignores_key_order() {
    // JSONB u Postgresu mijenja redoslijed ključeva; otisak se računa iz tipiziranog snimka, ne iz teksta.
    let s = snap();
    let text = serde_json::to_string(&s).unwrap();
    let reordered: serde_json::Value = serde_json::from_str(&text).unwrap(); // Value sortira ključeve
    let back: Snapshot = serde_json::from_value(reordered).unwrap();
    assert_eq!(back.fingerprint(), s.fingerprint());
}

#[test]
fn unknown_field_in_snapshot_is_rejected() {
    let mut v = serde_json::to_value(snap()).unwrap();
    v["catalog"]["economy"]["typo"] = json!(1);
    assert!(serde_json::from_value::<Snapshot>(v).is_err());
}
```
  U `tests/load.rs`, testu `repo_content_loads_and_validates`, dodaj:
  `assert!(c.situations.iter().all(|s| !s.unanswered.is_empty()));`

- [ ] **Step 6: Pokreni** `cargo test -p content; echo EXIT=$?` → ne prevodi se (`Snapshot` ne postoji), `EXIT=101`.

- [ ] **Step 7: `content` — DTO-i i snimak.** `Cargo.toml` (workspace) `[workspace.dependencies]`: `sha2 = "0.10"`;
  `crates/content/Cargo.toml`: `sha2.workspace = true`. U `lib.rs`:
  - `use serde::{Deserialize, Serialize};` i `use sha2::{Digest, Sha256};`
  - **svaki** DTO (`MonthsDto<T>`, `CategoryDto`, `EconomyDto`, `LevelDto`, `TrainingDto`, `ActionsDto`, `MechanicsDto`,
    `CatalogDto`, `ProfileDto`, `MechanicDto`, `AnswerDto`, `SituationDto`) dobiva
    `#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]` (postojeći `serde` atributi ostaju).
  - `SituationDto` dobiva polje `unanswered: Vec<MechanicDto>` (obavezno, bez `default`).
  - `#[derive(Debug, Clone, PartialEq, Eq)] pub struct Content` (dodaj `PartialEq, Eq`).
  - U `build_situations`, prije `out.push`: prazan `unanswered` → `invalid(format!("{ctx}: nema ishoda neodgovorene situacije"))`;
    inače `let unanswered = s.unanswered.into_iter().map(|m| build_mechanic(m, &ctx)).collect::<Result<Vec<_>, _>>()?;`
    i polje `unanswered` u `SituationTemplate`.
  - Snimak (zamjenjuje tijelo `load`):

```rust
/// Puni snimak pravila (ARHITEKTURA §7.2): stablo DTO-a iz `content/hotel/*.json`. Ulazi cijeli u `GameCreated`,
/// a obračun igre gradi katalog iz njega, nikad iz trenutnih datoteka.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    catalog: CatalogDto,
    profiles: BTreeMap<String, ProfileDto>,
    situations: Vec<SituationDto>,
}

impl Snapshot {
    pub fn read(dir: &Path) -> Result<Snapshot, ContentError> {
        Ok(Snapshot {
            catalog: read(dir, "catalog.json")?,
            profiles: read(dir, "profiles.json")?,
            situations: read(dir, "situations.json")?,
        })
    }

    /// Pretvori snimak u tipove jezgre i provjeri katalog.
    pub fn build(&self) -> Result<Content, ContentError> {
        let s = self.clone();
        let catalog = build_catalog(s.catalog, s.profiles)?;
        let situations = build_situations(s.situations)?;
        catalog
            .validate()
            .map_err(|e| ContentError::Invalid(format!("katalog: {e:?}")))?;
        Ok(Content { catalog, situations })
    }

    /// Otisak pravila (`RulesetId`): SHA-256 kanonskog JSON-a tipiziranog snimka. Mape su `BTreeMap`, polja struktura
    /// imaju stalan redoslijed, pa je izlaz isti na svakom računalu i ne ovisi o redoslijedu ključeva u izvoru.
    pub fn fingerprint(&self) -> String {
        let bytes = serde_json::to_vec(self).expect("snimak se uvijek serijalizira");
        format!("sha256:{:x}", Sha256::digest(&bytes))
    }
}

/// Učita `catalog.json`, `profiles.json` i `situations.json` iz `dir` i provjeri ih.
pub fn load(dir: &Path) -> Result<Content, ContentError> {
    Snapshot::read(dir)?.build()
}
```

- [ ] **Step 8: Podaci.** U `content/hotel/situations.json`, u svaki objekt situacije iza `"title"` umetni:
  - S1: `"unanswered": [{ "type": "ServiceRecovery", "compensation_cents": 0 }, { "type": "EquipmentRepair", "urgent": false }],`
  - S2: `"unanswered": [{ "type": "StaffRetention", "raise_one_pct": 0, "bonus_cents": 0 }],`
  - S3: `"unanswered": [{ "type": "GroupBooking", "accept": false, "rooms": 25, "nights": 7, "discount_bp": 2500, "month": "aug" }],`
  - S4: `"unanswered": [{ "type": "CrisisCommunication", "backup_cents": 0 }],`

  (Ignorirati gošću = isprika bez naknade i privremen popravak; ignorirati kuhara = odbijanje; ignorirati agenciju =
  ponuda propada; ignorirati najavu suše = bez pripreme. Nacrt čeka Leonovu provjeru — ADR-015 t. 2.)

- [ ] **Step 9: Pokreni** `cargo test -p content; echo EXIT=$?` → `EXIT=0`; zatim `cargo test --workspace; echo EXIT=$?`
  → `EXIT=0` (balans i sezona ne koriste `unanswered`: roboti uvijek odgovaraju);
  `cargo run -q -p balance --release -- --seeds 100 --check; echo EXIT=$?` → `EXIT=0`;
  `node scripts/probe-mutacija.mjs; echo EXIT=$?` → `proboja: 0`, `EXIT=0`.

- [ ] **Step 10: Provjere i commit.** fmt, clippy, `check-core-deps` (Global Constraints). Commit:
  `Cargo.toml Cargo.lock crates/sim-hotel/src/mechanic.rs crates/sim-hotel/src/lib.rs crates/sim-hotel/tests/robots.rs
  crates/sim-hotel/tests/mechanics.rs crates/content/Cargo.toml crates/content/src/lib.rs crates/content/tests/snapshot.rs
  crates/content/tests/load.rs content/hotel/situations.json`, poruka
  „Dnevnik 1/12: snimak pravila u content i ishod neodgovorene situacije (kvaliteta 1)".

---

### Task 2: Crate `game` — identifikatori, actor, događaji i DTO-i

**Files:**
- Create: `crates/game/Cargo.toml`, `crates/game/src/{lib,ids,actor,dto,event,error}.rs`
- Modify: `Cargo.toml` (member `crates/game`; workspace deps `game = { path = "crates/game" }`,
  `uuid = { version = "1", features = ["serde"] }`)
- Modify: `scripts/check-core-deps.mjs` (pravilo čistoće za `game`)
- Test: `crates/game/tests/events.rs`

**Interfaces:**
- Consumes: `content::Snapshot`, `sim_hotel::{HotelState, MonthPlan, MonthReport, Month, Strategy, Category, IndexSet, Index}`.
- Produces (sve `pub`, re-export iz `lib.rs`):
  - `GameId(pub Uuid)`, `HotelId(pub Uuid)`, `GuestId(pub Uuid)` — `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
    Serialize, Deserialize`, `#[serde(transparent)]`; `GameId::stream(self) -> String` (`"game:<uuid>"`).
  - `enum Actor { Guest(GuestId), Robot(Strategy), System }` s `as_db(&self) -> String` i `parse(&str) -> Option<Actor>`;
    `strategy_from_code(&str) -> Option<Strategy>`.
  - `dto`: `MonthCode`, `TrainingCode`, `MaintenanceCode`, `IndexCode`, `INDEX_ORDER: [IndexCode; 5]`, `PlanDto`,
    `HotelStateDto`, `PendingDto`, `GroupBlockDto`, `MarketDto`, `PeopleDto`, `MoneyDto`, `GuestDto`, `VarCode`,
    `CauseDto`, `UnitCode`, `LineDto`, `MonthReportDto`, `indices_dto(&IndexSet) -> [u16; 5]`.
  - `event`: `SCHEMA_VER: i32 = 1`, `GameCreated`, `HotelSetup`, `Controller`, `PlanSet`, `AnswerChosen`, `RoundClosed`,
    `RoundResolved`, `HotelResolution`, `SituationResult`, `enum GameEvent`, `GameEvent::{kind, to_payload, from_record}`,
    `EventError`, `struct Recorded { seq: i64, actor: Actor, event: GameEvent }`.
  - `error`: `enum GameError` (cijeli popis ovdje, koriste ga Task 3–4).

- [ ] **Step 1: Crate i ovisnosti.** `crates/game/Cargo.toml`:

```toml
[package]
name = "game"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
engine.workspace = true
sim-hotel.workspace = true
content.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true

[lints]
workspace = true
```
  `lib.rs`:

```rust
//! Igra kao dnevnik događaja (ARHITEKTURA §9.1, ADR-015): događaji, projekcija (fold), runde, roboti i obračun runde.
//! Čisto: bez baze, mreže, sata i vlastite slučajnosti — sve što treba dolazi iz dnevnika i snimka pravila.

pub mod actor;
pub mod dto;
pub mod error;
pub mod event;
pub mod ids;

pub use actor::{Actor, strategy_from_code};
pub use error::GameError;
pub use event::*;
pub use ids::{GameId, GuestId, HotelId};
```

- [ ] **Step 2: Testovi (crveni).** `crates/game/tests/events.rs`:

```rust
use content::{Snapshot, repo_dir};
use game::dto::*;
use game::*;
use serde_json::json;
use sim_hotel::{HotelState, Month, MonthPlan, Profile, Strategy, resolve_month};
use uuid::Uuid;

fn ids() -> (GameId, HotelId, GuestId) {
    (GameId(Uuid::from_u128(1)), HotelId(Uuid::from_u128(2)), GuestId(Uuid::from_u128(3)))
}

fn plan() -> PlanDto {
    PlanDto {
        price_cents: 14_500,
        marketing_cents: 500_000,
        seasonal_target: 18,
        wage_pct: 103,
        training: TrainingCode::Basic,
        maintenance: MaintenanceCode::Regular,
        renovation: false,
        reply_reviews: true,
    }
}

fn june_report() -> (HotelState, sim_hotel::MonthReport) {
    let c = Snapshot::read(&repo_dir()).unwrap().build().unwrap();
    let s = HotelState::from_profile(c.catalog.profile(Profile::C).unwrap()).unwrap();
    let p = MonthPlan::default_for(&s, Month::Jun, &c.catalog).unwrap();
    let r = resolve_month(&s, Month::Jun, &p, &[], &c.catalog).unwrap();
    (s, r)
}

fn samples() -> Vec<GameEvent> {
    let (g, h, guest) = ids();
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let (_, r) = june_report();
    vec![
        GameEvent::GameCreated(GameCreated {
            game_id: g,
            seed: u64::MAX,
            ruleset_id: snap.fingerprint(),
            core_version: sim_hotel::CORE_VERSION.into(),
            snapshot: snap,
            months: vec![MonthCode::Jun, MonthCode::Jul],
            hotels: vec![
                HotelSetup { hotel_id: h, slot: 1, name: "Tvoj hotel".into(), profile: "C".into(),
                             controller: Controller::Human { guest_id: guest } },
                HotelSetup { hotel_id: HotelId(Uuid::from_u128(4)), slot: 2, name: "Štediša".into(),
                             profile: "C".into(), controller: Controller::Robot { strategy: "stedisa".into() } },
            ],
        }),
        GameEvent::PlanSet(PlanSet { hotel_id: h, month: MonthCode::Jun, plan: plan() }),
        GameEvent::AnswerChosen(AnswerChosen { hotel_id: h, month: MonthCode::Jun, situation: "S1".into(), answer: 2 }),
        GameEvent::RoundClosed(RoundClosed { month: MonthCode::Jun }),
        GameEvent::RoundResolved(RoundResolved {
            month: MonthCode::Jun,
            hotels: vec![HotelResolution {
                hotel_id: h,
                plan: plan(),
                situations: vec![
                    SituationResult { code: "S1".into(), answer: Some(2), quality: 1 },
                    SituationResult { code: "S3".into(), answer: None, quality: 1 },
                ],
                report: MonthReportDto::from(&r),
                indices: [5000, 5100, 4900, 5000, 5200],
                progress_hundredths: -37,
            }],
        }),
    ]
}

#[test]
fn every_event_roundtrips_through_record() {
    for e in samples() {
        let back = GameEvent::from_record(e.kind(), SCHEMA_VER, e.to_payload()).unwrap();
        assert_eq!(back, e, "{}", e.kind());
    }
}

#[test]
fn record_kinds_are_stable_names() {
    let kinds: Vec<&str> = samples().iter().map(GameEvent::kind).collect();
    assert_eq!(kinds, ["GameCreated", "PlanSet", "AnswerChosen", "RoundClosed", "RoundResolved"]);
}

#[test]
fn bad_records_are_rejected() {
    let e = &samples()[1];
    assert!(matches!(GameEvent::from_record("Nepoznat", 1, e.to_payload()), Err(EventError::UnknownKind(_))));
    assert!(matches!(GameEvent::from_record("PlanSet", 2, e.to_payload()), Err(EventError::UnsupportedVersion { .. })));
    let mut p = e.to_payload();
    p["visak"] = json!(1);
    assert!(matches!(GameEvent::from_record("PlanSet", 1, p), Err(EventError::Payload { .. })));
}

#[test]
fn actor_roundtrip_and_rejects() {
    let (_, _, guest) = ids();
    let mut all = vec![Actor::Guest(guest), Actor::System];
    all.extend(Strategy::ALL.iter().map(|s| Actor::Robot(*s)));
    for a in all {
        assert_eq!(Actor::parse(&a.as_db()), Some(a.clone()), "{}", a.as_db());
    }
    assert_eq!(Actor::Guest(guest).as_db(), "guest:00000000-0000-0000-0000-000000000003");
    assert_eq!(Actor::Robot(Strategy::Saver).as_db(), "robot:stedisa");
    for bad in ["", "guest:xyz", "robot:nepoznat", "System", "guest:"] {
        assert_eq!(Actor::parse(bad), None, "{bad}");
    }
}

#[test]
fn state_and_plan_dtos_roundtrip_to_core() {
    let (s, r) = june_report();
    assert_eq!(HotelState::try_from(&HotelStateDto::from(&s)).unwrap(), s);
    assert_eq!(HotelState::try_from(&HotelStateDto::from(&r.after)).unwrap(), r.after);
    let p = MonthPlan::from(&plan());
    assert_eq!(PlanDto::from(&p), plan());
}

#[test]
fn report_dto_survives_json() {
    let (_, r) = june_report();
    let d = MonthReportDto::from(&r);
    let back: MonthReportDto = serde_json::from_value(serde_json::to_value(&d).unwrap()).unwrap();
    assert_eq!(back, d);
    assert_eq!(serde_json::to_value(CauseDto::Situation("S1".into())).unwrap(), json!({"situation": "S1"}));
    assert_eq!(serde_json::to_value(CauseDto::Price).unwrap(), json!("price"));
}
```

- [ ] **Step 3: Pokreni** `cargo test -p game; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 4: `ids.rs`, `actor.rs`, `error.rs`.**

```rust
// ids.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GameId(pub Uuid);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HotelId(pub Uuid);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GuestId(pub Uuid);

impl GameId {
    /// Jedan tok po igri: ukupan redoslijed svih događaja igre (ADR-015 t. 4).
    pub fn stream(self) -> String {
        format!("game:{}", self.0)
    }
}
```

```rust
// actor.rs
use sim_hotel::Strategy;
use uuid::Uuid;

use crate::ids::GuestId;

/// Tko je izazvao događaj; u bazi kao tekst `guest:<uuid>` · `robot:<kod>` · `system` (CHECK u migraciji 0002).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Actor {
    Guest(GuestId),
    Robot(Strategy),
    System,
}

pub fn strategy_from_code(code: &str) -> Option<Strategy> {
    Strategy::ALL.into_iter().find(|s| s.code() == code)
}

impl Actor {
    pub fn as_db(&self) -> String {
        match self {
            Actor::Guest(g) => format!("guest:{}", g.0),
            Actor::Robot(s) => format!("robot:{}", s.code()),
            Actor::System => "system".to_owned(),
        }
    }

    pub fn parse(s: &str) -> Option<Actor> {
        if s == "system" {
            return Some(Actor::System);
        }
        if let Some(id) = s.strip_prefix("guest:") {
            return Uuid::parse_str(id).ok().map(|u| Actor::Guest(GuestId(u)));
        }
        s.strip_prefix("robot:").and_then(strategy_from_code).map(Actor::Robot)
    }
}
```

```rust
// error.rs
use sim_hotel::PlanError;

/// Greške pravila igre. `Corrupt` znači da dnevnik ili snimak krše pravilo koje je provjereno pri upisu — to je
/// greška sustava (500), nikad korisnika.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameError {
    UnknownHotel,
    UnknownMonth,
    UnknownSituation,
    UnknownAnswer,
    RoundNotOpen,
    AlreadyAnswered,
    Plan(PlanError),
    Corrupt(String),
}

pub(crate) fn corrupt(msg: impl Into<String>) -> GameError {
    GameError::Corrupt(msg.into())
}
```

- [ ] **Step 5: `dto.rs`.** Svaki DTO: `#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]` i
  `#[serde(deny_unknown_fields)]` (strukture) ili `#[serde(rename_all = "snake_case")]` (enumi; `MonthCode` i
  `IndexCode` kako je navedeno). Polja **točno** ovim imenima i tipovima (zrcalo jezgre, polje za polje):

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MonthCode { Jun, Jul, Aug, Sep }
// From<Month> for MonthCode i From<MonthCode> for Month (iscrpni match, bez `_`).

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingCode { None, Basic, Full }          // ↔ sim_hotel::Training
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceCode { Deferred, Regular, Intensive } // ↔ sim_hotel::Maintenance

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexCode { G, R, Z, O, F }
/// Redoslijed `Index::ALL`; nizovi `[u16; 5]` u događajima slijede ga.
pub const INDEX_ORDER: [IndexCode; 5] = [IndexCode::G, IndexCode::R, IndexCode::Z, IndexCode::O, IndexCode::F];
pub fn indices_dto(s: &IndexSet) -> [u16; 5] {
    Index::ALL.map(|i| s.get(i).hundredths())
}

pub struct PlanDto { pub price_cents: i64, pub marketing_cents: i64, pub seasonal_target: u16, pub wage_pct: u16,
                     pub training: TrainingCode, pub maintenance: MaintenanceCode, pub renovation: bool,
                     pub reply_reviews: bool }
// From<&MonthPlan> for PlanDto, From<&PlanDto> for MonthPlan.

pub struct PendingDto { pub experience_pts: i32, pub morale_pts: i32, pub renovation: bool }
pub struct GroupBlockDto { pub month: MonthCode, pub room_nights: u32, pub price_cents: i64 }
pub struct HotelStateDto {
    pub category: String, pub rooms: u16, pub permanent: u16, pub seasonal: u16, pub wage_pct: u16,
    pub extra_wage_cents: i64, pub morale_pts: i32, pub experience_pts: i32, pub equipment_pts: i32,
    pub satisfaction_pts: i32, pub rating: u16, pub cash_cents: i64, pub fixed_cents: i64,
    pub load_hundredths: u16, pub turnover_bp: u16, pub renovated: bool, pub pending: PendingDto,
    pub groups: Vec<GroupBlockDto>,
}
// From<&HotelState> for HotelStateDto (category = Category::code()).
// TryFrom<&HotelStateDto> for HotelState, Error = GameError: nepoznata kategorija → corrupt("kategorija <x>").

pub struct MarketDto { pub rooms_out: u16, pub available: u32, pub f_price_bp: i32, pub f_reputation_bp: i32,
    pub f_marketing_bp: i32, pub base_demand: u32, pub demand: u32, pub staff_capacity: u32,
    pub group_room_nights: u32, pub sold: u32, pub group_sold: u32, pub individual_sold: u32,
    pub group_revenue_cents: i64 }
pub struct PeopleDto { pub hires: u16, pub released: u16, pub staff: u16, pub load_hundredths: u16,
    pub load_penalty_pts: i32, pub morale_target_pts: i32, pub turnover_bp: u16, pub departures: u16,
    pub experience_mix_pts: i32 }
pub struct MoneyDto { pub rooms_revenue_cents: i64, pub other_revenue_cents: i64, pub labor_cents: i64,
    pub costs_cents: i64, pub gop_cents: i64, pub capex_cents: i64, pub interest_cents: i64, pub goppar_cents: i64 }
pub struct GuestDto { pub quality_pts: i32, pub value_pts: i32, pub satisfaction_target_pts: i32, pub rating_target: u16 }
// From<&MarketOutcome>, From<&PeopleOutcome>, From<&MoneyOutcome>, From<&GuestOutcome> — samo u smjeru jezgra → DTO.

#[serde(rename_all = "snake_case")]
pub enum VarCode { Sold, Revenue, Costs, Cash, Staff, Morale, Experience, Equipment, Satisfaction, Rating }
#[serde(rename_all = "snake_case")]
pub enum CauseDto { Baseline, Price, Reputation, Marketing, Groups, Capacity, OtherRevenue, Wage, Load, Training,
    Hires, Release, Turnover, Maintenance, Renovation, ReplyReviews, FixedCosts, Interest, Situation(String) }
#[serde(rename_all = "snake_case")]
pub enum UnitCode { RoomNights, Cents, People, Points, RatingHundredths, Bp }
pub struct LineDto { pub var: VarCode, pub cause: CauseDto, pub amount: i64, pub unit: UnitCode }
// From<&Var>, From<&Cause>, From<&Unit>, From<&Line> — iscrpni matchevi bez `_`, da nova varijanta u jezgri
// ne prevede tiho.

pub struct MonthReportDto { pub month: MonthCode, pub before: HotelStateDto, pub after: HotelStateDto,
    pub market: MarketDto, pub people: PeopleDto, pub money: MoneyDto, pub guests: GuestDto,
    pub lines: Vec<LineDto> }
// From<&MonthReport> for MonthReportDto.
```

- [ ] **Step 6: `event.rs`.**

```rust
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::actor::Actor;
use crate::dto::{MonthCode, MonthReportDto, PlanDto};
use crate::ids::{GameId, GuestId, HotelId};

/// Verzija oblika svih događaja; promjena oblika = nova verzija i „podizanje" starih pri čitanju (ARHITEKTURA §9.1).
pub const SCHEMA_VER: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameCreated {
    pub game_id: GameId,
    pub seed: u64,
    /// `snapshot.fingerprint()`; fold ga provjerava.
    pub ruleset_id: String,
    pub core_version: String,
    pub snapshot: content::Snapshot,
    pub months: Vec<MonthCode>,
    pub hotels: Vec<HotelSetup>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotelSetup {
    pub hotel_id: HotelId,
    /// 1..=4; određuje redoslijed obračuna i tokove kocke (nikad uuid ni vrijeme dolaska, ARHITEKTURA §7.2).
    pub slot: u8,
    pub name: String,
    /// "A" | "B" | "C"
    pub profile: String,
    pub controller: Controller,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Controller {
    Human { guest_id: GuestId },
    /// `strategy` = `Strategy::code()`
    Robot { strategy: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanSet { pub hotel_id: HotelId, pub month: MonthCode, pub plan: PlanDto }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnswerChosen { pub hotel_id: HotelId, pub month: MonthCode, pub situation: String, pub answer: u8 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundClosed { pub month: MonthCode }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundResolved { pub month: MonthCode, pub hotels: Vec<HotelResolution> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotelResolution {
    pub hotel_id: HotelId,
    /// Plan koji je stvarno obračunat (zadnji `PlanSet` ili zadani iz DEMO §6.1).
    pub plan: PlanDto,
    pub situations: Vec<SituationResult>,
    pub report: MonthReportDto,
    /// Pet indeksa nakon mjeseca, redoslijedom `INDEX_ORDER`.
    pub indices: [u16; 5],
    pub progress_hundredths: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SituationResult {
    pub code: String,
    /// `None` = nitko nije odgovorio do zatvaranja (kvaliteta 1, DEMO §1.2).
    pub answer: Option<u8>,
    pub quality: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameEvent {
    GameCreated(GameCreated),
    PlanSet(PlanSet),
    AnswerChosen(AnswerChosen),
    RoundClosed(RoundClosed),
    RoundResolved(RoundResolved),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventError {
    UnknownKind(String),
    UnsupportedVersion { kind: String, version: i32 },
    Payload { kind: String, message: String },
}

/// Zapisan događaj: redni broj u toku igre i tko ga je izazvao.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorded {
    pub seq: i64,
    pub actor: Actor,
    pub event: GameEvent,
}

fn body<T: DeserializeOwned>(kind: &str, v: serde_json::Value) -> Result<T, EventError> {
    serde_json::from_value(v).map_err(|e| EventError::Payload { kind: kind.to_owned(), message: e.to_string() })
}

impl GameEvent {
    pub fn kind(&self) -> &'static str {
        match self {
            GameEvent::GameCreated(_) => "GameCreated",
            GameEvent::PlanSet(_) => "PlanSet",
            GameEvent::AnswerChosen(_) => "AnswerChosen",
            GameEvent::RoundClosed(_) => "RoundClosed",
            GameEvent::RoundResolved(_) => "RoundResolved",
        }
    }

    pub fn to_payload(&self) -> serde_json::Value {
        let v = match self {
            GameEvent::GameCreated(e) => serde_json::to_value(e),
            GameEvent::PlanSet(e) => serde_json::to_value(e),
            GameEvent::AnswerChosen(e) => serde_json::to_value(e),
            GameEvent::RoundClosed(e) => serde_json::to_value(e),
            GameEvent::RoundResolved(e) => serde_json::to_value(e),
        };
        v.expect("događaj nema mapu s ne-tekstualnim ključem pa se uvijek serijalizira")
    }

    pub fn from_record(kind: &str, schema_ver: i32, payload: serde_json::Value) -> Result<GameEvent, EventError> {
        if schema_ver != SCHEMA_VER {
            return Err(EventError::UnsupportedVersion { kind: kind.to_owned(), version: schema_ver });
        }
        Ok(match kind {
            "GameCreated" => GameEvent::GameCreated(body(kind, payload)?),
            "PlanSet" => GameEvent::PlanSet(body(kind, payload)?),
            "AnswerChosen" => GameEvent::AnswerChosen(body(kind, payload)?),
            "RoundClosed" => GameEvent::RoundClosed(body(kind, payload)?),
            "RoundResolved" => GameEvent::RoundResolved(body(kind, payload)?),
            other => return Err(EventError::UnknownKind(other.to_owned())),
        })
    }
}
```
  (`GameId`, `GuestId` su u `use` zbog polja; `use crate::actor::Actor` zbog `Recorded`.)

- [ ] **Step 7: Pokreni** `cargo test -p game; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 8: Brana čistoće u `check-core-deps.mjs`.** Izdvoji poziv `cargo tree` u funkciju `tree(pkg)` koja vraća
  `Set` imena (s istom obradom grešaka: izlaz 2). Postojeća petlja nad `CORE` je koristi. Dodaj iza nje:

```js
// `game` je čist sloj iznad jezgre (ADR-015): smije serde i uuid, ali nikad bazu, mrežu ni async izvođač.
const PURE = {
  game: ["tokio", "sqlx", "sqlx-core", "sqlx-postgres", "axum", "hyper", "reqwest", "mio", "socket2"],
};
for (const [pkg, banned] of Object.entries(PURE)) {
  const names = tree(pkg);
  for (const b of banned) {
    if (names.has(b)) {
      console.error(`NIJE DOPUŠTENO: ${pkg} ovisi o ${b}`);
      found += 1;
    }
  }
  console.log(`${pkg}: provjereno ${names.size} paketa (čist sloj)`);
}
```
  Ručna proba (ne ide u CI): privremeno dodaj `tokio.workspace = true` u `crates/game/Cargo.toml`,
  `node scripts/check-core-deps.mjs; echo EXIT=$?` → `NIJE DOPUŠTENO: game ovisi o tokio`, `EXIT=1`; vrati datoteku
  (`git diff --stat -- crates/game/Cargo.toml` prazno nakon vraćanja u odnosu na tvoju verziju).

- [ ] **Step 9: Pokreni** `node scripts/check-core-deps.mjs; echo EXIT=$?` → `game: provjereno … (čist sloj)`,
  `prekršaja: 0`, `EXIT=0`; zatim provjere iz Global Constraints.

- [ ] **Step 10: Commit** — `Cargo.toml Cargo.lock crates/game scripts/check-core-deps.mjs`, poruka
  „Dnevnik 2/12: crate game — događaji, DTO-i, actor i brana čistoće".

---

### Task 3: `game` — projekcija (fold), stanje runde, nova igra, potezi robota i naredbe

**Files:**
- Create: `crates/game/src/game.rs` (Game, HotelRecord, RoundStatus, fold/apply, upiti)
- Create: `crates/game/src/command.rs` (`new_game`, `robot_moves`, `set_plan`, `choose_answer`, `close_round`, `plan_limits`)
- Modify: `crates/game/src/lib.rs` (`pub mod game; pub mod command;` i re-exporti)
- Test: `crates/game/tests/round.rs`

**Interfaces:**
- Consumes: Task 2 (`GameEvent`, `Recorded`, `Actor`, DTO-i, `GameError`), `content::Snapshot::build`,
  `sim_hotel::{HotelState::from_profile, RefTable::compute, start_indices, MonthPlan::{default_for, validate},
  Strategy::{plan, choose}, Profile, Category}`, `engine::{Rng, label}`.
- Produces:
  - `pub enum RoundStatus { Scheduled, Open, Closing, Resolved }` (`Clone, Copy, Debug, PartialEq, Eq`).
  - `pub struct HotelRecord { pub setup: HotelSetup, pub profile: Profile, pub start_state: HotelState,
    pub start_indices: IndexSet, pub plans: BTreeMap<Month, MonthPlan>, pub answers: BTreeMap<(Month, String), u8> }`.
  - `pub struct Game { pub id: GameId, pub seed: u64, pub months: Vec<Month>, pub content: content::Content,
    pub hotels: Vec<HotelRecord>, pub closed: BTreeMap<Month, Actor>, pub resolved: BTreeMap<Month, RoundResolved>,
    pub last_seq: i64, pub history: Vec<Recorded>, refs: BTreeMap<Category, RefTable> }` (`Clone, Debug`).
  - `Game::fold(&[Recorded]) -> Result<Game, GameError>`, `Game::apply(&mut self, Recorded) -> Result<(), GameError>`,
    `hotel(HotelId) -> Result<&HotelRecord, GameError>`, `current_month() -> Option<Month>`,
    `status(Month) -> Result<RoundStatus, GameError>`, `state_at(HotelId, Month) -> Result<HotelState, GameError>`,
    `latest_state(HotelId) -> Result<HotelState, GameError>`, `effective_plan(HotelId, Month) -> Result<MonthPlan, GameError>`,
    `situations_in(Month) -> Vec<&SituationTemplate>`, `unanswered_count(HotelId, Month) -> Result<u8, GameError>`,
    `refs(Category) -> Result<&RefTable, GameError>`.
  - `command::new_game(game_id: GameId, seed: u64, snapshot: &Snapshot, guest: GuestId, hotel_ids: [HotelId; 4])
    -> Result<Vec<(Actor, GameEvent)>, GameError>`;
    `Game::robot_moves(&self, Month) -> Result<Vec<(Actor, GameEvent)>, GameError>`;
    `Game::set_plan(&self, HotelId, Month, MonthPlan) -> Result<GameEvent, GameError>`;
    `Game::choose_answer(&self, HotelId, &str, u8) -> Result<GameEvent, GameError>`;
    `Game::close_round(&self, Month) -> Result<Option<GameEvent>, GameError>` (`None` = već zatvoreno ili obračunato);
    `pub struct PlanLimits { pub price_min_cents: i64, pub price_max_cents: i64, pub marketing_max_cents: i64,
    pub seasonal_min: u16, pub seasonal_max: u16, pub wage_min_pct: u16, pub wage_max_pct: u16,
    pub renovation_available: bool }` i `plan_limits(&HotelState, Month, &Catalog) -> Result<PlanLimits, GameError>`.
  - Pravila nove igre (ADR-015 t. 13, Leon 05.10.: **različiti profili**): igrač dobiva
    `profile_at(Rng::derive(seed, &[label("game:profile")]).range(0, 3))`; roboti (slotovi 2–4) dobivaju
    `profile_at(rot + slot)` uz `rot = Rng::derive(seed, &[label("game:robot-profiles")]).range(0, 3)`, pa su među
    robotima **sva tri profila** A, B, C (`profile_at(n)`: `n % 3` → 0 A, 1 B, 2 C); slotovi: 1 čovjek „Tvoj hotel", 2 robot `stedisa` „Štediša", 3 robot `brizni`
    „Brižni", 4 robot `lovac` „Lovac na popunjenost"; mjeseci `[Jun, Jul]`.
  - Kocka robota: plan `Rng::derive(seed, &[label("game:robot-plan"), mjesec.position(), slot])`, odgovor
    `Rng::derive(seed, &[label("game:robot-answer"), mjesec.position(), slot, label(šifra)])` (sve `u64`).

- [ ] **Step 1: Testovi (crveni).** `crates/game/tests/round.rs`:

```rust
use content::{Snapshot, repo_dir};
use game::command::*;
use game::dto::*;
use game::game::*;
use game::*;
use sim_hotel::{Month, MonthPlan, PlanError};
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

fn me() -> HotelId {
    hotel_ids()[0]
}

#[test]
fn new_game_writes_created_then_robot_moves_for_june() {
    let log = created(42);
    let kinds: Vec<(&str, String)> = log.iter().map(|r| (r.event.kind(), r.actor.as_db())).collect();
    assert_eq!(kinds[0], ("GameCreated", "system".into()));
    assert_eq!(log.len(), 1 + 3 * 3, "svaki robot: PlanSet + odgovor na S1 i S3");
    for (i, robot) in ["robot:stedisa", "robot:brizni", "robot:lovac"].iter().enumerate() {
        let k: Vec<&str> = log[1 + i * 3..4 + i * 3].iter().map(|r| r.event.kind()).collect();
        assert_eq!(k, ["PlanSet", "AnswerChosen", "AnswerChosen"], "{robot}");
        assert!(log[1 + i * 3..4 + i * 3].iter().all(|r| r.actor.as_db() == *robot));
    }
    let GameEvent::GameCreated(c) = &log[0].event else { panic!() };
    assert_eq!(c.ruleset_id, c.snapshot.fingerprint());
    assert_eq!(c.months, [MonthCode::Jun, MonthCode::Jul]);
    let mut robot_profiles: Vec<&str> = c.hotels[1..].iter().map(|h| h.profile.as_str()).collect();
    robot_profiles.sort_unstable();
    assert_eq!(robot_profiles, ["A", "B", "C"], "roboti pokrivaju sva tri profila");
    assert!(["A", "B", "C"].contains(&c.hotels[0].profile.as_str()));
    assert_eq!(c.hotels[0].controller, Controller::Human { guest_id: GUEST });
}

#[test]
fn statuses_follow_the_calendar() {
    let g = Game::fold(&created(42)).unwrap();
    assert_eq!(g.current_month(), Some(Month::Jun));
    assert_eq!(g.status(Month::Jun), Ok(RoundStatus::Open));
    assert_eq!(g.status(Month::Jul), Ok(RoundStatus::Scheduled));
    assert_eq!(g.status(Month::Aug), Err(GameError::UnknownMonth));
    assert_eq!(g.unanswered_count(me(), Month::Jun), Ok(2));
    assert_eq!(g.unanswered_count(hotel_ids()[1], Month::Jun), Ok(0));
}

#[test]
fn untouched_plan_is_default_and_last_plan_wins() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    let s = g.state_at(me(), Month::Jun).unwrap();
    let default = MonthPlan::default_for(&s, Month::Jun, &g.content.catalog).unwrap();
    assert_eq!(g.effective_plan(me(), Month::Jun).unwrap(), default);
    let mut p1 = default.clone();
    p1.marketing_cents = 500_000;
    let ev = g.set_plan(me(), Month::Jun, p1.clone()).unwrap();
    push(&mut log, Actor::Guest(GUEST), ev);
    let mut p2 = default.clone();
    p2.wage_pct = 105;
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.set_plan(me(), Month::Jun, p2.clone()).unwrap());
    assert_eq!(Game::fold(&log).unwrap().effective_plan(me(), Month::Jun).unwrap(), p2);
}

#[test]
fn invalid_plan_is_rejected_with_plan_error() {
    let g = Game::fold(&created(42)).unwrap();
    let mut p = g.effective_plan(me(), Month::Jun).unwrap();
    p.price_cents += 50;
    assert_eq!(g.set_plan(me(), Month::Jun, p), Err(GameError::Plan(PlanError::PriceNotWholeEuro)));
}

#[test]
fn answers_once_per_situation() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.choose_answer(me(), "S1", 0).unwrap());
    let g = Game::fold(&log).unwrap();
    assert_eq!(g.choose_answer(me(), "S1", 1), Err(GameError::AlreadyAnswered));
    assert_eq!(g.choose_answer(me(), "S3", 9), Err(GameError::UnknownAnswer));
    assert_eq!(g.choose_answer(me(), "S2", 0), Err(GameError::RoundNotOpen), "S2 je srpanj");
    assert_eq!(g.choose_answer(me(), "S4", 0), Err(GameError::UnknownSituation), "kolovoz nije u igri");
    assert_eq!(g.choose_answer(me(), "S9", 0), Err(GameError::UnknownSituation));
    assert_eq!(g.unanswered_count(me(), Month::Jun), Ok(1));
}

#[test]
fn stale_commands_are_rejected() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    let ev = g.close_round(Month::Jun).unwrap().expect("otvorena runda se zatvara");
    push(&mut log, Actor::Guest(GUEST), ev);
    let g = Game::fold(&log).unwrap();
    assert_eq!(g.status(Month::Jun), Ok(RoundStatus::Closing));
    assert_eq!(g.close_round(Month::Jun), Ok(None), "drugi klik ne piše ništa");
    let p = g.effective_plan(me(), Month::Jun).unwrap();
    assert_eq!(g.set_plan(me(), Month::Jun, p), Err(GameError::RoundNotOpen));
    assert_eq!(g.choose_answer(me(), "S1", 0), Err(GameError::RoundNotOpen));
    assert_eq!(g.close_round(Month::Jul), Err(GameError::RoundNotOpen), "srpanj još nije otvoren");
}

#[test]
fn fold_rejects_broken_logs() {
    let mut log = created(42);
    log[3].seq = 99;
    assert!(matches!(Game::fold(&log), Err(GameError::Corrupt(_))));
    let mut log = created(42);
    let GameEvent::GameCreated(c) = &mut log[0].event else { panic!() };
    let mut v = serde_json::to_value(&c.snapshot).unwrap();
    v["catalog"]["economy"]["hire_cents"] = serde_json::json!(1);
    c.snapshot = serde_json::from_value(v).unwrap(); // ruleset_id ostaje stari
    match Game::fold(&log) {
        Err(GameError::Corrupt(m)) => assert!(m.contains("otisak"), "{m}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn plan_limits_are_exactly_the_validation_bounds() {
    let g = Game::fold(&created(42)).unwrap();
    let s = g.state_at(me(), Month::Jun).unwrap();
    let cat = &g.content.catalog;
    let l = plan_limits(&s, Month::Jun, cat).unwrap();
    let base = MonthPlan::default_for(&s, Month::Jun, cat).unwrap();
    let with = |f: &dyn Fn(&mut MonthPlan)| {
        let mut p = base.clone();
        f(&mut p);
        p.validate(&s, Month::Jun, cat)
    };
    assert_eq!(with(&|p| p.price_cents = l.price_min_cents), Ok(()));
    assert_eq!(with(&|p| p.price_cents = l.price_min_cents - 100), Err(PlanError::PriceOutOfRange));
    assert_eq!(with(&|p| p.price_cents = l.price_max_cents), Ok(()));
    assert_eq!(with(&|p| p.price_cents = l.price_max_cents + 100), Err(PlanError::PriceOutOfRange));
    assert_eq!(with(&|p| p.seasonal_target = l.seasonal_min), Ok(()));
    assert_eq!(with(&|p| p.seasonal_target = l.seasonal_max), Ok(()));
    assert!(with(&|p| p.seasonal_target = l.seasonal_max + 1).is_err());
    assert_eq!(with(&|p| p.wage_pct = l.wage_max_pct), Ok(()));
    assert_eq!(with(&|p| p.wage_pct = l.wage_max_pct + 1), Err(PlanError::WageOutOfRange));
    assert_eq!(with(&|p| p.marketing_cents = l.marketing_max_cents), Ok(()));
    assert!(l.renovation_available);
}
```
  (`seasonal_min` može biti 0 pa se `- 1` ne testira; `seasonal_max + 1` pada ili na `SeasonalOutOfRange` ili na
  `SeasonalStepTooBig`.)

- [ ] **Step 2: Pokreni** `cargo test -p game --test round; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 3: `game.rs` — fold i upiti.**

```rust
use std::collections::BTreeMap;

use content::Content;
use sim_hotel::{
    Category, HotelState, IndexSet, Month, MonthPlan, Profile, RefTable, SituationTemplate, start_indices,
};

use crate::actor::Actor;
use crate::error::{GameError, corrupt};
use crate::event::{GameCreated, GameEvent, HotelSetup, Recorded, RoundResolved};
use crate::ids::{GameId, HotelId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundStatus {
    /// ZAKAZANA (ARHITEKTURA §8.1): prethodni mjesec još nije obračunat.
    Scheduled,
    Open,
    /// ZATVARANJE: gumb pritisnut, posao obračuna čeka u redu.
    Closing,
    Resolved,
}

// HotelRecord i Game točno kako stoji u Interfaces (derive `Clone, Debug`).

impl Game {
    pub fn fold(records: &[Recorded]) -> Result<Game, GameError> {
        let (first, rest) = records.split_first().ok_or_else(|| corrupt("prazan dnevnik"))?;
        let GameEvent::GameCreated(c) = &first.event else {
            return Err(corrupt("prvi događaj nije GameCreated"));
        };
        if first.seq != 1 {
            return Err(corrupt("GameCreated nije na rednom broju 1"));
        }
        let mut g = Game::created(c, first)?;
        for r in rest {
            g.apply(r.clone())?;
        }
        Ok(g)
    }

    fn created(c: &GameCreated, r: &Recorded) -> Result<Game, GameError> {
        if c.snapshot.fingerprint() != c.ruleset_id {
            return Err(corrupt("otisak pravila se ne slaže sa snimkom"));
        }
        let content = c.snapshot.build().map_err(|e| corrupt(format!("snimak: {e:?}")))?;
        let mut setups: Vec<HotelSetup> = c.hotels.clone();
        setups.sort_by_key(|h| h.slot);
        let mut refs = BTreeMap::new();
        let mut hotels = Vec::with_capacity(setups.len());
        for setup in setups {
            let profile = Profile::from_code(&setup.profile)
                .ok_or_else(|| corrupt(format!("profil {}", setup.profile)))?;
            let spec = content.catalog.profile(profile).map_err(|e| corrupt(format!("{e:?}")))?;
            let state = HotelState::from_profile(spec).map_err(|e| corrupt(format!("{e:?}")))?;
            if !refs.contains_key(&state.category) {
                let t = RefTable::compute(&content.catalog, state.category).map_err(|e| corrupt(format!("{e:?}")))?;
                refs.insert(state.category, t);
            }
            let table = refs.get(&state.category).ok_or_else(|| corrupt("referentna tablica"))?;
            let start = start_indices(&state, &content.catalog, table).map_err(|e| corrupt(format!("{e:?}")))?;
            hotels.push(HotelRecord {
                setup,
                profile,
                start_state: state,
                start_indices: start,
                plans: BTreeMap::new(),
                answers: BTreeMap::new(),
            });
        }
        Ok(Game {
            id: c.game_id,
            seed: c.seed,
            months: c.months.iter().map(|&m| m.into()).collect(),
            content,
            hotels,
            closed: BTreeMap::new(),
            resolved: BTreeMap::new(),
            last_seq: 1,
            history: vec![r.clone()],
            refs,
        })
    }

    /// Primijeni jedan zapisani događaj. Pravila su provjerena pri upisu; prekršaj ovdje znači pokvaren dnevnik.
    pub fn apply(&mut self, r: Recorded) -> Result<(), GameError> {
        if r.seq != self.last_seq + 1 {
            return Err(corrupt(format!("redni broj {} nakon {}", r.seq, self.last_seq)));
        }
        match &r.event {
            GameEvent::GameCreated(_) => return Err(corrupt("drugi GameCreated")),
            GameEvent::PlanSet(p) => {
                let m = p.month.into();
                self.expect_open(m)?;
                let plan = MonthPlan::from(&p.plan);
                self.hotel_mut(p.hotel_id)?.plans.insert(m, plan);
            }
            GameEvent::AnswerChosen(a) => {
                let m = a.month.into();
                self.expect_open(m)?;
                let h = self.hotel_mut(a.hotel_id)?;
                if h.answers.insert((m, a.situation.clone()), a.answer).is_some() {
                    return Err(corrupt(format!("dvostruki odgovor na {}", a.situation)));
                }
            }
            GameEvent::RoundClosed(c) => {
                let m = c.month.into();
                self.expect_open(m)?;
                self.closed.insert(m, r.actor.clone());
            }
            GameEvent::RoundResolved(rr) => {
                let m = rr.month.into();
                if self.status(m)? != RoundStatus::Closing {
                    return Err(corrupt("obračun runde koja nije zatvorena"));
                }
                self.resolved.insert(m, rr.clone());
            }
        }
        self.last_seq = r.seq;
        self.history.push(r);
        Ok(())
    }

    fn expect_open(&self, m: Month) -> Result<(), GameError> {
        match self.status(m)? {
            RoundStatus::Open => Ok(()),
            s => Err(corrupt(format!("događaj za rundu u stanju {s:?}"))),
        }
    }

    pub fn current_month(&self) -> Option<Month> {
        self.months.iter().copied().find(|m| !self.resolved.contains_key(m))
    }

    pub fn status(&self, m: Month) -> Result<RoundStatus, GameError> {
        if !self.months.contains(&m) {
            return Err(GameError::UnknownMonth);
        }
        Ok(if self.resolved.contains_key(&m) {
            RoundStatus::Resolved
        } else if self.closed.contains_key(&m) {
            RoundStatus::Closing
        } else if self.current_month() == Some(m) {
            RoundStatus::Open
        } else {
            RoundStatus::Scheduled
        })
    }

    /// Stanje hotela na POČETKU mjeseca `m`: početno stanje ili `after` iz obračuna prethodnog mjeseca.
    pub fn state_at(&self, hotel: HotelId, m: Month) -> Result<HotelState, GameError> {
        let h = self.hotel(hotel)?;
        let pos = self.months.iter().position(|&x| x == m).ok_or(GameError::UnknownMonth)?;
        let Some(prev) = pos.checked_sub(1).and_then(|p| self.months.get(p)) else {
            return Ok(h.start_state.clone());
        };
        let rr = self.resolved.get(prev).ok_or(GameError::RoundNotOpen)?;
        let res = rr
            .hotels
            .iter()
            .find(|x| x.hotel_id == hotel)
            .ok_or_else(|| corrupt("hotel nedostaje u obračunu"))?;
        HotelState::try_from(&res.report.after)
    }
}
```
  Ostatak `impl Game` (svaki kratko, bez novih pravila):
  - `hotel(id)` / `hotel_mut(id)` (privatno) → `GameError::UnknownHotel` ako ga nema;
  - `latest_state(id)`: `after` zadnjeg obračunatog mjeseca (redoslijedom `months`) ili `start_state`;
  - `effective_plan(id, m)`: `plans.get(&m)` ili `MonthPlan::default_for(&self.state_at(id, m)?, m, &catalog)`
    (greška kataloga → `corrupt`);
  - `situations_in(m)`: `self.content.situations.iter().filter(|t| t.month == m).collect()` (redoslijed iz snimka);
  - `unanswered_count(id, m)`: broj predložaka mjeseca bez ključa `(m, code)` u `answers`, `u8::try_from` (→ `corrupt`);
  - `refs(c)`: `self.refs.get(&c).ok_or_else(|| corrupt("referentna tablica"))`.

- [ ] **Step 4: `command.rs` — nova igra, roboti, naredbe, granice plana.**

```rust
use engine::calc as c;
use engine::{Rng, label};
use sim_hotel::{Catalog, HotelState, Month, MonthPlan, Profile};

use crate::actor::{Actor, strategy_from_code};
use crate::dto::{MonthCode, PlanDto};
use crate::error::{GameError, corrupt};
use crate::event::*;
use crate::game::{Game, RoundStatus};
use crate::ids::{GameId, GuestId, HotelId};

const MONTHS: [Month; 2] = [Month::Jun, Month::Jul];
const ROBOTS: [(&str, &str); 3] = [("stedisa", "Štediša"), ("brizni", "Brižni"), ("lovac", "Lovac na popunjenost")];

fn pos(m: Month) -> Result<u64, GameError> {
    u64::try_from(m.position()).map_err(|_| corrupt("mjesec"))
}

pub fn new_game(
    game_id: GameId,
    seed: u64,
    snapshot: &content::Snapshot,
    guest: GuestId,
    hotel_ids: [HotelId; 4],
) -> Result<Vec<(Actor, GameEvent)>, GameError> {
    fn profile_at(n: u64) -> Profile {
        match n % 3 {
            0 => Profile::A,
            1 => Profile::B,
            _ => Profile::C,
        }
    }
    let profile = profile_at(Rng::derive(seed, &[label("game:profile")]).range(0, 3));
    let rot = Rng::derive(seed, &[label("game:robot-profiles")]).range(0, 3);
    let [h1, h2, h3, h4] = hotel_ids;
    let mut hotels = vec![HotelSetup {
        hotel_id: h1,
        slot: 1,
        name: "Tvoj hotel".into(),
        profile: profile.code().into(),
        controller: Controller::Human { guest_id: guest },
    }];
    for ((id, slot), (code, name)) in [h2, h3, h4].into_iter().zip(2_u8..).zip(ROBOTS) {
        hotels.push(HotelSetup {
            hotel_id: id,
            slot,
            name: name.into(),
            profile: profile_at(rot + u64::from(slot)).code().into(),
            controller: Controller::Robot { strategy: code.into() },
        });
    }
    let created = GameCreated {
        game_id,
        seed,
        ruleset_id: snapshot.fingerprint(),
        core_version: sim_hotel::CORE_VERSION.into(),
        snapshot: snapshot.clone(),
        months: MONTHS.iter().map(|&m| m.into()).collect(),
        hotels,
    };
    let first = Recorded { seq: 1, actor: Actor::System, event: GameEvent::GameCreated(created) };
    let game = Game::fold(std::slice::from_ref(&first))?;
    let mut out = vec![(first.actor, first.event)];
    out.extend(game.robot_moves(Month::Jun)?);
    Ok(out)
}

impl Game {
    /// Roboti igraju istim naredbama kao čovjek (DEMO §1.3): PlanSet i AnswerChosen za mjesec koji se upravo otvara.
    pub fn robot_moves(&self, m: Month) -> Result<Vec<(Actor, GameEvent)>, GameError> {
        let cat = &self.content.catalog;
        let mut out = Vec::new();
        for h in &self.hotels {
            let Controller::Robot { strategy } = &h.setup.controller else { continue };
            let st = strategy_from_code(strategy).ok_or_else(|| corrupt(format!("strategija {strategy}")))?;
            let id = h.setup.hotel_id;
            let state = self.state_at(id, m)?;
            let slot = u64::from(h.setup.slot);
            let mut rng = Rng::derive(self.seed, &[label("game:robot-plan"), pos(m)?, slot]);
            let plan = st.plan(&state, m, cat, &mut rng).map_err(|e| corrupt(format!("{e:?}")))?;
            plan.validate(&state, m, cat).map_err(|e| corrupt(format!("robot {strategy}: {e:?}")))?;
            out.push((Actor::Robot(st), GameEvent::PlanSet(PlanSet { hotel_id: id, month: m.into(), plan: (&plan).into() })));
            for t in self.situations_in(m) {
                let mut rng = Rng::derive(self.seed, &[label("game:robot-answer"), pos(m)?, slot, label(&t.code)]);
                let i = st.choose(&t.answers, cat, state.category, &mut rng).map_err(|e| corrupt(format!("{e:?}")))?;
                let answer = u8::try_from(i).map_err(|_| corrupt("indeks odgovora"))?;
                out.push((Actor::Robot(st), GameEvent::AnswerChosen(AnswerChosen {
                    hotel_id: id,
                    month: m.into(),
                    situation: t.code.clone(),
                    answer,
                })));
            }
        }
        Ok(out)
    }

    pub fn set_plan(&self, hotel: HotelId, m: Month, plan: MonthPlan) -> Result<GameEvent, GameError> {
        if self.status(m)? != RoundStatus::Open {
            return Err(GameError::RoundNotOpen);
        }
        let state = self.state_at(hotel, m)?;
        plan.validate(&state, m, &self.content.catalog).map_err(GameError::Plan)?;
        Ok(GameEvent::PlanSet(PlanSet { hotel_id: hotel, month: m.into(), plan: (&plan).into() }))
    }

    pub fn choose_answer(&self, hotel: HotelId, code: &str, answer: u8) -> Result<GameEvent, GameError> {
        self.hotel(hotel)?;
        let t = self
            .content
            .situations
            .iter()
            .find(|t| t.code == code && self.months.contains(&t.month))
            .ok_or(GameError::UnknownSituation)?;
        if self.status(t.month)? != RoundStatus::Open {
            return Err(GameError::RoundNotOpen);
        }
        if usize::from(answer) >= t.answers.len() {
            return Err(GameError::UnknownAnswer);
        }
        let h = self.hotel(hotel)?;
        if h.answers.contains_key(&(t.month, t.code.clone())) {
            return Err(GameError::AlreadyAnswered);
        }
        Ok(GameEvent::AnswerChosen(AnswerChosen { hotel_id: hotel, month: t.month.into(), situation: t.code.clone(), answer }))
    }

    /// `None` = runda je već zatvorena ili obračunata (drugi klik vraća stanje, ne piše ništa).
    pub fn close_round(&self, m: Month) -> Result<Option<GameEvent>, GameError> {
        match self.status(m)? {
            RoundStatus::Open => Ok(Some(GameEvent::RoundClosed(RoundClosed { month: m.into() }))),
            RoundStatus::Closing | RoundStatus::Resolved => Ok(None),
            RoundStatus::Scheduled => Err(GameError::RoundNotOpen),
        }
    }
}

// PlanLimits (polja iz Interfaces, derive Clone, Copy, Debug, PartialEq, Eq).

/// Granice izbornika radnji za ekran — iste brojke koje `MonthPlan::validate` provjerava (test ih veže).
pub fn plan_limits(s: &HotelState, m: Month, cat: &Catalog) -> Result<PlanLimits, GameError> {
    let spec = cat.category(s.category).map_err(|e| corrupt(format!("{e:?}")))?;
    let a = &cat.actions;
    let reference = i128::from(spec.price_ref_cents.get(m));
    let calc = |e: engine::calc::CalcError| corrupt(format!("{e:?}"));
    // cijena u centima mora biti cijeli euro: najmanji euro ≥ min %, najveći euro ≤ max %
    let lo_eur = c::div_floor(c::add(c::mul(i128::from(a.price_min_pct), reference).map_err(calc)?, 9_999).map_err(calc)?, 10_000).map_err(calc)?;
    let hi_eur = c::div_floor(c::mul(i128::from(a.price_max_pct), reference).map_err(calc)?, 10_000).map_err(calc)?;
    Ok(PlanLimits {
        price_min_cents: c::narrow(c::mul(lo_eur, 100).map_err(calc)?).map_err(calc)?,
        price_max_cents: c::narrow(c::mul(hi_eur, 100).map_err(calc)?).map_err(calc)?,
        marketing_max_cents: a.marketing_max_cents,
        seasonal_min: s.seasonal.saturating_sub(a.seasonal_step),
        seasonal_max: s.seasonal.saturating_add(a.seasonal_step).min(a.seasonal_max),
        wage_min_pct: a.wage_min_pct,
        wage_max_pct: a.wage_max_pct,
        renovation_available: !s.renovated,
    })
}
```
  (`MonthCode`/`PlanDto` u `use` služe pretvorbama `.into()`; ako ih clippy prijavi kao neiskorištene, ukloni ih.)

- [ ] **Step 5: Pokreni** `cargo test -p game; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 6: Provjere i commit** — `crates/game/src/game.rs crates/game/src/command.rs crates/game/src/lib.rs
  crates/game/tests/round.rs`, poruka „Dnevnik 3/12: game — fold, stanje runde, nova igra, roboti i naredbe".

---

### Task 4: `game` — obračun runde, ponovljivost, feed i pogledi

**Files:**
- Create: `crates/game/src/resolve.rs` (`resolve_round`, `resolve_job`)
- Create: `crates/game/src/feed.rs` (`FeedItem`, `Game::feed`)
- Create: `crates/game/src/view.rs` (`PublicHotel`, `Seat`, `MonthScore`, `PrivateHotel`, `RealNumbers`, `LastMonth`, `PlanState`)
- Modify: `crates/game/src/lib.rs`
- Test: `crates/game/tests/resolve.rs` (koristi pomoćnike iz `tests/round.rs` — premjesti `hotel_ids`, `GUEST`,
  `created`, `push` u `crates/game/tests/common/mod.rs` i u oba testa `mod common; use common::*;`)

**Interfaces:**
- Consumes: Task 3 (`Game`, `RoundStatus`, `robot_moves`, `state_at`, `effective_plan`, `situations_in`, `refs`,
  `plan_limits`), `sim_hotel::{resolve_month, month_indices, progress_pts, SituationOutcome::{from_answer, unanswered}}`.
- Produces:
  - `Game::resolve_round(&self, Month) -> Result<RoundResolved, GameError>` (samo za `Closing`; hoteli po slotu).
  - `Game::resolve_job(&self, Month) -> Result<Vec<(Actor, GameEvent)>, GameError>`: `Resolved` → prazno;
    `Closing` → `[(System, RoundResolved)]` + `robot_moves(idući mjesec)` ako idući postoji; `Open`/`Scheduled` → `Corrupt`.
  - `pub enum FeedItem` (`Clone, Debug, PartialEq, Eq`):
    `HotelReceived { seq: i64, name: String, profile: String }` ·
    `PlanChanged { seq: i64, month: MonthCode, plan: PlanDto, by: Actor }` ·
    `AnswerChosen { seq: i64, month: MonthCode, situation: String, title: String, answer: u8, label: String, by: Actor }` ·
    `RoundClosed { seq: i64, month: MonthCode, by: Actor }` ·
    `SituationIgnored { seq: i64, month: MonthCode, situation: String, title: String }` ·
    `MonthResolved { seq: i64, month: MonthCode, indices: [u16; 5], progress_hundredths: i32 }` ·
    `Change { seq: i64, month: MonthCode, line: LineDto }`; `Game::feed(&self, HotelId) -> Result<Vec<FeedItem>, GameError>`.
  - `view`: `pub enum Seat { Human, Robot { strategy: String } }` (bez `guest_id` — tuđi gost se ne otkriva);
    `pub struct MonthScore { month: MonthCode, indices: [u16; 5], progress_hundredths: i32 }`;
    `pub struct PublicHotel { hotel_id: HotelId, slot: u8, name: String, seat: Seat, start: [u16; 5], months: Vec<MonthScore> }`;
    `pub struct RealNumbers { rooms: u16, permanent: u16, seasonal: u16, wage_pct: u16, morale_pts: i32,
    experience_pts: i32, equipment_pts: i32, satisfaction_pts: i32, rating: u16, cash_cents: i64, load_hundredths: u16 }`;
    `pub struct LastMonth { month: MonthCode, sold: u32, available: u32, rooms_revenue_cents: i64, gop_cents: i64, goppar_cents: i64 }`;
    `pub struct PlanState { month: MonthCode, plan: PlanDto, limits: PlanLimits }`;
    `pub struct PrivateHotel { profile: String, now: RealNumbers, last_month: Option<LastMonth>, plan: Option<PlanState> }`
    (sva polja `pub`, derive `Clone, Debug, PartialEq, Eq`);
    `Game::public_hotel(&self, HotelId) -> Result<PublicHotel, GameError>`, `Game::private_hotel(&self, HotelId) -> Result<PrivateHotel, GameError>`
    (`plan` = `Some` samo dok je trenutni mjesec `Open`).

- [ ] **Step 1: Testovi (crveni).** `crates/game/tests/resolve.rs`:

```rust
mod common;
use common::*;
use content::{Snapshot, repo_dir};
use game::command::new_game;
use game::dto::*;
use game::feed::FeedItem;
use game::game::*;
use game::*;
use sim_hotel::Month;
use uuid::Uuid;

fn me() -> HotelId {
    hotel_ids()[0]
}

/// Lipanj: igrač odgovori na S1 (indeks 0), S3 ostavi; zatvori.
fn june_closed(seed: u64) -> Vec<Recorded> {
    let mut log = created(seed);
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.choose_answer(me(), "S1", 0).unwrap());
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.close_round(Month::Jun).unwrap().unwrap());
    log
}

fn run_job(log: &mut Vec<Recorded>, m: Month) {
    for (a, e) in Game::fold(log).unwrap().resolve_job(m).unwrap() {
        push(log, a, e);
    }
}

#[test]
fn june_resolution_covers_four_hotels_and_unanswered_is_quality_one() {
    let g = Game::fold(&june_closed(42)).unwrap();
    let rr = g.resolve_round(Month::Jun).unwrap();
    assert_eq!(rr.hotels.iter().map(|h| h.hotel_id).collect::<Vec<_>>(), hotel_ids());
    let mine = &rr.hotels[0];
    assert_eq!(mine.situations[0], SituationResult { code: "S1".into(), answer: Some(0), quality: 5 });
    assert_eq!(mine.situations[1], SituationResult { code: "S3".into(), answer: None, quality: 1 });
    assert!(rr.hotels[1..].iter().all(|h| h.situations.iter().all(|s| s.answer.is_some())));
}

#[test]
fn unanswered_s1_costs_satisfaction() {
    let mut log = created(42);
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.close_round(Month::Jun).unwrap().unwrap());
    let rr = Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap();
    let line = rr.hotels[0]
        .report
        .lines
        .iter()
        .find(|l| l.var == VarCode::Satisfaction && l.cause == CauseDto::Situation("S1".into()))
        .expect("neodgovorena S1 ima redak zadovoljstva");
    assert_eq!(line.amount, -400);
}

#[test]
fn replay_gives_byte_identical_round_resolved() {
    let log = june_closed(42);
    let first = Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap();
    // kao iz baze: JSON → tipizirano → JSON
    let stored: RoundResolved = serde_json::from_value(serde_json::to_value(&first).unwrap()).unwrap();
    let again = Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap();
    assert_eq!(serde_json::to_vec(&again).unwrap(), serde_json::to_vec(&first).unwrap());
    assert_eq!(serde_json::to_vec(&stored).unwrap(), serde_json::to_vec(&first).unwrap());
    let other = Game::fold(&june_closed(42)).unwrap().resolve_round(Month::Jun).unwrap();
    assert_eq!(other, first, "isto sjeme, iste naredbe → isti obračun");
}

#[test]
fn resolution_uses_snapshot_not_files() {
    let snap = Snapshot::read(&repo_dir()).unwrap();
    let mut v = serde_json::to_value(&snap).unwrap();
    for p in ["A", "B", "C"] {
        let f = v["profiles"][p]["fixed_cents"].as_i64().unwrap();
        v["profiles"][p]["fixed_cents"] = serde_json::json!(f + 100_000);
    }
    let changed: Snapshot = serde_json::from_value(v).unwrap();
    let log_of = |s: &Snapshot| {
        let mut log: Vec<Recorded> = new_game(GameId(Uuid::from_u128(1)), 42, s, GUEST, hotel_ids())
            .unwrap()
            .into_iter()
            .zip(1..)
            .map(|((actor, event), seq)| Recorded { seq, actor, event })
            .collect();
        let g = Game::fold(&log).unwrap();
        push(&mut log, Actor::Guest(GUEST), g.close_round(Month::Jun).unwrap().unwrap());
        Game::fold(&log).unwrap().resolve_round(Month::Jun).unwrap()
    };
    let a = log_of(&snap);
    let b = log_of(&changed);
    // fiksni troškovi dolaze iz snimka igre, ne iz datoteka u content/hotel
    assert_eq!(b.hotels[0].report.before.fixed_cents, a.hotels[0].report.before.fixed_cents + 100_000);
    assert_ne!(b.hotels[0].report.money.gop_cents, a.hotels[0].report.money.gop_cents);
}

#[test]
fn june_resolution_opens_july_with_robot_moves() {
    let mut log = june_closed(42);
    let before = log.len();
    run_job(&mut log, Month::Jun);
    let kinds: Vec<&str> = log[before..].iter().map(|r| r.event.kind()).collect();
    assert_eq!(kinds[0], "RoundResolved");
    assert_eq!(kinds.len(), 1 + 3 * 2, "svaki robot: PlanSet + S2");
    let g = Game::fold(&log).unwrap();
    assert_eq!(g.current_month(), Some(Month::Jul));
    assert_eq!(g.status(Month::Jul), Ok(RoundStatus::Open));
    let GameEvent::RoundResolved(rr) = &log[before].event else { panic!() };
    assert_eq!(HotelStateDto::from(&g.state_at(me(), Month::Jul).unwrap()), rr.hotels[0].report.after);
}

#[test]
fn resolve_job_twice_is_noop() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    assert_eq!(Game::fold(&log).unwrap().resolve_job(Month::Jun).unwrap(), vec![]);
}

#[test]
fn july_resolution_finishes_game() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    let g = Game::fold(&log).unwrap();
    push(&mut log, Actor::Guest(GUEST), g.close_round(Month::Jul).unwrap().unwrap());
    let before = log.len();
    run_job(&mut log, Month::Jul);
    assert_eq!(log.len(), before + 1, "nakon srpnja nema novih poteza robota");
    let g = Game::fold(&log).unwrap();
    assert_eq!(g.current_month(), None);
    assert_eq!(g.close_round(Month::Jul), Ok(None));
    assert_eq!(g.public_hotel(me()).unwrap().months.len(), 2);
    assert_eq!(g.private_hotel(me()).unwrap().plan, None, "igra je gotova");
}

#[test]
fn feed_is_chronological_and_only_mine() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    let feed = Game::fold(&log).unwrap().feed(me()).unwrap();
    assert!(matches!(feed[0], FeedItem::HotelReceived { seq: 1, .. }));
    assert!(feed.windows(2).all(|w| seq(&w[0]) <= seq(&w[1])));
    assert!(feed.iter().any(|f| matches!(f, FeedItem::AnswerChosen { situation, by: Actor::Guest(_), .. } if situation == "S1")));
    assert!(feed.iter().any(|f| matches!(f, FeedItem::SituationIgnored { situation, .. } if situation == "S3")));
    assert!(feed.iter().any(|f| matches!(f, FeedItem::MonthResolved { month: MonthCode::Jun, .. })));
    assert!(feed.iter().any(|f| matches!(f, FeedItem::Change { line, .. } if line.cause == CauseDto::Situation("S1".into()))));
    assert!(!feed.iter().any(|f| matches!(f, FeedItem::PlanChanged { by: Actor::Robot(_), .. })), "tuđi potezi nisu u mom feedu");
}

fn seq(f: &FeedItem) -> i64 {
    match f {
        FeedItem::HotelReceived { seq, .. }
        | FeedItem::PlanChanged { seq, .. }
        | FeedItem::AnswerChosen { seq, .. }
        | FeedItem::RoundClosed { seq, .. }
        | FeedItem::SituationIgnored { seq, .. }
        | FeedItem::MonthResolved { seq, .. }
        | FeedItem::Change { seq, .. } => *seq,
    }
}

#[test]
fn views_split_public_and_private() {
    let mut log = june_closed(42);
    run_job(&mut log, Month::Jun);
    let g = Game::fold(&log).unwrap();
    let p = g.public_hotel(hotel_ids()[1]).unwrap();
    assert_eq!(p.seat, game::view::Seat::Robot { strategy: "stedisa".into() });
    assert_eq!(p.months.len(), 1);
    let mine = g.private_hotel(me()).unwrap();
    let GameEvent::RoundResolved(rr) = &g.history.iter().rev().find(|r| r.event.kind() == "RoundResolved").unwrap().event else { panic!() };
    assert_eq!(mine.now.cash_cents, rr.hotels[0].report.after.cash_cents);
    assert_eq!(mine.last_month.as_ref().unwrap().month, MonthCode::Jun);
    let plan = mine.plan.expect("srpanj je otvoren");
    assert_eq!(plan.month, MonthCode::Jul);
}
```

- [ ] **Step 2: Pokreni** `cargo test -p game --test resolve; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 3: `resolve.rs`.**

```rust
use sim_hotel::{Month, SituationOutcome, month_indices, progress_pts, resolve_month};

use crate::actor::Actor;
use crate::dto::{MonthReportDto, indices_dto};
use crate::error::{GameError, corrupt};
use crate::event::{GameEvent, HotelResolution, Recorded, RoundResolved, SituationResult};
use crate::game::{Game, RoundStatus};

impl Game {
    /// Obračun runde za sve hotele, redom po slotu, iz snimka pravila igre (ARHITEKTURA §7.5; DEMO §1.2).
    pub fn resolve_round(&self, m: Month) -> Result<RoundResolved, GameError> {
        if self.status(m)? != RoundStatus::Closing {
            return Err(corrupt("obračun runde koja nije zatvorena"));
        }
        let cat = &self.content.catalog;
        let err = |e: &dyn std::fmt::Debug| corrupt(format!("{e:?}"));
        let mut hotels = Vec::with_capacity(self.hotels.len());
        for h in &self.hotels {
            let id = h.setup.hotel_id;
            let state = self.state_at(id, m)?;
            let plan = self.effective_plan(id, m)?;
            let mut outcomes = Vec::new();
            let mut results = Vec::new();
            for t in self.situations_in(m) {
                match h.answers.get(&(m, t.code.clone())) {
                    Some(&i) => {
                        let a = t.answers.get(usize::from(i)).ok_or_else(|| corrupt("indeks odgovora"))?;
                        results.push(SituationResult { code: t.code.clone(), answer: Some(i), quality: a.quality });
                        outcomes.push(SituationOutcome::from_answer(&t.code, a));
                    }
                    None => {
                        let o = SituationOutcome::unanswered(t);
                        results.push(SituationResult { code: t.code.clone(), answer: None, quality: o.quality });
                        outcomes.push(o);
                    }
                }
            }
            let report = resolve_month(&state, m, &plan, &outcomes, cat).map_err(|e| err(&e))?;
            let idx = month_indices(&report, cat, self.refs(state.category)?).map_err(|e| err(&e))?;
            let progress = progress_pts(&h.start_indices, &idx).map_err(|e| err(&e))?;
            hotels.push(HotelResolution {
                hotel_id: id,
                plan: (&plan).into(),
                situations: results,
                report: MonthReportDto::from(&report),
                indices: indices_dto(&idx),
                progress_hundredths: progress,
            });
        }
        Ok(RoundResolved { month: m.into(), hotels })
    }

    /// Odluka posla obračuna (red poslova, ADR-015 t. 8): idempotentna po stanju runde.
    pub fn resolve_job(&self, m: Month) -> Result<Vec<(Actor, GameEvent)>, GameError> {
        match self.status(m)? {
            RoundStatus::Resolved => Ok(Vec::new()),
            RoundStatus::Closing => {
                let rr = self.resolve_round(m)?;
                let mut next = self.clone();
                next.apply(Recorded { seq: self.last_seq + 1, actor: Actor::System, event: GameEvent::RoundResolved(rr.clone()) })?;
                let mut out = vec![(Actor::System, GameEvent::RoundResolved(rr))];
                if let Some(n) = next.current_month() {
                    out.extend(next.robot_moves(n)?);
                }
                Ok(out)
            }
            RoundStatus::Open | RoundStatus::Scheduled => Err(corrupt("posao obračuna za rundu koja nije zatvorena")),
        }
    }
}
```

- [ ] **Step 4: `feed.rs`.** `Game::feed(hotel)` prolazi `self.history` redom i za **taj** hotel izvodi stavke:
  `GameCreated` → `HotelReceived { seq, name, profile }`; `PlanSet` tog hotela → `PlanChanged { by: r.actor }`;
  `AnswerChosen` tog hotela → `AnswerChosen` s `title` i `label` iz predloška snimka (nepoznata šifra → `corrupt`);
  `RoundClosed` → `RoundClosed { by }` (zatvaranje je zajedničko, vidi ga svaki hotel); `RoundResolved` → za
  `HotelResolution` tog hotela redom: `SituationIgnored` za svaki `answer: None` (s naslovom), zatim `MonthResolved`,
  zatim `Change` za svaki redak `report.lines` (redoslijed iz jezgre). Sve stavke iz istog događaja nose isti `seq`.
  Nepoznat hotel → `GameError::UnknownHotel`.

- [ ] **Step 5: `view.rs`.** Strukture iz Interfaces. `public_hotel(id)`: `seat` iz `Controller` (bez `guest_id`),
  `start = indices_dto(&h.start_indices)`, `months` = za svaki obračunati mjesec redom `months` njegov `MonthScore`.
  `private_hotel(id)`: `profile = h.profile.code()`, `now` iz `latest_state(id)`; `last_month` iz zadnjeg obračunatog
  mjeseca (`report.market.sold`, `report.market.available`, `report.money.rooms_revenue_cents`, `gop_cents`,
  `goppar_cents`); `plan` = `Some(PlanState { month, plan: effective_plan, limits: plan_limits(state_at, …) })` samo
  ako je `current_month()` u stanju `Open`, inače `None`.

- [ ] **Step 6: Pokreni** `cargo test -p game; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 7: Provjere i commit** — `crates/game/src crates/game/tests`, poruka
  „Dnevnik 4/12: game — obračun runde, ponovljivost, feed i pogledi".

---
### Task 5: `app` — portovi, greške, lažna pohrana, gost i pogledi po ulozi

**Files:**
- Modify: `crates/app/Cargo.toml`, `crates/app/src/lib.rs` (postojeći kod zdravlja ostaje; dodaj module i `Deps`)
- Create: `crates/app/src/{ports,error,memory,demo,views}.rs`
- Test: `crates/app/tests/views.rs`

**Interfaces:**
- Consumes: Task 2–4 (`game::*`, `game::game::{Game, RoundStatus}`, `game::command::new_game`, `game::view::*`,
  `game::feed::FeedItem`), `content::Snapshot`.
- Produces (re-export iz `app` korijena; `pub use game::{GameId, HotelId, GuestId}` da `store` i `api` ne ovise o `game`):
  - `ports`: `RawEvent { seq: i64, kind: String, schema_ver: i32, payload: serde_json::Value, actor: String }`,
    `NewEvent { kind: String, schema_ver: i32, payload: serde_json::Value, actor: String }`,
    `NewJob { key: String, kind: String, game_id: GameId, payload: serde_json::Value }`,
    `Job { id: i64, key: String, kind: String, game_id: GameId, payload: serde_json::Value, attempts: i32 }`,
    `NewHotel { hotel_id: HotelId, slot: i16, guest_id: Option<GuestId> }`,
    `NewGame { game_id: GameId, created_by: GuestId, hotels: Vec<NewHotel>, events: Vec<NewEvent> }`,
    `#[derive(Default)] Decision { events: Vec<NewEvent>, enqueue: Option<NewJob>, finish_job: Option<i64> }`,
    `Decided { last_seq: i64, appended: usize }`, `StoreError(pub String)`,
    `enum DecideError { Rejected(AppError), Store(StoreError) }`,
    `type Decide<'a> = Box<dyn FnOnce(&[RawEvent]) -> Result<Decision, AppError> + Send + 'a>`,
    traitovi `EventLog`, `Guests`, `JobQueue`, `Ids` (potpisi u Step 3), `RandomIds`, `pub const MAX_ATTEMPTS: i32 = 5`.
  - `error`: `ErrorCode` (popis u Step 3), `ErrorKind { Unauthorized, Forbidden, NotFound, Conflict, Invalid, Unavailable, Internal }`,
    `ErrorCode::{kind, as_str}`, `AppError { code: ErrorCode, detail: String }` s `AppError::{new, internal}`,
    `From<GameError>`, `From<StoreError>`, `From<DecideError>`.
  - `Deps { status: Arc<dyn SystemStatus>, log: Arc<dyn EventLog>, guests: Arc<dyn Guests>, jobs: Arc<dyn JobQueue>,
    ids: Arc<dyn Ids>, snapshot: Arc<content::Snapshot> }` (`Clone`).
  - `demo`: `Caller { guest: Option<GuestId> }` s `Caller::require(&self) -> Result<GuestId, AppError>`;
    `token_hash(&str) -> String`; `caller(&Deps, Option<&str>) -> Result<Caller, AppError>`;
    `DemoStarted { game_id, hotel_id, new_token: Option<String> }`; `start_demo(&Deps, Option<&str>) -> Result<DemoStarted, AppError>`;
    `profile_indices(&str) -> Result<[u16; 5], AppError>`; `pub(crate) fold_raw(&[RawEvent]) -> Result<Game, AppError>`;
    `pub(crate) new_event(&Actor, &GameEvent) -> NewEvent`; `pub(crate) load_game(&Deps, GameId) -> Result<Game, AppError>`.
  - `views`: `HotelView { game_id: GameId, public: PublicHotel, private: Option<PrivateHotel> }`,
    `RoundView { month: MonthCode, status: RoundStatus, unanswered: u8 }`,
    `GameView { game_id: GameId, months: Vec<MonthCode>, current: Option<RoundView>, my_hotel: HotelId, hotels: Vec<PublicHotel> }`,
    `SituationItem { code: String, title: String, answers: Vec<String>, chosen: Option<u8> }`,
    `SituationsView { month: Option<MonthCode>, open: bool, items: Vec<SituationItem> }`;
    `my_hotel`, `game_view`, `hotel_view`, `situations`, `feed` (potpisi u Step 4);
    `pub(crate) member_hotel(&Game, GuestId) -> Result<HotelId, AppError>`,
    `pub(crate) require_owner(&Game, GuestId, HotelId) -> Result<(), AppError>`,
    `pub(crate) round_view(&Game, Month, HotelId) -> Result<RoundView, AppError>`.
  - `memory`: `Memory` (izvodi sva četiri porta + `SystemStatus`), `MemJob`, `JobStatus`, `SeqIds`,
    `deps(content::Snapshot) -> (Deps, Arc<Memory>)`; pomoćnici `Memory::{jobs, events, expire_leases, push_job}`.

- [ ] **Step 1: Ovisnosti.** `crates/app/Cargo.toml` `[dependencies]`: `game.workspace = true`,
  `content.workspace = true`, `sim-hotel.workspace = true`, `serde.workspace = true`, `serde_json.workspace = true`,
  `sha2.workspace = true`, `thiserror.workspace = true`, `uuid = { workspace = true, features = ["v4"] }`.

- [ ] **Step 2: Testovi (crveni).** `crates/app/tests/views.rs`:

```rust
use app::memory::{Memory, deps};
use app::*;
use content::{Snapshot, repo_dir};
use std::sync::Arc;

fn setup() -> (Deps, Arc<Memory>) {
    deps(Snapshot::read(&repo_dir()).unwrap())
}

async fn guest(d: &Deps) -> (Caller, DemoStarted) {
    let s = start_demo(d, None).await.unwrap();
    let c = caller(d, s.new_token.as_deref()).await.unwrap();
    (c, s)
}

fn robot_hotel(mem: &Memory, game: GameId) -> HotelId {
    let created = &mem.events(game)[0];
    HotelId(created.payload["hotels"][1]["hotel_id"].as_str().unwrap().parse().unwrap())
}

#[tokio::test]
async fn start_demo_creates_guest_then_reuses_it() {
    let (d, _) = setup();
    let first = start_demo(&d, None).await.unwrap();
    let tok = first.new_token.clone().expect("novi gost dobiva token");
    assert_eq!(tok.len(), 32);
    let second = start_demo(&d, Some(&tok)).await.unwrap();
    assert_eq!(second.new_token, None, "postojeći gost ne dobiva novi kolačić");
    assert_ne!(second.game_id, first.game_id);
    let c = caller(&d, Some(&tok)).await.unwrap();
    assert_eq!(my_hotel(&d, &c).await.unwrap().game_id, second.game_id, "moj hotel = zadnja igra");
    let unknown = start_demo(&d, Some("0123456789abcdef0123456789abcdef")).await.unwrap();
    assert!(unknown.new_token.is_some(), "nepoznat token = novi gost");
}

#[tokio::test]
async fn visibility_by_role() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let (other, _) = guest(&d).await;
    let anon = Caller { guest: None };
    let own = hotel_view(&d, &me, s.hotel_id).await.unwrap();
    assert!(own.private.is_some(), "vlasnik vidi stvarne brojke");
    let robot = hotel_view(&d, &me, robot_hotel(&mem, s.game_id)).await.unwrap();
    assert!(robot.private.is_none(), "tuđi hotel: samo ocjena i indeksi (ARHITEKTURA §12)");
    assert_eq!(hotel_view(&d, &other, s.hotel_id).await.unwrap_err().code, ErrorCode::NotInGame);
    assert_eq!(hotel_view(&d, &anon, s.hotel_id).await.unwrap_err().code, ErrorCode::GuestRequired);
    assert_eq!(game_view(&d, &other, s.game_id).await.unwrap_err().code, ErrorCode::NotInGame);
    assert_eq!(my_hotel(&d, &anon).await.unwrap_err().code, ErrorCode::GuestRequired);
}

#[tokio::test]
async fn game_view_lists_four_hotels_with_open_june() {
    let (d, _) = setup();
    let (me, s) = guest(&d).await;
    let g = game_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(g.hotels.len(), 4);
    assert_eq!(g.my_hotel, s.hotel_id);
    let r = g.current.unwrap();
    assert_eq!((r.month, r.status, r.unanswered), (game::dto::MonthCode::Jun, game::game::RoundStatus::Open, 2));
}

#[tokio::test]
async fn situations_and_feed_are_owner_only() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let v = situations(&d, &me, s.hotel_id).await.unwrap();
    assert!(v.open);
    assert_eq!(v.items.iter().map(|i| i.code.as_str()).collect::<Vec<_>>(), ["S1", "S3"]);
    assert!(v.items.iter().all(|i| i.answers.len() == 4 && i.chosen.is_none()));
    let robot = robot_hotel(&mem, s.game_id);
    assert_eq!(situations(&d, &me, robot).await.unwrap_err().code, ErrorCode::NotYourHotel);
    assert_eq!(feed(&d, &me, robot).await.unwrap_err().code, ErrorCode::NotYourHotel);
    assert!(!feed(&d, &me, s.hotel_id).await.unwrap().is_empty());
}

#[tokio::test]
async fn unknown_ids_and_profiles() {
    let (d, _) = setup();
    let (me, _) = guest(&d).await;
    let nobody = uuid::Uuid::from_u128(7);
    assert_eq!(hotel_view(&d, &me, HotelId(nobody)).await.unwrap_err().code, ErrorCode::UnknownHotel);
    assert_eq!(game_view(&d, &me, GameId(nobody)).await.unwrap_err().code, ErrorCode::UnknownGame);
    assert_eq!(profile_indices("C").unwrap(), [5000; 5]);
    assert_eq!(profile_indices("D").unwrap_err().code, ErrorCode::UnknownProfile);
}
```
  (`app` dev-ovisnosti: `tokio.workspace = true` postoji; dodaj `uuid.workspace = true` i `game.workspace = true`
  ako ih testovi ne vide kroz re-export.)

- [ ] **Step 3: Pokreni** `cargo test -p app --test views; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 4: `ports.rs` i `error.rs`.**

```rust
// ports.rs — sučelja prema vanjskom svijetu (ARHITEKTURA §6.2); izvodi ih `store`, u testovima `memory`.
use crate::{AppError, BoxFut};
use game::{GameId, GuestId, HotelId};

// RawEvent, NewEvent, NewJob, Job, NewHotel, NewGame, Decision, Decided — polja točno kao u Interfaces,
// derive `Clone, Debug, PartialEq` (Decision još i `Default`).

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("pohrana: {0}")]
pub struct StoreError(pub String);

#[derive(Debug)]
pub enum DecideError {
    /// Pravilo igre ili pravo pristupa odbilo je naredbu; ništa nije upisano.
    Rejected(AppError),
    Store(StoreError),
}

pub type Decide<'a> = Box<dyn FnOnce(&[RawEvent]) -> Result<Decision, AppError> + Send + 'a>;

/// Najviše pokušaja jednog posla prije statusa `failed` (ADR-015 t. 8).
pub const MAX_ATTEMPTS: i32 = 5;

pub trait EventLog: Send + Sync {
    /// `games`, `hotels` i prvi događaji igre (redni brojevi od 1) u JEDNOJ transakciji.
    fn create_game(&self, game: NewGame) -> BoxFut<'_, Result<(), StoreError>>;
    /// Cijeli tok `game:<id>` redom; prazan popis = igra ne postoji.
    fn load(&self, game: GameId) -> BoxFut<'_, Result<Vec<RawEvent>, StoreError>>;
    /// U JEDNOJ transakciji pod zaključavanjem igre: učitaj tok, pozovi `decide`, upiši događaje (redni brojevi
    /// nastavljaju tok), upiši posao (`ON CONFLICT` po ključu: ništa) i označi `finish_job` gotovim.
    /// `decide` koji vrati grešku ne upisuje ništa.
    fn decide<'a>(&'a self, game: GameId, decide: Decide<'a>) -> BoxFut<'a, Result<Decided, DecideError>>;
}

pub trait Guests: Send + Sync {
    fn create_guest(&self, id: GuestId, token_hash: String) -> BoxFut<'_, Result<(), StoreError>>;
    fn guest_by_token_hash(&self, token_hash: String) -> BoxFut<'_, Result<Option<GuestId>, StoreError>>;
    fn game_of_hotel(&self, hotel: HotelId) -> BoxFut<'_, Result<Option<GameId>, StoreError>>;
    /// Hotel gosta u njegovoj zadnjoj igri.
    fn latest_hotel_of(&self, guest: GuestId) -> BoxFut<'_, Result<Option<(GameId, HotelId)>, StoreError>>;
}

pub trait JobQueue: Send + Sync {
    /// Preuzmi najstariji posao u redu ili posao s isteklim zakupom; zakup traje `lease_secs` (ARHITEKTURA §9.4).
    fn claim(&self, lease_secs: i32) -> BoxFut<'_, Result<Option<Job>, StoreError>>;
    /// Vrati posao u red, ili ga označi `failed` kad je potrošio `MAX_ATTEMPTS`.
    fn fail(&self, id: i64, error: String) -> BoxFut<'_, Result<(), StoreError>>;
}

/// Izvor identifikatora i sjemena; u testovima predvidljiv (`memory::SeqIds`).
pub trait Ids: Send + Sync {
    fn uuid(&self) -> uuid::Uuid;
    fn seed(&self) -> u64;
}

pub struct RandomIds;

impl Ids for RandomIds {
    fn uuid(&self) -> uuid::Uuid {
        uuid::Uuid::new_v4()
    }
    fn seed(&self) -> u64 {
        let (hi, lo) = uuid::Uuid::new_v4().as_u64_pair();
        hi ^ lo.rotate_left(32)
    }
}
```

```rust
// error.rs
use game::GameError;
use sim_hotel::PlanError;

use crate::ports::{DecideError, StoreError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorCode {
    GuestRequired,
    NotInGame,
    NotYourHotel,
    UnknownGame,
    UnknownHotel,
    UnknownMonth,
    UnknownSituation,
    UnknownAnswer,
    UnknownProfile,
    RoundNotOpen,
    AlreadyAnswered,
    PriceNotWholeEuro,
    PriceOutOfRange,
    MarketingOutOfRange,
    SeasonalOutOfRange,
    SeasonalStepTooBig,
    WageOutOfRange,
    RenovationTwice,
    StorageUnavailable,
    Internal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind { Unauthorized, Forbidden, NotFound, Conflict, Invalid, Unavailable, Internal }

impl ErrorCode {
    pub const ALL: [ErrorCode; 20] = [/* svih 20 varijanti redom kako su navedene */];

    pub fn kind(self) -> ErrorKind {
        use ErrorCode::*;
        match self {
            GuestRequired => ErrorKind::Unauthorized,
            NotInGame | NotYourHotel => ErrorKind::Forbidden,
            UnknownGame | UnknownHotel | UnknownMonth | UnknownSituation | UnknownProfile => ErrorKind::NotFound,
            RoundNotOpen | AlreadyAnswered => ErrorKind::Conflict,
            UnknownAnswer | PriceNotWholeEuro | PriceOutOfRange | MarketingOutOfRange | SeasonalOutOfRange
            | SeasonalStepTooBig | WageOutOfRange | RenovationTwice => ErrorKind::Invalid,
            StorageUnavailable => ErrorKind::Unavailable,
            Internal => ErrorKind::Internal,
        }
    }

    /// Stabilan kod za API (snake_case); ekran ga prevodi.
    pub fn as_str(self) -> &'static str { /* iscrpni match: "guest_required", "not_in_game", … */ }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{code:?}: {detail}")]
pub struct AppError {
    pub code: ErrorCode,
    /// Za dnevnik poslužitelja; ne šalje se klijentu.
    pub detail: String,
}

impl AppError {
    pub fn new(code: ErrorCode) -> Self { AppError { code, detail: String::new() } }
    pub fn internal(detail: impl Into<String>) -> Self { AppError { code: ErrorCode::Internal, detail: detail.into() } }
}

impl From<GameError> for AppError {
    fn from(e: GameError) -> Self {
        use ErrorCode as C;
        let code = match &e {
            GameError::UnknownHotel => C::UnknownHotel,
            GameError::UnknownMonth => C::UnknownMonth,
            GameError::UnknownSituation => C::UnknownSituation,
            GameError::UnknownAnswer => C::UnknownAnswer,
            GameError::RoundNotOpen => C::RoundNotOpen,
            GameError::AlreadyAnswered => C::AlreadyAnswered,
            GameError::Plan(p) => match p {
                PlanError::PriceNotWholeEuro => C::PriceNotWholeEuro,
                PlanError::PriceOutOfRange => C::PriceOutOfRange,
                PlanError::MarketingOutOfRange => C::MarketingOutOfRange,
                PlanError::SeasonalOutOfRange => C::SeasonalOutOfRange,
                PlanError::SeasonalStepTooBig => C::SeasonalStepTooBig,
                PlanError::WageOutOfRange => C::WageOutOfRange,
                PlanError::RenovationTwice => C::RenovationTwice,
                PlanError::Calc => C::Internal,
            },
            GameError::Corrupt(_) => C::Internal,
        };
        AppError { code, detail: format!("{e:?}") }
    }
}

impl From<StoreError> for AppError {
    fn from(e: StoreError) -> Self { AppError { code: ErrorCode::StorageUnavailable, detail: e.0 } }
}

impl From<DecideError> for AppError {
    fn from(e: DecideError) -> Self {
        match e { DecideError::Rejected(a) => a, DecideError::Store(s) => s.into() }
    }
}
```

- [ ] **Step 5: `lib.rs`.** Iza postojećeg koda zdravlja (`BoxFut`, `StatusError`, `SystemStatus`, `check_health`
  ostaju gdje jesu):

```rust
pub mod demo;
pub mod error;
pub mod memory;
pub mod ports;
pub mod views;

pub use demo::*;
pub use error::*;
pub use game::{GameId, GuestId, HotelId};
pub use ports::*;
pub use views::*;

use std::sync::Arc;

/// Sve što slučajevi upotrebe trebaju; sastavlja ga `server` (Postgres) ili `memory::deps` (testovi).
#[derive(Clone)]
pub struct Deps {
    pub status: Arc<dyn SystemStatus>,
    pub log: Arc<dyn EventLog>,
    pub guests: Arc<dyn Guests>,
    pub jobs: Arc<dyn JobQueue>,
    pub ids: Arc<dyn Ids>,
    pub snapshot: Arc<content::Snapshot>,
}
```

- [ ] **Step 6: `demo.rs`.**

```rust
use game::command::new_game;
use game::game::Game;
use game::{Actor, GameEvent, GuestId, HotelId, GameId, Recorded, SCHEMA_VER};
use sha2::{Digest, Sha256};
use sim_hotel::{Index, Profile};

use crate::{AppError, Deps, ErrorCode, NewEvent, NewGame, NewHotel, RawEvent};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Caller { pub guest: Option<GuestId> }

impl Caller {
    pub fn require(&self) -> Result<GuestId, AppError> {
        self.guest.ok_or(AppError::new(ErrorCode::GuestRequired))
    }
}

/// U bazi je samo otisak tokena; sam token živi u kolačiću `httpOnly`.
pub fn token_hash(token: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(token.as_bytes()))
}

/// Gost iz kolačića; nepoznat ili pokvaren token = anoniman pozivatelj (nikad greška).
pub async fn caller(deps: &Deps, token: Option<&str>) -> Result<Caller, AppError> {
    let Some(t) = token else { return Ok(Caller { guest: None }) };
    Ok(Caller { guest: deps.guests.guest_by_token_hash(token_hash(t)).await? })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoStarted { pub game_id: GameId, pub hotel_id: HotelId, pub new_token: Option<String> }

/// `POST /v1/demo` (DEMO §1.4): gost (postojeći ili novi) i nova igra s tri robota u jednoj transakciji.
pub async fn start_demo(deps: &Deps, token: Option<&str>) -> Result<DemoStarted, AppError> {
    let (guest, new_token) = match caller(deps, token).await?.guest {
        Some(g) => (g, None),
        None => {
            let id = GuestId(deps.ids.uuid());
            let tok = deps.ids.uuid().simple().to_string();
            deps.guests.create_guest(id, token_hash(&tok)).await?;
            (id, Some(tok))
        }
    };
    let game_id = GameId(deps.ids.uuid());
    let hotel_ids = [(); 4].map(|_| HotelId(deps.ids.uuid()));
    let events = new_game(game_id, deps.ids.seed(), &deps.snapshot, guest, hotel_ids)?;
    let hotels = hotel_ids
        .iter()
        .zip(1_i16..)
        .map(|(&hotel_id, slot)| NewHotel { hotel_id, slot, guest_id: (slot == 1).then_some(guest) })
        .collect();
    deps.log
        .create_game(NewGame {
            game_id,
            created_by: guest,
            hotels,
            events: events.iter().map(|(a, e)| new_event(a, e)).collect(),
        })
        .await?;
    Ok(DemoStarted { game_id, hotel_id: hotel_ids[0], new_token })
}

/// Početni indeksi profila (DEMO §5) — prije u `api/src/demo.rs` mimo `app` (BACKLOG [dnevnik]).
pub fn profile_indices(code: &str) -> Result<[u16; 5], AppError> {
    let p = Profile::from_code(code).ok_or(AppError::new(ErrorCode::UnknownProfile))?;
    let s = p.starting_indices();
    Ok(Index::ALL.map(|i| s.get(i).hundredths()))
}

pub(crate) fn new_event(actor: &Actor, ev: &GameEvent) -> NewEvent {
    NewEvent { kind: ev.kind().to_owned(), schema_ver: SCHEMA_VER, payload: ev.to_payload(), actor: actor.as_db() }
}

pub(crate) fn fold_raw(raw: &[RawEvent]) -> Result<Game, AppError> {
    if raw.is_empty() {
        return Err(AppError::new(ErrorCode::UnknownGame));
    }
    let records = raw
        .iter()
        .map(|r| {
            let actor = Actor::parse(&r.actor).ok_or_else(|| AppError::internal(format!("actor {}", r.actor)))?;
            let event = GameEvent::from_record(&r.kind, r.schema_ver, r.payload.clone())
                .map_err(|e| AppError::internal(format!("{e:?}")))?;
            Ok(Recorded { seq: r.seq, actor, event })
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    Ok(Game::fold(&records)?)
}

pub(crate) async fn load_game(deps: &Deps, id: GameId) -> Result<Game, AppError> {
    fold_raw(&deps.log.load(id).await?)
}
```

- [ ] **Step 7: `views.rs`.** Strukture iz Interfaces (derive `Clone, Debug, PartialEq, Eq`), pomoćnici i slučajevi:

```rust
pub(crate) fn member_hotel(g: &Game, guest: GuestId) -> Result<HotelId, AppError> {
    g.hotels
        .iter()
        .find(|h| matches!(h.setup.controller, Controller::Human { guest_id } if guest_id == guest))
        .map(|h| h.setup.hotel_id)
        .ok_or(AppError::new(ErrorCode::NotInGame))
}

pub(crate) fn require_owner(g: &Game, guest: GuestId, hotel: HotelId) -> Result<(), AppError> {
    let mine = member_hotel(g, guest)?;
    g.hotel(hotel)?;
    if mine == hotel { Ok(()) } else { Err(AppError::new(ErrorCode::NotYourHotel)) }
}

pub(crate) fn round_view(g: &Game, m: Month, mine: HotelId) -> Result<RoundView, AppError> {
    let status = g.status(m)?;
    let unanswered = if status == RoundStatus::Open { g.unanswered_count(mine, m)? } else { 0 };
    Ok(RoundView { month: m.into(), status, unanswered })
}

async fn game_of_hotel(deps: &Deps, hotel: HotelId) -> Result<GameId, AppError> {
    deps.guests.game_of_hotel(hotel).await?.ok_or(AppError::new(ErrorCode::UnknownHotel))
}

pub async fn hotel_view(deps: &Deps, caller: &Caller, hotel: HotelId) -> Result<HotelView, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let g = load_game(deps, game_id).await?;
    let mine = member_hotel(&g, guest)?;
    let public = g.public_hotel(hotel)?;
    let private = if mine == hotel { Some(g.private_hotel(hotel)?) } else { None };
    Ok(HotelView { game_id, public, private })
}
```
  Ostali slučajevi, istim redom provjera (**gost → igra postoji → član → vlasnik**):
  - `my_hotel(deps, caller) -> Result<HotelView, AppError>`: `latest_hotel_of(guest)`; nema igre → `UnknownGame`;
    zatim kao `hotel_view`.
  - `game_view(deps, caller, game_id) -> Result<GameView, AppError>`: `load_game` (prazno → `UnknownGame`),
    `member_hotel`, `hotels` = `public_hotel` za svaki hotel po slotu, `current` = `round_view(current_month)` ili `None`.
  - `situations(deps, caller, hotel) -> Result<SituationsView, AppError>`: `require_owner`; `month = current_month()`;
    `open = status == Open`; stavke za `situations_in(month)`: `answers` su samo `label`-i (indeks = položaj), `chosen`
    iz `answers` hotela. **Kvaliteta i mehanike se ne kopiraju.** Igra gotova → `month: None`, prazno.
  - `feed(deps, caller, hotel) -> Result<Vec<FeedItem>, AppError>`: `require_owner`, pa `g.feed(hotel)`.

- [ ] **Step 8: `memory.rs`.** Ista pravila kao `store`, u `Mutex<Mem>`; zaključavanje se nikad ne drži preko `await`
  (svaka metoda izračuna rezultat sinkrono pa vrati `Box::pin(async move { r })`).

```rust
//! Pohrana u memoriji za testove `app` i `api`: ista pravila kao `store`, bez baze i bez sata.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobStatus { Queued, Running, Done, Failed }

#[derive(Clone, Debug, PartialEq)]
pub struct MemJob { pub id: i64, pub key: String, pub kind: String, pub game_id: GameId,
                    pub payload: serde_json::Value, pub status: JobStatus, pub attempts: i32 }

#[derive(Default)]
struct Mem {
    events: BTreeMap<GameId, Vec<RawEvent>>,
    guests: BTreeMap<String, GuestId>,
    hotels: BTreeMap<HotelId, (GameId, Option<GuestId>)>,
    games: Vec<(GameId, GuestId)>, // redom stvaranja
    jobs: Vec<MemJob>,
}

#[derive(Default)]
pub struct Memory { inner: Mutex<Mem> }

/// Predvidljivi identifikatori: uuid 1, 2, 3 …; sjeme uvijek 42.
#[derive(Default)]
pub struct SeqIds(AtomicU64);
impl Ids for SeqIds {
    fn uuid(&self) -> uuid::Uuid { uuid::Uuid::from_u128(u128::from(self.0.fetch_add(1, Ordering::SeqCst) + 1)) }
    fn seed(&self) -> u64 { 42 }
}

pub fn deps(snapshot: content::Snapshot) -> (Deps, Arc<Memory>) {
    let m = Arc::new(Memory::default());
    let d = Deps { status: m.clone(), log: m.clone(), guests: m.clone(), jobs: m.clone(),
                   ids: Arc::new(SeqIds::default()), snapshot: Arc::new(snapshot) };
    (d, m)
}
```
  Pravila izvedbe (moraju odgovarati `store`-u, testovi Taska 6 i 9–10 ovise o njima):
  - `create_game`: upiši događaje s `seq` 1..=n, hotele u `hotels`, `(game, created_by)` na kraj `games`.
  - `decide`: kloniraj tok, pozovi `decide`; `Err` → `DecideError::Rejected` bez promjene; `Ok` → dodaj događaje s
    `seq = last + 1 …`, `enqueue` dodaj samo ako posao s istim `key` ne postoji (status `Queued`, `attempts` 0, `id` =
    broj poslova + 1), `finish_job` → `Done`. Vrati `Decided { last_seq, appended }`.
  - `claim`: prvi posao po `id` sa statusom `Queued` i `attempts < MAX_ATTEMPTS` → `Running`, `attempts += 1`, vrati
    ga (zakup u memoriji nikad ne istječe sam — vidi `expire_leases`).
  - `fail`: `attempts >= MAX_ATTEMPTS` → `Failed`, inače `Queued`.
  - `latest_hotel_of`: zadnja igra gosta u `games`, pa hotel s tim gostom u toj igri.
  - `SystemStatus::events_count` = ukupan broj događaja.
  - Pomoćnici za testove: `jobs() -> Vec<MemJob>`, `events(GameId) -> Vec<RawEvent>`, `expire_leases()` (sve
    `Running` → `Queued`: kao da je radnik pao i zakup istekao), `push_job(NewJob)` (doda posao **bez** provjere ključa:
    simulira dvostruku isporuku).

- [ ] **Step 9: Pokreni** `cargo test -p app; echo EXIT=$?` → `EXIT=0` (i stari testovi zdravlja).

- [ ] **Step 10: Provjere i commit** — `crates/app`, `Cargo.lock`, poruka
  „Dnevnik 5/12: app — portovi, greške, lažna pohrana, gost i pogledi po ulozi".

---

### Task 6: `app` — naredbe runde, zatvaranje i radnik obračuna

**Files:**
- Create: `crates/app/src/commands.rs`, `crates/app/src/jobs.rs`
- Modify: `crates/app/src/lib.rs` (`pub mod commands; pub mod jobs;` + `pub use`)
- Test: `crates/app/tests/rounds.rs`

**Interfaces:**
- Consumes: Task 5 (`Deps`, `Caller`, `Decision`, `NewJob`, `fold_raw`, `new_event`, `member_hotel`, `require_owner`,
  `round_view`, `situations`, `load_game`, `MAX_ATTEMPTS`), Task 3–4 (`set_plan`, `choose_answer`, `close_round`, `resolve_job`).
- Produces:
  - `set_plan(&Deps, &Caller, HotelId, MonthCode, PlanDto) -> Result<PlanDto, AppError>`
  - `answer(&Deps, &Caller, HotelId, &str, u8) -> Result<SituationsView, AppError>`
  - `round_status(&Deps, &Caller, GameId, MonthCode) -> Result<RoundView, AppError>`
  - `CloseOutcome { accepted: bool, round: RoundView }`; `close_round(&Deps, &Caller, GameId, MonthCode) -> Result<CloseOutcome, AppError>`
  - `RESOLVE_KIND = "resolve_round"`, `resolve_key(GameId, Month) -> String` (`"resolve:<uuid>:<jun|jul>"`),
    `ResolvePayload { month: MonthCode }` (serde), `LEASE_SECS: i32 = 30`,
    `run_next_job(&Deps) -> Result<bool, AppError>` (`true` = obradio posao; greška posla se zapiše u red pa vrati `Err`).

- [ ] **Step 1: Testovi (crveni).** `crates/app/tests/rounds.rs` (pomoćnici `setup`, `guest`, `robot_hotel` kao u
  `views.rs` — kopiraj ih, ne uvozi među test-datotekama):

```rust
use game::dto::MonthCode::{Aug, Jul, Jun};
use game::game::RoundStatus;

async fn events_of(mem: &Memory, g: GameId, kind: &str) -> usize {
    mem.events(g).iter().filter(|e| e.kind == kind).count()
}

#[tokio::test]
async fn plan_answer_close_then_worker_resolves_june() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let mut plan = my_hotel(&d, &me).await.unwrap().private.unwrap().plan.unwrap().plan;
    plan.marketing_cents = 500_000;
    assert_eq!(set_plan(&d, &me, s.hotel_id, Jun, plan.clone()).await.unwrap(), plan);
    answer(&d, &me, s.hotel_id, "S1", 0).await.unwrap();
    assert_eq!(round_status(&d, &me, s.game_id, Jun).await.unwrap().unanswered, 1);
    let c = close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert!(c.accepted);
    assert_eq!(c.round.status, RoundStatus::Closing);
    assert!(run_next_job(&d).await.unwrap());
    assert!(!run_next_job(&d).await.unwrap(), "red je prazan");
    assert_eq!(round_status(&d, &me, s.game_id, Jun).await.unwrap().status, RoundStatus::Resolved);
    let g = game_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(g.current.unwrap().month, Jul);
    assert!(g.hotels.iter().all(|h| h.months.len() == 1));
    assert_eq!(events_of(&mem, s.game_id, "RoundResolved").await, 1);
}

#[tokio::test]
async fn double_close_is_idempotent() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    assert!(close_round(&d, &me, s.game_id, Jun).await.unwrap().accepted);
    let again = close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert!(!again.accepted);
    assert_eq!(again.round.status, RoundStatus::Closing);
    assert_eq!(mem.jobs().len(), 1, "jedan posao po ključu");
    run_next_job(&d).await.unwrap();
    let after = close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert_eq!((after.accepted, after.round.status), (false, RoundStatus::Resolved));
    assert_eq!(events_of(&mem, s.game_id, "RoundClosed").await, 1);
}

#[tokio::test]
async fn plan_for_closed_month_rejected() {
    let (d, _) = setup();
    let (me, s) = guest(&d).await;
    let plan = my_hotel(&d, &me).await.unwrap().private.unwrap().plan.unwrap().plan;
    close_round(&d, &me, s.game_id, Jun).await.unwrap();
    assert_eq!(set_plan(&d, &me, s.hotel_id, Jun, plan).await.unwrap_err().code, ErrorCode::RoundNotOpen);
    assert_eq!(answer(&d, &me, s.hotel_id, "S1", 0).await.unwrap_err().code, ErrorCode::RoundNotOpen);
    assert_eq!(close_round(&d, &me, s.game_id, Jul).await.unwrap_err().code, ErrorCode::RoundNotOpen);
    assert_eq!(close_round(&d, &me, s.game_id, Aug).await.unwrap_err().code, ErrorCode::UnknownMonth);
}

#[tokio::test]
async fn crashed_worker_job_is_reclaimed_and_duplicate_job_is_noop() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    close_round(&d, &me, s.game_id, Jun).await.unwrap();
    let claimed = d.jobs.claim(30).await.unwrap().unwrap(); // radnik preuzme i „padne"
    assert_eq!(claimed.attempts, 1);
    assert!(!run_next_job(&d).await.unwrap(), "zakup još traje");
    mem.expire_leases();
    assert!(run_next_job(&d).await.unwrap());
    mem.push_job(NewJob { key: "dupli".into(), kind: RESOLVE_KIND.into(), game_id: s.game_id,
                          payload: serde_json::json!({"month": "jun"}) });
    assert!(run_next_job(&d).await.unwrap());
    assert_eq!(events_of(&mem, s.game_id, "RoundResolved").await, 1, "drugi posao ne obračunava ponovno");
    assert!(mem.jobs().iter().all(|j| j.status == app::memory::JobStatus::Done));
}

#[tokio::test]
async fn failing_job_is_recorded_and_eventually_failed() {
    let (d, mem) = setup();
    let (_, s) = guest(&d).await;
    mem.push_job(NewJob { key: "los".into(), kind: "nepoznat".into(), game_id: s.game_id, payload: serde_json::json!({}) });
    for _ in 0..MAX_ATTEMPTS {
        assert!(run_next_job(&d).await.is_err());
    }
    assert_eq!(mem.jobs()[0].status, app::memory::JobStatus::Failed);
    assert!(!run_next_job(&d).await.unwrap());
}

#[tokio::test]
async fn rights_by_role() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let (other, _) = guest(&d).await;
    let plan = my_hotel(&d, &me).await.unwrap().private.unwrap().plan.unwrap().plan;
    let robot = robot_hotel(&mem, s.game_id);
    assert_eq!(set_plan(&d, &me, robot, Jun, plan.clone()).await.unwrap_err().code, ErrorCode::NotYourHotel);
    assert_eq!(answer(&d, &me, robot, "S1", 0).await.unwrap_err().code, ErrorCode::NotYourHotel);
    assert_eq!(close_round(&d, &other, s.game_id, Jun).await.unwrap_err().code, ErrorCode::NotInGame);
    let anon = Caller { guest: None };
    assert_eq!(close_round(&d, &anon, s.game_id, Jun).await.unwrap_err().code, ErrorCode::GuestRequired);
    let mut bad = plan;
    bad.price_cents += 50;
    assert_eq!(set_plan(&d, &me, s.hotel_id, Jun, bad).await.unwrap_err().code, ErrorCode::PriceNotWholeEuro);
}

#[tokio::test]
async fn full_demo_season_june_and_july() {
    let (d, _) = setup();
    let (me, s) = guest(&d).await;
    close_round(&d, &me, s.game_id, Jun).await.unwrap();
    run_next_job(&d).await.unwrap();
    let v = situations(&d, &me, s.hotel_id).await.unwrap();
    assert_eq!(v.items.iter().map(|i| i.code.as_str()).collect::<Vec<_>>(), ["S2"]);
    close_round(&d, &me, s.game_id, Jul).await.unwrap();
    run_next_job(&d).await.unwrap();
    let g = game_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(g.current, None, "igra je gotova nakon srpnja");
    assert!(g.hotels.iter().all(|h| h.months.len() == 2));
    assert!(!close_round(&d, &me, s.game_id, Jul).await.unwrap().accepted);
}
```

- [ ] **Step 2: Pokreni** `cargo test -p app --test rounds; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 3: `commands.rs`.** Svaka naredba je **jedan** `decide`: provjera prava i pravila igre događa se nad
  tokom učitanim pod zaključavanjem, pa nema utrke između provjere i upisa.

```rust
use game::dto::{MonthCode, PlanDto};
use game::{Actor, GameId, HotelId};
use sim_hotel::{Month, MonthPlan};

use crate::views::{member_hotel, require_owner, round_view};
use crate::jobs::{RESOLVE_KIND, ResolvePayload, resolve_key};
use crate::{AppError, Caller, Decision, Deps, ErrorCode, NewJob, RawEvent, SituationsView, fold_raw, load_game, new_event};

async fn game_of_hotel(deps: &Deps, hotel: HotelId) -> Result<GameId, AppError> {
    deps.guests.game_of_hotel(hotel).await?.ok_or(AppError::new(ErrorCode::UnknownHotel))
}

pub async fn set_plan(deps: &Deps, caller: &Caller, hotel: HotelId, month: MonthCode, plan: PlanDto) -> Result<PlanDto, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let m: Month = month.into();
    let core = MonthPlan::from(&plan);
    deps.log
        .decide(game_id, Box::new(move |raw: &[RawEvent]| {
            let g = fold_raw(raw)?;
            require_owner(&g, guest, hotel)?;
            let ev = g.set_plan(hotel, m, core)?;
            Ok(Decision { events: vec![new_event(&Actor::Guest(guest), &ev)], ..Decision::default() })
        }))
        .await?;
    Ok(plan)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloseOutcome { pub accepted: bool, pub round: crate::RoundView }

/// „Zatvori mjesec" (BitLifeov „Age +"): `RoundClosed` i posao obračuna u istoj transakciji; drugi klik ne piše ništa.
pub async fn close_round(deps: &Deps, caller: &Caller, game_id: GameId, month: MonthCode) -> Result<CloseOutcome, AppError> {
    let guest = caller.require()?;
    let m: Month = month.into();
    let decided = deps
        .log
        .decide(game_id, Box::new(move |raw: &[RawEvent]| {
            let g = fold_raw(raw)?;
            member_hotel(&g, guest)?;
            match g.close_round(m)? {
                None => Ok(Decision::default()),
                Some(ev) => Ok(Decision {
                    events: vec![new_event(&Actor::Guest(guest), &ev)],
                    enqueue: Some(NewJob {
                        key: resolve_key(game_id, m),
                        kind: RESOLVE_KIND.to_owned(),
                        game_id,
                        payload: serde_json::to_value(ResolvePayload { month }).map_err(|e| AppError::internal(e.to_string()))?,
                    }),
                    finish_job: None,
                }),
            }
        }))
        .await?;
    let g = load_game(deps, game_id).await?;
    let mine = member_hotel(&g, guest)?;
    Ok(CloseOutcome { accepted: decided.appended > 0, round: round_view(&g, m, mine)? })
}
```
  - `answer(...)`: kao `set_plan`, s `g.choose_answer(hotel, &code, answer)` (kloniraj `code` u `String` prije
    `move`), pa vrati `situations(deps, caller, hotel).await`.
  - `round_status(...)`: `caller.require()`, `load_game`, `member_hotel`, `round_view(&g, month.into(), mine)`.
  - Redoslijed provjera u `decide`: `fold_raw` (nepostojeća igra → `UnknownGame`) → pravo (`member_hotel` /
    `require_owner`) → pravilo igre. Prazna igra se nikad ne stvara kroz `decide`.

- [ ] **Step 4: `jobs.rs`.**

```rust
use game::dto::MonthCode;
use game::GameId;
use serde::{Deserialize, Serialize};
use sim_hotel::Month;

use crate::{AppError, Decision, Deps, Job, RawEvent, fold_raw, new_event};

pub const RESOLVE_KIND: &str = "resolve_round";
/// Zakup posla (ARHITEKTURA §9.4): radnik koji ga ne završi u roku vraća ga u red.
pub const LEASE_SECS: i32 = 30;

pub fn resolve_key(game: GameId, m: Month) -> String {
    format!("resolve:{}:{}", game.0, m.code())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvePayload { pub month: MonthCode }

/// Jedan korak radnika: preuzmi posao, obradi ga, a grešku zapiši u red (ponovni pokušaj ili `failed`).
pub async fn run_next_job(deps: &Deps) -> Result<bool, AppError> {
    let Some(job) = deps.jobs.claim(LEASE_SECS).await? else { return Ok(false) };
    match run_job(deps, &job).await {
        Ok(()) => Ok(true),
        Err(e) => {
            deps.jobs.fail(job.id, format!("{e}")).await?;
            Err(e)
        }
    }
}

async fn run_job(deps: &Deps, job: &Job) -> Result<(), AppError> {
    if job.kind != RESOLVE_KIND {
        return Err(AppError::internal(format!("nepoznata vrsta posla {}", job.kind)));
    }
    let p: ResolvePayload = serde_json::from_value(job.payload.clone()).map_err(|e| AppError::internal(e.to_string()))?;
    let id = job.id;
    deps.log
        .decide(job.game_id, Box::new(move |raw: &[RawEvent]| {
            let g = fold_raw(raw)?;
            // idempotentno: već obračunata runda daje praznu odluku, ali posao se ipak zatvara
            let events = g.resolve_job(p.month.into())?.iter().map(|(a, e)| new_event(a, e)).collect();
            Ok(Decision { events, enqueue: None, finish_job: Some(id) })
        }))
        .await?;
    Ok(())
}
```

- [ ] **Step 5: Pokreni** `cargo test -p app; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 6: Provjere i commit** — `crates/app/src/commands.rs crates/app/src/jobs.rs crates/app/src/lib.rs
  crates/app/tests/rounds.rs`, poruka „Dnevnik 6/12: app — naredbe runde, zatvaranje i radnik obračuna".

---

### Task 7: `store` — migracije, dnevnik i gosti u Postgresu

**Files:**
- Create: `crates/store/migrations/0002_events_append_only.sql`, `crates/store/migrations/0003_platform.sql`
- Modify: `crates/store/migrations/0001_events.sql` — **ne dirati** (primijenjene migracije se ne mijenjaju; sqlx
  provjerava kontrolni zbroj)
- Modify: `crates/store/src/lib.rs`; Create: `crates/store/src/log.rs`, `crates/store/src/guests.rs`
- Modify: `crates/store/Cargo.toml`, `Cargo.toml` (sqlx značajke `uuid`, `json`)
- Modify: `crates/store/tests/events.rs`; Create: `crates/store/tests/log.rs`
- Modify: `.sqlx/` (regenerirano)

**Interfaces:**
- Consumes: Task 5 (`app::{EventLog, Guests, RawEvent, NewEvent, NewGame, NewHotel, Decision, Decided, DecideError,
  Decide, StoreError, GameId, GuestId, HotelId}`).
- Produces: `impl EventLog for Store`, `impl Guests for Store`; tablice `guests`, `games` (`seq BIGSERIAL`), `hotels`,
  `jobs` (stupac `job_key`, ne `key`), indeks `events_game_idx`, okidači `events_no_update_delete` i
  `events_no_truncate` (SQLSTATE `42501`), CHECK `events_actor_format` (SQLSTATE `23514`).

- [ ] **Step 1: Ovisnosti.** Workspace `sqlx` značajke: `["runtime-tokio", "postgres", "macros", "migrate", "uuid", "json"]`.
  `crates/store/Cargo.toml`: `serde_json.workspace = true`, `uuid.workspace = true`; dev: `serde_json`, `uuid`, `tokio`.

- [ ] **Step 2: Migracije.** `0002_events_append_only.sql`:

```sql
-- Dnevnik je samo za dopisivanje (ARHITEKTURA §9.1, ADR-015 t. 5): baza odbija izmjenu, brisanje i pražnjenje.
-- Uloga bez prava UPDATE/DELETE dolazi prije Railwaya (BACKLOG); okidač radi već sad, i u sqlx::test bazama.
CREATE INDEX events_game_idx ON events (game_id, id);

ALTER TABLE events ADD CONSTRAINT events_actor_format
    CHECK (actor ~ '^(guest:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}|robot:[a-z_]+|system)$');

CREATE FUNCTION events_append_only() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'tablica events je samo za dopisivanje (% odbijen)', TG_OP
        USING ERRCODE = 'insufficient_privilege';
END
$$;

CREATE TRIGGER events_no_update_delete BEFORE UPDATE OR DELETE ON events
    FOR EACH ROW EXECUTE FUNCTION events_append_only();
CREATE TRIGGER events_no_truncate BEFORE TRUNCATE ON events
    FOR EACH STATEMENT EXECUTE FUNCTION events_append_only();
```
  `0003_platform.sql`:

```sql
-- Platformske tablice (ARHITEKTURA §9.3): obične tablice za pretragu; istina o igri je u events.
CREATE TABLE guests (
    id          UUID PRIMARY KEY,
    token_hash  TEXT NOT NULL UNIQUE,          -- "sha256:<hex>" tokena iz kolačića; sam token se ne sprema
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE games (
    id          UUID PRIMARY KEY,
    seq         BIGSERIAL UNIQUE,              -- redoslijed stvaranja („zadnja igra gosta"), neovisno o satu
    created_by  UUID NOT NULL REFERENCES guests (id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE hotels (
    id        UUID PRIMARY KEY,
    game_id   UUID NOT NULL REFERENCES games (id),
    slot      SMALLINT NOT NULL CHECK (slot BETWEEN 1 AND 4),
    guest_id  UUID REFERENCES guests (id),     -- NULL = robot
    UNIQUE (game_id, slot)
);
CREATE INDEX hotels_guest_idx ON hotels (guest_id);

-- Red poslova (ARHITEKTURA §9.4, ADR-015 t. 8).
CREATE TABLE jobs (
    id            BIGSERIAL PRIMARY KEY,
    job_key       TEXT NOT NULL UNIQUE,        -- npr. resolve:<game>:<mjesec>; drugi upis istog ključa se preskače
    kind          TEXT NOT NULL,
    game_id       UUID NOT NULL,
    payload       JSONB NOT NULL,
    status        TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'done', 'failed')),
    attempts      INTEGER NOT NULL DEFAULT 0,
    locked_until  TIMESTAMPTZ,
    last_error    TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX jobs_ready_idx ON jobs (id) WHERE status IN ('queued', 'running');
```

- [ ] **Step 3: Testovi (crveni).** U `tests/events.rs` zamijeni `stream_sequence_is_unique`:

```rust
fn db_code(e: sqlx::Error) -> String {
    e.as_database_error().and_then(|d| d.code()).map(|c| c.into_owned()).unwrap_or_default()
}

#[sqlx::test]
async fn stream_sequence_is_unique_per_stream(pool: PgPool) {
    let insert = |stream: &'static str| {
        sqlx::query(
            "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
             VALUES ('00000000-0000-0000-0000-000000000001', $1, 1, 'X', 1, '{}'::jsonb, 'system')",
        )
        .bind(stream)
    };
    insert("game:1").execute(&pool).await.unwrap();
    let err = insert("game:1").execute(&pool).await.unwrap_err();
    assert_eq!(db_code(err), "23505", "isti (stream, stream_seq) ne smije proći dvaput");
    insert("game:2").execute(&pool).await.expect("isti redni broj u drugom toku je dopušten");
}

#[sqlx::test]
async fn events_are_append_only(pool: PgPool) {
    sqlx::query(
        "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
         VALUES ('00000000-0000-0000-0000-000000000001', 'game:1', 1, 'X', 1, '{}'::jsonb, 'system')",
    )
    .execute(&pool)
    .await
    .unwrap();
    for sql in ["UPDATE events SET type = 'Y'", "DELETE FROM events", "TRUNCATE events"] {
        let err = sqlx::query(sql).execute(&pool).await.unwrap_err();
        assert_eq!(db_code(err), "42501", "{sql}");
    }
    assert_eq!(Store::from_pool(pool).count_events().await.unwrap(), 1);
}

#[sqlx::test]
async fn actor_must_follow_convention(pool: PgPool) {
    let err = sqlx::query(
        "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
         VALUES ('00000000-0000-0000-0000-000000000001', 'game:1', 1, 'X', 1, '{}'::jsonb, 'leon')",
    )
    .execute(&pool)
    .await
    .unwrap_err();
    assert_eq!(db_code(err), "23514");
}
```
  `tests/log.rs`:

```rust
use app::{Decision, DecideError, EventLog, GameId, Guests, GuestId, HotelId, NewEvent, NewGame, NewHotel, NewJob, RawEvent};
use serde_json::json;
use sqlx::PgPool;
use store::Store;
use uuid::Uuid;

fn ev(kind: &str) -> NewEvent {
    NewEvent { kind: kind.into(), schema_ver: 1, payload: json!({ "k": kind }), actor: "system".into() }
}

async fn game(store: &Store, n: u128) -> (GameId, GuestId, HotelId) {
    let (g, guest, h) = (GameId(Uuid::from_u128(n)), GuestId(Uuid::from_u128(n + 1000)), HotelId(Uuid::from_u128(n + 2000)));
    store.create_guest(guest, format!("sha256:{n}")).await.unwrap();
    store
        .create_game(NewGame { game_id: g, created_by: guest,
            hotels: vec![NewHotel { hotel_id: h, slot: 1, guest_id: Some(guest) }],
            events: vec![ev("GameCreated")] })
        .await
        .unwrap();
    (g, guest, h)
}

fn append(kind: &'static str) -> app::Decide<'static> {
    Box::new(move |_: &[RawEvent]| Ok(Decision { events: vec![ev(kind)], ..Decision::default() }))
}

#[sqlx::test]
async fn create_then_decide_continues_the_stream(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    let d = s.decide(g, append("PlanSet")).await.unwrap();
    assert_eq!((d.last_seq, d.appended), (2, 1));
    let raw = s.load(g).await.unwrap();
    assert_eq!(raw.iter().map(|r| (r.seq, r.kind.as_str())).collect::<Vec<_>>(), [(1, "GameCreated"), (2, "PlanSet")]);
    assert_eq!(raw[1].payload, json!({ "k": "PlanSet" }));
    assert!(s.load(GameId(Uuid::from_u128(77))).await.unwrap().is_empty());
}

#[sqlx::test]
async fn decide_rejection_writes_nothing(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    let r = s
        .decide(g, Box::new(|_: &[RawEvent]| Err(app::AppError::new(app::ErrorCode::RoundNotOpen))))
        .await;
    assert!(matches!(r, Err(DecideError::Rejected(e)) if e.code == app::ErrorCode::RoundNotOpen));
    assert_eq!(s.load(g).await.unwrap().len(), 1);
}

#[sqlx::test]
async fn concurrent_decides_are_serialized(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    let tasks: Vec<_> = (0..20)
        .map(|_| {
            let s = s.clone();
            tokio::spawn(async move { s.decide(g, append("X")).await.map(|d| d.last_seq).map_err(|e| format!("{e:?}")) })
        })
        .collect();
    for t in tasks {
        t.await.unwrap().expect("zaključavanje igre: nijedan sukob rednog broja");
    }
    let seqs: Vec<i64> = s.load(g).await.unwrap().iter().map(|r| r.seq).collect();
    assert_eq!(seqs, (1..=21).collect::<Vec<i64>>());
}

#[sqlx::test]
async fn decide_enqueues_job_once_per_key(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    for _ in 0..2 {
        s.decide(g, Box::new(move |_: &[RawEvent]| Ok(Decision {
            events: vec![],
            enqueue: Some(NewJob { key: "resolve:x:jun".into(), kind: "resolve_round".into(), game_id: g, payload: json!({"month": "jun"}) }),
            finish_job: None,
        })))
        .await
        .unwrap();
    }
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs").fetch_one(&pool).await.unwrap();
    assert_eq!(n, 1);
}

#[sqlx::test]
async fn guests_and_hotels_lookup(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g1, guest, h1) = game(&s, 1).await;
    assert_eq!(s.guest_by_token_hash("sha256:1".into()).await.unwrap(), Some(guest));
    assert_eq!(s.guest_by_token_hash("sha256:nema".into()).await.unwrap(), None);
    assert_eq!(s.game_of_hotel(h1).await.unwrap(), Some(g1));
    assert_eq!(s.latest_hotel_of(guest).await.unwrap(), Some((g1, h1)));
    let (g2, h2) = (GameId(Uuid::from_u128(5)), HotelId(Uuid::from_u128(6)));
    s.create_game(NewGame { game_id: g2, created_by: guest,
        hotels: vec![NewHotel { hotel_id: h2, slot: 1, guest_id: Some(guest) }], events: vec![ev("GameCreated")] })
        .await
        .unwrap();
    assert_eq!(s.latest_hotel_of(guest).await.unwrap(), Some((g2, h2)), "zadnja igra po games.seq");
}
```

- [ ] **Step 4: Pokreni** `bash scripts/baza.sh status` (ako ne radi: `bash scripts/baza.sh start`), pa
  `set -a; source .env; set +a; cargo sqlx migrate run --source crates/store/migrations; echo EXIT=$?` → `EXIT=0`;
  `cargo test -p store; echo EXIT=$?` → ne prevodi se (`EventLog` nije izveden), `EXIT=101`.

- [ ] **Step 5: `log.rs`.**

```rust
use app::{BoxFut, Decide, DecideError, Decided, EventLog, GameId, NewEvent, NewGame, RawEvent, StoreError};
use sqlx::PgConnection;

use crate::Store;

pub(crate) fn store_err(e: sqlx::Error) -> StoreError {
    StoreError(e.to_string())
}

async fn load_in(conn: &mut PgConnection, stream: &str) -> Result<Vec<RawEvent>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT stream_seq, type AS "kind", schema_ver, payload, actor
           FROM events WHERE stream = $1 ORDER BY stream_seq"#,
        stream
    )
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| RawEvent { seq: i64::from(r.stream_seq), kind: r.kind, schema_ver: r.schema_ver, payload: r.payload, actor: r.actor })
        .collect())
}

async fn insert_events(conn: &mut PgConnection, game: GameId, stream: &str, last: i64, events: &[NewEvent]) -> Result<(), sqlx::Error> {
    for (e, seq) in events.iter().zip(last + 1..) {
        let seq = i32::try_from(seq).map_err(|_| sqlx::Error::Protocol("redni broj izvan INTEGER".into()))?;
        sqlx::query!(
            "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            game.0, stream, seq, e.kind, e.schema_ver, e.payload, e.actor
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

impl EventLog for Store {
    fn create_game(&self, g: NewGame) -> BoxFut<'_, Result<(), StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(store_err)?;
            sqlx::query!("INSERT INTO games (id, created_by) VALUES ($1, $2)", g.game_id.0, g.created_by.0)
                .execute(&mut *tx).await.map_err(store_err)?;
            for h in &g.hotels {
                sqlx::query!("INSERT INTO hotels (id, game_id, slot, guest_id) VALUES ($1, $2, $3, $4)",
                    h.hotel_id.0, g.game_id.0, h.slot, h.guest_id.map(|x| x.0))
                    .execute(&mut *tx).await.map_err(store_err)?;
            }
            insert_events(&mut tx, g.game_id, &g.game_id.stream(), 0, &g.events).await.map_err(store_err)?;
            tx.commit().await.map_err(store_err)
        })
    }

    fn load(&self, game: GameId) -> BoxFut<'_, Result<Vec<RawEvent>, StoreError>> {
        Box::pin(async move {
            let mut conn = self.pool.acquire().await.map_err(store_err)?;
            load_in(&mut conn, &game.stream()).await.map_err(store_err)
        })
    }

    fn decide<'a>(&'a self, game: GameId, decide: Decide<'a>) -> BoxFut<'a, Result<Decided, DecideError>> {
        Box::pin(async move {
            let st = |e: sqlx::Error| DecideError::Store(store_err(e));
            let stream = game.stream();
            let mut tx = self.pool.begin().await.map_err(st)?;
            // Jedna naredba po igri u isto vrijeme (ADR-015 t. 4); UNIQUE(stream, stream_seq) je zadnja brana.
            sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
                .bind(&stream)
                .execute(&mut *tx)
                .await
                .map_err(st)?;
            let raw = load_in(&mut tx, &stream).await.map_err(st)?;
            let last = raw.last().map_or(0, |r| r.seq);
            let d = decide(&raw).map_err(DecideError::Rejected)?; // `tx` se odbacuje = ROLLBACK
            insert_events(&mut tx, game, &stream, last, &d.events).await.map_err(st)?;
            if let Some(j) = &d.enqueue {
                sqlx::query!(
                    "INSERT INTO jobs (job_key, kind, game_id, payload) VALUES ($1, $2, $3, $4) \
                     ON CONFLICT (job_key) DO NOTHING",
                    j.key, j.kind, j.game_id.0, j.payload
                )
                .execute(&mut *tx).await.map_err(st)?;
            }
            if let Some(id) = d.finish_job {
                sqlx::query!("UPDATE jobs SET status = 'done', locked_until = NULL, updated_at = now() WHERE id = $1", id)
                    .execute(&mut *tx).await.map_err(st)?;
            }
            tx.commit().await.map_err(st)?;
            let appended = d.events.len();
            let added = i64::try_from(appended).map_err(|_| DecideError::Store(StoreError("previše događaja".into())))?;
            Ok(Decided { last_seq: last + added, appended })
        })
    }
}
```
  (`pg_advisory_xact_lock` vraća `void`, zato je to funkcija `sqlx::query`, ne makro — makro ne zna tip `void`.)

- [ ] **Step 6: `guests.rs`** — `impl Guests for Store` s `query!` upitima:
  `INSERT INTO guests (id, token_hash) VALUES ($1, $2)` · `SELECT id FROM guests WHERE token_hash = $1` ·
  `SELECT game_id FROM hotels WHERE id = $1` ·
  `SELECT h.game_id, h.id FROM hotels h JOIN games g ON g.id = h.game_id WHERE h.guest_id = $1 ORDER BY g.seq DESC LIMIT 1`
  (`fetch_optional`, rezultat u `GuestId`/`GameId`/`HotelId`). U `lib.rs`: `mod guests; mod log;`, polje `pool`
  ostaje privatno ali vidljivo modulima (`pub(crate)` ako treba), komentar zaglavlja „dnevnik, gosti, red poslova".

- [ ] **Step 7: Offline cache.** `set -a; source .env; set +a; SQLX_OFFLINE=false cargo sqlx prepare --workspace; echo EXIT=$?`
  → `EXIT=0`, nove datoteke u `.sqlx/`.

- [ ] **Step 8: Pokreni** `cargo test -p store; echo EXIT=$?` → `EXIT=0`; zatim provjere iz Global Constraints i
  `cargo sqlx prepare --workspace --check; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 9: Commit** — `Cargo.toml Cargo.lock crates/store .sqlx`, poruka
  „Dnevnik 7/12: store — nepromjenjiv dnevnik, platformske tablice, decide pod zaključavanjem igre".

---

### Task 8: `store` — red poslova (`FOR UPDATE SKIP LOCKED`, zakup, pokušaji)

**Files:**
- Create: `crates/store/src/jobs.rs`; Modify: `crates/store/src/lib.rs` (`mod jobs;`)
- Test: `crates/store/tests/jobs.rs`
- Modify: `.sqlx/`

**Interfaces:**
- Consumes: Task 5 (`app::{JobQueue, Job, MAX_ATTEMPTS}`), Task 7 (tablica `jobs`, `decide` s `enqueue`/`finish_job`).
- Produces: `impl JobQueue for Store`.

- [ ] **Step 1: Testovi (crveni).** `crates/store/tests/jobs.rs` (uvozi i pomoćnike `game` i `ev` kopiraj iz `tests/log.rs`; dodatno `use app::JobQueue;`):

```rust
async fn enqueue(s: &Store, g: GameId, key: &str) {
    let key = key.to_owned();
    s.decide(g, Box::new(move |_: &[RawEvent]| Ok(Decision { events: vec![], finish_job: None,
        enqueue: Some(NewJob { key, kind: "resolve_round".into(), game_id: g, payload: json!({"month": "jun"}) }) })))
        .await
        .unwrap();
}

#[sqlx::test]
async fn empty_queue_gives_none(pool: PgPool) {
    assert!(Store::from_pool(pool).claim(30).await.unwrap().is_none());
}

#[sqlx::test]
async fn claimed_job_is_leased(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let j = s.claim(30).await.unwrap().unwrap();
    assert_eq!((j.key.as_str(), j.attempts, j.game_id), ("a", 1, g));
    assert_eq!(j.payload, json!({"month": "jun"}));
    assert!(s.claim(30).await.unwrap().is_none(), "zakup traje");
}

#[sqlx::test]
async fn concurrent_claims_take_one_job_once(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let tasks: Vec<_> = (0..8).map(|_| { let s = s.clone(); tokio::spawn(async move { s.claim(30).await.unwrap() }) }).collect();
    let mut got = 0;
    for t in tasks {
        if t.await.unwrap().is_some() { got += 1; }
    }
    assert_eq!(got, 1, "dvostruko preuzimanje nije moguće");
}

#[sqlx::test]
async fn expired_lease_is_reclaimed(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let first = s.claim(30).await.unwrap().unwrap(); // radnik „padne" usred posla
    sqlx::query("UPDATE jobs SET locked_until = now() - interval '1 second'").execute(&pool).await.unwrap();
    let again = s.claim(30).await.unwrap().unwrap();
    assert_eq!((again.id, again.attempts), (first.id, 2));
}

#[sqlx::test]
async fn finished_job_is_not_claimed(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let j = s.claim(30).await.unwrap().unwrap();
    s.decide(g, Box::new(move |_: &[RawEvent]| Ok(Decision { events: vec![], enqueue: None, finish_job: Some(j.id) }))).await.unwrap();
    // i kad bi zakup „istekao", gotov posao se više ne preuzima
    sqlx::query("UPDATE jobs SET locked_until = now() - interval '1 second'").execute(&pool).await.unwrap();
    assert!(s.claim(30).await.unwrap().is_none());
}

#[sqlx::test]
async fn failing_job_is_retried_then_failed(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    for i in 1..=app::MAX_ATTEMPTS {
        let j = s.claim(30).await.unwrap().expect("posao se vraća u red");
        assert_eq!(j.attempts, i);
        s.fail(j.id, format!("pokušaj {i}")).await.unwrap();
    }
    assert!(s.claim(30).await.unwrap().is_none());
    let (status, err): (String, Option<String>) =
        sqlx::query_as("SELECT status, last_error FROM jobs").fetch_one(&pool).await.unwrap();
    assert_eq!(status, "failed");
    assert_eq!(err.as_deref(), Some("pokušaj 5"));
}

#[sqlx::test]
async fn expired_last_attempt_becomes_failed(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    sqlx::query("UPDATE jobs SET status = 'running', attempts = $1, locked_until = now() - interval '1 second'")
        .bind(app::MAX_ATTEMPTS).execute(&pool).await.unwrap();
    assert!(s.claim(30).await.unwrap().is_none());
    let status: String = sqlx::query_scalar("SELECT status FROM jobs").fetch_one(&pool).await.unwrap();
    assert_eq!(status, "failed");
}
```
- [ ] **Step 2: Pokreni** `cargo test -p store --test jobs; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 3: `jobs.rs`.**

```rust
use app::{BoxFut, GameId, Job, JobQueue, MAX_ATTEMPTS, StoreError};

use crate::Store;
use crate::log::store_err;

impl JobQueue for Store {
    fn claim(&self, lease_secs: i32) -> BoxFut<'_, Result<Option<Job>, StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(store_err)?;
            // posao kojem je istekao zakup nakon zadnjeg dopuštenog pokušaja ne vraća se u red
            sqlx::query!(
                "UPDATE jobs SET status = 'failed', locked_until = NULL, updated_at = now(), \
                 last_error = COALESCE(last_error, 'istekao zakup nakon zadnjeg pokušaja') \
                 WHERE status = 'running' AND locked_until < now() AND attempts >= $1",
                MAX_ATTEMPTS
            )
            .execute(&mut *tx).await.map_err(store_err)?;
            let row = sqlx::query!(
                r#"UPDATE jobs SET status = 'running', attempts = attempts + 1,
                       locked_until = now() + make_interval(secs => $1), updated_at = now()
                   WHERE id = (
                       SELECT id FROM jobs
                       WHERE (status = 'queued' OR (status = 'running' AND locked_until < now()))
                         AND attempts < $2
                       ORDER BY id
                       FOR UPDATE SKIP LOCKED
                       LIMIT 1
                   )
                   RETURNING id, job_key, kind, game_id, payload, attempts"#,
                f64::from(lease_secs),
                MAX_ATTEMPTS
            )
            .fetch_optional(&mut *tx).await.map_err(store_err)?;
            tx.commit().await.map_err(store_err)?;
            Ok(row.map(|r| Job { id: r.id, key: r.job_key, kind: r.kind, game_id: GameId(r.game_id), payload: r.payload, attempts: r.attempts }))
        })
    }

    fn fail(&self, id: i64, error: String) -> BoxFut<'_, Result<(), StoreError>> {
        Box::pin(async move {
            sqlx::query!(
                "UPDATE jobs SET status = CASE WHEN attempts >= $2 THEN 'failed' ELSE 'queued' END, \
                 locked_until = NULL, last_error = $3, updated_at = now() WHERE id = $1",
                id, MAX_ATTEMPTS, error
            )
            .execute(&self.pool).await.map_err(store_err)?;
            Ok(())
        })
    }
}
```
  Ponovni pokušaj je odmah (bez odgode); odgoda za AI i usporedba s apalis/pgmq su korak 4 (BACKLOG, Task 12).

- [ ] **Step 4: Offline cache i testovi.** `set -a; source .env; set +a; SQLX_OFFLINE=false cargo sqlx prepare --workspace; echo EXIT=$?`
  → `EXIT=0`; `cargo test -p store; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 5: Provjere i commit** — `crates/store .sqlx`, poruka
  „Dnevnik 8/12: store — red poslova sa zakupom i SKIP LOCKED".

---
### Task 9: `api` — temelj (Deps, problem kodovi, kolačić gosta, zamjenska ruta) i rute gosta i hotela

**Files:**
- Modify: `crates/api/Cargo.toml` (makni `sim-hotel`; dodaj `uuid.workspace = true`, `tracing.workspace = true`;
  dev: `content.workspace = true`)
- Modify: `crates/api/src/lib.rs`, `crates/api/src/problem.rs`, `crates/api/src/health.rs`, `crates/api/src/demo.rs`
- Create: `crates/api/src/auth.rs`, `crates/api/src/schema.rs`, `crates/api/src/hotels.rs`
- Modify: `crates/server/Cargo.toml` (`app`, `content`), `crates/server/src/main.rs` (sastavljanje `Deps`, sadržaj pri startu)
- Modify: `crates/api/tests/http.rs`; Create: `crates/api/tests/common/mod.rs`, `crates/api/tests/hotels.rs`
- Modify: `openapi.json`, `packages/api-client/src/schema.d.ts` (generirano)
- Modify: `docs/records/BACKLOG.md` (brišu se riješene stavke `[dnevnik]`: HealthBody `required`, OpenAPI `info`,
  fallback ruta, test OpenAPI-ja za 503/404, `api` zove `sim-hotel` mimo `app`, enum kodovi u OpenAPI-ju)

**Interfaces:**
- Consumes: Task 5 (`app::{Deps, caller, start_demo, my_hotel, hotel_view, profile_indices, AppError, ErrorCode,
  ErrorKind, HotelView, memory}`), `game::view::*` i `game::dto::*` **samo kroz** tipove koje `app` vraća (api dodaje
  ovisnost `game.workspace = true` za te tipove — dopušteno: `game` je čist).
- Produces:
  - `AppState { pub deps: app::Deps }` (`Clone`); `router(AppState) -> Router` sa zamjenskom rutom `unknown_route`.
  - `problem::ProblemCode` — enum u OpenAPI-ju: svih 20 `app::ErrorCode` + `UnknownRoute` + `InvalidBody`
    (snake_case); `Problem { type, title, status, code: ProblemCode }`; `impl From<AppError> for Problem`;
    `Problem::new(StatusCode, ProblemCode)`.
  - `auth::{COOKIE = "hotelixsim_guest", Token(pub Option<String>), set_cookie(&str) -> HeaderValue}`; `Token` je
    `FromRequestParts` (nikad ne odbija; token prihvaća samo ako je točno 32 heksadecimalna znaka).
  - `schema`: `MonthCode`, `IndexCode`, `IndexEntry { code: IndexCode, hundredths: u16 }`, `entries([u16; 5]) -> Vec<IndexEntry>`,
    `TrainingCode`, `MaintenanceCode`, `PlanBody`, `PlanLimitsBody`, `SeatKind`, `StrategyCode`, `SeatBody`,
    `MonthScoreBody`, `PublicHotelBody`, `ProfileCode`, `RealNumbersBody`, `LastMonthBody`, `PlanStateBody`,
    `PrivateHotelBody`, `HotelBody`, `DemoBody`; `parse_game(&str)`, `parse_hotel(&str)`, `parse_month(&str)` →
    `Result<_, Problem>` (neispravno = 404 `unknown_game` / `unknown_hotel` / `unknown_month`).
  - Rute: `POST /v1/demo` (201 + `Set-Cookie` kad je gost nov) · `GET /v1/me/hotel` · `GET /v1/hotels/{hotel_id}` ·
    `GET /v1/demo/profiles/{profile}/indices` (sad preko `app::profile_indices`) · `GET /v1/health`.

- [ ] **Step 1: Pomoćnik testova.** `crates/api/tests/common/mod.rs`:

```rust
#![allow(dead_code)]
use std::sync::Arc;

use api::{AppState, router};
use app::memory::{Memory, deps};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use content::{Snapshot, repo_dir};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

pub struct Resp {
    pub status: StatusCode,
    pub ctype: Option<String>,
    pub set_cookie: Option<String>,
    pub body: Value,
}

pub fn app() -> (Router, app::Deps, Arc<Memory>) {
    let (d, mem) = deps(Snapshot::read(&repo_dir()).unwrap());
    (router(AppState { deps: d.clone() }), d, mem)
}

pub async fn call(app: &Router, method: &str, uri: &str, cookie: Option<&str>, body: Option<Value>) -> Resp {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(t) = cookie {
        req = req.header("cookie", format!("hotelixsim_guest={t}"));
    }
    let req = match body {
        Some(b) => req.header("content-type", "application/json").body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let header = |n: &str| res.headers().get(n).map(|v| v.to_str().unwrap().to_owned());
    let (status, ctype, set_cookie) = (res.status(), header("content-type"), header("set-cookie"));
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    Resp { status, ctype, set_cookie, body }
}

/// Novi gost s igrom: (token, game_id, hotel_id).
pub async fn demo(app: &Router) -> (String, String, String) {
    let r = call(app, "POST", "/v1/demo", None, None).await;
    assert_eq!(r.status, StatusCode::CREATED);
    let c = r.set_cookie.expect("novi gost dobiva kolačić");
    let tok = c.split(';').next().unwrap().strip_prefix("hotelixsim_guest=").unwrap().to_owned();
    (tok, r.body["game_id"].as_str().unwrap().into(), r.body["hotel_id"].as_str().unwrap().into())
}

pub fn robot_hotel(mem: &Memory, game: &str) -> String {
    let g = app::GameId(game.parse().unwrap());
    mem.events(g)[0].payload["hotels"][1]["hotel_id"].as_str().unwrap().to_owned()
}

pub fn assert_problem(r: &Resp, status: u16, code: &str) {
    assert_eq!(r.status.as_u16(), status, "{:?}", r.body);
    assert_eq!(r.ctype.as_deref(), Some("application/problem+json"));
    assert_eq!(r.body["code"], code);
    assert_eq!(r.body["status"], status);
}
```

- [ ] **Step 2: Testovi (crveni).** `crates/api/tests/hotels.rs`:

```rust
mod common;
use common::*;

#[tokio::test]
async fn demo_sets_httponly_cookie_once() {
    let (app, _, _) = app();
    let r = call(&app, "POST", "/v1/demo", None, None).await;
    let c = r.set_cookie.clone().unwrap();
    for part in ["hotelixsim_guest=", "HttpOnly", "SameSite=Lax", "Path=/", "Max-Age=604800"] {
        assert!(c.contains(part), "{c}");
    }
    let tok = c.split(';').next().unwrap().strip_prefix("hotelixsim_guest=").unwrap();
    let again = call(&app, "POST", "/v1/demo", Some(tok), None).await;
    assert_eq!(again.status.as_u16(), 201);
    assert_eq!(again.set_cookie, None, "postojeći gost ne dobiva novi kolačić");
    assert_ne!(again.body["game_id"], r.body["game_id"]);
}

#[tokio::test]
async fn me_hotel_shows_private_numbers_and_june_plan() {
    let (app, _, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    let r = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    assert_eq!(r.status.as_u16(), 200);
    assert_eq!(r.body["game_id"], game.as_str());
    assert_eq!(r.body["hotel"]["hotel_id"], hotel.as_str());
    assert_eq!(r.body["hotel"]["seat"]["kind"], "human");
    assert!(r.body["private"]["now"]["cash_cents"].is_i64());
    assert_eq!(r.body["private"]["plan"]["month"], "jun");
    assert_eq!(r.body["hotel"]["start"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn foreign_and_anonymous_requests() {
    let (app, _, mem) = app();
    let (tok, game, hotel) = demo(&app).await;
    let (other, _, _) = demo(&app).await;
    assert_problem(&call(&app, "GET", "/v1/me/hotel", None, None).await, 401, "guest_required");
    assert_problem(&call(&app, "GET", "/v1/me/hotel", Some("nije-token"), None).await, 401, "guest_required");
    let uri = format!("/v1/hotels/{hotel}");
    assert_problem(&call(&app, "GET", &uri, Some(&other), None).await, 403, "not_in_game");
    let robot = call(&app, "GET", &format!("/v1/hotels/{}", robot_hotel(&mem, &game)), Some(&tok), None).await;
    assert_eq!(robot.status.as_u16(), 200);
    assert!(robot.body["private"].is_null(), "tuđi hotel bez stvarnih brojki");
    assert!(robot.body["hotel"]["start"].is_array());
}

#[tokio::test]
async fn unknown_ids_and_routes_are_problem_404() {
    let (app, _, _) = app();
    let (tok, _, _) = demo(&app).await;
    let nobody = "00000000-0000-0000-0000-00000000abcd";
    assert_problem(&call(&app, "GET", &format!("/v1/hotels/{nobody}"), Some(&tok), None).await, 404, "unknown_hotel");
    assert_problem(&call(&app, "GET", "/v1/hotels/nije-uuid", Some(&tok), None).await, 404, "unknown_hotel");
    assert_problem(&call(&app, "GET", "/v1/nema-me", None, None).await, 404, "unknown_route");
}
```
  `tests/http.rs`: zamijeni lažni `Fake` sustavom `common::app()` uz zamjenu zdravlja
  (`let (_, mut d, _) = app(); d.status = Arc::new(Fake(Err(..))); router(AppState { deps: d })`); postojeći testovi
  zdravlja i profila ostaju istog sadržaja. Test OpenAPI-ja proširi:

```rust
#[test]
fn openapi_documents_contract() {
    let doc: Value = serde_json::from_str(&openapi_json()).unwrap();
    assert_eq!(doc["openapi"], "3.1.0", "TS generator čita samo 3.0 i 3.1");
    assert!(doc["info"]["description"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(doc["info"]["license"]["name"].is_string());
    for (path, method) in [("/v1/health", "get"), ("/v1/demo", "post"), ("/v1/me/hotel", "get"),
                           ("/v1/hotels/{hotel_id}", "get"), ("/v1/demo/profiles/{profile}/indices", "get")] {
        assert!(doc["paths"][path][method].is_object(), "{method} {path}");
    }
    assert!(doc["paths"]["/v1/health"]["get"]["responses"]["503"].is_object());
    let not_found = &doc["paths"]["/v1/hotels/{hotel_id}"]["get"]["responses"]["404"]["content"];
    assert!(not_found["application/problem+json"].is_object());
    let required = doc["components"]["schemas"]["HealthBody"]["required"].as_array().unwrap();
    assert!(required.contains(&json!("events")), "events je obavezan (može biti null)");
    let codes = doc["components"]["schemas"]["ProblemCode"]["enum"].as_array().unwrap();
    for c in ["unknown_route", "invalid_body", "guest_required", "round_not_open", "price_not_whole_euro"] {
        assert!(codes.contains(&json!(c)), "{c}");
    }
    assert_eq!(codes.len(), 22);
    assert!(doc["components"]["schemas"]["IndexCode"]["enum"].is_array());
}
```

- [ ] **Step 3: Pokreni** `cargo test -p api; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 4: `problem.rs`.**

```rust
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

/// Stabilni kodovi grešaka; ekran ih prevodi (ARHITEKTURA §11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProblemCode {
    GuestRequired, NotInGame, NotYourHotel, UnknownGame, UnknownHotel, UnknownMonth, UnknownSituation,
    UnknownAnswer, UnknownProfile, RoundNotOpen, AlreadyAnswered, PriceNotWholeEuro, PriceOutOfRange,
    MarketingOutOfRange, SeasonalOutOfRange, SeasonalStepTooBig, WageOutOfRange, RenovationTwice,
    StorageUnavailable, Internal,
    /// Samo HTTP sloj: ruta ne postoji.
    UnknownRoute,
    /// Samo HTTP sloj: tijelo zahtjeva nije ispravan JSON traženog oblika.
    InvalidBody,
}

impl From<app::ErrorCode> for ProblemCode {
    fn from(c: app::ErrorCode) -> Self {
        use app::ErrorCode as E;
        match c { E::GuestRequired => Self::GuestRequired, /* … iscrpno, svih 20, bez `_` … */ E::Internal => Self::Internal }
    }
}

impl ProblemCode {
    /// Naslov za čovjeka (hrvatski); sučelje ionako prevodi po kodu.
    pub fn title(self) -> &'static str {
        match self {
            Self::GuestRequired => "Potreban je gost: otvori demo poveznicu",
            Self::NotInGame => "Nisi u ovoj igri",
            Self::NotYourHotel => "To nije tvoj hotel",
            Self::UnknownGame => "Igra ne postoji",
            Self::UnknownHotel => "Hotel ne postoji",
            Self::UnknownMonth => "Mjesec nije u ovoj igri",
            Self::UnknownSituation => "Situacija ne postoji",
            Self::UnknownAnswer => "Taj odgovor nije ponuđen",
            Self::UnknownProfile => "Nepoznat profil",
            Self::RoundNotOpen => "Runda nije otvorena",
            Self::AlreadyAnswered => "Na situaciju je već odgovoreno",
            Self::PriceNotWholeEuro => "Cijena mora biti cijeli euro",
            Self::PriceOutOfRange => "Cijena je izvan dopuštenog raspona",
            Self::MarketingOutOfRange => "Marketing je izvan dopuštenog raspona",
            Self::SeasonalOutOfRange => "Broj sezonaca je izvan dopuštenog raspona",
            Self::SeasonalStepTooBig => "Prevelika promjena broja sezonaca u jednom mjesecu",
            Self::WageOutOfRange => "Plaća je izvan dopuštenog raspona",
            Self::RenovationTwice => "Obnova je moguća samo jednom u sezoni",
            Self::StorageUnavailable => "Baza nije dostupna",
            Self::Internal => "Greška sustava",
            Self::UnknownRoute => "Nepoznata ruta",
            Self::InvalidBody => "Neispravno tijelo zahtjeva",
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct Problem {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub title: &'static str,
    pub status: u16,
    pub code: ProblemCode,
}

impl Problem {
    pub fn new(status: StatusCode, code: ProblemCode) -> Self {
        Problem { kind: "about:blank", title: code.title(), status: status.as_u16(), code }
    }
}

impl From<app::AppError> for Problem {
    fn from(e: app::AppError) -> Self {
        use app::ErrorKind as K;
        let status = match e.code.kind() {
            K::Unauthorized => StatusCode::UNAUTHORIZED,
            K::Forbidden => StatusCode::FORBIDDEN,
            K::NotFound => StatusCode::NOT_FOUND,
            K::Conflict => StatusCode::CONFLICT,
            K::Invalid => StatusCode::UNPROCESSABLE_ENTITY,
            K::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            K::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if status.is_server_error() {
            tracing::error!(code = ?e.code, detail = %e.detail, "greška poslužitelja");
        }
        Problem::new(status, e.code.into())
    }
}
```
  `IntoResponse for Problem` ostaje kakav jest (`application/problem+json`).

- [ ] **Step 5: `auth.rs`.**

```rust
use std::convert::Infallible;

use axum::extract::FromRequestParts;
use axum::http::{HeaderMap, HeaderValue, header, request::Parts};

pub const COOKIE: &str = "hotelixsim_guest";

/// Token gosta iz kolačića; pokvaren ili nepostojeći = `None` (anoniman pozivatelj, odluku donosi `app`).
pub struct Token(pub Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for Token {
    type Rejection = Infallible;
    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Token(read(&parts.headers)))
    }
}

fn read(h: &HeaderMap) -> Option<String> {
    h.get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(k, _)| *k == COOKIE)
        .map(|(_, v)| v.to_owned())
        .filter(|v| v.len() == 32 && v.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// `Secure` dolazi s HTTPS-om prije Railwaya (BACKLOG); lokalno je http.
pub fn set_cookie(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!("{COOKIE}={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age=604800"))
        .expect("token je heksadecimalan")
}
```

- [ ] **Step 6: `schema.rs`.** Svi tipovi `#[derive(Serialize, ToSchema)]` (+ `Deserialize` za `PlanBody` i enume koje
  tijelo zahtjeva nosi; `PlanBody` uz to `#[serde(deny_unknown_fields)]`). **Svako `Option` polje odgovora nosi
  `#[schema(required = true)]`** (TS tip `T | null`, ne `T?`). Enumi: `MonthCode` (`lowercase`: jun, jul, aug, sep),
  `IndexCode` (G, R, Z, O, F), `TrainingCode` i `MaintenanceCode` (`snake_case`), `SeatKind` (human, robot),
  `StrategyCode` (`lowercase`: stedisa, brizni, lovac, nasumicni), `ProfileCode` (A, B, C). Strukture zrcale
  `game::view` polje za polje, s ovim razlikama: identifikatori su `String` (uuid u tekstu),
  indeksi su `Vec<IndexEntry>` (`entries` po `game::dto::INDEX_ORDER`), `PublicHotelBody.seat: SeatBody { kind,
  strategy: Option<StrategyCode> }`. `HotelBody { game_id: String, hotel: PublicHotelBody, private: Option<PrivateHotelBody> }`,
  `DemoBody { game_id: String, hotel_id: String }`. Pretvorbe `From<&game::view::X>`/`From<&app::HotelView>` i
  `From<&PlanBody> for game::dto::PlanDto` žive ovdje. `parse_*`:

```rust
pub fn parse_hotel(s: &str) -> Result<app::HotelId, Problem> {
    s.parse().map(app::HotelId).map_err(|_| Problem::new(StatusCode::NOT_FOUND, ProblemCode::UnknownHotel))
}
// parse_game isto s UnknownGame; parse_month: "jun"|"jul"|"aug"|"sep" → game::dto::MonthCode, inače UnknownMonth.
```

- [ ] **Step 7: Rute.** `health.rs`: `#[schema(required = true)] pub events: Option<i64>` i `state.deps.status`.
  `demo.rs`: `profile_indices` zove `app::profile_indices` i vraća `IndicesBody { profile: String, indices: Vec<IndexEntry> }`
  (`IndexEntry.code` je sad `IndexCode`; JSON isti, test profila C ostaje nepromijenjen); nova ruta:

```rust
/// Novi gost (ako nema kolačić) i nova igra s tri robota (DEMO §1.4).
#[utoipa::path(
    post,
    path = "/v1/demo",
    responses(
        (status = 201, description = "Igra stvorena", body = DemoBody,
         headers(("set-cookie" = String, description = "hotelixsim_guest (HttpOnly) — samo za novog gosta"))),
        (status = 503, description = "Baza nije dostupna", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn start(State(s): State<AppState>, Token(t): Token) -> Result<Response, Problem> {
    let d = app::start_demo(&s.deps, t.as_deref()).await?;
    let body = DemoBody { game_id: d.game_id.0.to_string(), hotel_id: d.hotel_id.0.to_string() };
    let mut res = (StatusCode::CREATED, Json(body)).into_response();
    if let Some(tok) = d.new_token {
        res.headers_mut().insert(header::SET_COOKIE, set_cookie(&tok));
    }
    Ok(res)
}
```
  `hotels.rs`: `GET /v1/me/hotel` (`app::my_hotel`) i `GET /v1/hotels/{hotel_id}` (`app::hotel_view`), oba vraćaju
  `HotelBody`; u `responses` navedi 200, 401, 403, 404 (problem+json). Obrazac rukovatelja:

```rust
pub async fn hotel(State(s): State<AppState>, Token(t): Token, Path(id): Path<String>) -> Result<Json<HotelBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let v = app::hotel_view(&s.deps, &caller, parse_hotel(&id)?).await?;
    Ok(Json(HotelBody::from(&v)))
}
```
  `lib.rs`:

```rust
#[derive(Clone)]
pub struct AppState {
    pub deps: app::Deps,
}

#[derive(OpenApi)]
#[openapi(info(
    title = "HotelixSim API",
    version = "0.1.0",
    description = "Simulacija vođenja hotela za učenje (FMTU Opatija). Igra je dnevnik događaja; ovaj API ga čita i piše.",
    license(name = "Vlasnički — sva prava pridržana (Leon Kreso)")
))]
struct ApiDoc;

fn parts() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health::health))
        .routes(routes!(demo::profile_indices))
        .routes(routes!(demo::start))
        .routes(routes!(hotels::me_hotel))
        .routes(routes!(hotels::hotel))
}

async fn unknown_route() -> Problem {
    Problem::new(StatusCode::NOT_FOUND, ProblemCode::UnknownRoute)
}

pub fn router(state: AppState) -> Router {
    let (router, _) = parts().split_for_parts();
    router.fallback(unknown_route).with_state(state)
}
```
  Ako `ProblemCode`/`IndexCode` ne uđu u `components.schemas` sami (utoipa ih skuplja kroz `Problem`/`IndexEntry`),
  dodaj `components(schemas(ProblemCode, IndexCode))` u `#[openapi(...)]`.

- [ ] **Step 8: Server sastavlja `Deps`.** `crates/server/Cargo.toml`: `app.workspace = true`, `content.workspace = true`.
  U `main.rs`:

```rust
/// Sadržaj se učitava i provjerava pri startu: server s pokvarenim katalogom ne smije ni krenuti (ADR-015 t. 12).
fn content_snapshot() -> anyhow::Result<content::Snapshot> {
    let dir = std::env::var("HOTELIXSIM_CONTENT").map(std::path::PathBuf::from).unwrap_or_else(|_| content::repo_dir());
    let snap = content::Snapshot::read(&dir).map_err(|e| anyhow::anyhow!("sadržaj u {}: {e:?}", dir.display()))?;
    snap.build().map_err(|e| anyhow::anyhow!("sadržaj u {} nije ispravan: {e:?}", dir.display()))?;
    Ok(snap)
}

fn deps(store: store::Store, snapshot: content::Snapshot) -> app::Deps {
    let s = Arc::new(store);
    app::Deps {
        status: s.clone(),
        log: s.clone(),
        guests: s.clone(),
        jobs: s,
        ids: Arc::new(app::RandomIds),
        snapshot: Arc::new(snapshot),
    }
}
```
  `serve()`: `let snapshot = content_snapshot()?;` **prije** `connect_lazy`, zatim
  `api::router(api::AppState { deps: deps(store, snapshot) })`; u log upiši `tracing::info!(ruleset = %snapshot.fingerprint(), …)`
  (izračunaj prije premještanja). Migracije se pri startu **ne** pokreću (naredba `migrate` ostaje zasebna — ADR-015 t. 12).

- [ ] **Step 9: Pokreni** `cargo test -p api; echo EXIT=$?` → `EXIT=0`; `corepack pnpm gen:api; echo EXIT=$?` → `EXIT=0`;
  `corepack pnpm typecheck; echo EXIT=$?` i `corepack pnpm test; echo EXIT=$?` → `EXIT=0` (sučelje čita `code` kao
  string, unija literala mu je dodjeljiva); provjere iz Global Constraints.

- [ ] **Step 10: BACKLOG.** Obriši šest riješenih stavki navedenih u **Files** (povijest je u gitu).

- [ ] **Step 11: Commit** — `crates/api crates/server Cargo.lock openapi.json packages/api-client/src/schema.d.ts
  docs/records/BACKLOG.md`, poruka „Dnevnik 9/12: api — Deps, problem kodovi kao enum, kolačić gosta, rute gosta i hotela".

---

### Task 10: `api` — rute igre: plan, situacije, runda, zatvaranje i feed

**Files:**
- Create: `crates/api/src/games.rs`; Modify: `crates/api/src/hotels.rs`, `crates/api/src/schema.rs`, `crates/api/src/lib.rs`
- Test: `crates/api/tests/games.rs`
- Modify: `openapi.json`, `packages/api-client/src/schema.d.ts`

**Interfaces:**
- Consumes: Task 6 (`app::{set_plan, answer, round_status, close_round, CloseOutcome, game_view, situations, feed,
  run_next_job}`), Task 9 (`schema`, `Problem`, `Token`, `parse_*`).
- Produces (rute; imena po ARHITEKTURA §11, mjesec u putanji radi prirodne idempotentnosti — ADR-015 t. 10/11):

| metoda i putanja | tijelo → odgovor | greške |
|---|---|---|
| `GET /v1/games/{game_id}` | → `GameBody { game_id, months, current: Option<RoundBody>, my_hotel_id, hotels: Vec<PublicHotelBody> }` | 401, 403, 404 |
| `GET /v1/games/{game_id}/rounds/{month}` | → `RoundBody { month, status: RoundStatusCode, unanswered: u8 }` | 401, 403, 404 |
| `POST /v1/games/{game_id}/rounds/{month}/close` | → **202** `RoundBody` (zatvoreno sad) · **200** `RoundBody` (već zatvoreno/obračunato) | 401, 403, 404, 409 |
| `PUT /v1/hotels/{hotel_id}/plans/{month}` | `PlanBody` → 200 `PlanBody` | 401, 403, 404, 409, 422 |
| `GET /v1/hotels/{hotel_id}/situations` | → `SituationsBody { month: Option<MonthCode>, open: bool, items: Vec<SituationBody { code, title, answers: Vec<String>, chosen: Option<u8> }> }` | 401, 403, 404 |
| `POST /v1/hotels/{hotel_id}/situations/{code}/answer` | `AnswerBody { answer: u8 }` → 200 `SituationsBody` | 401, 403, 404, 409, 422 |
| `GET /v1/hotels/{hotel_id}/feed` | → `FeedBody { items: Vec<FeedItemBody> }` | 401, 403, 404 |

  - `RoundStatusCode` (`snake_case`): `scheduled`, `open`, `closing`, `resolved`.
  - `FeedItemBody` — `#[serde(tag = "kind", rename_all = "snake_case")]` enum (u OpenAPI-ju `oneOf` s diskriminatorom):
    `HotelReceived { seq, name, profile: ProfileCode }` · `PlanChanged { seq, month, plan: PlanBody, by: ActorKind }` ·
    `AnswerChosen { seq, month, situation, title, answer: u8, label, by: ActorKind }` · `RoundClosed { seq, month, by }` ·
    `SituationIgnored { seq, month, situation, title }` · `MonthResolved { seq, month, indices: Vec<IndexEntry>, progress_hundredths: i32 }` ·
    `Change { seq, month, var: VarCode, cause: CauseCode, situation: Option<String>, amount: i64, unit: UnitCode }`.
    `ActorKind` (guest, robot, system), `VarCode`/`UnitCode` zrcala, `CauseCode` = zrcalo `CauseDto` bez podatka, a
    šifra situacije ide u `situation`.
  - Neispravno tijelo (loš JSON, nepoznato polje, nepoznata vrijednost enuma) → 422 `invalid_body` (rukovatelj prima
    `Result<Json<T>, JsonRejection>`).

- [ ] **Step 1: Testovi (crveni).** `crates/api/tests/games.rs`:

```rust
mod common;
use common::*;
use serde_json::json;

async fn run_worker(d: &app::Deps) {
    while app::run_next_job(d).await.unwrap() {}
}

#[tokio::test]
async fn game_view_lists_four_hotels_and_open_june() {
    let (app, _, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    let r = call(&app, "GET", &format!("/v1/games/{game}"), Some(&tok), None).await;
    assert_eq!(r.status.as_u16(), 200);
    assert_eq!(r.body["months"], json!(["jun", "jul"]));
    assert_eq!(r.body["current"], json!({"month": "jun", "status": "open", "unanswered": 2}));
    assert_eq!(r.body["my_hotel_id"], hotel.as_str());
    assert_eq!(r.body["hotels"].as_array().unwrap().len(), 4);
    assert!(r.body["hotels"].as_array().unwrap().iter().all(|h| h.get("private").is_none()));
}

#[tokio::test]
async fn plan_put_validates_and_is_visible() {
    let (app, _, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    let me = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    let mut plan = me.body["private"]["plan"]["plan"].clone();
    plan["marketing_cents"] = json!(500_000);
    plan["training"] = json!("basic");
    let uri = format!("/v1/hotels/{hotel}/plans/jun");
    let r = call(&app, "PUT", &uri, Some(&tok), Some(plan.clone())).await;
    assert_eq!((r.status.as_u16(), &r.body), (200, &plan));
    let me = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    assert_eq!(me.body["private"]["plan"]["plan"], plan, "zadnja promjena vrijedi");
    let mut bad = plan.clone();
    bad["price_cents"] = json!(plan["price_cents"].as_i64().unwrap() + 50);
    assert_problem(&call(&app, "PUT", &uri, Some(&tok), Some(bad)).await, 422, "price_not_whole_euro");
    assert_problem(&call(&app, "PUT", &format!("/v1/hotels/{hotel}/plans/aug"), Some(&tok), Some(plan.clone())).await, 404, "unknown_month");
    assert_problem(&call(&app, "PUT", &format!("/v1/hotels/{hotel}/plans/xyz"), Some(&tok), Some(plan)).await, 404, "unknown_month");
}

#[tokio::test]
async fn bad_plan_body_is_422_problem() {
    let (app, _, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    let uri = format!("/v1/hotels/{hotel}/plans/jun");
    let me = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    let mut plan = me.body["private"]["plan"]["plan"].clone();
    plan["training"] = json!("mega");
    assert_problem(&call(&app, "PUT", &uri, Some(&tok), Some(plan)).await, 422, "invalid_body");
    assert_problem(&call(&app, "PUT", &uri, Some(&tok), Some(json!({"price_cents": 1}))).await, 422, "invalid_body");
}

#[tokio::test]
async fn situations_hide_quality_and_answer_once() {
    let (app, _, mem) = app();
    let (tok, game, hotel) = demo(&app).await;
    let uri = format!("/v1/hotels/{hotel}/situations");
    let r = call(&app, "GET", &uri, Some(&tok), None).await;
    assert_eq!(r.body["month"], "jun");
    assert_eq!(r.body["items"].as_array().unwrap().len(), 2);
    assert!(!r.body.to_string().contains("quality") && !r.body.to_string().contains("mechanics"));
    let a = call(&app, "POST", &format!("{uri}/S1/answer"), Some(&tok), Some(json!({"answer": 0}))).await;
    assert_eq!(a.body["items"][0]["chosen"], 0);
    assert_problem(&call(&app, "POST", &format!("{uri}/S1/answer"), Some(&tok), Some(json!({"answer": 1}))).await, 409, "already_answered");
    assert_problem(&call(&app, "POST", &format!("{uri}/S3/answer"), Some(&tok), Some(json!({"answer": 9}))).await, 422, "unknown_answer");
    assert_problem(&call(&app, "POST", &format!("{uri}/S9/answer"), Some(&tok), Some(json!({"answer": 0}))).await, 404, "unknown_situation");
    let robot = robot_hotel(&mem, &game);
    assert_problem(&call(&app, "GET", &format!("/v1/hotels/{robot}/situations"), Some(&tok), None).await, 403, "not_your_hotel");
}

#[tokio::test]
async fn close_twice_is_202_then_200_and_worker_resolves() {
    let (app, d, _) = app();
    let (tok, game, _) = demo(&app).await;
    let round = format!("/v1/games/{game}/rounds/jun");
    assert_eq!(call(&app, "GET", &round, Some(&tok), None).await.body["unanswered"], 2);
    let first = call(&app, "POST", &format!("{round}/close"), Some(&tok), None).await;
    assert_eq!((first.status.as_u16(), &first.body["status"]), (202, &json!("closing")));
    let second = call(&app, "POST", &format!("{round}/close"), Some(&tok), None).await;
    assert_eq!((second.status.as_u16(), &second.body["status"]), (200, &json!("closing")));
    run_worker(&d).await;
    assert_eq!(call(&app, "GET", &round, Some(&tok), None).await.body["status"], "resolved");
    let g = call(&app, "GET", &format!("/v1/games/{game}"), Some(&tok), None).await;
    assert_eq!(g.body["current"]["month"], "jul");
    assert_problem(&call(&app, "POST", &format!("/v1/games/{game}/rounds/aug/close"), Some(&tok), None).await, 404, "unknown_month");
}

#[tokio::test]
async fn plan_for_closed_month_is_409() {
    let (app, _, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    let plan = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await.body["private"]["plan"]["plan"].clone();
    call(&app, "POST", &format!("/v1/games/{game}/rounds/jun/close"), Some(&tok), None).await;
    assert_problem(&call(&app, "PUT", &format!("/v1/hotels/{hotel}/plans/jun"), Some(&tok), Some(plan)).await, 409, "round_not_open");
    assert_problem(&call(&app, "POST", &format!("/v1/hotels/{hotel}/situations/S1/answer"), Some(&tok), Some(json!({"answer": 0}))).await, 409, "round_not_open");
}

#[tokio::test]
async fn feed_is_owner_only_and_tells_the_month() {
    let (app, d, mem) = app();
    let (tok, game, hotel) = demo(&app).await;
    call(&app, "POST", &format!("/v1/hotels/{hotel}/situations/S1/answer"), Some(&tok), Some(json!({"answer": 0}))).await;
    call(&app, "POST", &format!("/v1/games/{game}/rounds/jun/close"), Some(&tok), None).await;
    run_worker(&d).await;
    let f = call(&app, "GET", &format!("/v1/hotels/{hotel}/feed"), Some(&tok), None).await;
    let kinds: Vec<&str> = f.body["items"].as_array().unwrap().iter().map(|i| i["kind"].as_str().unwrap()).collect();
    assert_eq!(kinds[0], "hotel_received");
    for k in ["answer_chosen", "round_closed", "situation_ignored", "month_resolved", "change"] {
        assert!(kinds.contains(&k), "{k}");
    }
    let robot = robot_hotel(&mem, &game);
    assert_problem(&call(&app, "GET", &format!("/v1/hotels/{robot}/feed"), Some(&tok), None).await, 403, "not_your_hotel");
}
```
  Proširi `openapi_documents_contract` u `tests/http.rs` putanjama iz tablice i provjerom
  `doc["components"]["schemas"]["FeedItemBody"]["oneOf"].is_array()` i `RoundStatusCode.enum == ["scheduled","open","closing","resolved"]`.

- [ ] **Step 2: Pokreni** `cargo test -p api --test games; echo EXIT=$?` → `EXIT=101` (rute ne postoje: 404 `unknown_route`
  ili ne prevodi se).

- [ ] **Step 3: Rukovatelji.** Obrazac za naredbu s tijelom (`hotels.rs`):

```rust
#[utoipa::path(
    put,
    path = "/v1/hotels/{hotel_id}/plans/{month}",
    params(("hotel_id" = String, Path, description = "Hotel (uuid)"), ("month" = MonthCode, Path, description = "Mjesec runde")),
    request_body = PlanBody,
    responses(
        (status = 200, description = "Plan je zapisan; zadnji vrijedi", body = PlanBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Runda nije otvorena", body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "Plan izvan granica ili neispravno tijelo", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn put_plan(
    State(s): State<AppState>,
    Token(t): Token,
    Path((id, month)): Path<(String, String)>,
    body: Result<Json<PlanBody>, JsonRejection>,
) -> Result<Json<PlanBody>, Problem> {
    let Json(plan) = body.map_err(|_| Problem::new(StatusCode::UNPROCESSABLE_ENTITY, ProblemCode::InvalidBody))?;
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let saved = app::set_plan(&s.deps, &caller, parse_hotel(&id)?, parse_month(&month)?, (&plan).into()).await?;
    Ok(Json(PlanBody::from(&saved)))
}
```
  Zatvaranje (`games.rs`): `app::close_round` → `if out.accepted { StatusCode::ACCEPTED } else { StatusCode::OK }` uz
  `Json(RoundBody::from(&out.round))`. Ostali rukovatelji istim obrascem (token → `caller` → parsiranje putanje →
  slučaj upotrebe → `From`). Registriraj sve rute u `parts()`.

- [ ] **Step 4: Pokreni** `cargo test -p api; echo EXIT=$?` → `EXIT=0`; `corepack pnpm gen:api; echo EXIT=$?`;
  `corepack pnpm typecheck; echo EXIT=$?`; `corepack pnpm test; echo EXIT=$?` → svi `EXIT=0`; provjere iz Global Constraints.

- [ ] **Step 5: Commit** — `crates/api openapi.json packages/api-client/src/schema.d.ts`, poruka
  „Dnevnik 10/12: api — plan, situacije, runda, zatvaranje (202) i feed hotela".

---

### Task 11: `server` — radnik u procesu, brana nepraćenih generiranih datoteka, upute

**Files:**
- Modify: `crates/server/src/main.rs`, `Cargo.toml` (workspace tokio značajka `time`)
- Create: `scripts/check-clean.mjs`; Modify: `package.json` (`check:api`)
- Modify: `README.md` („Lokalni razvoj": sadržaj, radnik), `.env.example` (`HOTELIXSIM_CONTENT` zakomentiran)
- Modify: `docs/records/BACKLOG.md` (briše se stavka „`check:api` ne hvata nepraćenu generiranu datoteku")

**Interfaces:**
- Consumes: Task 6 (`app::run_next_job`), Task 9 (`deps`, `content_snapshot`).
- Produces: `serve` pokreće radnika kao tokio zadatak; `node scripts/check-clean.mjs <putanje…>` → izlaz 1 ako
  `git status --porcelain -- <putanje>` nije prazan.

- [ ] **Step 1: Radnik.** Workspace `tokio` značajke + `"time"`. U `main.rs`:

```rust
/// Radnik reda poslova u istom procesu (demo; zaseban način `work` je za pilot). Pad procesa ne gubi posao:
/// zakup istekne i posao se preuzme ponovno (ARHITEKTURA §9.4).
fn spawn_worker(deps: app::Deps) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match app::run_next_job(&deps).await {
                Ok(true) => {}
                Ok(false) => tokio::time::sleep(std::time::Duration::from_millis(250)).await,
                Err(e) => {
                    tracing::warn!(error = %e, "posao reda nije uspio");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    })
}
```
  U `serve()`: `let deps = deps(store, snapshot); let worker = spawn_worker(deps.clone());` prije `axum::serve`, a
  nakon gašenja `worker.abort();`.

- [ ] **Step 2: Brana.** `scripts/check-clean.mjs`:

```js
// check:api: generirane datoteke moraju biti praćene i nepromijenjene. `git diff --exit-code` ne vidi
// NOVU nepraćenu datoteku (BACKLOG [dnevnik], Task 6 #2 kostura); `git status --porcelain` je vidi.
import { execFileSync } from "node:child_process";

const paths = process.argv.slice(2);
if (paths.length === 0) {
  console.error("upotreba: node scripts/check-clean.mjs <putanja>…");
  process.exit(2);
}
const out = execFileSync("git", ["status", "--porcelain", "--", ...paths], { encoding: "utf8" });
if (out.trim() !== "") {
  console.error(`generirane datoteke nisu commitane:\n${out}`);
  process.exit(1);
}
console.log(`čisto: ${paths.join(", ")}`);
```
  `package.json`: `"check:api": "node scripts/pnpm.mjs gen:api && node scripts/check-clean.mjs openapi.json packages/api-client/src"`.
  Proba: `touch packages/api-client/src/proba.ts; node scripts/check-clean.mjs openapi.json packages/api-client/src; echo EXIT=$?`
  → ispis `?? packages/api-client/src/proba.ts`, `EXIT=1`; `rm packages/api-client/src/proba.ts`.

- [ ] **Step 3: Upute.** README „Lokalni razvoj", korak 3 dopuni: „`serve` pri startu učitava i provjerava
  `content/hotel` (ili mapu iz `HOTELIXSIM_CONTENT`) i pokreće radnika koji obračunava zatvorene mjesece; migracije se
  ne pokreću same — uvijek `migrate` prije `serve`." `.env.example`: `# HOTELIXSIM_CONTENT=content/hotel`.

- [ ] **Step 4: Pokreni** `cargo build -p server; echo EXIT=$?` → `EXIT=0`; `corepack pnpm check:api; echo EXIT=$?` →
  `čisto: …`, `EXIT=0` (nakon commita Taska 10); provjere iz Global Constraints.

- [ ] **Step 5: Commit** — `crates/server Cargo.toml Cargo.lock scripts/check-clean.mjs package.json README.md
  .env.example docs/records/BACKLOG.md`, poruka „Dnevnik 11/12: radnik u serveru, check:api vidi nepraćene datoteke".

---

### Task 12: Kraj do kraja na pravoj bazi, regeneracija i sve provjere CI-ja

**Files:**
- Create: `crates/api/tests/e2e.rs`; Modify: `crates/api/Cargo.toml` (dev: `sqlx.workspace = true`, `store.workspace = true`)
- Modify: `docs/architecture/ARHITEKTURA.md` (§6.1 „Postoje:", §6.2 dijagram, §9.1 `actor`, §9.4 izbor izvedbe)
- Modify: `docs/records/BACKLOG.md` (obriši preostale riješene `[dnevnik]` stavke; dodaj odgođeno — Step 6)
- Modify: `.sqlx/`, `openapi.json`, `packages/api-client/src/schema.d.ts` (ako se promijene)

**Interfaces:**
- Consumes: sve prethodno; `store::Store::from_pool`, `app::RandomIds`, `game::game::Game`.
- Produces: dokaz da cijeli put radi nad Postgresom; nijedan novi javni tip.

- [ ] **Step 1: Test kraja do kraja.** `crates/api/tests/e2e.rs`:

```rust
mod common;
use std::sync::Arc;

use api::{AppState, router};
use common::{assert_problem, call};
use content::{Snapshot, repo_dir};
use serde_json::json;
use sqlx::PgPool;

fn deps(pool: PgPool) -> app::Deps {
    let s = Arc::new(store::Store::from_pool(pool));
    app::Deps { status: s.clone(), log: s.clone(), guests: s.clone(), jobs: s,
                ids: Arc::new(app::RandomIds), snapshot: Arc::new(Snapshot::read(&repo_dir()).unwrap()) }
}

async fn run_worker(d: &app::Deps) {
    while app::run_next_job(d).await.unwrap() {}
}

#[sqlx::test(migrations = "../store/migrations")]
async fn guest_plays_june_and_july_against_three_robots(pool: PgPool) {
    let d = deps(pool.clone());
    let app = router(AppState { deps: d.clone() });
    // gost → igra
    let r = call(&app, "POST", "/v1/demo", None, None).await;
    let tok = r.set_cookie.unwrap().split(';').next().unwrap().strip_prefix("hotelixsim_guest=").unwrap().to_owned();
    let (game, hotel) = (r.body["game_id"].as_str().unwrap().to_owned(), r.body["hotel_id"].as_str().unwrap().to_owned());
    // lipanj: plan + jedan odgovor, S3 ostaje neodgovorena
    let me = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    let mut plan = me.body["private"]["plan"]["plan"].clone();
    plan["marketing_cents"] = json!(500_000);
    assert_eq!(call(&app, "PUT", &format!("/v1/hotels/{hotel}/plans/jun"), Some(&tok), Some(plan)).await.status.as_u16(), 200);
    call(&app, "POST", &format!("/v1/hotels/{hotel}/situations/S1/answer"), Some(&tok), Some(json!({"answer": 0}))).await;
    assert_eq!(call(&app, "GET", &format!("/v1/games/{game}/rounds/jun"), Some(&tok), None).await.body["unanswered"], 1);
    assert_eq!(call(&app, "POST", &format!("/v1/games/{game}/rounds/jun/close"), Some(&tok), None).await.status.as_u16(), 202);
    assert_eq!(call(&app, "POST", &format!("/v1/games/{game}/rounds/jun/close"), Some(&tok), None).await.status.as_u16(), 200);
    run_worker(&d).await;
    // srpanj
    let g = call(&app, "GET", &format!("/v1/games/{game}"), Some(&tok), None).await;
    assert_eq!(g.body["current"], json!({"month": "jul", "status": "open", "unanswered": 1}));
    let s = call(&app, "GET", &format!("/v1/hotels/{hotel}/situations"), Some(&tok), None).await;
    assert_eq!(s.body["items"][0]["code"], "S2");
    call(&app, "POST", &format!("/v1/games/{game}/rounds/jul/close"), Some(&tok), None).await;
    run_worker(&d).await;
    // ljestvica i feed
    let g = call(&app, "GET", &format!("/v1/games/{game}"), Some(&tok), None).await;
    assert!(g.body["current"].is_null(), "igra je gotova");
    for h in g.body["hotels"].as_array().unwrap() {
        assert_eq!(h["months"].as_array().unwrap().len(), 2, "{h}");
    }
    let f = call(&app, "GET", &format!("/v1/hotels/{hotel}/feed"), Some(&tok), None).await;
    let resolved = f.body["items"].as_array().unwrap().iter().filter(|i| i["kind"] == "month_resolved").count();
    assert_eq!(resolved, 2);
    assert_problem(&call(&app, "PUT", &format!("/v1/hotels/{hotel}/plans/jul"), Some(&tok),
        Some(me.body["private"]["plan"]["plan"].clone())).await, 409, "round_not_open");
    // baza: dva obračuna, nijedan posao nije ostao
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE type = 'RoundResolved'").fetch_one(&pool).await.unwrap();
    assert_eq!(n, 2);
    let open: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs WHERE status <> 'done'").fetch_one(&pool).await.unwrap();
    assert_eq!(open, 0);
}

#[sqlx::test(migrations = "../store/migrations")]
async fn replay_from_database_matches(pool: PgPool) {
    use app::EventLog;
    use game::game::Game;
    let d = deps(pool);
    let app = router(AppState { deps: d.clone() });
    let r = call(&app, "POST", "/v1/demo", None, None).await;
    let tok = r.set_cookie.unwrap().split(';').next().unwrap().strip_prefix("hotelixsim_guest=").unwrap().to_owned();
    let game = app::GameId(r.body["game_id"].as_str().unwrap().parse().unwrap());
    call(&app, "POST", &format!("/v1/games/{}/rounds/jun/close", game.0), Some(&tok), None).await;
    run_worker(&d).await;
    // pročitaj iz baze (JSONB je promijenio redoslijed ključeva), obračunaj lipanj ponovno i usporedi tipizirano
    let raw = d.log.load(game).await.unwrap();
    let recs: Vec<game::Recorded> = raw
        .iter()
        .map(|r| game::Recorded {
            seq: r.seq,
            actor: game::Actor::parse(&r.actor).unwrap(),
            event: game::GameEvent::from_record(&r.kind, r.schema_ver, r.payload.clone()).unwrap(),
        })
        .collect();
    let at = recs.iter().position(|r| r.event.kind() == "RoundResolved").unwrap();
    let again = Game::fold(&recs[..at]).unwrap().resolve_round(sim_hotel::Month::Jun).unwrap();
    let game::GameEvent::RoundResolved(stored) = &recs[at].event else { panic!() };
    assert_eq!(serde_json::to_vec(&again).unwrap(), serde_json::to_vec(stored).unwrap());
}
```
  (dev-ovisnosti `api`: `game`, `sim-hotel` ako ih test ne vidi; `common/mod.rs` je iz Taska 9.)

- [ ] **Step 2: Pokreni** `bash scripts/baza.sh status`; `cargo test -p api --test e2e; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 3: Regeneracija.** `set -a; source .env; set +a; cargo sqlx migrate run --source crates/store/migrations; echo EXIT=$?`;
  `SQLX_OFFLINE=false cargo sqlx prepare --workspace; echo EXIT=$?`; `corepack pnpm gen:api; echo EXIT=$?` — svi `EXIT=0`.

- [ ] **Step 4: Sve provjere CI-ja, lokalno, svaka zasebno:** `cargo fmt --all -- --check` · `cargo clippy --workspace
  --all-targets -- -D warnings` · `node scripts/check-core-deps.mjs` · `node scripts/probe-mutacija.mjs` ·
  `cargo sqlx prepare --workspace --check` · `cargo test --workspace` · `cargo run -q -p balance --release -- --seeds 100 --check` ·
  `corepack pnpm check:docs` · `corepack pnpm test:docs` · `corepack pnpm check:api` (nakon commita) · `corepack pnpm lint` ·
  `corepack pnpm typecheck` · `corepack pnpm test` — svaka s `; echo EXIT=$?`, sve `EXIT=0`.

- [ ] **Step 5: Ručna proba pravog procesa** (radnik u serveru, kolačić kroz HTTP). Prvi terminal (pozadina):
  `set -a; source .env; set +a; cargo run -p server -- migrate && cargo run -p server -- serve`. Drugi:

```bash
J=/tmp/hx-jar; rm -f $J
curl -s -c $J -X POST http://127.0.0.1:8080/v1/demo                      # {"game_id":…,"hotel_id":…}
G=$(curl -s -b $J http://127.0.0.1:8080/v1/me/hotel | node -pe 'JSON.parse(require("fs").readFileSync(0)).game_id')
curl -s -b $J -o /dev/null -w '%{http_code}\n' -X POST http://127.0.0.1:8080/v1/games/$G/rounds/jun/close   # 202
sleep 1; curl -s -b $J http://127.0.0.1:8080/v1/games/$G/rounds/jun       # "status":"resolved"
```
  Očekivano: 202, pa `resolved` unutar sekunde (radnik radi u procesu). Zaustavi server (Ctrl+C).

- [ ] **Step 6: Dokumenti.** ARHITEKTURA: §6.1 „Postoje:" dodaj `game`; §6.2 dijagram — `game` između `sim-hotel`/`content`
  i `app` (`app ──► game ──► sim-hotel, content`); §9.1 redak `actor` → `text: guest:<uuid> · robot:<strategija> · system
  (CHECK u bazi)` i rečenica da nepromjenjivost sad čuva okidač, a uloga bez prava dolazi prije Railwaya; §9.4 na kraj:
  „Za demo izabrana vlastita tablica `jobs` (ADR-015); probni test s apalis/pgmq i zakašnjeli AI rezultat su korak 4."
  BACKLOG: obriši preostale riješene `[dnevnik]` stavke (`stream_sequence_is_unique` preslab, oblik tablice `events`);
  dodaj odgođeno:
  - `[drugi val]` Limit 3 demo igre po IP-u na dan i brisanje gosta i igre nakon 7 dana (DEMO §1.4; Leon 05.10.).
  - `[prije Railwaya]` Uloga baze bez prava UPDATE/DELETE nad `events` (okidač ostaje); kolačić `Secure` uz HTTPS;
    `content::repo_dir()` je put iz vremena prevođenja — na Railwayu `HOTELIXSIM_CONTENT`; zaseban način `work`.
  - `[dnevnik]` → korak 4: zaglavlje `Idempotency-Key` (ARHITEKTURA §11); ponovni pokušaj s odgodom; probni test
    apalis/pgmq (ARHITEKTURA §9.4); zakašnjela AI presuda za zatvorenu rundu (§8.3).
  - `[ekrani]` Posao obračuna u statusu `failed` ostavlja rundu u „closing" zauvijek — ekran i profesor trebaju to
    vidjeti; SSE „runda je obračunata" umjesto ispitivanja (ARHITEKTURA §11).

- [ ] **Step 7: Commit** — `crates/api .sqlx openapi.json packages/api-client/src/schema.d.ts docs/architecture/ARHITEKTURA.md
  docs/records/BACKLOG.md`, poruka „Dnevnik 12/12: kraj do kraja na pravoj bazi, regeneracija i dokumenti".

---

## Završni pregled (opus)

Nakon Taska 12 orkestrator pokreće **završni pregled cijele grane** `gradnja/dnevnik-runde` na najjačem dostupnom modelu
(opus), po [AGENTI.md](../workflow/AGENTI.md) §1 t. 5. Pregled posebno gleda: (1) Review Focus gore, točku po točku;
(2) svaku naredbu u `app` — je li cijela provjera unutar `decide`; (3) vidljivost — postoji li ijedan put do tuđih
stvarnih brojki, kvalitete ili mehanika; (4) jesu li događaji samodostatni za ponovni obračun bez datoteka;
(5) jezgra i `game` bez novih ovisnosti (`check-core-deps`). Nalazi koji se ne popravljaju idu u
[BACKLOG](../records/BACKLOG.md); zatim `/kraj` (ledger u `docs/archive/`, CHANGELOG, plan u arhivu s pečatom).
**Push i spajanje u `main` samo uz Leonov izričit OK.**

## Što ovaj plan namjerno ne radi

AI presuda, slobodan tekst, likovi i mentor (korak 4; nema `VerdictRecorded`) · ekrani (korak 5) · profesorov pogled i
`?uloga=profesor` (korak 6) · kolovoz i rujan, splet okolnosti i tragovi (drugi val) · limit po IP-u i brisanje gostiju
(drugi val) · projekcijske tablice (fold pri čitanju je dovoljan za 4 hotela × 2 mjeseca).
