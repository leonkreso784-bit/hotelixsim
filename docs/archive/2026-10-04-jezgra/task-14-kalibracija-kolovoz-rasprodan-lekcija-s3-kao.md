### Task 14: Kalibracija — kolovoz rasprodan, lekcija S3 kao provjera

**Files:**
- Modify: `crates/balance/src/lib.rs` (provjera S3), `crates/balance/tests/balance.rs`
- Modify: `content/hotel/catalog.json`, `content/hotel/profiles.json`
- Modify: `docs/research/KALIBRACIJA_2026-10-04.md`, `docs/records/BACKLOG.md` ([leon] S3 i obnova → riješeno, obrisati)

- [ ] **Step 1: Provjera S3 u `failures`** (ADR-014 t. 1): referentni profil, zadani plan: dobit GOP-a od prihvaćanja S3
  grupe (odgovor „kako je ponuđeno", 175 soba-noći, −25 %) u kolovozu mora biti **manja** od dobiti u lipnju; poruka
  „§7 S3: …". Postojeća metrika iz završnog pregleda je ulaz. Test: na sadašnjem katalogu `failures` sadrži „§7 S3".
- [ ] **Step 2: Kalibracija (samo podaci).** Kolovoz rasprodan: `base_occupancy_bp` za kolovoz tako da potražnja
  referentnog hotela bude **veća od raspoloživih soba** (cilj ≥ 105 % — npr. 10 500+ bp; formula dopušta > 10 000) i
  osoblja dovoljno da kapacitet osoblja ne bude uže grlo (`seasonal_plan` kolovoza). Zatim fiksni troškovi / sezonci da
  marža ostane: sezona VI–IX 35–45 %, nijedan mjesec iznad 50 %. Zatim profili A i B do ciljeva §5 (±2) — sad uz
  likvidnost bez granice (Task 13). §11.3–§11.5 moraju ostati zeleni.
- [ ] **Step 3:** `cargo run -q -p balance --release -- --seeds 1000 --check; echo EXIT=$?` → `EXIT=0`, uključujući S3;
  u izvještaj: `render`, dobit S3 kolovoz/lipanj, marže po mjesecima, učinak obnove na napredak A/B/C (proba „obnova u lipnju").
- [ ] **Step 4: Dokumenti.** KALIBRACIJA: odjeljak rezultata ažuriran (stare vrijednosti → nove i zašto); BACKLOG: obrisati
  [leon] stavke S3 i obnove (riješeno ADR-014), dodati novi nalaz ako obnova i dalje dominira. `check:docs` → 0.
- [ ] **Step 5:** sve provjere iz Global Constraints + `corepack pnpm lint`. **Commit** — „Jezgra 14: kolovoz rasprodan, S3 provjera u balansu (ADR-014)".
