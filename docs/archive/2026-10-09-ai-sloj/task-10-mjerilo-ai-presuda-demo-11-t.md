### Task 10: Mjerilo AI presuda (DEMO §11 t. 7)

Profesor ne smije vidjeti AI koji se ne slaže s čovjekom. Alat mjeri: za S1 i S2 po 20 odgovora koje je ocijenio
**Leon**; AI se mora slagati unutar ±1 u barem 80 % slučajeva. Plan piše 20 + 20 nacrta odgovora (raznih kvaliteta);
**ocjene upisuje Leon** — alat bez njih ne mjeri i to jasno kaže.

**Files:**
- Create: `content/hotel/mjerilo/S1.json`, `content/hotel/mjerilo/S2.json`
- Create: `crates/ai/src/bin/mjerilo.rs`, `crates/ai/src/benchmark.rs`
- Test: `crates/ai/tests/benchmark.rs`
- Modify: `README.md` (jedan redak u „Lokalni razvoj": kako se pokreće mjerilo)

**Interfaces:**
- Produces:

```rust
// benchmark.rs
#[derive(Deserialize)] #[serde(deny_unknown_fields)]
pub struct Case { pub text: String, pub leon_quality: Option<u8>, #[serde(default)] pub note: String }
pub struct Outcome { pub ai_quality: u8, pub leon_quality: u8 }
pub fn ungraded(cases: &[Case]) -> usize;
/// Udio slaganja unutar ±1, u promilima (cjelobrojno; 800 = 80 %).
pub fn agreement_permille(outcomes: &[Outcome]) -> u32;
pub const THRESHOLD_PERMILLE: u32 = 800;
```

- [ ] **Step 1: Napiši testove koji padaju** —

```rust
#[test]
fn agreement_counts_within_one() {
    use ai::benchmark::*;
    let o = |a, l| Outcome { ai_quality: a, leon_quality: l };
    assert_eq!(agreement_permille(&[o(5, 5), o(4, 5), o(2, 4), o(1, 1)]), 750);
    assert_eq!(agreement_permille(&[]), 0);
}

#[test]
fn shipped_cases_parse_and_have_twenty_each() {
    for code in ["S1", "S2"] {
        let path = content::repo_dir().join(format!("mjerilo/{code}.json"));
        let cases: Vec<ai::benchmark::Case> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(cases.len(), 20, "{code}");
        assert!(cases.iter().all(|c| c.leon_quality.is_none_or(|q| (1..=5).contains(&q))));
    }
}
```

- [ ] **Step 2: Pokreni i provjeri da pada.**
- [ ] **Step 3: Nacrti odgovora** — za S1 i S2 po 20 odgovora studenata, realni (kratki, dugi, s pravopisnim greškama,
  jedan pokušaj prompt-injectiona, jedan potpuno promašen, jedan koji ispuni sve), raspoređeni otprilike 4 po svakoj
  razini kvalitete 1–5 prema rubrici. Svaki `{"text": "...", "leon_quality": null, "note": "nacrt: ciljano q≈N"}`.
  `leon_quality` **ostaje `null`** — upisuje ga Leon.
- [ ] **Step 4: Alat** — `cargo run -p ai --bin mjerilo -- S1 [S2]`: učita sadržaj (`content::load`) i slučajeve; ako
  `ungraded > 0` → ispiše „Leon još nije ocijenio N od 20 odgovora za S1 — mjerenje nije moguće" i izlaz **2**; bez
  `ANTHROPIC_API_KEY` → poruka i izlaz 2. Inače za svaki slučaj složi `AssessInput` (bez razgovora; isti `render_assess`
  kao u igri), zove `Anthropic::assess`, provjeri kroz `game`-ovu `validate_verdict` (igra iz `new_game` sa sjemenom 42),
  ispiše tablicu (`#`, Leon, AI, razlika, prvih 60 znakova teksta) i ukupno slaganje; izlaz 0 ako ≥ 800 ‰, inače 1.
  Nevaljana presuda broji se kao neslaganje (i ispiše razlog).
- [ ] **Step 5: README** — „Mjerilo AI-ja (DEMO §11 t. 7): u `content/hotel/mjerilo/*.json` upiši `leon_quality`, pa
  `set -a; source .env; set +a; cargo run -p ai --bin mjerilo -- S1 S2`."
- [ ] **Step 6: Pokreni sve provjere** — sve `EXIT=0` (testovi ne zovu API).
- [ ] **Step 7: Commit** — „AI sloj 10/11: mjerilo AI presuda i nacrti odgovora za Leonove ocjene".

---

