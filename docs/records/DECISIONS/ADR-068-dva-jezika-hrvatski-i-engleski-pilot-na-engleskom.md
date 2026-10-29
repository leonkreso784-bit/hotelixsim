# ADR-068 — Dva jezika (hrvatski i engleski); pilot na engleskom za prvu godinu
**Datum:** 2026-10-29 · **Status:** ✅ **Leonove odluke 29.10.** (t. 1–2), 🟨 t. 3–8 ČEKA LEONOV PREGLED ·
**Vezano:** [ADR-066](ADR-066-pilot-s-profesorovim-tempom-grupa-igara.md) (pilot), [ADR-067](ADR-067-tim-rino-tomic-i-simon-seifert-simpolis-kao-izvor.md) (tim),
[ADR-037](ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md) (razred)

**Povod.** Leon 29.10.: „cilj nam je da sljedeći semestar budemo spremni za Management i da bude na engleskom sve". Analiza
(dva read-only agenta, `.superpowers/sdd/2026-10-29-tim/engleski-29-10.md`) pokazala je: sučelje ide kroz `t()` (746 ključeva,
samo `hr.ts`); sadržaj ~43 500 riječi bez polja za jezik; ~350 hrvatskih nizova u Rustu izvan jezgre (`crates/game/src/mentor*.rs`,
`season_mentor.rs`); AI upute traže hrvatski odgovor. Snimak sadržaja ulazi u `GameCreated`, pa igra trajno ostaje na jeziku na
kojem je stvorena.

## A — Leonove odluke (29.10.)
1. **Ciljna skupina pilota: prva godina studija Hospitality Management, smjer na engleskom jeziku** (FMTU), ljetni semestar 2027.
2. **Igra mora biti spremna na oba jezika, hrvatskom i engleskom.**

## B — Pretpostavke voditelja 🟨
3. 🟨 **Jezik se bira po igri** (profesor ga bira pri stvaranju grupe; gost u demou bira na početku). Sučelje prati jezik igre.
   Jedna grupa = jedan jezik.
4. 🟨 **Sadržaj po jeziku u zasebnim mapama** (`content/hotel/` hr, `content/hotel-en/` en) s istim šiframa, ključevima, redoslijedom
   i brojem stavki; test istovjetne strukture čuva da se ne raziđu. Tako brojke ostaju iste, `CORE_VERSION` se ne diže, mijenja se samo `ruleset_id`.
5. 🟨 **Rust vraća kodove i brojke, web prevodi** (objašnjenja „zašto", mentor kad AI ne radi); gdje to nije izvedivo, tablica nizova po jeziku.
6. 🟨 **AI upute u verzijama po jeziku** (npr. `presuda.en.v1`, `lik.en.v1`, `mentor.en.v1`), birane po jeziku igre; za engleski novo
   mjerilo suca (ocjene engleskih odgovora, s profesorom).
7. 🟨 **Banka ljudi na engleskom kasnije** — igra je danas ne koristi; imena likova ostaju hrvatska (hotel je u Opatiji).
8. 🟨 **Prva godina:** studenti još ne znaju USALI, ADR, RevPAR ni GOPPAR. Sadržaj i mentor trebaju kratko objašnjenje pojmova
   (npr. „?" uz svaki pokazatelj); to je pravilo koje student osjeti, pa ide Leonu na anketu prije gradnje.

## Posljedice
- Procjena: ~110–200 h (sučelje i infrastruktura 18–30, sadržaj 20–35, AI upute i mjerilo 16–30, Rust tekstovi 25–45, testovi 10–20,
  banka ljudi 15–25 kasnije), usporedo s planom pilota (~145 h) i izgledom po Simpolisu (~110–180 h web).
- Novi plan do pilota spaja: plan „Pilot s profesorom", engleski i izgled; čeka Leonove odgovore na pitanja 1–9 plana.
- Prijevod sadržaja: AI prijevod + stručni pregled (Simon; terminologija s profesorom).
