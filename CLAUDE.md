# CLAUDE.md — HotelixSim

> Učitava se svaku sesiju. Samo pravila, TRENUTNO stanje i pokazivači; povijest ide u `docs/records/`.
> Budžet: 12 000 znakova (čuva `corepack pnpm check:docs`).

## Što je projekt
**HotelixSim** — ⚠️ nikad skraćeno na „Hotelix" (postojeći hotelski softver; [ADR-002](docs/records/DECISIONS/ADR-002-ime-hotelixsim-nikad-skraceno-na-hotelix.md)).
Simulacija vođenja hotela za učenje (FMTU Opatija, Hospitality Management): student dobije hotel, rješava situacije
i donosi odluke uz AI; ocjenjuje se **napredak u odnosu na početak, ne zarada**. Vlasnik i jedini autor: **Leon Kreso**.
**Zaseban projekt** — ne dira kod Sokrat Studyja ([ADR-001](docs/records/DECISIONS/ADR-001-hotelixsim-je-zaseban-projekt-odvojen-od.md)); Leon mu daje više
vremena nego Sokratu.

**Dokumentacija:** jedini indeks je [docs/README.md](docs/README.md) — tamo piše gdje što ide (jedna činjenica,
jedno mjesto). Pokretanje i provjere prije commita: [README.md](README.md) → „Lokalni razvoj".

## Stanje
- **Sesija počinje** s `/clear` → `/newsession` (čita [docs/SLJEDECA_SESIJA.md](docs/SLJEDECA_SESIJA.md)), **završava**
  s `/kraj` (dnevnik, ADR-ovi, predaja, `check:docs`, commit).
- **Gotovo i na GitHubu:** DEMO koraci 1–5 — cijela igra u pregledniku (mobitel i računalo, [ADR-027](docs/records/DECISIONS/ADR-027-raspored-za-racunalo-prije-demoa.md)),
  AI sloj (lik, presuda `presuda.v3`, mentor), i **sloj 0 puta B**: hotelski pokazatelji (ADR, RevPAR, GOPPAR, točka pokrića)
  ([CHANGELOG](docs/records/CHANGELOG.md)). Modeli: presuda Sonnet, lik Haiku, mentor Sonnet ([ADR-024](docs/records/DECISIONS/ADR-024-modeli-presuda-na-sonnetu-5-5.md)).
  Pretpostavke 🟨: ADR-013, 015, 017, 019, 022, 023, 025, 026, 027 i pravilo o nacionalnosti u [ADR-028](docs/records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md).
  Balans: `cargo run -p balance --release -- --seeds 1000 --check`.
- **AI ključ** je u `.env` (`ANTHROPIC_API_KEY`), nikad u `.env.example` ni u gitu; živi test:
  `cargo test -p ai --test live -- --ignored` (uz `set -a; source .env; set +a`).
- **Sljedeće** ([SLJEDECA_SESIJA.md](docs/SLJEDECA_SESIJA.md)): (1) **UI val 1** za demo na grani `gradnja/ui-popravci`
  (UI čuvar: `docs/product/UI_PRAVILA.md` na toj grani) → `main` prije demoa **~14.10.** ([ADR-012](docs/records/DECISIONS/ADR-012-ciljani-rok-demoa-14-10-prvo.md));
  (2) **put B, sloj 1** ([plan](docs/plan/2026-10-10-v2-sloj1.md), grana `model-v2/sloj1`, spec
  [MODEL_V2.md](docs/architecture/MODEL_V2.md)); (3) **paralelna sesija dokumentacije** ([plan](docs/plan/2026-10-10-dokumentacija.md)) —
  dok radi, sesija koda ne dira `docs/` (bilješke u `.superpowers/sdd/2026-10-10-v2-sloj1/za-dokumentaciju.md`).
- **Čeka Leona:** opis S3 (BACKLOG), ADR-ovi 🟨, ocjene 40 odgovora za mjerilo (`content/hotel/mjerilo/*.json`),
  [BACKLOG.md](docs/records/BACKLOG.md) → „Otvoreno za Leona".
- **Git:** GitHub `leonkreso784-bit/hotelixsim` (privatni) kao `origin`. Neobjavljeno: `git log origin/main..main` (demo, računalo i sloj 0 su na GitHubu).
  CI gledaj sam: `"/c/Program Files/GitHub CLI/gh.exe" run list -R leonkreso784-bit/hotelixsim -L 3`
  (`gh` je prijavljen, nije na PATH-u).
- **Okolina:** VS Code panel ([ADR-010](docs/records/DECISIONS/ADR-010-rad-u-claude-code-dodatku-za.md)), Windows, Git Bash + PowerShell.
  Baza se nakon restarta pali s `bash scripts/baza.sh start` ([ADR-007](docs/records/DECISIONS/ADR-007-lokalna-baza-zaseban-postgresql-16-klaster.md)).
  pnpm samo kao `corepack pnpm`. Sve lokalno; Railway tek za pilot ([ADR-004](docs/records/DECISIONS/ADR-004-sve-se-gradi-i-pokazuje-lokalno.md)).
- **Rad s agentima:** [docs/workflow/AGENTI.md](docs/workflow/AGENTI.md).

## Tko je Leon i kako radimo — POČNI OVDJE
Moja memorija o Leonu je vezana uz Sokratov folder pa se **ovdje ne učitava sama**. Pročitaj na početku sesije
(samo čitanje): `C:\Users\leonk\.claude\projects\c--Users-leonk-Documents-sokratstudy-dev\memory\` →
`tko-je-leon.md` · `komunikacija-s-leonom.md` · `temelji-prije-brzine.md` · `windows-git-bash-zamke.md` ·
`hotelsim-projekt.md` (samo odbijena imena; ostalo je zastarjelo — vrijede ovaj repo i `docs/`).
Ne prepisuj ih ovamo — ondje su izvor.

## Pravila rada s Leonom
- Odgovaraj na **hrvatskom**, kratko, bez žargona kad pita nešto obično; pojam koji možda ne zna objasni od početka.
- **Radi bez pitanja** za tok rada: pretpostavke zapiši (ADR sa statusom 🟨 ČEKA LEONOV PREGLED) ([ADR-005](docs/records/DECISIONS/ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)).
- **Anketa (AskUserQuestion) samo za jednostavne proizvodne odluke s pravim opcijama**: preporučena prva, svaka s
  obrazloženjem, bez lažnih dilema. Arhitektura ide kroz dokument i raspravu; brojke i činjenice pitaj tekstom,
  numerirano ([ADR-006](docs/records/DECISIONS/ADR-006-arhitektura-kroz-dokument-i-raspravu-ne.md)).
- **Push i deploy samo uz Leonov izričit OK** — odobrenje jedne radnje ne proteže se na sljedeću.
- Uz svako „gotovo" reci **gdje se vidi** (lokalno / GitHub / preview / produkcija).
- **Nikad „zeleno" prije nego si vidio izlaz**; „jesi siguran?" = novi, drugačiji prolaz provjere.
- **AI ključ nikad u pregledniku** ni u gitu (`.env` je u `.gitignore`).
- Temelj prije brzine: kad se bira između zakrpe i temelja, ponudi temelj i reci cijenu.
- Neovisne poslove pokreći kao **paralelne agente** u jednoj poruci, svaki na svoje datoteke (Leon ih prati).
