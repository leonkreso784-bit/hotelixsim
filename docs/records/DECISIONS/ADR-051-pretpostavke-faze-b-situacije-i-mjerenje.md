# ADR-051 — Pretpostavke faze B (B2a–B1a): broj situacija, mjerenje težine, prostor odjela
**Datum:** 2026-10-15 · **Status:** 🟨 ČEKA LEONOV PREGLED · **Vezano:** [ADR-042](ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 1, 12–14,
[ADR-049](ADR-049-situacije-dvadeset-posto-mjeri-se-od-ravnala.md), [ADR-050](ADR-050-popravak-podize-odjel-iznos-u-l1-13.md),
[plan popravka](../../plan/2026-10-14-popravak-temelja.md) (B2a, B2b, B3a, B3b, B1a)

Pretpostavke koje je voditelj donio bez pitanja (ADR-005) dok su se gradile cigle B2a–B1a. Svaka ima cijenu ako je krivo.

**B2a (broj situacija, jedan blok):**
1. Mjesečni krug je jedan blok (nema virtualnih tjedana); razmak 4 tjedna između mjeseci uvijek prolazi, predložak s razmakom 5–6 preskače mjesec.
2. Prilika (`opportunity`) barem jednom u krugu vrijedi i u mjesečnoj igri **od srpnja**, svaki krug.
3. Dopušteno je 3–8 situacija po krugu (3 = obvezne); **broj ne pada s popunjenošću** (izvedeno iz ADR-042 t. 13, nije pitano).
4. Granica odjela u mjesečnom krugu je jedna, ⌈2n/5⌉ (za 6 situacija 3 iz istog odjela; prije 2 po virtualnom tjednu).
5. Broj situacija je izričito zapisan u svakoj novoj v2 igri; **v1 igra s izričitim brojem situacija se odbija** (`InvalidSettings`).
6. Test „kolektivna situacija iza 3. mjesta može se prepustiti" vrti mjesečnu igru **sjemena 43** (tjedna igra 42 je nema).

**B3a / B3b (mjerenje):**
7. Horizont mjerenja puta: tjedno 4 kruga, mjesečno 2 (mjerni izbor, ne pravilo igre).
8. Predznak težine: brana težine gleda predznak Δ; Δ ≤ 0 ruši „B3 odgovori", ne prolazi kao pojas.
9. Prepušteno ima kvalitetu 2 (`DELEGATED_QUALITY`) i 3 obvezne situacije (`MANDATORY_SITUATIONS`) — zrcale igru, čuva ih test pariteta.
10. Medijan razlike računa se po sjemenu (ne razlika medijana) — ispravak nalaza C1 pregleda B3a.

**B1a (prostor odjela):**
11. Hitni popravak vrijedi bez faktora kruga, isto tjedno i mjesečno; privremeni bonus ostaje −100 × faktor i djeluje samo u odjel situacije.
12. Situacija bez odjela ili neraspoređena ide u Smještaj; stari snimak bez ključa = bonus 0; odjel izvan stanja hotela je greška kataloga.
13. Načelo šutnje (v2 `situation_effect`) vrijedi za mjesečni krug i prvi tjedni krug mjeseca; dug načela nepromijenjen (16 šifara).

**Cijena ako krivo:** po stavci jedna konstanta ili ključ sadržaja; stavke 1–6 i 11–12 diraju `CORE_VERSION` (2.3.0, 2.4.0) pa i pečat.
