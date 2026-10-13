### Task 4: Snimak pravila — polje `model` i sadržaj v2 (koeficijenti razdoblja)

**Veličina:** S–M, 2–3 h · **Ovisi o:** Task 1, Task 2 (test `v1_fixture()` čita
`crates/game/tests/fixtures/v1-snapshot.json` iz Taska 1) · **Paralelno s:** Task 3, Task 5 · **Model:** sonnet

**Cilj:** snimak zna kojim se modelom igra računa (`"v1"` kad ga nema) i nosi odjeljak v2; stari snimci daju isti
otisak bajt po bajt.

**Files:**
- Create: `crates/content/src/v2.rs`, `content/hotel/v2/periods.json`, `crates/content/tests/v2.rs`
- Modify: `crates/content/src/lib.rs`, `crates/content/Cargo.toml` (`sim-hotel-v2.workspace = true`)
- Ne dirati: `content/hotel/{catalog,profiles,situations}.json`, `crates/content/tests/fixtures/*`, `crates/game/**`
  (uključujući fixture Taska 1 — samo se čita).

**Interfaces (Produces):**

```rust
// crates/content/src/lib.rs
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Model { #[default] V1, V2 }

pub struct Content {
    pub catalog: Catalog,
    pub situations: Vec<SituationTemplate>,
    pub ai: BTreeMap<String, SituationAi>,
    /// Katalog v2 (`Some` samo za snimak s `model = v2`); `base` je isti `catalog`.
    pub v2: Option<sim_hotel_v2::Catalog>,
}

pub struct Snapshot {
    catalog: CatalogDto,
    profiles: BTreeMap<String, ProfileDto>,
    situations: Vec<SituationDto>,
    #[serde(default, skip_serializing_if = "is_v1")]
    model: Model,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    v2: Option<v2::V2Dto>,
}
impl Snapshot {
    pub fn model(&self) -> Model;
    /// v1 datoteke iz `dir` + `dir/v2/*.json`; `model = v2`.
    pub fn read_v2(dir: &Path) -> Result<Snapshot, ContentError>;
    /// v2 snimak „kao v1" za ekvivalencijske testove (MODEL_V2 §2 t. 6); odjeljak v2 = `sim_hotel_v2::degenerate`.
    pub fn degenerate_v2(&self) -> Snapshot;
}
```

`content/hotel/v2/periods.json`:

```json
{
  "month": { "morale_step_bp": 3000, "satisfaction_step_bp": 5000, "rating_step_bp": 2500, "seasonal_step": 10 },
  "week": { "morale_step_bp": 800, "satisfaction_step_bp": 1500, "rating_step_bp": 700, "seasonal_step": 5 }
}
```

- [ ] **Step 1: Testovi (padaju)** — `crates/content/tests/v2.rs`:

```rust
use content::{Model, Snapshot, fingerprint_value, repo_dir};
use sim_hotel_v2::PeriodKind;

fn v1_fixture() -> serde_json::Value {
    let p = repo_dir().join("../../crates/game/tests/fixtures/v1-snapshot.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

#[test]
fn old_v1_snapshot_round_trips_without_new_keys() {
    let v = v1_fixture();
    let s = Snapshot::from_value(&v).unwrap();
    assert_eq!(s.model(), Model::V1);
    let again = s.to_value();
    assert!(again.get("model").is_none() && again.get("v2").is_none());
    assert_eq!(fingerprint_value(&again), fingerprint_value(&v));
}

#[test]
fn current_v1_snapshot_has_no_model_key() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    assert_eq!(s.model(), Model::V1);
    assert!(s.to_value().get("model").is_none());
    assert!(s.build().unwrap().v2.is_none());
}

#[test]
fn v2_snapshot_builds_with_period_coefficients() {
    let s = Snapshot::read_v2(&repo_dir()).unwrap();
    assert_eq!(s.model(), Model::V2);
    assert_eq!(s.to_value()["model"], "v2");
    let c = s.build().unwrap();
    let v2 = c.v2.unwrap();
    assert_eq!(v2.base, c.catalog);
    let m = v2.coeffs(PeriodKind::Month).unwrap();
    assert_eq!((m.morale_step_bp, m.satisfaction_step_bp, m.rating_step_bp), (3_000, 5_000, 2_500));
    let w = v2.coeffs(PeriodKind::Week).unwrap();
    assert!(w.morale_step_bp < m.morale_step_bp && w.satisfaction_step_bp < m.satisfaction_step_bp);
}

#[test]
fn v2_snapshot_round_trip_keeps_fingerprint() {
    let v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    let again = Snapshot::from_value(&v).unwrap().to_value();
    assert_eq!(fingerprint_value(&again), fingerprint_value(&v));
}

#[test]
fn model_and_section_must_agree() {
    let mut only_model = Snapshot::read(&repo_dir()).unwrap().to_value();
    only_model["model"] = "v2".into();
    assert!(Snapshot::from_value(&only_model).unwrap().build().is_err());
    let mut only_section = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    only_section.as_object_mut().unwrap().remove("model");
    assert!(Snapshot::from_value(&only_section).unwrap().build().is_err());
}

#[test]
fn degenerate_v2_matches_core_degenerate() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let c = s.degenerate_v2().build().unwrap();
    assert_eq!(c.v2.unwrap(), sim_hotel_v2::degenerate(&c.catalog));
}
```

- [ ] **Step 2:** `cargo test -p content --test v2` → FAIL.
- [ ] **Step 3: Implementiraj.** `v2.rs`: `pub(crate) struct V2Dto { periods: BTreeMap<String, PeriodCoefficientsDto> }`
  (`deny_unknown_fields`), `read_v2_dir(dir)` čita `dir.join("v2/periods.json")` u `periods`, `build_v2(&V2Dto,
  &Catalog) -> Result<sim_hotel_v2::Catalog, ContentError>` (nepoznata vrsta razdoblja → `Invalid`), `degenerate_dto(&Catalog)`.
  `build_v2` **ne piše literal** `sim_hotel_v2::Catalog { base, periods }`, nego `let mut c =
  sim_hotel_v2::degenerate(&base); c.periods = <iz DTO-a>;` — tako Task 9 smije dodati polja u `Catalog` (odjeli,
  profili v2, ekonomija; do Taska 10 ostaju degenerirana) bez diranja `crates/content`.
  `build()` gradi v1 katalog kao danas, pa: `model == V2 && v2.is_some()` → `Some(build_v2)` i `validate()`;
  `model == V1 && v2.is_none()` → `None`; inače `ContentError::Invalid("model i odjeljak v2 se ne slažu")`.
- [ ] **Step 4:** `cargo test -p content; echo EXIT=$?` → zeleno, uključujući **postojeće** testove otiska
  (`crates/content/tests/snapshot.rs`, korak 3).
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/4: snimak pravila zna model; sadržaj v2 s koeficijentima razdoblja".

**Prihvaćanje:** postojeći testovi otiska nepromijenjeni i zeleni; Task 1 `v1_frozen` zelen (Task 1 je spojen prije
ovog taska); `build_v2` počinje od `sim_hotel_v2::degenerate(&base)` (u `crates/content/src/v2.rs` nema literala
`sim_hotel_v2::Catalog { … }` — recenzent provjerava).

---

