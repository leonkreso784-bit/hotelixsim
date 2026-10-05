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
- **Gotovo:** kostur od kraja do kraja, jezgra s alatom za balans i DEMO korak 3 — cijela igra preko API-ja na pravom
  Postgresu (gost, igrač + 3 robota, lipanj i srpanj, „Zatvori mjesec", radnik, ljestvica, feed) ([CHANGELOG](docs/records/CHANGELOG.md));
  dokumentacija po Sokratovom sustavu ([ADR-011](docs/records/DECISIONS.md#adr-011)). Ekrana još nema.
  Pretpostavke: [ADR-013](docs/records/DECISIONS.md#adr-013), [ADR-014](docs/records/DECISIONS.md#adr-014),
  [ADR-015](docs/records/DECISIONS.md#adr-015). Balans: `cargo run -p balance --release -- --seeds 1000 --check`.
- **Sljedeće:** DEMO korak 4 — AI sloj (presuda na slobodni odgovor, likovi, mentor); aktivnog plana još nema
  ([ROADMAP](docs/plan/ROADMAP.md)). **Prvo** popraviti pravilo otiska snimka iz BACKLOG-a `[korak 4]` prije ikakve
  promjene content DTO-a. **Rok demoa ~14.10.**, prvo profesorovih 15 minuta (DEMO §1.5) — [ADR-012](docs/records/DECISIONS.md#adr-012).
- **Čeka Leona:** [BACKLOG.md](docs/records/BACKLOG.md) → „Otvoreno za Leona" (domena, arhitektura v2, AI ključ, inkubator)
  i ADR-ovi sa statusom 🟨 u [DECISIONS.md](docs/records/DECISIONS.md), među njima ADR-015 i [ADR-017](docs/records/DECISIONS.md#adr-017).
- **Git:** GitHub `leonkreso784-bit/hotelixsim` (privatni) kao `origin`. Neobjavljeno: `git log origin/main..main` (osim zadnjeg `/kraj` commita, korak 3 je na GitHubu).
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
