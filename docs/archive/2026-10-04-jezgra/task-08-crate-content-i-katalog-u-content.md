### Task 8: Crate `content` i katalog u `content/hotel/` (ARHITEKTURA §9.5)

**Files:**
- Create: `crates/content/Cargo.toml`, `crates/content/src/lib.rs`, `crates/content/tests/load.rs`
- Create: `content/hotel/catalog.json`, `content/hotel/profiles.json`, `content/hotel/situations.json`
- Modify: `Cargo.toml` (workspace `members` + `content = { path = "crates/content" }`)

**Interfaces:**
- Consumes: svi tipovi iz `sim_hotel`.
- Produces: `pub struct Content { pub catalog: Catalog, pub situations: Vec<SituationTemplate> }` ·
  `pub fn load(dir: &Path) -> Result<Content, ContentError>` · `pub fn repo_dir() -> PathBuf`
  (`env!("CARGO_MANIFEST_DIR")/../../content/hotel`) · `ContentError { Io(path, String), Json(path, String), Invalid(String) }`.

- [ ] **Step 1: JSON.** Ključevi mjeseci `"jun" "jul" "aug" "sep"`, kategorija `"3" "4" "5"`, profila `"A" "B" "C"`;
  sve brojke u jedinicama koda (centi, bp, stotinke). Početne vrijednosti = testni katalog (Task 3, korak 1); Task 11
  ih kalibrira. Oblik:

```json
{
  "categories": {
    "4": {
      "price_ref_cents": { "jun": 13500, "jul": 16500, "aug": 17500, "sep": 13500 },
      "base_occupancy_bp": { "jun": 7200, "jul": 7800, "aug": 8300, "sep": 7300 },
      "seasonal_plan": { "jun": 20, "jul": 20, "aug": 20, "sep": 20 },
      "elasticity_bp": -8000, "wage_cents": 220000, "cpor_cents": 2200
    }
  },
  "reference_profiles": { "4": "C" },
  "economy": { "other_revenue_bp": 3000, "hire_cents": 150000, "interest_bp": 100,
               "marketing_half_cents": 1500000, "marketing_max_bp": 1500, "goppar_floor_cents": 1000 },
  "actions": { "maintenance_deferred": { "cost_cents": 300000, "equipment_pts": 0 }, "...": "svako polje ActionSpec" },
  "mechanics": { "recovery_pts_per_q": 200, "...": "svako polje MechanicSpec" }
}
```
(„...": napiši SVA polja `ActionSpec`/`MechanicSpec` s vrijednostima iz Taska 3 — JSON nema komentare ni skraćenice.)
`situations.json` — S1 i S2 po DEMO §7 (q iz tablica); **S3 i S4 su nacrt (ADR-013 t. 7)**, Leon provjerava:

| kod, mjesec | odgovor | q | mehanike |
|---|---|---|---|
| S1, jun | Ispričati se, odmah premjestiti obitelj, ponuditi večeru, poslati tehničara, sutra nazvati gošću | 5 | `ServiceRecovery{12000}` + `EquipmentRepair{urgent:true}` |
| | Ispričati se, premjestiti obitelj i poslati tehničara | 3 | `ServiceRecovery{0}` + `EquipmentRepair{urgent:true}` |
| | Vratiti novac za cijeli boravak | 1 | `ServiceRecovery{94500}` + `EquipmentRepair{urgent:false}` |
| | Objasniti da su zbog toplinskog vala svi uređaji opterećeni i poslati tehničara kad stigne | 1 | `ServiceRecovery{0}` + `EquipmentRepair{urgent:false}` |
| S2, jul | Razgovor; 8 % povišice i bonus na kraju sezone; on bira i obučava zamjenika | 5 | `StaffRetention{8, 200000}` |
| | Bonus na kraju sezone ako ostane, bez razgovora | 2 | `StaffRetention{0, 300000}` |
| | Odmah dati 20 % | 1 | `StaffRetention{20, 0}` |
| | Odbiti: „nitko nije nezamjenjiv" | 1 | `StaffRetention{0, 0}` |
| S3, jun | Protuponuda: grupa u rujnu, 20 % popusta, depozit i rok otkaza | 5 | `GroupBooking{true, 25, 7, 2000, sep}` |
| | Prihvatiti uz 15 % popusta i depozit | 3 | `GroupBooking{true, 25, 7, 1500, aug}` |
| | Prihvatiti ponudu kako je | 1 | `GroupBooking{true, 25, 7, 2500, aug}` |
| | Odbiti | 2 | `GroupBooking{false, 25, 7, 2500, aug}` |
| S4, aug | Voda i hlađenje za goste i osoblje, iskrena obavijest unaprijed, štednja u praonici i bazenu, cisterna kao rezerva, kraće smjene | 5 | `CrisisCommunication{300000}` |
| | Obavijest gostima i štednja u praonici | 3 | `CrisisCommunication{0}` |
| | Noću zatvoriti tuševe i bazen bez objašnjenja | 1 | `CrisisCommunication{0}` |
| | Ne poduzimati ništa dok ograničenja ne počnu | 1 | `CrisisCommunication{0}` |

Mehanike u JSON-u: `{ "type": "ServiceRecovery", "compensation_cents": 12000 }` (serde `tag = "type"`; ako
`deny_unknown_fields` ne radi s unutarnjom oznakom, koristi `#[serde(tag = "type", content = "params")]` i tako piši JSON).

- [ ] **Step 2: Testovi** `crates/content/tests/load.rs`:

```rust
use sim_hotel::*;

#[test]
fn repo_content_loads_and_validates() {
    let c = content::load(&content::repo_dir()).unwrap();
    assert_eq!(c.catalog.validate(), Ok(()));
    assert_eq!(c.catalog.profiles.len(), 3);
    let codes: Vec<&str> = c.situations.iter().map(|s| s.code.as_str()).collect();
    assert_eq!(codes, ["S1", "S3", "S2", "S4"]); // redoslijed: po mjesecu, pa po datoteci
    assert!(c.situations.iter().all(|s| s.answers.len() == 4 && s.answers.iter().all(|a| (1..=5).contains(&a.quality))));
}

#[test]
fn invalid_catalog_is_rejected_with_field_name() {
    let dir = std::env::temp_dir().join(format!("hotelixsim-content-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for f in ["profiles.json", "situations.json"] {
        std::fs::copy(content::repo_dir().join(f), dir.join(f)).unwrap();
    }
    let cat = std::fs::read_to_string(content::repo_dir().join("catalog.json")).unwrap();
    std::fs::write(dir.join("catalog.json"), cat.replacen("\"wage_cents\": 220000", "\"wage_cents\": 0", 1)).unwrap();
    let err = content::load(&dir).unwrap_err();
    assert!(format!("{err:?}").contains("wage_cents"), "{err:?}");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn unknown_field_is_rejected() {
    let dir = std::env::temp_dir().join(format!("hotelixsim-content-u-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for f in ["catalog.json", "situations.json"] {
        std::fs::copy(content::repo_dir().join(f), dir.join(f)).unwrap();
    }
    let p = std::fs::read_to_string(content::repo_dir().join("profiles.json")).unwrap();
    std::fs::write(dir.join("profiles.json"), p.replacen("\"rooms\"", "\"rooms_typo\": 1, \"rooms\"", 1)).unwrap();
    assert!(matches!(content::load(&dir), Err(content::ContentError::Json(..))));
    std::fs::remove_dir_all(&dir).unwrap();
}
```
Situacije se nakon učitavanja **stabilno** poredaju po mjesecu (S1, S3 u lipnju redom iz datoteke). Provjera
učitavanja: svaki odgovor ima q 1–5 i bar jednu mehaniku; mjesec grupe postoji; zatim `catalog.validate()`.

- [ ] **Step 3–5:** pokreni (pada) → implementiraj (DTO s `#[serde(deny_unknown_fields)]`, pretvorba u `sim_hotel`
  tipove, `validate`) → `cargo test -p content; echo EXIT=$?` i `node scripts/check-core-deps.mjs` (content NIJE jezgra,
  brana i dalje 0).
- [ ] **Step 6: Commit** — „Jezgra 8/12: crate content i katalog u content/hotel (S3/S4 nacrt za Leona)".

---

