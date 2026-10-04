# CLAUDE.md — HotelixSim

> Učitava se svaku sesiju. Samo pravila, TRENUTNO stanje i pokazivači; povijest ide u `docs/records/`.
> Budžet: 12 000 znakova (čuva `corepack pnpm check:docs`).

## Što je projekt
**HotelixSim** — ⚠️ nikad skraćeno na „Hotelix" (postojeći hotelski softver; [ADR-002](docs/records/DECISIONS.md#adr-002)).
Simulacija vođenja hotela za učenje (FMTU Opatija, Hospitality Management): student dobije hotel, rješava situacije
i donosi odluke uz AI; ocjenjuje se **napredak u odnosu na početak, ne zarada**. Vlasnik i jedini autor: **Leon Kreso**.
**Zaseban projekt** — ne dira kod Sokrat Studyja ([ADR-001](docs/records/DECISIONS.md#adr-001)); Leon mu daje više
vremena nego Sokratu.

**Dokumentacija:** jedini indeks je [docs/README.md](docs/README.md) — tamo piše gdje što ide (jedna činjenica,
jedno mjesto). Pokretanje i provjere prije commita: [README.md](README.md) → „Lokalni razvoj".

## Stanje
- **Sesija počinje** s `/clear` → `/newsession` (čita [docs/SLJEDECA_SESIJA.md](docs/SLJEDECA_SESIJA.md)), **završava**
  s `/kraj` (dnevnik, ADR-ovi, predaja, `check:docs`, commit).
- **Gotovo:** kostur od kraja do kraja ([CHANGELOG 0.1.0](docs/records/CHANGELOG.md)); dokumentacija po Sokratovom
  sustavu ([ADR-011](docs/records/DECISIONS.md#adr-011)).
- **Sljedeće:** plan „jezgra i alat za balans" (DEMO korak 2, [ROADMAP](docs/plan/ROADMAP.md)); Task 1 = ojačati
  brane determinizma ([ADR-009](docs/records/DECISIONS.md#adr-009)). Spec: [DEMO.md](docs/product/DEMO.md) §2–§12.
- **Čeka Leona:** [BACKLOG.md](docs/records/BACKLOG.md) → „Otvoreno za Leona" (domena, arhitektura v2, AI ključ, rokovi)
  i ADR-ovi sa statusom 🟨 u [DECISIONS.md](docs/records/DECISIONS.md).
- **Git:** GitHub `leonkreso784-bit/hotelixsim` (privatni) kao `origin`. Neobjavljeno: `git log origin/main..main`.
  CI gledaj sam: `"/c/Program Files/GitHub CLI/gh.exe" run list -R leonkreso784-bit/hotelixsim -L 3`
  (`gh` je prijavljen, nije na PATH-u).
- **Okolina:** VS Code panel ([ADR-010](docs/records/DECISIONS.md#adr-010)), Windows, Git Bash + PowerShell.
  Baza se nakon restarta pali s `bash scripts/baza.sh start` ([ADR-007](docs/records/DECISIONS.md#adr-007)).
  pnpm samo kao `corepack pnpm`. Sve lokalno; Railway tek za pilot ([ADR-004](docs/records/DECISIONS.md#adr-004)).
- **Rad s agentima:** [docs/workflow/AGENTI.md](docs/workflow/AGENTI.md).

## Tko je Leon i kako radimo — POČNI OVDJE
Moja memorija o Leonu je vezana uz Sokratov folder pa se **ovdje ne učitava sama**. Pročitaj na početku sesije
(samo čitanje): `C:\Users\leonk\.claude\projects\c--Users-leonk-Documents-sokratstudy-dev\memory\` →
`tko-je-leon.md` · `komunikacija-s-leonom.md` · `temelji-prije-brzine.md` · `windows-git-bash-zamke.md` ·
`hotelsim-projekt.md` (samo odbijena imena; ostalo je zastarjelo — vrijede ovaj repo i `docs/`).
Ne prepisuj ih ovamo — ondje su izvor.

## Pravila rada s Leonom
- Odgovaraj na **hrvatskom**, kratko, bez žargona kad pita nešto obično; pojam koji možda ne zna objasni od početka.
- **Radi bez pitanja** za tok rada: pretpostavke zapiši (ADR sa statusom 🟨 ČEKA LEONOV PREGLED) ([ADR-005](docs/records/DECISIONS.md#adr-005)).
- **Anketa (AskUserQuestion) samo za jednostavne proizvodne odluke s pravim opcijama**: preporučena prva, svaka s
  obrazloženjem, bez lažnih dilema. Arhitektura ide kroz dokument i raspravu; brojke i činjenice pitaj tekstom,
  numerirano ([ADR-006](docs/records/DECISIONS.md#adr-006)).
- **Push i deploy samo uz Leonov izričit OK** — odobrenje jedne radnje ne proteže se na sljedeću.
- Uz svako „gotovo" reci **gdje se vidi** (lokalno / GitHub / preview / produkcija).
- **Nikad „zeleno" prije nego si vidio izlaz**; „jesi siguran?" = novi, drugačiji prolaz provjere.
- **AI ključ nikad u pregledniku** ni u gitu (`.env` je u `.gitignore`).
- Temelj prije brzine: kad se bira između zakrpe i temelja, ponudi temelj i reci cijenu.
- Neovisne poslove pokreći kao **paralelne agente** u jednoj poruci, svaki na svoje datoteke (Leon ih prati).
