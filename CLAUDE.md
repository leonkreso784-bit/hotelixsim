# CLAUDE.md — HotelixSim

> Učitava se svaku sesiju. Samo pravila, TRENUTNO stanje i pokazivači; povijest ide u `docs/records/`.
> Budžet: 12 000 znakova (čuva `corepack pnpm check:docs`).

## Što je projekt
**HotelixSim** — ⚠️ nikad skraćeno na „Hotelix" (postojeći hotelski softver; [ADR-002](docs/records/DECISIONS/ADR-002-ime-hotelixsim-nikad-skraceno-na-hotelix.md)).
Simulacija vođenja hotela za učenje (FMTU Opatija, Hospitality Management): student dobije hotel, rješava situacije
i donosi odluke uz AI; ocjenjuje se **napredak u odnosu na početak, ne zarada**. Vlasnik: **Leon Kreso**; od 29.10. u timu i Rino Tomić (@rtomic20) i Simon Seifert (@simonseifert) — [TEAM.md](TEAM.md),
[ADR-067](docs/records/DECISIONS/ADR-067-tim-rino-tomic-i-simon-seifert-simpolis-kao-izvor.md) (Simpolis s RI-HACK-a kao izvor ideja).
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
  Pretpostavke 🟨: ADR-003 (nakon E4), ADR-032 t. 9, ADR-055 t. 6, ADR-058, ADR-059, ADR-060 t. 17–20, ADR-061 t. 7–12, ADR-062 t. 10–18, ADR-063 t. 7–19 i ADR-064 t. 2–7; Leonove odluke:
  ADR-056, 057 (17.10.), ADR-060, ADR-061 (18.10.), ADR-062, ADR-063 (19.10.) ADR-064 (23.10.) i ADR-065 (24.10.) (popis: [DECISIONS.md](docs/records/DECISIONS.md)).
  Balans: `cargo run -q -p balance --release -- --model v2 --round week --seeds 1000 --check [--situations L,M,H]` (i `--round month`; v1:
  `--seeds 100 --check`, kao CI). `--check` ruši samo brane modela; mjere robota se samo ispisuju
  ([ADR-033](docs/records/DECISIONS/ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md) Dopuna 14.10.).
- **AI ključ** je u `.env` (`ANTHROPIC_API_KEY`), nikad u `.env.example` ni u gitu; živi test:
  `cargo test -p ai --test live -- --ignored` (uz `set -a; source .env; set +a`).
- **Raspored i način rada:** [ROADMAP](docs/plan/ROADMAP.md) — dva kolosijeka (IGRA, PLATFORMA) + kvaliteta, faze i
  cigle, prekretnice P1–P4 (verzija za profesore ~siječanj, studentski pilot ljetni semestar); razred po
  [ADR-037](docs/records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md), rad po
  [ADR-038](docs/records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) (čuvar testova prije graditelja; staje se na kraju faze).
- **Sljedeće** ([SLJEDECA_SESIJA.md](docs/SLJEDECA_SESIJA.md)): **PILOT S PROFESOROM** — [plan](docs/plan/2026-10-24-pilot-s-profesorom.md)
  (🟨 čeka Leonov OK; [ADR-066](docs/records/DECISIONS/ADR-066-pilot-s-profesorovim-tempom-grupa-igara.md), [AI_TEMELJ](docs/architecture/AI_TEMELJ.md),
  [TROSKOVI_I_AI](docs/architecture/TROSKOVI_I_AI.md)); popravak temelja ([ADR-043](docs/records/DECISIONS/ADR-043-popravak-temelja-ne-gradnja-ispocetka.md)) je u
  [arhivi](docs/archive/2026-10-14-popravak-temelja.md), ledger `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md`.
  **Faza A gotova** (na GitHubu). **Faza B** (sve na GitHubu, `model-v2/sloj1`): B1–B4, PRAVILO-a/b/c, IZGLED (a+b) i IZGLED-c (uniforme vanjskih),
  **L1-13** (kalibracija, svi dijelovi; situacije ~20 % napretka), **L1-13b** (izmiješani odgovori, jedan gumb „Spremi plan", opis Smještaja,
  pojačano održavanje +5), banka ljudi (**930**) i **pilot dijelovi 1–7** (kolačić 120 dana, lozinka, stropovi AI 5 $/dan · 30 $/mj, 3 igre po gostu,
  Docker, CI deploy, kopije; [ADR-063](docs/records/DECISIONS/ADR-063-pilot-jedna-adresa-lozinka-stropovi-ai-tri-igre.md)). `main` = `sloj1` = 9b6f047
  (Leon pushao 19.10.). `CORE_VERSION` **2.6.0**. Osnovica balansa `nakon-pilot-4/`. L1-14 Leon odigrao (`l114-popis-izmjena.md`).
  **Sesija 14 (24.10.):** opseg pilota (a) — profesorov tempo kao grupa igara, profesorski kod (ADR-066); plan napisan (val 0 testovi → AI temelj →
  grupa → generator → isporuka). **Sesija 15:** Leon odgovara na pitanja plana 1–9 → „kreni" → val 0; deploy samo uz izričit OK.
  Za trajanje pilota jezgra se zamrzava.
  **Banka ljudi:** radi **paralelna sesija** ([SESIJA_BANKE](docs/workflow/SESIJA_BANKE.md), ledger `.superpowers/sdd/banka-ljudi-k1/progress.md`, stablo `../hotelixsim-ljudi`);
  glavna sesija spaja kad ledger kaže „SPREMNO ZA SPAJANJE".
  **Pozor:** najmanji korak lanca je 0,04 — svaka promjena sadržaja vrti `--check` 1000 tjedno i mjesečno prije spajanja.
  **Svako radno stablo svoj `CARGO_TARGET_DIR=…/hotelixsim/target/<stablo>`** (B12), briše se odmah nakon spajanja; **jedan Rust slot po sesiji** (najviše 2 cargo
  ukupno); **disk: ispod 50 GB ne gradi** (B16). Lažno crveno pod opterećenjem (vitest timeout, 0xC0000142) samo se ponovi (B15).
  Stare probne v2 igre daju 409 (jezgra 2.6.0). Baza se može ugasiti uz prekid sesije (B24). Gradnja (prevođenje) samo na punjaču. Roboti su samo za demo (Leon 13.10.).
  Pogreške i pouke: [BUGS.md](docs/records/BUGS.md). Kopije grana: `Documents/hotelixsim-kopije/<datum>/`.
- **Dokumentacija:** sesija piše `docs/` sama, po [docs/README.md](docs/README.md); **nijedan `.md` preko 1200 redaka**, **plan = jedna datoteka ≤ 1000** (bez mape),
  kod ≤ 1000 (`check:kod`). Bilješke izvođača idu u `.superpowers/…/za-dokumentaciju.md` i prenose se u `docs/` na `/kraj`.
- **Čeka Leona:** opseg pilota (ADR-065 t. 8); novi video u Higgsfieldu (kasnije); pilot tekst privatnosti (s profesorima); 🟨 ADR-063, 064, 065; zadano održavanje nije najbolje ni u jednom hotelu; nalaz E4 ([provjera arhitekture](docs/architecture/PROVJERA_ARHITEKTURE_2026-10-17.md), 5 pitanja); C3 poznate osobe
  ([nacrt rizika](docs/architecture/BANKA_LJUDI/07-poznate-osobe-rizik.md)); ADR-058 t. 10, 13, 19, 20, 🟨 ADR-059 i ADR-060 t. 17–20; uzorci banke;
  pregled lica i uniformi na `/likovi`; slijepi pregled tekstova B4a–c; ocjene 40 odgovora za mjerilo (`content/hotel/mjerilo/*.json`); 5 pitanja [plana platforme](docs/plan/2026-11-platforma.md) §7;
  [BACKLOG.md](docs/records/BACKLOG.md) → „Otvoreno za Leona".
- **Git:** GitHub `leonkreso784-bit/hotelixsim` (**javan** — push je objava) kao `origin`. Neobjavljeno: `git log origin/main..main`.
  CI gledaj sam: `"/c/Program Files/GitHub CLI/gh.exe" run list -R leonkreso784-bit/hotelixsim -L 3`
  (`gh` je prijavljen, nije na PATH-u).
- **Okolina:** VS Code panel ([ADR-010](docs/records/DECISIONS/ADR-010-rad-u-claude-code-dodatku-za.md)), Windows, Git Bash + PowerShell.
  Baza se nakon restarta pali s `bash scripts/baza.sh start` ([ADR-007](docs/records/DECISIONS/ADR-007-lokalna-baza-zaseban-postgresql-16-klaster.md)).
  pnpm samo kao `corepack pnpm`. Sve lokalno do pilota na kraju faze B ([ADR-004](docs/records/DECISIONS/ADR-004-sve-se-gradi-i-pokazuje-lokalno.md), ADR-060 t. 12–14).
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
