# ADR-067 — Tim: Rino Tomić i Simon Seifert; Simpolis kao izvor ideja
**Datum:** 2026-10-29 · **Status:** ✅ **Leonove odluke 29.10.** (t. 1–3), 🟨 t. 4–9 ČEKA LEONOV PREGLED ·
**Vezano:** [ADR-001](ADR-001-hotelixsim-je-zaseban-projekt-odvojen-od.md) (zaseban projekt), [ADR-066](ADR-066-pilot-s-profesorovim-tempom-grupa-igara.md) (pilot),
[ADR-037](ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md) (razred), [TEAM.md](../../../TEAM.md)

**Povod.** Leon, Rino Tomić i Simon Seifert (tim Pojedinci) osvojili su 3. mjesto od 8 na RI-HACK 4.0 (Rijeka, 23.–25. 10. 2026.)
s igrom **Simpolis** (repo `leonkreso784-bit/riihack`): profesor otvori igru, razred uđe kodom, studenti vode hotele u zajedničkom
gradu, AI glumi goste, sudi po rubrici i kao mentor prvo pita. Do sada je HotelixSim imao jednog autora.

## A — Leonove odluke (29.10.)
1. **Rino i Simon rade s Leonom na HotelixSimu.**
2. **Što su napravili na RI-HACK-u želimo slično ugraditi u HotelixSim** (ideje i tokovi, popis u [TEAM.md](../../../TEAM.md) „Što iz Simpolisa prenosimo").
3. **Rino i Simon dobivaju pristup repou** (poziv kao suradnici, GitHub `rtomic20` i `simonseifert`) i **TEAM.md** koji opisuje tim i stanje.

## B — Pretpostavke voditelja 🟨
4. 🟨 **Uloge po Simpolisu:** Rino sučelje (`apps/web`), Simon AI sloj (`prompts/`, mjerilo, mentor; Rust u `crates/ai` uz Leona),
   Leon jezgra, backend, baza, spajanje i deploy. Pravo pristupa: pisanje (*write*), ne administracija.
5. 🟨 **Kod Simpolisa** pripada svoj trojici zajedno (njegova LICENSE). U HotelixSim se prenose **ideje i tokovi**; doslovan kod ili
   tekst samo uz pisanu suglasnost sve trojice (komentar na PR-u). Tehnologija se ne mijenja: jezgra ostaje Rust i deterministička.
6. 🟨 **Rad preko grana i PR-a u `main`**, spaja Leon; push, spajanje u `main` i deploy i dalje samo uz Leonov izričit OK (CLAUDE.md).
   Preporuka: zaštita grane `main` (PR obavezan, CI zelen) — Leon uključuje u postavkama repoa.
7. 🟨 **Plan pilota se ne mijenja** samim ulaskom tima: Simpolisove ideje ulaze kroz plan (QR za ulaz, profesorov ispravak presude,
   CSV, mentor koji pita) tek nakon Leonova OK; mentor koji pita je pravilo koje student osjeti i ide kroz anketu. Jezgra ostaje zamrznuta do pilota.
8. 🟨 **Licenca HotelixSima ostaje Leonova** dok Leon ne odluči o suautorstvu (TEAM.md pitanje 2); `LICENSE` se ne mijenja ovim ADR-om.
9. 🟨 **Dokumentacija za tim:** `TEAM.md` u korijenu (vidljiv na GitHubu); `CLAUDE.md` navodi tim. Leonove lokalne bilješke ostaju samo njegove.

## Posljedice
- Tri osobe u repou: veći rizik sukoba datoteka → podjela po mapama kao u Simpolisu (`RADNI-DOGOVOR`), mali PR-ovi.
- Otvoreno za Leona: vlasništvo i licenca, jedna prijava u inkubator (HotelixSim ili Simpolis), sudbina Simpolisa, zaštita `main`.
