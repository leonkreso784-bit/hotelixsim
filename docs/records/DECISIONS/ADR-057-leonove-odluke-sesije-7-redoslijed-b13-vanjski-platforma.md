# ADR-057 — Leonove odluke sesije 7 (16.–17.10.): redoslijed, B13, stil lika, vanjski, sesija banke, platforma, disk
**Datum:** 2026-10-17 · **Status:** ✅ **Leonove odluke 16. i 17.10.** (ankete i tekst, sesija 7; dopuna 17.10. u sesiji 8) ·
**Vezano:** [ADR-054](ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md) t. 2–3, [ADR-043](ADR-043-popravak-temelja-ne-gradnja-ispocetka.md) t. 2,
[ADR-004](ADR-004-sve-se-gradi-i-pokazuje-lokalno.md), [ADR-052](ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md) t. 2,
[ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md), [ADR-058](ADR-058-pretpostavke-sesije-7-izgled-vanjski-platforma-k2.md),
[plan popravka](../../plan/2026-10-14-popravak-temelja.md)

Odluke iz sesije 7 koje su čekale ADR na `/kraj` (izvor: ledger popravka temelja i ledger banke ljudi). Pretpostavke izvođača iz istih cigli su u ADR-058.

1. **Redoslijed (anketa 16.10.; rješava ADR-054 t. 3).** Cigla **IZGLED** (web i API, crtić iz roda, dobi i podrijetla) ide **odmah, usporedo s
   PRAVILO-om**. Plan PRAVILO → L1-13 → L1-14 → L1-15 → C ostaje; L1-13b gubi „izgled lika" (ostaje samo ako jezgra zatreba rod u `RoleFill`).
2. **B13, zalijepljeni odgovori (anketa 16.10.; rješava ADR-054 t. 2).** Tko u polje vlastitog odgovora zalijepi **dva ili više** ponuđenih odgovora
   dobije **neutralnu 3**, kao šutnja: 0 €, bez učinka; studentu poruka, mentor to zna. Vrijedi uz oba pravila odgovora (to je kvar, ne pravilo).
   Prepoznaje **kod**, sudac se ne zove (0 tokena). Granica je **parovi susjednih riječi** (svaka oznaka barem 70 %), jer pojedinačne riječi lažno
   hvataju dobar vlastiti odgovor ([BUGS](../BUGS.md) B13); Leon ju je potvrdio 17.10. (A6, ADR-056). Spojeno s PRAVILO-a (9a275d8, `CORE_VERSION` 2.5.0).
3. **Stil lika (anketa 16.10.):** **crtić iz podataka** (A), ne slike; [IZGLED_LIKA](../../architecture/IZGLED_LIKA.md) §4 riješen.
4. **Vanjski su treća vrsta banke ljudi** (Leon 16.10.: *„pod vanjski mislim na inspekciju, policiju i vatrogasce"*), uz radnike i goste; pregled
   ljudi u Excelu ima tri lista. Uloge i brojke: ADR-056 C9–C10.
5. **Tri agenta, svaki svoju vrstu** (gosti, radnici, vanjski) pri punjenju banke.
6. **Sesija banke ljudi radi PARALELNO s glavnom.** 16.10.: *„kasnije zasebna sesija koja paralelno radi samo na banci ljudi i provjerava ih"*, kad
   vanjski (K2v) slete. 17.10. (razgovor): sesija radi **istodobno** s glavnom, nije nova ni zamjenska; *„što više ljudi, to bolje, uvijek se mogu
   koristiti"*. Prompt: `.superpowers/sdd/banka-ljudi-k1/PROMPT-PARALELNA-SESIJA.md`, pravila: [SESIJA_BANKE](../../workflow/SESIJA_BANKE.md).
   Glavna sesija gradi po planu (dokumenti → IZGLED-a → PRAVILO-b i c → L1-13) i spaja ono što banka pripremi.
7. **Dva dana samo za Leonove odluke** nakon PRAVILO-a i IZGLED-a, prije faze C (🟨 ADR-ovi, tekstovi B4, 120 ljudi, voditelj i tržište) →
   [paket od 43 pitanja](../../plan/2026-10-17-leonov-pregled.md), odgovoren 17.10. (ADR-056).
8. **Platforma počinje u studenom 2026., usporedo s fazom C** (P3 o njoj ovisi više nego o C4). Mijenja [ADR-043](ADR-043-popravak-temelja-ne-gradnja-ispocetka.md)
   t. 2 i dopunjuje [ADR-004](ADR-004-sve-se-gradi-i-pokazuje-lokalno.md) (odjeljci „Izmjena 17.10." u obje datoteke). Nacrt: [plan platforme](../../plan/2026-11-platforma.md).
9. **Disk: granica je 50 GB slobodnog**, ne 100 (Leon 16.10.): ispod 50 GB najprije čišćenje (stabla spojenih grana i njihovi `target/<stablo>`).
   Upisano u [AGENTI](../../workflow/AGENTI.md) §1 t. 6, [SESIJA_BANKE](../../workflow/SESIJA_BANKE.md) i [BUGS](../BUGS.md) B11.

**Cijena ako krivo:** (1) redoslijed se mijenja u planu; (2) pravilo zalijepljenih je jedna grana koda i jedna konstanta (70 %); (3) stil je
web modul `look.ts`; (4–6) način punjenja banke, ne igra; (8) dva kolosijeka dijele stroj (najviše 2 Rust agenta ukupno, ADR-052 t. 2);
(9) uz 50 GB ostaje manje mjesta za mjerenja osnovice (B8) — mjerenje se briše odmah.
