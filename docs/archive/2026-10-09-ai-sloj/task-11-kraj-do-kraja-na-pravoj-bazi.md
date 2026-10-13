### Task 11: Kraj do kraja na pravoj bazi, sve provjere i dokumenti

**Files:**
- Modify: `crates/api/tests/e2e.rs` (ili novi `crates/api/tests/e2e_ai.rs`)
- Modify: `docs/architecture/ARHITEKTURA.md` (§9.4: „Za demo izabrana vlastita tablica…" — dopuni što je izgrađeno u koraku 4:
  odgoda, čekanje presuda, `VerdictLate`), `docs/product/DEMO.md` §11 t. 7 (gdje je mjerilo)

- [ ] **Step 1: Napiši e2e test** na pravoj bazi (`#[sqlx::test]`, `Store` + `app::memory::FakeAi::good()` kao `ai`):
  gost → igra → 2 poruke liku S1 (radnik odgovori) → slobodan odgovor S1 → presuda (q4) → ponuđeni S3 → „Zatvori
  mjesec" → obračun → mentor u `last_month` → srpanj: slobodan odgovor S2 s `FakeAi::failing()` (drugi `Deps` nad istom
  bazom) → zatvori → nakon `UPDATE jobs SET created_at = now() - interval '10 minutes', run_after = now()` obračun piše
  neutralnu presudu (`review: true`) i `RoundResolved`. Na kraju: ponovno čitanje (fold) cijele igre iz baze i
  `resolve_round` za lipanj daje **bajt-isti** `RoundResolved` kao zapisani (ponovljivost uz presude).
- [ ] **Step 2: Pokreni** — `bash scripts/baza.sh status`, `cargo test -p api --test e2e; echo EXIT=$?` → PASS.
- [ ] **Step 3: Ručna proba s pravim ključem (samo ako Leon ima ključ u `.env`)** — `cargo test -p ai --test live -- --ignored; echo EXIT=$?`.
  Bez ključa: preskoči i zapiši u ledger „živi test nije pokrenut — nema ključa".
- [ ] **Step 4: Sve provjere iz Global Constraints** + `corepack pnpm check:api` + `corepack pnpm check:docs` — sve
  `EXIT=0`, svaki izlaz pročitan.
- [ ] **Step 5: Dokumenti** — ARHITEKTURA §9.4 i DEMO §11 t. 7 kako je navedeno gore (po jedna-dvije rečenice, bez
  ponavljanja plana). CHANGELOG i ROADMAP mijenja `/kraj`, ne ovaj task.
- [ ] **Step 6: Commit** — „AI sloj 11/11: kraj do kraja s AI-jem na pravoj bazi i dokumenti".

---

