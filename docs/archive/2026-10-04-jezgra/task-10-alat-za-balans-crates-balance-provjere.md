### Task 10: Alat za balans (`crates/balance`) — provjere DEMO §11

**Files:**
- Create: `crates/balance/Cargo.toml`, `crates/balance/src/lib.rs`, `crates/balance/src/main.rs`, `crates/balance/tests/balance.rs`
- Modify: `Cargo.toml` (members)

**Interfaces:**
- Consumes: `content::load`, `sim_hotel::{run_season, run_season_with, RefTable, Strategy, …}`.
- Produces:

```rust
pub struct Options { pub seeds: u64 }
pub struct Report {
    pub starts: Vec<(Profile, IndexSet)>,
    pub reference_margin_bp: Vec<(Month, i32)>, pub reference_season_margin_bp: i32,
    pub august_displacement: bool,                    // grupa iz S3 u kolovozu istiskuje individualne goste
    pub medians: Vec<(Profile, Strategy, i32)>,       // medijan napretka po profilu i strategiji
    pub profile_medians: Vec<(Profile, i32)>,         // medijan preko svih strategija i sjemena
    pub probes: Vec<(String, Vec<(Profile, i32)>)>,   // jedna krajnja radnja cijelu sezonu, napredak po profilu
    pub reproducible: bool,
}
pub fn run(content: &content::Content, opts: &Options) -> Result<Report, sim_hotel::ResolveError>;
pub fn failures(report: &Report, cat: &Catalog) -> Vec<String>;   // prazno = sve provjere prolaze
pub fn render(report: &Report) -> String;                          // čitljiva tablica na hrvatskom
```
CLI: `cargo run -p balance -- [--seeds N] [--content DIR] [--check]` — ispiše `render`, a s `--check` izlazi 1 ako
`failures` nije prazan (i ispiše ih). Zadano `--seeds 1000`, `--content content/hotel`.

- [ ] **Step 1: Provjere** (`failures`):
  1. **Početak (§11.1):** C = 50 ±1 u svakom indeksu; A i B unutar ±2 od `Profile::starting_indices()`.
  2. **Ekonomija (§11.2, ADR-013 t. 5):** referentni profil, zadani plan: sezonska marža VI–IX 3 500–4 500 bp, nijedan
     mjesec iznad 5 000 bp. Marža = `div_round(Σ GOP · 10000, Σ (rooms_revenue + other))`.
  3. **Istiskivanje (ADR-013 posljedice):** referentni profil, kolovoz, uz grupu 175 soba-noći: `individual_sold` manji
     nego bez grupe.
  4. **Pravednost (§11.3):** razlika najvećeg i najmanjeg `profile_medians` < 500 stotinki (5 bodova); i ne postoji
     profil koji ima najveći medijan pod SVE ČETIRI strategije.
  5. **Nema dominantne radnje (§11.4):** probe (svaka = zadani plan + jedna radnja na krajnjoj vrijednosti svaki mjesec;
     situacije odgovara „srednji po q"): cijena 60 %, cijena 150 %, marketing 4 000 000, sezonci +korak, sezonci 0,
     plaća 115, plaća 90, obuka Full, održavanje Intensive, održavanje Deferred, odgovaranje da, obnova u lipnju. Pada ako
     ista proba ima najveći napredak za sva tri profila.
  6. **Ponovljivost (§11.5):** svaka (profil, strategija) za sjeme 7 pokrenuta dvaput daje jednak `SeasonResult`.
  Medijan: sortiraj, za paran broj uzmi niži od dva srednja (cijeli broj, bez prosjeka).

- [ ] **Step 2: Testovi** `crates/balance/tests/balance.rs`:

```rust
#[test]
fn report_runs_on_repo_content_and_is_reproducible() {
    let c = content::load(&content::repo_dir()).unwrap();
    let r = balance::run(&c, &balance::Options { seeds: 5 }).unwrap();
    assert_eq!(r.starts.len(), 3);
    assert_eq!(r.medians.len(), 12);
    assert_eq!(r.probes.len(), 12);
    assert!(r.reproducible);
    let r2 = balance::run(&c, &balance::Options { seeds: 5 }).unwrap();
    assert_eq!(balance::render(&r), balance::render(&r2));
}

#[test]
fn failures_name_the_broken_check() {
    let mut c = content::load(&content::repo_dir()).unwrap();
    c.catalog.profiles.get_mut(&sim_hotel::Profile::C).unwrap().satisfaction_pts = 9_000;
    let r = balance::run(&c, &balance::Options { seeds: 2 }).unwrap();
    let f = balance::failures(&r, &c.catalog);
    assert!(f.iter().any(|m| m.contains("§11.1")), "{f:?}");
}
```
(Drugi test: referentni hotel je i sam C, pa promjena C-a pomiče i referencu — ali A i B tada odu od cilja, pa §11.1 pada.
Ako ne padne, promijeni A umjesto C-a i zapiši zašto.)

- [ ] **Step 3–5:** pokreni (pada) → implementiraj → `cargo test -p balance; echo EXIT=$?`; zatim
  `cargo run -p balance --release -- --seeds 1000; echo EXIT=$?` → ispiše tablicu (provjere smiju još pasti — to je ulaz za Task 11).
- [ ] **Step 6: Commit** — „Jezgra 10/12: alat za balans s provjerama DEMO §11".

---

