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

