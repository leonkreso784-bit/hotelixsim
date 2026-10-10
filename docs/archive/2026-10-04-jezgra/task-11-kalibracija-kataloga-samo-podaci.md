### Task 11: Kalibracija kataloga (samo podaci)

**Files:**
- Modify: `content/hotel/catalog.json`, `content/hotel/profiles.json`
- Modify: `docs/product/DEMO.md` §2.1 i §5 (brojke → pokazivač na JSON), `docs/research/KALIBRACIJA_2026-10-04.md` (rezultat)

- [ ] **Step 1:** `cargo run -p balance --release -- --seeds 1000 --check; echo EXIT=$?` → zapiši koje provjere padaju.
- [ ] **Step 2: Ekonomija i istiskivanje.** Mijenjaj samo `seasonal_plan`, `base_occupancy_bp` (kolovoz smije do
  9 500 — 4* u Opatiji u vrhu sezone puniji je od prosjeka DZS-a) i fiksne troškove profila. Polazište iz modela
  (`scripts/oracle-jezgra.mjs`): plan sezonaca 28/33/37/29 uz fiksne 120 000 € daje sezonu ≈ 38,8 %, kolovoz ≈ 48,5 %.
  Profil C: `seasonal` = plan lipnja (inače lipanj plaća zapošljavanje).
- [ ] **Step 3: Profili A i B** prema ciljevima §5 (±2) — mijenjaj samo njihova polja u `profiles.json`, smjer iz DEMO §5
  (A: ocjena, zadovoljstvo, moral, opterećenje, blagajna, fiksni; B: oprema, iskustvo, ocjena, blagajna, plaća).
- [ ] **Step 4: Pravednost i dominacija.** Ako §11.3/§11.4 padnu, NE mijenjaj formule: zapiši rezultat (tablica iz
  `render`) u KALIBRACIJA kao nalaz za Leona i u BACKLOG `[jezgra]`; smiju se mijenjati samo podaci iz koraka 2–3.
- [ ] **Step 5:** `cargo run -p balance --release -- --seeds 1000 --check; echo EXIT=$?` → `EXIT=0` za §11.1, §11.2,
  istiskivanje i §11.5 (obavezno); §11.3/§11.4 obavezno samo ako se mogu postići podacima — inače izričit nalaz.
  `cargo test --workspace; echo EXIT=$?` → 0 (testni katalog se NE mijenja).
- [ ] **Step 6: Dokumenti.** DEMO §2.1 i §5: brojke zamijeni rečenicom „Brojke žive u `content/hotel/*.json`
  (kalibrirane alatom za balans); ovdje je samo smjer." (jedna činjenica, jedno mjesto). U KALIBRACIJA dodaj
  „Rezultat kalibracije" s izlazom `render` (bez ISO datuma u `product/`; `check:docs` mora ostati zelen).
- [ ] **Step 7: Commit** — „Jezgra 11/12: kalibracija kataloga — marža, profili, istiskivanje u kolovozu".

---

