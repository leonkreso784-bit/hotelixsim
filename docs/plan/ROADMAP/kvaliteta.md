# ROADMAP — kolosijek KVALITETA

**Što je ovo:** ZALIHA CIGLI za dug i poliranje: temelj rada (faza 0), stalne cigle poliranja koje se vežu uz faze igre i platforme, i popis pravih
grešaka koje valja popraviti prije siječnja. Faze iza TEMELJA RADA su **smjer** koji se ponovno slaže nakon što profesor vidi igru; kartica cigle se
razrađuje u planu faze kad faza krene. Sati su [PROC]. Kolosijeci se dijele po **datotekama** ([ADR-038](../../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)):
kvaliteta dira alate, CI, dokumentaciju i **čisto premještanje** koda (nulta promjena ponašanja). Susjedi: [igra.md](./igra.md),
[platforma.md](./platforma.md), [ROADMAP.md](../ROADMAP.md). Popis stavki duga je u [BACKLOG.md](../../records/BACKLOG.md); ovdje su samo pokazivači (`BACKLOG.md:redak`
po stanju 13.10.), ne prepis. Kod ~58 000 redaka, 0 TODO, 2 `#[allow]`, 15 `expect("razlog")` — stanje je dobro, dug je u veličini datoteka i rupama u testovima.

**Prekretnice (stupac P):** P1 kraj listopada · P2 kraj studenog · P3 sredina siječnja · P4 ožujak · poslije.

---
## FAZA KVALITETA — 0 TEMELJ RADA (prije P1/P2; 8 cigli, ~26 h)

**Gotovo kad:** CI ne puca zbog novog Ubuntua, dva kolosijeka (igra, platforma) rade usporedno bez sudara na istim datotekama, nijedna nova datoteka ne
prelazi granicu veličine, a Leon je prošao ADR-ove 🟨. Nemaju dodira s pravilima igre: svaka cigla dokazuje „nulta promjena ponašanja".

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| K0-1 | **CI Ubuntu 26 (HITNO do 19.10.)** | `ubuntu-latest` prelazi na Ubuntu 26 od 19.10. ([BACKLOG.md:211](../../records/BACKLOG.md)): provjeriti `ci.yml` (Postgres, sqlx-cli, Rust 1.98.1, pnpm), po potrebi pin `ubuntu-24.04`; usput Node 20 akcije ([:210](../../records/BACKLOG.md)) | `gh run list` zelen na `main` i na grani prije 19.10. | — | 1,5 | P1 | — |
| K0-2 | Pravila rada s agentima i područja | `docs/workflow/AGENTI.md` + `check:docs`: područja `igra` i `platforma` uz `kod`/`dokumentacija` (pravilo 3 u [docs/README.md](../../README.md)), „jedan aktivni plan po području"; tablica vlasništva po datotekama (`game/*`, `api/schema.rs`, migracije, `app/platform`…); max 2 Rust izvođača | `check:docs` pada na planu s nepoznatim područjem; test u `scripts/*.test.mjs` | ADR-038 | 3 | P1 | — |
| K0-3 | Brana veličine koda | `scripts/check-kod.mjs` (`check:kod`): upozorenje > 600, tvrda > 1000 redaka za `.rs/.ts/.tsx/.mjs` (broji kao `docs-granice.mjs`), izuzeci: generirano, `openapi.json`, fixtures, `content/**/*.json`; **ratchet popis duga** (dopuštena gornja granica po datoteci, ne smije rasti); CI korak u poslu `web` ([BACKLOG.md:204](../../records/BACKLOG.md)) | `node --test`: 1001 redak pada, izuzetak prolazi, ratchet raste pada | — | 2 | P1 | brana > 1000: potvrda (BACKLOG „Otvoreno za Leona") |
| K0-4 | `api/src/schema.rs` → mapa `schema/` | 1687 redaka u `codes`, `plan`, `hotel`, `game`, `season`, `situations`, `feed`; `mod.rs` re-exportira pa putanje `schema::X` ostaju; `mod tests` uz svoj dio | `cargo test -p api`; `check:api` daje ISTI `openapi.json` (`git diff` prazan) | K0-3 | 3,5 | P1 | — |
| K0-5 | Mentor u mape | `game/src/mentor.rs` (1367) i `season_mentor.rs` (856) → `mentor/{mod,types,words,numbers,facts,fallback}.rs`, `season_mentor/{…}` sa zajedničkim `words.rs`; javni API (`DebriefFacts`, `debrief_facts`, `fallback_debrief`, `wants_ai_mentor`) ostaje u `mod.rs` | postojeći testovi nepromijenjeni i zeleni; diff samo premještanje | K0-3 | 5 | P1 | — |
| K0-6 | Testovi mentora po temama | `tests/mentor.rs` (1670) i `season_mentor.rs` (605) → po temama (činjenice, fallback, osoblje, tjedni krug, odlasci, plaće) + `tests/common`; svaka datoteka u `tests/` je zaseban binary | broj testova prije = poslije (`cargo test -p game -- --list`) | K0-5 | 3 | P1 | — |
| K0-7 | Arhitektura platforme u `docs/` | prenijeti `platforma-arhitektura.md` iz ledgera (`.superpowers/sdd/…`, nije u gitu) u `docs/architecture` **uz prepis po ispravku** (razred = jedna igra; sesije; Railway Hobby; račun + lozinka). Zastarjelo: §1 t.5, §2.1–2.6, §4.2 t.5 i L7, §6.2b, §6.4, §6.6, §7.1, §7.4, §8, §9.1 L1/L2/L7; ARHITEKTURA §9.3, §11, §12, §14, §16; MODEL_V2 §7.1; ADR-015 t.14, ADR-016 t.3; granica 300 redaka → indeks + mapa | `check:docs` zeleno; nijedna rečenica o „partiji po studentu" ni „i mod 4" ni „3 robota u razredu" | ADR-037 | 6 | P1 | — |
| K0-8 | Leonova sjednica ADR-ova 🟨 | ~1 h razgovora, 18 ADR-ova u 4 grupe (jezgra/model: 026, 013, 029, 031–033 · platforma: 003, 015, 017, 008, 036 · AI/sadržaj: 019, 023, 025 · UI i proces: 022, 027, 035, 030); priprema ~1 h; popis: [DECISIONS.md](../../records/DECISIONS.md) | statusi ✅/izmijenjeno; `docs:adr` svjež | — | 2 | P2 | **Leon, ~1 h** |

Raspored: K0-1, K0-2, K0-3, K0-7 i K0-8 odmah (ne diraju kod igre). **K0-4, K0-5 i K0-6 tek NAKON spajanja grana obnove i situacija u
`sloj1`** (L1-10…L1-12): obje grane mijenjaju `schema.rs` (obnova ga diže na 1889 redaka) i mentora, pa bi rez prije spajanja stvorio velike
sukobe. Zatim K0-4 prije F1-11/F1-12 (platforma dira `api`), K0-5/6 prije prve faze koja dira mentora (M-1, LJUDI); K0-4, K0-5 i F1-10 ne paralelizirati (isti Rust objekti, najviše 2 izvođača). Ostala velika datoteka ispod 1000: `balance/lib.rs` 710,
`department.rs` 685 (uz E), `content/lib.rs` 684, `game/model.rs` 671 — drže se na ratchetu; `situation_templates.json` ~3630 je podatak na grani s1-sit.

---
## FAZA KVALITETA — stalne cigle poliranja (uz faze; 9 cigli, 25,5 h)

**Gotovo kad:** svaka faza igre/platforme završava istim ritualom, a rupe u testovima su zatvorene prije faze koja ih dira. Pravilo na kraju svake faze:
„datoteke koje je faza dirnula i narasle su iznad 600 → rez", provjereno branom K0-3 (upozorenje), ne sjećanjem. Testovi, provjeravatelj i balans se **nikad ne režu**.

| id | cigla | što se radi | dokaz | uz fazu | h | P | Leon |
|---|---|---|---|---|---|---|---|
| Q-1 | Mjerenje pokrivenosti | `cargo llvm-cov` (lokalno) + `vitest --coverage`; polazna brojka u `docs/`, prag tek poslije | izvještaj po paketu | prije RAZREDA U JEZGRI | 2 | P1 | — |
| Q-2 | `DeptInputs` umjesto `#[allow(too_many_arguments)]` | `sim-hotel-v2/src/department.rs:486`; `type_complexity` u `balance/lib.rs:372` | oracle/zlatne vrijednosti nepromijenjene; isti `--check` izlaz | uz LJUDI | 2,5 | P3 | — |
| Q-3 | Playwright E2E preglednik → API | dev server + API + test baza; start → plan → situacija → zatvaranje; AI preko `FakeAi`; jedna cijela tjedna sezona | zeleno u CI-ju ili `pnpm e2e` | prije SPREMNO ZA INTERNET izlaska | 6 | P2 | — |
| Q-4 | Testovi ekrana bez testa | `StartScreen`, `SituationsScreen`, `CloseMonth`, `NotFound`, `AvatarGallery`, `api/queries.ts` (341 r.) | „renderira, greška, klik šalje poziv" | uz svaku fazu koja dira ekran (StartScreen prije RAZREDA) | 3 | P2 | — |
| Q-5 | Mrtve ovisnosti i audit | `cargo machete`, `pnpm audit` kao upozorenje u CI-ju | izlaz alata | prije izlaska na internet | 2 | P2 | — |
| Q-6 | `ScrollTable` + `engine/rng.rs` | 3× kopirani `biome-ignore` (KpiScreen, DepartmentHistory, PlTable) → jedna komponenta; `unwrap`→`checked_*` (`rng.rs:54-60`) | testovi ekrana + snimke | filler | 2 | poslije | — |
| Q-7 | Dug uz fazu platforme | „Railway priprema" (konfiguracija, SIGTERM, TLS, uloga baze: [BACKLOG.md:81-92](../../records/BACKLOG.md)); dupli izvori istine ([:39](../../records/BACKLOG.md), [:43](../../records/BACKLOG.md), [:60](../../records/BACKLOG.md)); provjere kataloga ([:18](../../records/BACKLOG.md), [:31-32](../../records/BACKLOG.md)) | vidi platforma.md | uz SPREMNO ZA INTERNET i PILOT | uračunato | — | — |
| Q-8 | Sitni nalazi brane dokumentacije | 5 točaka ([BACKLOG.md:214](../../records/BACKLOG.md)); Sokratis `docs` mjeri staro pravilo ([:200](../../records/BACKLOG.md)) | `check:docs` + Sokratis | uz kraj sesije | 3 | poslije | pokrenuti `check:docs` umjesto vlastitog pravila? |
| Q-9 | Testovi koji fale | popis a–h ([BACKLOG.md:47](../../records/BACKLOG.md)); AI sloj ([:59](../../records/BACKLOG.md)); `/situations.unanswered` ([:77](../../records/BACKLOG.md)); pending/500/null ([:68](../../records/BACKLOG.md)) | testovi zeleni | uz PROFESOR i PILOT | 5 | P3 | — |

Postojeće brane (ne dirati, samo održavati): `cargo fmt`/`clippy -D warnings`, `check-core-deps`, `probe-mutacija` (ADR-009), `check:api`, `check:docs`,
`oracle-jezgra.mjs`, CI `rust` i `web`, `deny.toml`, Biome + `tsc`. Pokrivenost: najrjeđe `app` (3 test-datoteke za 2311 LOC), `server` (2 testa), web ekrani u Q-4.

---
## FAZA KVALITETA — prave greške prije siječnja (P3)

**Gotovo kad:** svaka stavka ispod je ili popravljena ili zapisana kao svjesno odgođena; popis se briše iz BACKLOG-a kad se popravi (BACKLOG briše riješeno).
Pokazivači su na [BACKLOG.md](../../records/BACKLOG.md) (broj retka po stanju 13.10.); cigla se razrađuje u planu faze u koju je stavka svrstana.

| prije | stavka | BACKLOG | gdje se rješava | P |
|---|---|---|---|---|
| 19.10. | `ubuntu-latest` → Ubuntu 26 | [:211](../../records/BACKLOG.md) | K0-1 | P1 |
| P1 | `hr.ts` bez `problem.game_outdated` / `invalid_settings` | [:119](../../records/BACKLOG.md) | ZAVRŠETAK SLOJA 1 | P1 |
| P1 | mentor kritizira odgovor koji student nije dao (`DebriefSituation.answered`) | [:58](../../records/BACKLOG.md) | L1-3 | P1 |
| P1 | mentor kruga sprema odrezan AI tekst kao `source: ai` | [:127](../../records/BACKLOG.md) | ZAVRŠETAK SLOJA 1 | P1 |
| P1 | S3 bez opisa situacije (polje u API-ju) | [:162](../../records/BACKLOG.md) | L1-4 | P1 |
| P1 | redak Equipment/Maintenance miješa −30 % Smještaja | [:116](../../records/BACKLOG.md) | L1-10 | P1 |
| P1 | indeks likvidnosti krivo skalira jednokratni trošak u tjednu sa zapošljavanjem | [:121](../../records/BACKLOG.md) | L1-13 (balans) | P1 |
| P1 | lom „7.\|lipnja", poruke bez `h1`; izbornik `md:top-28` | [:125](../../records/BACKLOG.md), [:164](../../records/BACKLOG.md) | L1-6 | P1 |
| P1 | sezonski ugovor do datuma troši tjedni korak | [:139](../../records/BACKLOG.md) | LJUDI (L2a-4) | P3 |
| P1 | probne v2 igre bez zaštite verzijom jezgre | [:114](../../records/BACKLOG.md) | L1-10 (jedno podizanje `CORE_VERSION`) | P1 |
| P2 | `ctrl_c` ignoriran (`server/main.rs:157`) | [:85](../../records/BACKLOG.md) | F0-2 | P2 |
| P2 | `failed` posao ostavlja rundu u „closing" zauvijek, profesor ne vidi (**najopasnije za pilot**) + ručni „ponovi obračun" | [:70](../../records/BACKLOG.md), [:44](../../records/BACKLOG.md) | PROFESOR (uz F2-2) | P3 |
| P2 | `check:api` pada na staged promjenama (alat) | [:41](../../records/BACKLOG.md) | Q-8 | P1 |
| latentno | siroče gost i čišćenje nakon 7 dana ([:38](../../records/BACKLOG.md), [:189](../../records/BACKLOG.md)); serijski radnik ([:61](../../records/BACKLOG.md)); deadlock expiry ([:81](../../records/BACKLOG.md)); latencija Opusa vs rok 120 s ([:62](../../records/BACKLOG.md)) | | F0-8, G-5 | P4 |

Pokazivači na stavke koje su zastarjele ili se dupliciraju (npr. [:25](../../records/BACKLOG.md), [:150](../../records/BACKLOG.md)) čisti sesija `/kraj`, ne ovaj plan.

---
## FAZA KVALITETA — poslije (smjer)

Mjerenje pokrivenosti postaje prag u CI-ju nakon polazne brojke (Q-1); `cargo machete` i `pnpm audit` iz upozorenja u prepreku; rez ostalih datoteka s ratchet popisa
(`balance/lib.rs`, `department.rs`, `content/lib.rs`, `model.rs`) kad ih faza dirne; i18n sučelja (en.ts, format brojeva) je u [platforma.md](./platforma.md).
Popis ADR-ova 🟨 se ponovno prolazi prije pilota.

## Sažetak ([PROC])

| faza | cigli | h |
|---|---|---|
| 0 TEMELJ RADA | 8 | 26,5 |
| stalne cigle poliranja | 9 (Q-7 uračunat drugdje) | 25,5 |
| prave greške prije siječnja | 14 stavki (pokazivači) | rješavaju se unutar faza igre i platforme |

Linije reza: kvaliteta se ne reže na račun dokaza; ako zapne, odgađaju se Q-6, Q-8 i prag pokrivenosti.
