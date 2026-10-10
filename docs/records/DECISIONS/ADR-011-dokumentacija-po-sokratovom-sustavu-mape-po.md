# ADR-011 — Dokumentacija po Sokratovom sustavu: mape po ulozi, jedan indeks, brana, komande
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon, „kasnije da" na prijedlog) · **Vezano:** [ADR-010](ADR-010-rad-u-claude-code-dodatku-za.md),
[ADR-001](ADR-001-hotelixsim-je-zaseban-projekt-odvojen-od.md)

**Kontekst.** Popis 04.10. (20 .md): ista činjenica živi na 3–6 mjesta (stanje CI-ja, push, baza 5433, pravila rada),
nema indeksa ni numeriranih odluka, a `SLJEDECA_SESIJA` se prepisuje pa gubi povijest. Leon: *„ne sviđa mi se kako imamo
dokumentaciju posloženu, moramo imat isti pristup kao i u Sokrat Studyju pošto će ovo biti veliki razvoj, mislim da
dokumentacije ne idu dobro kroz sesiju."*
**Odluka.** Sokratov raspored 1:1: mape po ulozi dokumenta (`product`, `architecture`, `plan`, `workflow`, `records`,
`research`, `archive`); jedini indeks `docs/README.md`; jedna činjenica jedno mjesto; najviše jedan aktivni plan
(`ROADMAP.md` se ne broji); brana `scripts/check-docs.mjs` (mrtve poveznice, .md izvan indeksa, >1 aktivni plan,
>3 ISO datuma u `product/`, budžet `CLAUDE.md`) s vlastitim testom i korakom u CI-ju; komande `/newsession` i nova
`/kraj`. Poboljšanje nad Sokratom: predaja `docs/SLJEDECA_SESIJA.md` ostaje u repou, ne u memoriji, pa ne zastarijeva.
**Posljedice.** Datoteke premještene s `git mv` (povijest ostaje), poveznice ispravljene. Povijest odluka je ovdje,
dnevnik sesija u `PROGRESS.md`, odgođeno u [BACKLOG.md](../BACKLOG.md). Provedeno 04.10. (unos u [PROGRESS.md](../PROGRESS.md)).
