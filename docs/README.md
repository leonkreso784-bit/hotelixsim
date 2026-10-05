# HotelixSim — dokumentacija

**Ovo je jedini ulaz.** Sve ostalo je u mapama ispod, složeno **po ulozi dokumenta**, ne po temi
(model preuzet iz Sokrat Studyja — [ADR-011](./records/DECISIONS.md#adr-011)).

> **Brzi kontekst za sesiju:** [`../CLAUDE.md`](../CLAUDE.md) (učitava se sam: pravila, trenutno stanje, pokazivači).
> **Početak sesije:** [SLJEDECA_SESIJA.md](./SLJEDECA_SESIJA.md) (`/newsession`) · **kraj sesije:** `/kraj`.
> **Pokretanje i provjere prije commita:** korijenski [`README.md`](../README.md) → „Lokalni razvoj".

---

## Kako je ovo složeno

| mapa | uloga | mijenja se |
|---|---|---|
| **`product/`** | **ŠTO** gradimo — vizija, dizajn igre, sadržaj | rijetko, uz odluku |
| **`architecture/`** | **KAKO** je građeno — sustav, granice, ugovori | kad kod pomakne granicu |
| **`plan/`** | **ŠTO SADA** — `ROADMAP.md` + najviše **jedan** aktivni plan | stalno |
| **`workflow/`** | **KAKO RADIMO** — agenti, tok rada | povremeno |
| **`records/`** | **POVIJEST** — dnevnik, isporuke, odluke, backlog | stalno |
| **`research/`** | istraživanja — podloga za brojke i izbore | uz istraživanje |
| **`archive/`** | **ISPUNJENO** — referenca, **nikad izvor istine** | isti dan kad dokument ispuni svrhu |

### Gdje što ide — jedna činjenica, jedno mjesto

| vrsta znanja | JEDINO mjesto | ostali |
|---|---|---|
| **što sustav radi** | **kod + testovi** | dokument to samo *opisuje*, nikad ne *definira* |
| **što vrijedi sad** | `../CLAUDE.md` + `product/` + aktivni plan | nikad dnevnik, nikad `archive/` |
| **zašto je tako** | `records/DECISIONS.md` (ADR) | ostali linkaju ADR |
| **što se kad dogodilo** | `records/PROGRESS.md` (sesija) · `records/CHANGELOG.md` (isporuka) | `CLAUDE.md` ih ne ponavlja |
| **što nije riješeno** | `records/BACKLOG.md` (i pitanja za Leona) | plan uzima stavke po oznaci, ne prepisuje ih |
| **kako pokrenuti** | korijenski `README.md` | ostali linkaju |
| **što radi sljedeća sesija** | `SLJEDECA_SESIJA.md` | prepisuje se na svakom `/kraj` |

**Duplikat se briše, ne sinkronizira.**

### Pravila (čuva ih `corepack pnpm check:docs`, i u CI-ju)

1. **Svaki `.md` pod `docs/` je naveden ovdje.** Dokument koji nije u indeksu je duh.
2. **Jedan aktivni plan.** `plan/` ima `ROADMAP.md` i najviše jedan plan. Ispunjen → `archive/` isti dan, s pečatom
   `✅ ISPUNJEN datum`. Plan koji čeka nosi `**Status:** ⏸️ PAUZIRAN`.
3. **`product/` nije dnevnik** — najviše 3 ISO datuma po datoteci; kronologija ide u `records/`.
4. **Nema mrtvih poveznica** ni ćirilice u tekstu; `CLAUDE.md` ima budžet znakova.

---

## Korijen `docs/`

| Dokument | Svrha |
|---|---|
| [SLJEDECA_SESIJA.md](./SLJEDECA_SESIJA.md) | Predaja: cilj sljedeće sesije, prvi korak, otvorena pitanja. Prepisuje se na kraju svake sesije |

## `product/` — što gradimo

| Dokument | Svrha |
|---|---|
| [VIZIJA.md](./product/VIZIJA.md) | Zašto i za koga: dobiveni hotel, fer start, pet osi situacija, AI u četiri uloge, relativna ocjena |
| [DEMO.md](./product/DEMO.md) | **Spec demoa**: stvarni sloj hotela, pet indeksa, profili A/B/C, mehanike, situacije S1–S8, roboti, provjere (§11), pravila zaokruživanja (§12, nacrt) |
| [SITUACIJE_S1b_S5-S8.md](./product/SITUACIJE_S1b_S5-S8.md) | Nacrt ponuđenih odgovora i rubrika za S1b i S5–S8 — čeka Leonovu provjeru |
| [PITANJA_ZA_PROFESORE.md](./product/PITANJA_ZA_PROFESORE.md) | Pitanja za profesore FMTU-a — čeka Leonov pregled prije slanja |

## `architecture/` — kako je građeno

| Dokument | Svrha |
|---|---|
| [ARHITEKTURA.md](./architecture/ARHITEKTURA.md) | Arhitektura v2: Rust (jezgra + backend) + TypeScript (sučelje), granice crateova, determinizam, dnevnik događaja, AI iza porta, redoslijed gradnje (§20) |
| [RASPRAVA_ARHITEKTURE.md](./architecture/RASPRAVA_ARHITEKTURE.md) | Zapis rasprave o nacrtu: primjedbe, odgovori, otvoreni upiti |

## `plan/` — što sada

| Dokument | Svrha |
|---|---|
| [ROADMAP.md](./plan/ROADMAP.md) | 🟩 **živ** — faze (TEMELJI → DEMO → PREZENTACIJA → PILOT → VIZIJA), redoslijed gradnje demoa, gdje smo |

## `workflow/` — kako radimo

| Dokument | Svrha |
|---|---|
| [AGENTI.md](./workflow/AGENTI.md) | Tok rada s agentima: plan → subagenti na grani → pregledi → `/kraj`; ledger, model po poslu, paralelni agenti, CI preko `gh` |

## `records/` — povijest

| Dokument | Svrha |
|---|---|
| [PROGRESS.md](./records/PROGRESS.md) | Dnevnik sesija, najnoviji na vrhu |
| [CHANGELOG.md](./records/CHANGELOG.md) | Isporuke (ono što radi) |
| [DECISIONS.md](./records/DECISIONS.md) | Odluke (ADR-001…), najnovija na vrhu; uključuje one koje čekaju Leonov pregled |
| [BACKLOG.md](./records/BACKLOG.md) | Sve neriješeno: odgođeni nalazi pregleda po oznakama, rupe u specu, pitanja za Leona |

## `research/` — istraživanja

| Dokument | Svrha |
|---|---|
| [HOTELSKI_POKAZATELJI.md](./research/HOTELSKI_POKAZATELJI.md) | Hotelski pokazatelji (popunjenost, ADR, RevPAR, GOPPAR…), tipične vrijednosti, uzročne veze |
| [POSTOJECE_SIMULACIJE.md](./research/POSTOJECE_SIMULACIJE.md) | Postojeće hotelske simulacije i što naučiti od njih |
| [RUST_ALATI_2026-10.md](./research/RUST_ALATI_2026-10.md) | Stanje Rust alata (listopad 2026.) za izbor ovisnosti |
| [KALIBRACIJA_2026-10-04.md](./research/KALIBRACIJA_2026-10-04.md) | Kalibracija početnih indeksa i marže — opcije s brojkama za rupe u DEMO specu (prijedlog, nije odlučeno) |
| [BRANE_DETERMINIZMA_2026-10-04.md](./research/BRANE_DETERMINIZMA_2026-10-04.md) | Pokus izvedivosti brana determinizma — ulaz za Task 1 plana jezgre |

## `archive/` — ispunjeno

| Dokument | Svrha |
|---|---|
| [2026-10-04-kostur.md](./archive/2026-10-04-kostur.md) | ✅ Plan „kostur od kraja do kraja" (8 taskova) |
| [2026-10-04-kostur-ledger.md](./archive/2026-10-04-kostur-ledger.md) | ✅ Radni ledger kostura: odluke i nalazi po taskovima |
| [2026-10-04-jezgra.md](./archive/2026-10-04-jezgra.md) | ✅ Plan „jezgra i alat za balans" (14 taskova, zlatne vrijednosti iz `scripts/oracle-jezgra.mjs`) |
| [2026-10-04-jezgra-ledger.md](./archive/2026-10-04-jezgra-ledger.md) | ✅ Radni ledger jezgre: odluke, nalazi pregleda, odgođene sitnice |
| [2026-10-05-dnevnik-runde.md](./archive/2026-10-05-dnevnik-runde.md) | ✅ Plan „dnevnik, runde i red poslova" (12 taskova, DEMO korak 3) |
| [2026-10-05-dnevnik-runde-ledger.md](./archive/2026-10-05-dnevnik-runde-ledger.md) | ✅ Radni ledger koraka 3: odluke, nalazi pregleda, sažetak završnog pregleda |
| [HOTEL_SIM_2026-08-22.md](./archive/HOTEL_SIM_2026-08-22.md) | Izvorna ideja iz kolovoza (iz Sokrata) — zamijenjena s `product/VIZIJA.md` |
