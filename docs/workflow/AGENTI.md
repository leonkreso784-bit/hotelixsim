# AGENTI — kako HotelixSim gradimo s agentima

> Leon (04.10.): *„koristi agente i subagente da ti pomažu"* i *„napravi da je u pixel agentu puno agenata koji
> rade simultano"*. Ovo je protokol rada. Naredbe za pokretanje i provjere prije commita **nisu ovdje** —
> jedino su mjesto [Lokalni razvoj u README-u](../../README.md#lokalni-razvoj).

## 1 · Tok rada

1. **Brainstorming** (`superpowers:brainstorming`) — što i zašto; arhitektura kroz dokument i raspravu, ne kroz anketu.
2. **Plan** (`superpowers:writing-plans`) u `docs/plan/`, zaglavlje `**Status:** 🟩 AKTIVAN` i `**Područje:** igra|platforma|kvaliteta|dokumentacija` (bez retka = `igra`; `kod` = stari naziv). Najviše jedan aktivni plan **po području**;
   dugoročni smjer je u [ROADMAP-u](../plan/ROADMAP.md). Plan unutar granice živog dokumenta ([tablica](../README.md#granice-veličine)) smije biti jedna datoteka. Veći se **prije
   aktivacije** razdvaja u pregled + mapu taskova: prvo `node scripts/razdvoji-plan.mjs <plan.md> --provjeri`, pa
   `corepack pnpm docs:plan <plan.md>`. Pregled razdvojenog plana ima granicu indeksa iz iste tablice, pa
   velik plan treba pisati tako da pregled stane, inače se ne može aktivirati.
3. **Cigla** (komad od ~2–8 h) u planu faze ima **karticu**: što se radi, koje datoteke smije dirati, dokaz, veličina.
   Svaki korak radi **svjež agent** ([ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) t. 3):
   1. **voditelj** (glavna sesija) otvara granu s integracijske grane, daje karticu, vodi ledger i spaja isti dan kad
      provjeravatelj kaže „da" (pravila grana: t. 6);
   2. **čuvar testova** piše testove iz kartice **prije koda** i dokazuje da padaju iz pravog razloga;
   3. **graditelj** piše kod dok testovi ne prođu i polira što dira. **Testove ne mijenja**: ako misli da je test kriv,
      vraća ga voditelju;
   4. **provjeravatelj** sam ponovno pokreće sve provjere i čita kod naspram kartice (za jezgru i probe-mutaciju);
      nalaz ide graditelju, a tek njegov „da" pušta ciglu dalje;
   5. **čuvar dokumentacije** upisuje promjene (CHANGELOG, ADR, MODEL_V2, BACKLOG) i pokreće `check:docs`; ne prevodi
      Rust pa radi usporedno.
4. **Cigla je gotova** kad su testovi čuvara i sve stalne provjere zeleni (fmt, clippy, testovi, balans, `check:api`,
   web), provjeravatelj kaže „da", dokumentacija je upisana i nijedna dirana datoteka nije prešla granicu veličine (t. 4).
5. **Ritam.** Cigle unutar faze idu jedna za drugom bez čekanja; voditelj se javlja Leonu **na kraju faze**, kad treba
   odluka o igri ili brojkama, ili kad nešto pukne (t. 7). **Kraj faze:** UI čuvar pregleda ekrane, završni pregled
   radi najjači model, Leon igra u pregledniku (jedna adresa i put do novoga); odgođeni nalazi idu u
   [BACKLOG](../records/BACKLOG.md) (t. 5).
6. **Grane i integracija.** Integracijska grana je `model-v2/sloj1` (do L1-15), poslije `main`. Grana cigle ne živi dulje
   od ~1 dana; ako cigla ne stane u dan, dijeli se. `corepack pnpm grane` pokazuje stare i spojene grane.
   **Nakon svakog spajanja** na integracijsku granu balans mora proći: `cargo run -q -p balance --release -- --model v2
   --round week --seeds 1000 --check` i isto s `--round month` (v1: `--seeds 100 --check`). Pad brane = spajanje se vraća.
   **Dvije cigle koje diraju napredak** (jezgra, sadržaj s učinkom, balans) ne rade usporedno u dva stabla; usporedno
   smiju samo cigle s odvojenim datotekama i bez utjecaja na napredak (npr. web i AI uputa). **Svako radno stablo ima
   svoju bazu** kad pokreće poslužitelj. **Prevedeni kod je zajednički:** sva stabla koriste
   `CARGO_TARGET_DIR=C:/Users/leonk/Documents/hotelixsim/target`; samo mjerenje osnovice (B8) ima privremeni target koji
   se briše odmah nakon mjerenja. **Disk:** prije otvaranja stabla `df -h /c`; ispod 100 GB slobodno najprije čišćenje.
   Stablo cigle se **briše isti dan kad se spoji** ([BUGS](../records/BUGS.md) B11).
7. **`/kraj`** — dnevnik, ADR-ovi, predaja, brana `check:docs`, lokalni commit; push tek uz Leonov izričit OK.

## 2 · Radni ledger, brief i pretinac

Orkestrator vodi ledger u `.superpowers/sdd/<plan>/` (**gitignoriran**): spec i grana na vrhu, „Ruling" za svaku
odluku koju je donio umjesto Leona (s cijenom ako je kriva), „Preflight scan" (tablica parova taskova: što jedan
proizvodi, što drugi troši, nalaz) i stanje po tasku. Primjer oblika: [ledger kostura](../archive/2026-10-04-kostur-ledger.md).
Na kraju plana ga `/kraj` kopira u `docs/archive/<datum>-<ime>-ledger.md`, jer git ledger ne vidi. Ledger se zove po
pregledu plana (`<datum>-<ime>`), i kad je plan razdvojen.

- **Ledger je stanje, ne dnevnik:** na vrhu `progress.md` je jedna tablica „cigla | stanje | grana | commit | sljedeći
  korak" koju voditelj prepisuje; povijest ide ispod jednog naslova „Povijest" i ne čita se na početku sesije.
- **Kopija na kraju faze:** `git bundle create ../hotelixsim-kopije/<datum>/hotelixsim-sve-grane.bundle --all` i
  `tar -czf …/superpowers.tar.gz .superpowers` ([ADR-043](../records/DECISIONS/ADR-043-popravak-temelja-ne-gradnja-ispocetka.md) t. 6).
- **Brief = task datoteka.** Izvođač dobiva putanju task datoteke (`docs/plan/<datum>-<ime>/task-NN-….md`) i pregleda.
  Ako je potreban `task-brief`, uvijek s trećim argumentom, inače radni prostor dobije ime taska:
  `task-brief <mapa>/task-NN-x.md N "$(sdd-workspace <pregled>.md)/task-N-brief.md"`
- **Jedna sesija, dva kolosijeka.** Igra i platforma ne dijele datoteke (vlasništvo nad datotekama sprječava sudar,
  ne tema), a svako područje ima najviše jedan aktivan plan. Zasebna sesija dokumentacije otpada: dokumentaciju piše
  čuvar dokumentacije unutar cigle.
- **Pretinac** (`.superpowers/sdd/<plan>/za-dokumentaciju.md`) služi samo za bilješke **nespojenih grana**, u odjeljcima
  `## ADR`, `## BACKLOG`, `## CHANGELOG`, `## PROGRESS`; prenosi se kad se grana spoji.

## 3 · Model po poslu

| uloga | model |
|---|---|
| voditelj | glavna sesija |
| čuvar testova, graditelj | sonnet |
| provjeravatelj | sonnet; opus za rizične cigle (jezgra, balans, sigurnost, prijava) |
| čuvar dokumentacije | haiku ili sonnet |
| kratki ponovni pregled ispravka | haiku |
| završni pregled faze | opus (najjači dostupan) |

Svaki korak dobiva **novog** agenta; nastavak samo za mali popravak. Izlazi idu u datoteke (ADR-038 t. 7).

## 4 · Paralelni agenti

- Neovisne poslove (provjere, revizije, istraživanja, nacrti) pokreći **u jednoj poruci**, u pozadini.
- Najviše **2 agenta istodobno prevode Rust**, zbrojeno preko oba kolosijeka (računalo).
- **Jedan pisac po datoteci**: svaki agent dobiva popis datoteka koje smije dirati i ništa drugo. Eksperimenti nad
  kodom idu u worktree.
- **Vlasništvo datoteka među kolosijecima** ([ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) t. 1;
  izvor: [platforma.md](../plan/ROADMAP/platforma.md)). Tko piše datoteku, odlučuje kolosijek, ne tema:

  | kolosijek | datoteke |
  |---|---|
  | **IGRA** | `crates/engine`, `sim-hotel`, `sim-hotel-v2`, `game`, `content`, `balance`; `content/`, `prompts/` |
  | **PLATFORMA** | `crates/server`, `store` (migracije); `app/src/platform`, `api/src/platform`; nove migracije; web ekrani razreda i profesora |
  | **KVALITETA** | `scripts/`, `.github/`, alati u `docs/`; čisto premještanje koda (nulta promjena ponašanja) |
  | **zajedničko, samo uz dogovor** | `api/src/schema.rs` + `openapi.json` + `packages/api-client` (poslije spajanja **uvijek regenerirati** s `gen:api`, nikad ručno; DTO-i platforme ne idu u `schema.rs`); migracije (dogovoreni raspon brojeva: platforma od 0005, igra dogovorom prije pisanja) |

- Tvrdnje agenata nisu dokaz: ključni nalaz provjeri sam ili ga daj drugom agentu da ga neovisno ponovi.
- Leon prati rad u VS Code dodatku Pixel Agents: imenovani agenti dobiju vlastiti lik i sjedalo, neimenovani su
  „Subtask" likovi uz glavnog. Broj agenata nije ograničen postavkom, nego računalom (vidi gore).

## 5 · CI i GitHub

CI gledaj **sam**, Leona ne pitaj da otvori Actions. `gh` je u `C:\Program Files\GitHub CLI\gh.exe` (nije na PATH-u
Bash alata):

```bash
"/c/Program Files/GitHub CLI/gh.exe" run list -R leonkreso784-bit/hotelixsim -L 3
"/c/Program Files/GitHub CLI/gh.exe" run view <id> -R leonkreso784-bit/hotelixsim
"/c/Program Files/GitHub CLI/gh.exe" run view --job=<id> --log-failed -R leonkreso784-bit/hotelixsim
```

GitHub MCP plugin pada („Authorization header is badly formatted", token nije postavljen) — nije potreban.

## 6 · Što agenti smiju, a što ne

Pravila rada s Leonom u [`CLAUDE.md`](../../CLAUDE.md) vrijede i za svakog agenta (radi bez pitanja, push i deploy
samo uz izričit OK, nikad „zeleno" prije viđenog izlaza). Lokalni commit je u redu; agent commita samo ako mu je to
izričito u zadatku.

## 7 · Zamke ovog stroja

- `pnpm` nije na PATH-u: samo `corepack pnpm …`.
- Cijev maskira exit kod: pokreni golu naredbu pa `echo EXIT=$?` (ne `| tail`).
- Bash alat resetira radni direktorij između poziva: u svakoj naredbi `cd` na apsolutnu putanju ili koristi apsolutne putanje.
- Agentima u promptu daj apsolutne putanje i izričit popis dopuštenih datoteka.
