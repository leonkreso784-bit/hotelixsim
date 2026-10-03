# CLAUDE.md — HotelixSim

> Učitava se svaku sesiju. Drži ga kratkim: samo trenutno stanje, pravila i pokazivači.

## Što je projekt
**Ime: HotelixSim** (Leon, 2026-10-04). ⚠️ Nikad skraćeno na „Hotelix" — to je postojeći hotelski softver; `hotelixsim.com` je bio slobodan 04.10.
Simulacija vođenja hotela za učenje (FMTU Opatija, Hospitality Management). Student dobije hotel, rješava situacije
i donosi odluke uz AI; ocjenjuje se **napredak u odnosu na početak, ne zarada**. Vlasnik i jedini autor: **Leon Kreso**.
**Zaseban projekt** — nema veze s kodom Sokrat Studyja (`Documents\sokratstudy.*`) i ne dira ga (ADR-040 u Sokratu).

- **Vizija:** [docs/VIZIJA.md](docs/VIZIJA.md) · **plan i rokovi:** [docs/PLAN.md](docs/PLAN.md) ·
  izvorna ideja: [docs/izvor/HOTEL_SIM_2026-08-22.md](docs/izvor/HOTEL_SIM_2026-08-22.md)

## Stanje (2026-10-03)
- Prazan repozitorij, samo dokumenti. **Arhitektura nije odabrana** (Python backend — FastAPI ili drugo — otvoreno).
- Sljedeće: faza **PROTOTIP** (rok ~10.10.) — prvo kratka usporedba stacka i Leonova presuda, tek onda kod.

## Pravila rada s Leonom (prenesena iz Sokrata)
- Odgovaraj na **hrvatskom**, kratko, bez žargona kad pita nešto obično.
- **Odluke kroz anketu** (AskUserQuestion): preporučena opcija prva, svaka s obrazloženjem, bez lažnih dilema.
- **Push i deploy samo uz Leonov izričit OK** — odobrenje jedne radnje ne proteže se na sljedeću.
- Uz svako „gotovo" reci **gdje se vidi** (lokalno / preview / produkcija).
- **Nikad „zeleno" prije nego si vidio izlaz**; „jesi siguran?" = novi prolaz provjere.
- **AI ključ nikad u pregledniku** ni u gitu (`.env` je u `.gitignore`).
- Temelj prije brzine: kad se bira između zakrpe i temelja, ponudi temelj i reci cijenu.
