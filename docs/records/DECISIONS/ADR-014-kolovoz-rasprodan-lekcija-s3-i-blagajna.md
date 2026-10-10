# ADR-014 — Kolovoz rasprodan (lekcija S3) i blagajna bez gornje granice u financijama
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon, anketa) · **Vezano:** mijenja [ADR-013](ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md) t. 1 (formula F) i
kalibraciju · **Plan:** [2026-10-04-jezgra.md](../../archive/2026-10-04-jezgra.md) Task 13 i 14

**Kontekst.** Završni pregled jezgre izmjerio je dvije strukturne greške. (1) Lekcija S3 radi naopako: prihvatiti grupu
od 175 soba-noći uz 25 % popusta u kolovozu donosi profilu C +25 598 € GOP-a, a u lipnju +19 184 € — kolovoz nije pun pa
grupa nikoga stvarno ne istiskuje. (2) Obnova od 150 000 € pred kraj sezone ne košta ništa: likvidnost je ograničena na
100, a do rujna je blagajna svima toliko puna da i hotel i referentni hotel stoje na 100, pa razlika nestane.
**Odluka (Leon, anketa):**
1. **Kolovoz je rasprodan:** potražnja u kolovozu veća je od broja soba (kao stvarni 4* u Opatiji u vrhu sezone), pa grupa
   s popustom istiskuje goste koji plaćaju punu cijenu. Samo podaci; kalibracija se ponavlja (više osoblja u kolovozu da
   marža ostane ≤ 50 %). Alat za balans dobiva provjeru: prihvaćanje S3 u kolovozu donosi manje od prihvaćanja u lipnju.
2. **Likvidnost se ne ograničava odozgo prije usporedbe s referentnim hotelom:** sirovi F = 0,7 × GOPPAR-indeks +
   0,3 × likvidnost, gdje likvidnost = blagajna / (2 × (fiksni + trošak rada)) × 100 **bez gornje granice** (negativna
   blagajna → 0); granica 0–100 vrijedi samo za konačni, centrirani indeks. Potrošen novac tako ostaje vidljiv do kraja sezone.
**Posljedice.** Mijenja se jedna formula (`indices.rs`) i `scripts/oracle-jezgra.mjs`; zlatne vrijednosti F referentne
tablice od srpnja i početni F profila B mijenjaju se. DEMO §4 i §12 (P30) se usklađuju.
