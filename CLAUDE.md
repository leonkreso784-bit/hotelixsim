# CLAUDE.md — HotelixSim

> Učitava se svaku sesiju. Drži ga kratkim: samo trenutno stanje, pravila i pokazivači.

## Što je projekt
**Ime: HotelixSim** (Leon, 2026-10-04). ⚠️ Nikad skraćeno na „Hotelix" — to je postojeći hotelski softver; `hotelixsim.com` je bio slobodan 04.10.
Simulacija vođenja hotela za učenje (FMTU Opatija, Hospitality Management). Student dobije hotel, rješava situacije
i donosi odluke uz AI; ocjenjuje se **napredak u odnosu na početak, ne zarada**. Vlasnik i jedini autor: **Leon Kreso**.
**Zaseban projekt** — nema veze s kodom Sokrat Studyja (`Documents\sokratstudy.*`) i ne dira ga (ADR-040 u Sokratu).
**Glavni projekt od 2026-10-03:** Leon radi paralelno, ali **više vremena daje ovome nego Sokratu** (Sokrat se dovršava
po `sokratstudy.f6/docs/plan/ZAVRSNICA.md` pa ide u održavanje).

- **Vizija:** [docs/VIZIJA.md](docs/VIZIJA.md) · **plan i rokovi:** [docs/PLAN.md](docs/PLAN.md) ·
  izvorna ideja: [docs/izvor/HOTEL_SIM_2026-08-22.md](docs/izvor/HOTEL_SIM_2026-08-22.md)

## Stanje (2026-10-04)
- **Početak nove sesije:** pročitaj [docs/SLJEDECA_SESIJA.md](docs/SLJEDECA_SESIJA.md) — tamo je redoslijed rada.
  Leon to pokreće s `/clear` pa `/newsession` (`.claude/skills/newsession/`). Na kraju sesije osvježi tu predaju.
- GitHub: `leonkreso784-bit/hotelixsim` (privatni), spojen kao `origin`. Kostur pushan 04.10.; **prvi CI zelen**. CI gledaj sam:
  `"/c/Program Files/GitHub CLI/gh.exe" run list -R leonkreso784-bit/hotelixsim` (`gh` prijavljen, nije na PATH-u).
  Commitovi nakon toga čekaju Leonov OK za push (`git log origin/main..main`).
- **Kostur od kraja do kraja je gotov** (04.10., plan `docs/superpowers/plans/2026-10-04-kostur.md` + ledger s
  odlukama). Pokretanje: README → „Lokalni razvoj". Baza: zaseban klaster na **5433**, nakon restarta
  `bash scripts/baza.sh start` (sistemski PostgreSQL na 5432 se ne dira). pnpm samo kao `corepack pnpm`.
- Sljedeće: plan „jezgra i alat za balans"; **Task 1 = ojačati brane determinizma** (vidi SLJEDECA_SESIJA §3).
- Arhitektura: **Rust (jezgra + backend) + TypeScript (sučelje)** — **v2** u [docs/ARHITEKTURA.md](docs/ARHITEKTURA.md),
  rasprava u [docs/RASPRAVA_ARHITEKTURE.md](docs/RASPRAVA_ARHITEKTURE.md). Dizajn igre i opseg demoa:
  [docs/DEMO.md](docs/DEMO.md) (nacrt, čeka Leonovu provjeru domene). Plan: [docs/PLAN.md](docs/PLAN.md) (usklađen 04.10.).
- **Sve se gradi i pokazuje lokalno; Railway tek za pilot** (Leon, 04.10.). Istraživanja: `docs/istrazivanje/`.
- **Radi bez pitanja** (Leon, 04.10.): pretpostavke zapiši u dokument; push/deploy i dalje samo uz izričit OK.

## Tko je Leon i kako radimo — POČNI OVDJE
Moja memorija je vezana uz Sokratov folder pa se **ovdje ne učitava sama**. Pročitaj na početku sesije (samo čitanje):
`C:\Users\leonk\.claude\projects\c--Users-leonk-Documents-sokratstudy-dev\memory\` →
`tko-je-leon.md` · `komunikacija-s-leonom.md` · `temelji-prije-brzine.md` · `windows-git-bash-zamke.md` ·
`hotelsim-projekt.md` (samo odbijena imena; ostalo je zastarjelo od 04.10. — vrijede ovaj repo i `docs/`).
Ne prepisuj ih ovamo — ondje su izvor.

## Pravila rada s Leonom (prenesena iz Sokrata)
- Odgovaraj na **hrvatskom**, kratko, bez žargona kad pita nešto obično.
- **Anketa (AskUserQuestion) samo za jednostavne proizvodne odluke s pravim opcijama**: preporučena prva, svaka s
  obrazloženjem, bez lažnih dilema. Arhitektura ide kroz dokument i raspravu; brojke i činjenice pitaj tekstom.
- **Push i deploy samo uz Leonov izričit OK** — odobrenje jedne radnje ne proteže se na sljedeću.
- Uz svako „gotovo" reci **gdje se vidi** (lokalno / preview / produkcija).
- **Nikad „zeleno" prije nego si vidio izlaz**; „jesi siguran?" = novi prolaz provjere.
- **AI ključ nikad u pregledniku** ni u gitu (`.env` je u `.gitignore`).
- Temelj prije brzine: kad se bira između zakrpe i temelja, ponudi temelj i reci cijenu.
