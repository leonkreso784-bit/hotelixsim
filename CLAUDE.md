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
jedno mjesto), **granice veličine** dokumenata ([tablica](docs/README.md#granice-veličine)) i pravilo „indeks + mapa istog
imena" za dokumente koji prerastu granicu. Pokretanje i provjere prije commita: [README.md](README.md) → „Lokalni razvoj".

## Stanje
- **Sesija počinje** s `/clear` → `/newsession` (čita [docs/SLJEDECA_SESIJA.md](docs/SLJEDECA_SESIJA.md)), **završava**
  s `/kraj` (dnevnik, ADR-ovi, predaja, `check:docs`, commit).
- **Gotovo i na GitHubu:** DEMO koraci 1–5 — cijela igra u pregledniku (mobitel i računalo, [ADR-027](docs/records/DECISIONS/ADR-027-raspored-za-racunalo-prije-demoa.md)),
  AI sloj (lik, presuda `presuda.v3`, mentor), i **sloj 0 puta B**: hotelski pokazatelji (ADR, RevPAR, GOPPAR, točka pokrića)
  ([CHANGELOG](docs/records/CHANGELOG.md)). Modeli: presuda Sonnet, lik Haiku, mentor Sonnet ([ADR-024](docs/records/DECISIONS/ADR-024-modeli-presuda-na-sonnetu-5-5.md)).
  Pretpostavke 🟨: ADR-013, 015, 017, 019, 022, 023, 025–027, 029–033, 035, 036, 039 i pravilo o nacionalnosti u [ADR-028](docs/records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md)
  (popis: [DECISIONS.md](docs/records/DECISIONS.md)).
  Balans: `cargo run -q -p balance --release -- --model v2 --round week --seeds 1000 --check` (i `--round month`; v1:
  `--seeds 100 --check`, kao CI). `--check` ruši samo brane modela; mjere robota se samo ispisuju
  ([ADR-033](docs/records/DECISIONS/ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md) Dopuna 14.10.).
- **AI ključ** je u `.env` (`ANTHROPIC_API_KEY`), nikad u `.env.example` ni u gitu; živi test:
  `cargo test -p ai --test live -- --ignored` (uz `set -a; source .env; set +a`).
- **Raspored i način rada:** [ROADMAP](docs/plan/ROADMAP.md) — dva kolosijeka (IGRA, PLATFORMA) + kvaliteta, faze i
  cigle, prekretnice P1–P4 (verzija za profesore ~siječanj, studentski pilot ljetni semestar); razred po
  [ADR-037](docs/records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md), rad po
  [ADR-038](docs/records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) (čuvar testova prije graditelja; staje se na kraju faze).
- **Sljedeće** ([SLJEDECA_SESIJA.md](docs/SLJEDECA_SESIJA.md)): **POPRAVAK TEMELJA** — samo to dok nije gotovo
  ([ADR-043](docs/records/DECISIONS/ADR-043-popravak-temelja-ne-gradnja-ispocetka.md), [revizija](docs/architecture/REVIZIJA_TEMELJA.md),
  [plan](docs/plan/2026-10-14-popravak-temelja.md), ledger `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md`).
  Faza A (brane koje mjere pravu stvar, kratke grane) → B (v2 mehanike po odjelu, jedna kalibracija L1-13, ADR-042) → C
  (igra za semestar). Situacije čekaju na `model-v2/sloj1-l111` i ulaze s A1. Roboti su samo za demo (Leon 13.10.).
  Pogreške i pouke: [BUGS.md](docs/records/BUGS.md). Kopije grana: `Documents/hotelixsim-kopije/<datum>/`.
- **Dokumentacija:** sesija piše `docs/` sama, po [docs/README.md](docs/README.md); **nijedan `.md` preko 1200 redaka**,
  kod ≤ 1000 (`check:kod`). Pretinac `za-dokumentaciju.md` samo za nespojene grane (Situacije).
- **Čeka Leona:** 5 pitanja o voditelju odjela i grafu tržišta ([nacrt](docs/architecture/VODITELJ_I_TRZISTE.md)), opis S3 (BACKLOG),
  ADR-ovi 🟨, 🟨 tekstovi iz grane situacija, ocjene 40 odgovora za mjerilo (`content/hotel/mjerilo/*.json`),
  [BACKLOG.md](docs/records/BACKLOG.md) → „Otvoreno za Leona". Banka ljudi ([nacrt](docs/architecture/BANKA_LJUDI.md)) čeka znak za K1.
- **Git:** GitHub `leonkreso784-bit/hotelixsim` (**javan** — push je objava) kao `origin`. Neobjavljeno: `git log origin/main..main`.
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
