### Task 16: Balans v2 — pokretač i nove provjere

**Veličina:** M, 4–5 h · **Ovisi o:** Task 12, Task 13 · **Paralelno s:** Task 14, 15 · **Model:** sonnet

**Cilj:** `balance` vrti v2 za tjedan i mjesec, ispisuje izvještaj i provjerava DEMO §11.1–11.5 i nove kriterije
sloja 1. U ovom tasku provjere **smiju padati** (kalibrira ih Task 17); prihvaćanje je da alat radi i ispisuje.

**Files:**
- Create: `crates/balance/src/v2.rs`, `crates/balance/tests/balance_v2.rs`
- Modify: `crates/balance/src/{lib,main}.rs`, `crates/balance/Cargo.toml` (`sim-hotel-v2.workspace = true`)
- Ne dirati: v1 dio `balance` (izlaz `cargo run -p balance --release -- --seeds 1000 --check` mora ostati bajt-isti),
  sadržaj, jezgra.

**Interfaces (Produces):**

```rust
// crates/balance/src/main.rs: nove zastavice
//   --model v1|v2   (zadano v1; v1 put potpuno nepromijenjen)
//   --round month|week (samo uz v2; zadano week)
//   --content DIR   (za v2 čita DIR i DIR/v2)
// crates/balance/src/v2.rs
pub struct OptionsV2 { pub seeds: u64, pub kind: PeriodKind }
pub struct ReportV2 {
    pub starts: Vec<(Profile, IndexSet)>,
    pub reference_season_gop_margin_bp: i32,
    pub reference_season_ebitda_margin_bp: i32,
    pub reference_dept_margins_bp: Vec<(DeptCode, UsaliClass, Option<i32>)>,  // sezona, referentni profil, zadani plan
    pub reference_labor_share_bp: i32,
    pub medians: Vec<(Profile, Strategy, i32)>, pub profile_medians: Vec<(Profile, i32)>,
    pub probes: Vec<(String, Vec<(Profile, i32)>)>,
    pub week_vs_month: Vec<(Profile, Strategy, i32, i32)>, // medijan napretka tjedno, mjesečno
    pub reproducible: bool,
}
pub fn run_v2(content: &content::Content, opts: &OptionsV2) -> Result<ReportV2, sim_hotel_v2::ResolveError>;
pub fn failures_v2(r: &ReportV2) -> Vec<String>;
pub fn render_v2(r: &ReportV2) -> String;
```

Provjere (`failures_v2`): §11.1 početni indeksi kao v1 (ciljevi `Profile::starting_indices`, tolerancija C ±100, A/B
±200); **GOP marža sezone referentnog hotela 3 500–4 500 bp** (🟨 P4); EBITDA marža ispod GOP marže; marže odjela
(sezona, referentni profil, zadani plan): Smještaj 6 500–8 000, hrana i piće 1 500–3 500 (svaki odjel tog razreda),
wellness/ostali operativni 2 000–4 000 [PROC, MODEL_V2 §8]; udio troška rada u ukupnom prihodu 3 000–4 000 bp;
§11.3 pravednost (razlika medijana profila ≤ 499, nijedan profil najbolji pod svim strategijama); §11.4 nijedna
proba strogo najbolja za sva tri profila — probe v1 (`Probe::ALL` preneseni na `PeriodPlan`) **plus** odjelne: „zatvori
sve odjele koji se smiju zatvoriti", „razina cijene 120 % u svim odjelima", „razina cijene 90 %", „održavanje odgođeno
svugdje", „održavanje pojačano svugdje", „sezonci samo u Smještaju"; **tjedno ≈ mjesečno:** za svaki profil i
strategiju |medijan tjedno − medijan mjesečno| ≤ 300 bodova (stotinki; 3 boda) [PROC]; §11.5 ponovljivost (sjeme 7,
dvaput isto).

- [ ] **Step 1: Testovi (padaju)** — `crates/balance/tests/balance_v2.rs`:

```rust
use balance::v2::{OptionsV2, failures_v2, render_v2, run_v2};
use content::{Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;

fn content_v2() -> content::Content {
    Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap()
}

#[test]
fn v2_balance_runs_for_both_kinds_and_is_reproducible() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let r = run_v2(&content_v2(), &OptionsV2 { seeds: 3, kind }).unwrap();
        assert!(r.reproducible);
        assert_eq!(r.starts.len(), 3);
        assert!(render_v2(&r).contains("Marže odjela"));
        let _ = failures_v2(&r); // u ovom tasku smiju postojati
    }
}

#[test]
fn degenerate_v2_monthly_balance_matches_v1_numbers() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let c = s.degenerate_v2().build().unwrap();
    let r2 = run_v2(&c, &OptionsV2 { seeds: 5, kind: PeriodKind::Month }).unwrap();
    let r1 = balance::run(&s.build().unwrap(), &balance::Options { seeds: 5 }).unwrap();
    assert_eq!(r2.starts, r1.starts);
    assert_eq!(r2.profile_medians, r1.profile_medians);
}
```

- [ ] **Step 2:** `cargo test -p balance --test balance_v2` → FAIL.
- [ ] **Step 3: Implementiraj** (`run_v2` kao `run` u v1, nad `sim_hotel_v2::run_season`; marže iz
  `usali::statement` + `sum_statements` po sezoni; probe nad `PeriodPlan`).
- [ ] **Step 4:** `cargo run -q -p balance --release -- --seeds 1000 --check; echo EXIT=$?` → EXIT=0 i izlaz isti kao
  prije taska (prije prve izmjene spremi izlaz te naredbe u datoteku u scratchpadu i na kraju usporedi s `diff`);
  `cargo run -q -p balance --release -- --model v2 --round week --seeds 100; echo EXIT=$?` → ispisuje izvještaj
  (EXIT=0 bez `--check`); zapiši popis provjera koje padaju u izvještaj taska — to je ulaz za Task 17.
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/16: balans v2 — tjedan i mjesec, marže odjela, udio rada, tjedno prema mjesečnom".

---

