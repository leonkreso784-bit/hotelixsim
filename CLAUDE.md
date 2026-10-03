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
- **Prvi posao nove sesije:** napraviti privatni GitHub repo `hotelixsim`, spojiti ga s lokalnim gitom i pushati
  (uz Leonov OK). GitHub MCP čita token iz Windows varijable `GITHUB_PERSONAL_ACCESS_TOKEN` (postavljena 03.10.).
- Lokalni repozitorij, samo dokumenti. **Arhitektura nije odabrana** (Python backend — FastAPI ili drugo — otvoreno).
- Sljedeće: faza **PROTOTIP** (rok ~10.10.) — prvo kratka usporedba stacka i Leonova presuda, tek onda kod.

## Tko je Leon i kako radimo — POČNI OVDJE
Moja memorija je vezana uz Sokratov folder pa se **ovdje ne učitava sama**. Pročitaj na početku sesije (samo čitanje):
`C:\Users\leonk\.claude\projects\c--Users-leonk-Documents-sokratstudy-dev\memory\` →
`tko-je-leon.md` · `komunikacija-s-leonom.md` · `temelji-prije-brzine.md` · `windows-git-bash-zamke.md` ·
`hotelsim-projekt.md` (sažetak ovog projekta i odbijena imena). Ne prepisuj ih ovamo — ondje su izvor.

## Pravila rada s Leonom (prenesena iz Sokrata)
- Odgovaraj na **hrvatskom**, kratko, bez žargona kad pita nešto obično.
- **Odluke kroz anketu** (AskUserQuestion): preporučena opcija prva, svaka s obrazloženjem, bez lažnih dilema.
- **Push i deploy samo uz Leonov izričit OK** — odobrenje jedne radnje ne proteže se na sljedeću.
- Uz svako „gotovo" reci **gdje se vidi** (lokalno / preview / produkcija).
- **Nikad „zeleno" prije nego si vidio izlaz**; „jesi siguran?" = novi prolaz provjere.
- **AI ključ nikad u pregledniku** ni u gitu (`.env` je u `.gitignore`).
- Temelj prije brzine: kad se bira između zakrpe i temelja, ponudi temelj i reci cijenu.
