### Task 13: Likvidnost bez gornje granice u sirovom F

**Files:**
- Modify: `crates/sim-hotel/src/indices.rs` (`raw_indices`, likvidnost), `crates/sim-hotel/tests/indices.rs` (zlatne vrijednosti)
- Modify: `scripts/oracle-jezgra.mjs` (već izmijenjen u radnoj kopiji: redak `liq` bez `clamp` — commitaj ga s ovim taskom)
- Modify: `docs/product/DEMO.md` §4 (redak „likvidnost = …, ograničeno 0–100") i §12 P30 (likvidnost)

**Interfaces:** nepromijenjena; mijenja se samo vrijednost `RawIndices.f`.

- [ ] **Step 1: Testovi.** U `tests/indices.rs` promijeni zlatne vrijednosti (iz `node scripts/oracle-jezgra.mjs` nakon izmjene):
  referentna tablica `Jul` F `8_394`, `Aug` F `10_876`, `Sep` F `11_868` (ostale vrijednosti i GOPPAR-i isti); početni
  indeksi profila B `[5_000, 3_571, 5_405, 3_950, 8_195]` (A i C isti). Dodaj test da velika blagajna nije ograničena:
  `state_c()` s `cash_cents = 300_000_000` → `raw_indices(&s, 4_163, 11_000_000, 4_163, &cat).unwrap().f`
  = `div_round(7000·5000 + 3000·div_round(5000·300 000 000, 12 000 000 + 11 000 000), 10 000)` = **`23_065`**
  (likvidnost 65 217). Postojeći test negativne blagajne (F = 3 500) ostaje.
- [ ] **Step 2: Pokreni** `cargo test -p sim-hotel --test indices; echo EXIT=$?` → pada na novim vrijednostima.
- [ ] **Step 3: Implementacija.** `liq = cash ≤ 0 ? 0 : div_round(5000·cash, fixed + labor)` — bez `.clamp(0, 10_000)`.
  Granica 0–100 ostaje samo u `centred`. Komentar uz formulu: zašto nema gornje granice (ADR-014: zasićenje je
  skrivalo potrošen novac, npr. obnovu).
- [ ] **Step 4: Dokumenti.** DEMO §4: `likvidnost = blagajna / (2 × (fiksni + trošak_rada)) × 100, najmanje 0 (bez gornje
  granice — ADR-014)`; P30: isto za likvidnost. Bez novih ISO datuma.
- [ ] **Step 5: Pokreni** sve provjere iz Global Constraints + `cargo run -q -p balance --release -- --seeds 1000; echo EXIT=$?`
  (bez `--check`: §11.1 za B može pasti — popravlja Task 14). Zapiši novi `render` u izvještaj.
- [ ] **Step 6: Commit** — „Jezgra 13: likvidnost bez gornje granice u sirovom F (ADR-014)".

