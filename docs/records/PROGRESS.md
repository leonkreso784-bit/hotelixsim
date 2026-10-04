# Dnevnik rada (PROGRESS)

Najnoviji unos na vrhu. Svaka sesija: što je napravljeno, što je provjereno, što slijedi. **Povijest, nikad izvor istine** — što vrijedi sad piše u [CLAUDE.md](../../CLAUDE.md).

---

## 2026-10-04 — Preslagivanje dokumentacije po Sokratovom sustavu (ADR-011)

- Povod: Leon nije zadovoljan slaganjem dokumentacije („dokumentacije ne idu dobro kroz sesiju") pa dokumenti idu po ulozi u mape `product`, `architecture`, `plan`, `workflow`, `records`, `research`, `archive` ([ADR-011](DECISIONS.md#adr-011)). Premješteno s `git mv` (povijest ostaje), 33 relativne poveznice prepisane skriptom.
- Novo: jedini indeks [docs/README.md](../README.md); zapisi `PROGRESS.md`, [CHANGELOG.md](CHANGELOG.md), [DECISIONS.md](DECISIONS.md) (ADR-001…011), [BACKLOG.md](BACKLOG.md) (46 odgođenih nalaza + pitanja za Leona + rupe u specu); [AGENTI.md](../workflow/AGENTI.md); komanda `/kraj`; `/newsession` usklađen; `.sokratis/profile.json`.
- CLAUDE.md sveden na pravila, trenutno stanje i pokazivače (3 993 znaka, budžet 12 000); `SLJEDECA_SESIJA.md` skraćen na cilj, prvi korak i pitanja; ROADMAP više ne nosi pitanja (žive u BACKLOG-u); arhiva dobila pečate `✅ ISPUNJEN`.
- Brana `scripts/check-docs.mjs` (mrtve poveznice, indeks, jedan aktivni plan, datumi u `product/`, budžet CLAUDE.md, ćirilica, arhiva proglašena aktivnom) + 11 testova (`node:test`), svaka provjera dokazana mutacijom; korak u CI-ju (`web`). Usput nađeno: Sokratov regex `AKTIVN(O|I|A)` nikad ne hvata „AKTIVAN" — ovdje ispravljeno.
- Provjereno lokalno: `check:docs` EXIT=0 (26 .md, 158 poveznica), `test:docs` 11/11, `lint` EXIT=0, ručna mutacija mrtve poveznice → EXIT=1. CI na GitHubu nije viđen (nije pushano).
- Obrisana spojena grana `gradnja/kostur`.

## 2026-10-04 — Predaja: VS Code panel i plan preslagivanja dokumentacije

- Dogovoreno da se radi u Claude Code dodatku za VS Code (panel, ne terminal) — [ADR-010](DECISIONS.md#adr-010).
- Leon zatražio preslagivanje dokumentacije po Sokratovom sustavu; plan i dijagnoza (ista činjenica na 3–6 mjesta, nema dnevnika ni numeriranih odluka) upisani u [SLJEDECA_SESIJA](../SLJEDECA_SESIJA.md) §2b.
- Provjereno: popis 20 `.md` datoteka, samo 2 mrtve poveznice.
- Otvoreno: sam posao preslagivanja; zatim plan jezgre i alata za balans.

## 2026-10-04 — Push i prvi CI zelen

- Uz Leonov izričit OK pushano na GitHub (`leonkreso784-bit/hotelixsim`, privatni) — [ADR-005](DECISIONS.md#adr-005).
- Provjereno: CI run 37167540215 zelen, `rust` 4m51s, `web` 1m44s; gledano preko `gh` (nije na PATH-u Bash alata, putanja u SLJEDECA_SESIJA §5).
- Upozorenje: akcije `checkout@v4` i `setup-node@v4` na Node 20 zastarjele; `ubuntu-latest` prelazi na Ubuntu 26 od 19.10.
- Revizija stanja (4 agenta) pa 7 paralelnih agenata: nađene rupe u specu [DEMO](../product/DEMO.md) (profil C ne kreće s 50, kalibracija bez redovnog održavanja, nema pravila zaokruživanja); upisane kao ulaz za plan jezgre. Ankete samo za proizvodne odluke — [ADR-006](DECISIONS.md#adr-006).
- Dodana komanda `/newsession`.

## 2026-10-04 — Gradnja kostura od kraja do kraja (8 taskova)

- Po planu [kostur](../archive/2026-10-04-kostur.md): Rust workspace (`engine`, `sim-hotel`, `app`, `store`, `api`, `server`) → `openapi.json` → generirani TS klijent → i18n → jedan React ekran.
- Svaki task s pregledom, završni pregled cijele grane i jedan krug završnih popravaka (LF za sve datoteke, `SQLX_OFFLINE`, dozvole CI-a). Odluke koje je kostur donio umjesto Leona: [ADR-008](DECISIONS.md#adr-008); lokalna baza na 5433: [ADR-007](DECISIONS.md#adr-007); brane determinizma kao Task 1 sljedećeg plana: [ADR-009](DECISIONS.md#adr-009).
- Provjereno lokalno: testovi Rust 28, sučelje 10; zdravlje 200, profil D → 404, kriva lozinka baze → 503 bez pada, Vite proxy.
- Odgođeni nalazi pregleda (47 stavki) zapisani u [BACKLOG](BACKLOG.md).
- Otvoreno: ojačati brane determinizma (allowlist ovisnosti, mutacijski probni testovi).

## 2026-10-04 — Ime, arhitektura v2 i dizajn demoa

- Ime odlučeno: HotelixSim — [ADR-002](DECISIONS.md#adr-002). Pokušaj „Xenokrat" vraćen (revert) prije toga, 03.10. navečer.
- Arhitektura nacrt v1 (Rust + TypeScript) i rasprava, zatim v2 s ugrađenim krugom 1 rasprave — [ADR-003](DECISIONS.md#adr-003); sve lokalno, Railway tek za pilot — [ADR-004](DECISIONS.md#adr-004). Vidi [ARHITEKTURA](../architecture/ARHITEKTURA.md).
- Istraživanja: hotelski pokazatelji, postojeće simulacije, Rust alati ([research](../research/HOTELSKI_POKAZATELJI.md)).
- [DEMO](../product/DEMO.md): opseg demoa i dizajn igre; kasnije kategorija hotela (3*/4*/5*), demo svi 4*. Pitanja za profesore napisana ([PITANJA_ZA_PROFESORE](../product/PITANJA_ZA_PROFESORE.md)).
- Arhitektura se vodi kroz dokument, ne ankete — [ADR-006](DECISIONS.md#adr-006); od 04.10. radi bez pitanja — [ADR-005](DECISIONS.md#adr-005).
- Otvoreno: Leonova provjera domene (pitanja 2–6 u DEMO-u).

## 2026-10-03 — Početak: HotelixSim kao zaseban projekt

- Sesija planiranja u Sokratu: HotelixSim postaje glavni projekt, zaseban repo bez veze s kodom Sokrat Studyja; Sokrat ide u održavanje — [ADR-001](DECISIONS.md#adr-001).
- Prvi commit repoa: [VIZIJA](../product/VIZIJA.md) (dobiveni hotel, fer start, pet osi situacija, AI u četiri uloge, relativna ocjena), [ROADMAP](../plan/ROADMAP.md) (tada PLAN: prototip, prezentacija, pilot, vizija), izvorna ideja u [archive](../archive/HOTEL_SIM_2026-08-22.md), CLAUDE.md.
- Pokušano ime „Xenokrat" (xenia + -krat), isti dan vraćeno.
- Otvoreno: GitHub repo još ne postoji (nema `gh`); backend simulacije; vrijeme „1–7 dana" u simulaciji.
