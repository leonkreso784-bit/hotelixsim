# ADR-053 — Pretpostavke 16.10. (B4): sastojak samo u najboljem, oznake iz sadržaja, graditelji tekstova opus
**Datum:** 2026-10-16 · **Status:** ✅ **Potvrdio Leon 17.10.** (t. 1–5 F1; t. 6 riješio ADR-054 t. 1) · **Vezano:** [ADR-051](ADR-051-pretpostavke-faze-b-situacije-i-mjerenje.md),
[ADR-052](ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md), [plan popravka](../../plan/2026-10-14-popravak-temelja.md) (B1b, B4a–c)

Pretpostavke voditelja bez pitanja (ADR-005); svaka ima cijenu ako je krivo.
1. **Test T4b „sastojak samo u najboljem":** popis riječi-korijena (sutra, provjer, ubuduće, potvrd, plan, tjed…) koje smiju stajati
   samo u najboljem odgovoru; prag ≤ 20 % jedinica. Razlog: pregled B4a našao je „za poslije" samo u najboljem u 9 od 19 jedinica, a T4 to nije vidio.
   Ograničenje: popis riječi zaobilazi sinonim ([BACKLOG](../BACKLOG.md)). Ako krivo: popis ili prag, jedan redak.
2. **Testovi čitaju oznake iz sadržaja repoa**, ne doslovno (`mentor.rs` test čita S2 oznaku iz `Snapshot::read`). Inače svaki prepis oznake ruši test.
3. **Graditelji tekstova su opus**, ne sonnet kao u kartici: oznake čita student, Leon ih slijepo pregledava, sklad s mehanikom traži najjači model.
4. **Osnovica balansa bez novog mjerenja** kad se mijenjaju samo `docs/` i CLAUDE.md (B1b koristi mjerenje B1a); ako krivo, `cmp` pokaže razliku.
5. **K1a (čuvar):** S1–S3 i R4 ne mogu dati točno jedan retak (dopušteni `stereotip:`/`prekretnica:`); sastav testova 400/50/50; O7, J1, O6 dopuštaju
   posljedičnu grešku raspona; JSON oblik okvira (`raspodjele.jezici`, `zemlje_*`, `gosti.rod`) pretpostavka čuvara — prilagođava se u jednom krugu ako graditelj javi BLOCKED.
6. **Prag 65 za P-BRAVE, P-CURENJE-VODE, P-DIZALO** (B4c) čeka Leonovu anketu s brojkama; do tada commit 2 B4c se ne spaja. **Riješeno: prag 80, [ADR-054](ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md).**

**Potvrdio Leon 17.10.** ([ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)): t. 1–5 (F1); t. 6 je riješen pragom 80 (ADR-054 t. 1, F2).
