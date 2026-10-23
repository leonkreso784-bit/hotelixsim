# HotelixSim — roadmap

**Status:** 🟩 živ (faze projekta i put do pilota; nije plan taskova) · **Otvoren:** 2026-10-03
Što gradimo: [VIZIJA.md](../product/VIZIJA.md) · kako: [ARHITEKTURA.md](../architecture/ARHITEKTURA.md) i
[MODEL_V2.md](../architecture/MODEL_V2.md) · način rada: [ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)

> Faza ima ime, ne slovo. Svaka faza ima uvjet „gotovo kad…" izražen kroz ono što **čovjek može napraviti**,
> nikad kroz „test je zelen". Čvrsto se planira samo blisko (TEMELJ RADA, ZAVRŠETAK SLOJA 1, lokalni dio SPREMNO ZA
> INTERNET). Sve iza toga je **smjer**: raspored se ponovno slaže kad profesor vidi igru. Sati su [PROC].

## Faze projekta

| faza | gotovo kad… |
|---|---|
| **TEMELJI** ✅ | …arhitektura i dizajn igre su zapisani. Arhitektura v2 i domena DEMO-a još čekaju Leonovu potvrdu (ADR-003, BACKLOG) |
| **DEMO** ✅ | …igra radi u pregledniku: hotel, situacije, razgovor s likom, AI ocjena, mentor, napredak uz tri robota (koraci 1–5 i sloj 0, [CHANGELOG](../records/CHANGELOG.md), planovi u [archive/](../archive/2026-10-10-ekrani.md)) |
| **PREZENTACIJA** | …profesor je vidio igru i dobio [pitanja](../product/PITANJA_ZA_PROFESORE.md) |
| **VERZIJA ZA PROFESORE** | …profesori na internetu probaju igru s razredom (cilj sredina siječnja 2027., poželjno, ne nužno) — prekretnica P3 |
| **PILOT** | …studenti ljetnog semestra igraju jedni protiv drugih, na satu ili kroz semestar, a profesor prati razred — prekretnica P4 |
| **VIZIJA** | zajedničko tržište gostiju, više hotela i tržišta ([VIZIJA.md](../product/VIZIJA.md) §8) |

## Put do pilota: dva kolosijeka

Kolosijeci su podijeljeni **po datotekama** ([ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)):
**IGRA** drži srce igre (jezgra, `crates/game`, sadržaj, balans), a **PLATFORMA** server, bazu, prijavu i ekrane
razreda. Pravila razreda: [ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md).
Cigle po fazama (zaliha; kartica se razrađuje u planu faze kad faza krene):

| kolosijek | cigle |
|---|---|
| kvaliteta i TEMELJ RADA | [§ kvaliteta](#roadmap--kolosijek-kvaliteta) |
| IGRA | [§ igra](#roadmap--kolosijek-igra) |
| PLATFORMA | [§ platforma](#roadmap--kolosijek-platforma) |

**Prije oba kolosijeka: TEMELJ RADA** (~26 h). Pravila rada s agentima, popravak CI-ja (Ubuntu 26 od 19.10.), brana
veličine koda, prijenos platformskog nacrta iz ledgera u `docs/` i Leonova sjednica ADR-ova 🟨. Razdvajanje
`api/src/schema.rs` i mentora ide tek kad se grane obnove i situacija spoje u `sloj1` (obje te datoteke mijenjaju). **Gotovo kad:** oba kolosijeka mogu krenuti bez ijedne zajedničke velike datoteke.

| okvirno | IGRA | PLATFORMA | prekretnica: što čovjek može |
|---|---|---|---|
| listopad | **ZAVRŠETAK SLOJA 1** (~50 h): situacije u igri, obnova, kraj sezone, spajanje tri grane, zajednički balans, profesorov račun dobiti | **SPREMNO ZA INTERNET** (~48 h, lokalno, 0 €): postavke, gašenje, zaštita, server servira web, kopije, granica AI troška, brza igra za 30 igrača | **P1** · Leon odigra cijelu sezonu Sloja 1 sa situacijama, obnovom i krajem sezone |
| studeni | **RAZRED U JEZGRI** (~45 h): N studentskih hotela, profesor otvara krug, obračun po hotelu | Platforma kreće u studenom usporedo s fazom C — Leon 16.10., [nacrt plana](./2026-11-platforma.md). **RAZRED** (~116 h): računi studenata (korisničko ime i lozinka), kod igre, čekaonica, lista kolega, raspored otvaranja i rokovi, ekrani studenta i profesora | **P2** · profesor na internetu stvori igru, studenti uđu kodom i igraju jedni protiv drugih (izlazak samo uz Leonov OK, ~5–10 €/mj) |
| prosinac – siječanj | **LJUDI** (~64 h) · **RAZGOVORI** (~35 h) · **NATJECANJE ZA LJUDE** (~45 h) | **PROFESOR** (~83 h do P3): pregled razreda, detalj studenta, pregled AI ocjena, profesor dodaje vlastite situacije | **P3** · profesori probaju verziju s ljudima i razgovorima i vode razred |
| veljača – ožujak | **JEZICI I AGENCIJA** · **VRSTE SOBA I CIJENE** · **RECENZIJE** (~85 h) | **PILOT** (~95 h + ostatak PROFESORA): GDPR, opterećenje 30 studenata, domena, ispravak AI ocjene, izvoz | **P4** · studentski pilot |
| poslije pilota | sobe (tlocrt), Hoteli u okolici, pošta s agencijama, događaji, dobavljači, novine; igra na engleskom (AI i sadržaj) | generator hotela (puni banku hotela, 17.10.), AAI@EduHr (čim fakultet odobri), sučelje na engleskom | — |

**Do P3 je ~525 h** (IGRA ~240, PLATFORMA ~250, kvaliteta ~40), više od okvirnih 425 h iz razgovora: liste su
detaljnije. Zato su linije reza dio plana, ne iznimka.

**Linije reza** ako tempo zapne, redom: vlastite situacije profesora P3 → P4 · razgovori samo tri vrste ·
NATJECANJE ZA LJUDE P3 → P4. **Nikad se ne režu** testovi, provjeravatelj i balans.

## Pilot → produkcija i banka hotela (Leon 17.10.2026., [ADR-056](../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md))

**Leon:** *„ovo što već imamo je pilot, a ovo što gradimo je već produkcijska platforma; spremamo se za profesorov DA jer znam
da će biti da."* Standard „gotovo" zato nije „radi na Leonovu računalu" nego **40 studenata igra tri mjeseca bez Leona uz
računalo**. Igra bogata sadržajem bez prijave i hostinga nije produkcija.

**🟨 Redoslijed (prijedlog voditelja, Leon ga nije izričito potvrdio):**
1. **L1-13 ✅ → L1-13b ✅ → L1-14 → L1-15** (kraj popravka temelja, faza B), zatim priprema i deploy pilota (Railway, [ADR-062](../records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md) t. 9).
2. **PLATFORMA do najmanje igrive razredne igre:** prijava i pristup (računi, profesorovi vlastiti podaci za prijavu), hosting
   s **izmjerenim** troškom (uz AI trošak po igri, AI-M), oporavak od pada na pravoj bazi (kopije i proba vraćanja), profesorovo
   sučelje u najmanjem obliku (otvori krug, vidi hotele, vidi razgovore), GDPR i brisanje. Cigle: [platforma.md](#roadmap--kolosijek-platforma)
   i [nacrt plana platforme](./2026-11-platforma.md); uz to RAZRED U JEZGRI ([igra.md](#roadmap--kolosijek-igra)).
3. **Tek onda** bogatstvo igre: sučelje soba, voditelji i obuka, smještaj radnika, **BANKA HOTELA**, tržište, potrošnja hotela
   ([igra.md](#roadmap--kolosijek-igra) „Nove cigle 17.10.").

**BANKA HOTELA** (Leon: *„profesor će moći dodijeliti hotele studentima koje želi"*; rješava A10 „da studenti ne dobiju uvijek
isti hotel sa 120 soba"): popis gotovih, pregledanih hotela raznih veličina. 🟨 Mjesto: nakon L1-13 (i L1-14/15), prije kraja
faze C; zamjenjuje GENERATOR HOTELA kao prvi korak (generator kasnije samo puni banku). Tri komada: **sadržaj** (BH-S: prvo
istraživanje pravih hotela i plaća, A10 i A12, pa 10–15 hotela 4* u serijama koje Leon pregleda i pečati, kao banka ljudi),
**jezgra** (BH-J: ključ hotela kao tekst, referenca napretka i balans po hotelu; pravednost 80 prema 160 soba kroz indekse po
sobi ili udjelu, [PLATFORMA §4.3](../architecture/PLATFORMA/04-generirani-hoteli.md)), **platforma** (BH-P: dodjela po
studentu ili nasumce, tek s razredom). Sati: 🟨 [PROC], procjenu daje kartica.

## Gdje smo

Aktivan plan: [Pilot s profesorom](./2026-10-24-pilot-s-profesorom.md) (24.10., [ADR-066](../records/DECISIONS/ADR-066-pilot-s-profesorovim-tempom-grupa-igara.md): testovi, AI temelj, grupa igara, L1-15, pilot). Prethodni: [Popravak temelja](../archive/2026-10-14-popravak-temelja.md) (14.10., u arhivi; [ADR-043](../records/DECISIONS/ADR-043-popravak-temelja-ne-gradnja-ispocetka.md):
faze A → B → C, samo popravak dok nije gotov; preuzima ostatak ZAVRŠETKA SLOJA 1). [Sloj 1](../archive/2026-10-10-v2-sloj1.md) je pauziran.
Nakon popravka: TEMELJ RADA i SPREMNO ZA INTERNET, pa RAZRED (🟨 17.10.: platforma do najmanje igrive razredne igre prije novih mehanika igre, odjeljak iznad).
Otvorena pitanja za Leona žive na jednom mjestu: [BACKLOG.md](../records/BACKLOG.md) → „Otvoreno za Leona".

## ROADMAP — kolosijek KVALITETA

**Što je ovo:** ZALIHA CIGLI za dug i poliranje: temelj rada (faza 0), stalne cigle poliranja koje se vežu uz faze igre i platforme, i popis pravih
grešaka koje valja popraviti prije siječnja. Faze iza TEMELJA RADA su **smjer** koji se ponovno slaže nakon što profesor vidi igru; kartica cigle se
razrađuje u planu faze kad faza krene. Sati su [PROC]. Kolosijeci se dijele po **datotekama** ([ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)):
kvaliteta dira alate, CI, dokumentaciju i **čisto premještanje** koda (nulta promjena ponašanja). Susjedi: [igra.md](#roadmap--kolosijek-igra),
[platforma.md](#roadmap--kolosijek-platforma), [ROADMAP.md](./ROADMAP.md). Popis stavki duga je u [BACKLOG.md](../records/BACKLOG.md); ovdje su samo pokazivači (po imenu stavke, jer se retci pomiču), ne prepis. Kod ~58 000 redaka, 0 TODO, 2 `#[allow]`, 15 `expect("razlog")` — stanje je dobro, dug je u veličini datoteka i rupama u testovima.

**Prekretnice (stupac P):** P1 kraj listopada · P2 kraj studenog · P3 sredina siječnja · P4 ožujak · poslije.

---
### FAZA KVALITETA — 0 TEMELJ RADA (prije P1/P2; 8 cigli, ~26 h)

**Gotovo kad:** CI ne puca zbog novog Ubuntua, dva kolosijeka (igra, platforma) rade usporedno bez sudara na istim datotekama, nijedna nova datoteka ne
prelazi granicu veličine, a Leon je prošao ADR-ove 🟨. Nemaju dodira s pravilima igre: svaka cigla dokazuje „nulta promjena ponašanja".

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| K0-1 ✅ | **CI Ubuntu 26 (HITNO do 19.10.)** (✅ 16c4ac9 + b495c8d; CI 38446984164 zelen na `ubuntu-24.04` 14.10.) | `ubuntu-latest` prelazi na Ubuntu 26 od 19.10. (BACKLOG „Ubuntu 26“): provjeriti `ci.yml` (Postgres, sqlx-cli, Rust 1.98.1, pnpm), po potrebi pin `ubuntu-24.04`; usput Node 20 akcije (BACKLOG „Node 20 akcije“) | `gh run list` zelen na `main` i na grani prije 19.10. | — | 1,5 | P1 | — |
| K0-2 ✅ | Pravila rada s agentima i područja (✅ a910ea2) | `docs/workflow/AGENTI.md` + `check:docs`: područja `igra` i `platforma` uz `kod`/`dokumentacija` (pravilo 3 u [docs/README.md](../README.md)), „jedan aktivni plan po području"; tablica vlasništva po datotekama (`game/*`, `api/schema.rs`, migracije, `app/platform`…); max 2 Rust izvođača | `check:docs` pada na planu s nepoznatim područjem; test u `scripts/*.test.mjs` | ADR-038 | 3 | P1 | — |
| K0-3 ✅ | Brana veličine koda (✅ 9d830ba) | `scripts/check-kod.mjs` (`check:kod`): upozorenje > 600, tvrda > 1000 redaka za `.rs/.ts/.tsx/.mjs/.js` (broji kao `docs-granice.mjs`), izuzeci: samo generirano (`schema.d.ts`); `openapi.json` i `content/**/*.json` nisu kod; `.claude/` se ne broji; **ratchet popis duga** samo za tri datoteke preko 1000 (dopuštena gornja granica, ne smije rasti); CI korak u poslu `web` (u `ci.yml`); granicu i popis Leon još nije potvrdio ([ADR-039](../records/DECISIONS/ADR-039-brana-velicine-koda-upozorenje-600-granica.md)) | `node --test`: 1001 redak pada, izuzetak prolazi, ratchet raste pada | — | 2 | P1 | potvrda granice 600/1000 i popisa duga (BACKLOG „Otvoreno za Leona“, ADR-039) |
| K0-4 | `api/src/schema.rs` → mapa `schema/` | 1687 redaka u `codes`, `plan`, `hotel`, `game`, `season`, `situations`, `feed`; `mod.rs` re-exportira pa putanje `schema::X` ostaju; `mod tests` uz svoj dio | `cargo test -p api`; `check:api` daje ISTI `openapi.json` (`git diff` prazan) | K0-3 | 3,5 | P1 | — |
| K0-5 | Mentor u mape | `game/src/mentor.rs` (1367) i `season_mentor.rs` (856) → `mentor/{mod,types,words,numbers,facts,fallback}.rs`, `season_mentor/{…}` sa zajedničkim `words.rs`; javni API (`DebriefFacts`, `debrief_facts`, `fallback_debrief`, `wants_ai_mentor`) ostaje u `mod.rs` | postojeći testovi nepromijenjeni i zeleni; diff samo premještanje | K0-3 | 5 | P1 | — |
| K0-6 | Testovi mentora po temama | `tests/mentor.rs` (1670) i `season_mentor.rs` (605) → po temama (činjenice, fallback, osoblje, tjedni krug, odlasci, plaće) + `tests/common`; svaka datoteka u `tests/` je zaseban binary | broj testova prije = poslije (`cargo test -p game -- --list`) | K0-5 | 3 | P1 | — |
| K0-7 ✅ | Arhitektura platforme u `docs/` (✅ 2ab1c4b/e3e9ebe) | prenijeti `platforma-arhitektura.md` iz ledgera (`.superpowers/sdd/…`, nije u gitu) u `docs/architecture` **uz prepis po ispravku** (razred = jedna igra; sesije; Railway Hobby; račun + lozinka). Zastarjelo: §1 t.5, §2.1–2.6, §4.2 t.5 i L7, §6.2b, §6.4, §6.6, §7.1, §7.4, §8, §9.1 L1/L2/L7; ARHITEKTURA §9.3, §11, §12, §14, §16; MODEL_V2 §7.1; ADR-015 t.14, ADR-016 t.3; granica 300 redaka → indeks + mapa | `check:docs` zeleno; nijedna rečenica o „partiji po studentu" ni „i mod 4" ni „3 robota u razredu" | ADR-037 | 6 | P1 | — |
| K0-8 | Leonova sjednica ADR-ova 🟨 | ~1 h razgovora, 18 ADR-ova u 4 grupe (jezgra/model: 026, 013, 029, 031–033 · platforma: 003, 015, 017, 008, 036 · AI/sadržaj: 019, 023, 025 · UI i proces: 022, 027, 035, 030); priprema ~1 h; popis: [DECISIONS.md](../records/DECISIONS.md) | statusi ✅/izmijenjeno; `docs:adr` svjež | — | 2 | P2 | **Leon, ~1 h** |

Raspored: K0-1, K0-2, K0-3, K0-7 i K0-8 odmah (ne diraju kod igre). **K0-4, K0-5 i K0-6 tek NAKON spajanja grana obnove i situacija u
`sloj1`** (L1-10…L1-12): obje grane mijenjaju `schema.rs` (obnova ga diže na 1889 redaka) i mentora, pa bi rez prije spajanja stvorio velike
sukobe. Zatim K0-4 prije F1-11/F1-12 (platforma dira `api`), K0-5/6 prije prve faze koja dira mentora (M-1, LJUDI); K0-4, K0-5 i F1-10 ne paralelizirati (isti Rust objekti, najviše 2 izvođača). Ostala velika datoteka ispod 1000: `balance/lib.rs` 710,
`department.rs` 685 (uz E), `content/lib.rs` 684, `game/model.rs` 671 — nisu na popisu duga: ispod 1000 smiju rasti uz upozorenje (> 600), a brana pada tek preko 1000; `situation_templates.json` ~3630 je podatak na grani s1-sit.

---
### FAZA KVALITETA — stalne cigle poliranja (uz faze; 9 cigli, 25,5 h)

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
| Q-7 | Dug uz fazu platforme | „Railway priprema" (konfiguracija, SIGTERM, TLS, uloga baze: BACKLOG, odjeljak „prije Railwaya“); dupli izvori istine (BACKLOG „ErrorCode::ALL“, BACKLOG „unanswered na dva mjesta“, BACKLOG „AssessInput na dva mjesta“); provjere kataloga (BACKLOG „provjera kataloga: poruke granica“, [:31-32](../records/BACKLOG.md)) | vidi platforma.md | uz SPREMNO ZA INTERNET i PILOT | uračunato | — | — |
| Q-8 | Sitni nalazi brane dokumentacije | 5 točaka (BACKLOG „sitni nalazi brane dokumentacije“); Sokratis `docs` mjeri staro pravilo (BACKLOG „Sokratis mjeri staro pravilo“) | `check:docs` + Sokratis | uz kraj sesije | 3 | poslije | pokrenuti `check:docs` umjesto vlastitog pravila? |
| Q-9 | Testovi koji fale | popis a–h (BACKLOG „testovi koji fale (dnevnik)“); AI sloj (BACKLOG „testovi koji nedostaju (ai-sloj)“); `/situations.unanswered` (BACKLOG „HTTP test /situations.unanswered“); pending/500/null (BACKLOG „pending/500/null u ekranima“) | testovi zeleni | uz PROFESOR i PILOT | 5 | P3 | — |

Postojeće brane (ne dirati, samo održavati): `cargo fmt`/`clippy -D warnings`, `check-core-deps`, `probe-mutacija` (ADR-009), `check:api`, `check:docs`,
`oracle-jezgra.mjs`, CI `rust` i `web`, `deny.toml`, Biome + `tsc`. Pokrivenost: najrjeđe `app` (3 test-datoteke za 2311 LOC), `server` (2 testa), web ekrani u Q-4.

---
### FAZA KVALITETA — prave greške prije siječnja (P3)

**Gotovo kad:** svaka stavka ispod je ili popravljena ili zapisana kao svjesno odgođena; popis se briše iz BACKLOG-a kad se popravi (BACKLOG briše riješeno).
Pokazivači su na [BACKLOG.md](../records/BACKLOG.md) (po imenu stavke); cigla se razrađuje u planu faze u koju je stavka svrstana.

| prije | stavka | BACKLOG | gdje se rješava | P |
|---|---|---|---|---|
| 19.10. | `ubuntu-latest` → Ubuntu 26 | BACKLOG „Ubuntu 26“ | K0-1 | P1 |
| P1 | `hr.ts` bez `problem.game_outdated` / `invalid_settings` | BACKLOG „hr.ts bez problem.game_outdated“ | ZAVRŠETAK SLOJA 1 | P1 |
| P1 | mentor kritizira odgovor koji student nije dao (`DebriefSituation.answered`) | BACKLOG „DebriefSituation.answered“ | L1-3 | P1 |
| P1 | mentor kruga sprema odrezan AI tekst kao `source: ai` | BACKLOG „mentor sprema odrezan AI tekst“ | ZAVRŠETAK SLOJA 1 | P1 |
| P1 | S3 bez opisa situacije (polje u API-ju) | BACKLOG „S3 bez opisa situacije“ | L1-4 | P1 |
| P1 | redak Equipment/Maintenance miješa −30 % Smještaja | BACKLOG „redak Equipment/Maintenance“ | L1-10 | P1 |
| P1 | indeks likvidnosti krivo skalira jednokratni trošak u tjednu sa zapošljavanjem | BACKLOG „indeks likvidnosti“ | L1-13 (balans) | P1 |
| P1 | lom „7.\|lipnja", poruke bez `h1`; izbornik `md:top-28` | BACKLOG „lom teksta s točkom i razmakom“, BACKLOG „izbornik md:top-28“ | L1-6 | P1 |
| P1 | sezonski ugovor do datuma troši tjedni korak | BACKLOG „sezonski ugovor do datuma“ | LJUDI (L2a-4) | P3 |
| P1 | probne v2 igre bez zaštite verzijom jezgre | BACKLOG „probne v2 igre“ | L1-10 (jedno podizanje `CORE_VERSION`) | P1 |
| P2 | `ctrl_c` ignoriran (`server/main.rs:157`) | BACKLOG „ctrl_c ignoriran“ | F0-2 | P2 |
| P2 | `failed` posao ostavlja rundu u „closing" zauvijek, profesor ne vidi (**najopasnije za pilot**) + ručni „ponovi obračun" | BACKLOG „failed posao u closing“, BACKLOG „ručni ponovi obračun“ | PROFESOR (uz F2-2) | P3 |
| P2 | `check:api` pada na staged promjenama (alat) | BACKLOG „check:api na staged“ | Q-8 | P1 |
| latentno | siroče gost i čišćenje nakon 7 dana (BACKLOG „siroče gost“, BACKLOG „čišćenje gosta“); serijski radnik (BACKLOG „serijski radnik“); deadlock expiry (BACKLOG „deadlock expiry“); latencija Opusa vs rok 120 s (BACKLOG „latencija Opusa“) | | F0-8, G-5 | P4 |

Stavke BACKLOG-a koje su zastarjele ili se dupliciraju (npr. „duplicirani ključevi sadržaja“, „pitanja za profesore“) čisti sesija `/kraj`, ne ovaj plan.

---
### FAZA KVALITETA — poslije (smjer)

Mjerenje pokrivenosti postaje prag u CI-ju nakon polazne brojke (Q-1); `cargo machete` i `pnpm audit` iz upozorenja u prepreku; rez datoteka s popisa duga i onih iznad 600
(`balance/lib.rs`, `department.rs`, `content/lib.rs`, `model.rs`) kad ih faza dirne; i18n sučelja (en.ts, format brojeva) je u [platforma.md](#roadmap--kolosijek-platforma).
Popis ADR-ova 🟨 se ponovno prolazi prije pilota.

### Sažetak ([PROC])

| faza | cigli | h |
|---|---|---|
| 0 TEMELJ RADA | 8 | 26,5 |
| stalne cigle poliranja | 9 (Q-7 uračunat drugdje) | 25,5 |
| prave greške prije siječnja | 14 stavki (pokazivači) | rješavaju se unutar faza igre i platforme |

Linije reza: kvaliteta se ne reže na račun dokaza; ako zapne, odgađaju se Q-6, Q-8 i prag pokrivenosti.

## ROADMAP — kolosijek IGRA

**Što je ovo:** ZALIHA CIGLI po fazama za jedan kolosijek (igra: srce igre, `crates/game`, `sim-hotel-v2`, `content`, sadržaj i
ekrani igre). Faze iza ZAVRŠETKA SLOJA 1 su **smjer** koji se ponovno slaže nakon što profesor vidi igru; kartica cigle
se razrađuje u planu faze kad faza krene. Sati su [PROC] (rad agenata + pregled). Podjela kolosijeka je po **datotekama**,
ne po temi ([ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)): sve što dira jezgru igre je ovdje, i razredne
cigle. Susjedi: [platforma.md](#roadmap--kolosijek-platforma), [kvaliteta.md](#roadmap--kolosijek-kvaliteta); pregled faza: [ROADMAP.md](./ROADMAP.md).
Razred i obračun: [ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md). Spec: [MODEL_V2.md](../architecture/MODEL_V2.md).

**Prekretnice (stupac P):** P1 kraj listopada (cijela sezona Sloja 1 igriva) · P2 kraj studenog (profesor na internetu stvori igru,
studenti uđu kodom) · P3 sredina siječnja (verzija za profesore: ljudi, razgovori, profesor vodi razred i dodaje situacije) ·
P4 ožujak (studentski pilot) · poslije = iza pilota.

**Pravilo za SVE cigle jezgre:** `equivalence.rs` zelen, `v1_frozen` + `fixtures_sealed` zeleni, v1 balans bajt-isti, v2
`--seeds 1000 --check` (tjedan i mjesec) prije/poslije s opisanim pomakom; `CORE_VERSION` se podiže jednom po valu. Rust: najviše 2 izvođača istodobno.

---
### FAZA IGRA — ZAVRŠETAK SLOJA 1 (P1; 17 cigli, ~49,5 h)

**Gotovo kad:** Leon odigra cijelu sezonu u pregledniku (situacije, obnova, kraj sezone) i `main` ima sve tri grane spojene.

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| L1-1 | ✅ S-4 dovršetak | probe-mutacija, pregled S-4 (e9e349b, Approved 13.10.) | 686/0; probe-mutacija 14/14; v1 100 + v2 1000 `--check` | — | 3 | P1 | odlučeno: [ADR-040](../records/DECISIONS/ADR-040-leonove-odluke-iz-pregleda-situacija-s.md) |
| L1-2 | ✅ G-1 SituationsDrawn (707d524 + popravak 0bbec33: kolektivna situacija jednom za sve hotele, poredak po važnosti; pregled Approved 13.10.) | događaj pri otvaranju kruga; `situations_in` po hotelu iz dnevnika | fold dvaput isto; stara v2 igra bez događaja = staro; robot isti odgovor | L1-1 | 3 | P1 | — |
| L1-3 | ✅ G-2 AI ulaz (c2ed74e + popravak 1 e95393b: rezervni odgovor lika i rubrika za primjerak, naziv uloge, opis; ponovni pregled Approved 14.10.) | `CharacterInput`/`AssessInput` iz popunjenog primjerka; `DebriefSituation.odjel` | nijedan AI ulaz s neispunjenim `{…}`; živi test 1 predloška | L1-2 | 2 | P1 | OK za ključ |
| L1-4 | ✅ A-1 API situacije + naredba „prepusti voditelju" (P4) (6f44e25; pregled Approved 14.10.; balansi bajt-isti) | `SituationsBody` (+department, kind, weight, uloge), `openapi.json` | `check:api`; ruta s 3–5 situacija | L1-3 | 1 | P1 | — |
| L1-5 | ✅ W-1 SituationsScreen (33353fb + popravak 1 ea19d54: obvezne samo uz voditelja, odjel za sidra; web 430/0) | 3–5 kartica po važnosti, „riješeno 2 od 4", upozorenje pri „Zatvori tjedan", „prepusti voditelju" | vitest | L1-4 | 3 | P1 | — |
| L1-6 | ✅ UI čuvar situacije (1. SPREMNO ne → popravak; UI čuvar 2: SPREMNO DA 14.10.) | mjerenja 360 px/laptop (`UI_PRAVILA.md`) | UI čuvar DA | L1-5 | 1,5 | P1 | — |
| L1-7 | ✅ O-5 dovršetak | KS-3 spojen u granu obnove (1d982c1); `mentor-sezona.v2` nadograđen (bez v3); 89f8092, 07f1a2b | pregled clean; 703/0, balansi, web | — | 3 | P1 | tekst „Obnova je odlučena…"; lokal B: prag ili 100 % pokrivanje |
| L1-8 | ✅ UI čuvar O-4+O-5 (popravak OKS-1 f76905c: redak „Obnova" na Kraju sezone čita v2 obnove; UI čuvar 2: SPREMNO DA 13.10.; `finance_note` svjesno u W-fazu) | popravak (vitest prvo) pa ponovni UI čuvar | UI čuvar DA | L1-7 | 2 | P1 | — |
| L1-9 | ✅ KS-3 živi test (14.10.: `season_mentor_writes_four_parts` ok; tekst Leon ocjenjuje igrajući) | sezonski mentor s ključem (stablo spremno, 600/0) | 1 živi poziv, ocjena teksta | — | 1 | P1 | OK za ključ |
| L1-10 | ✅ Spoji OBNOVA (s KS-3) → sloj1 (24b4e2a 14.10.: bez sukoba, 703/0, balansi prolaze; `CORE_VERSION` 2.1.0 → probne v2 igre na 2.0.0 daju 409 `game_outdated`) | prvo obnova; sukobi u `schema.rs`, `model.rs`, `PlanScreen.tsx`; **jedno** podizanje `CORE_VERSION` | workspace zelen; v2 1000 `--check`; F pomak izmjeren | L1-7, L1-8 | 4 | P1 | „demo gotov"; potvrda da se probne v2 igre gase |
| L1-11a | ✅ Osnovica ostalih odjela bruto (ADR-040 t. 2; SB-1 ca31005, balans bajt-isti) | `share_basis` = prihod soba prije provizije (`resolve/mod.rs` ~218-233) u `../hotelixsim-s1-sit` | test; balans isti | L1-2 | 1 | P1 | — |
| L1-11b | ✅ Dijagnoza tjedan ≈ mjesec (ADR-040 t. 3; uzrok: mjesečni Brižni bez silaska iz ZP-6; popravak RB-1 252fe8b po [ADR-041](../records/DECISIONS/ADR-041-mjesecni-brizni-otpusta-prije-pada.md), B Brižni 2,96 → 0,18) | zašto Brižni u profilu B igra drukčije po tjednima i mjesecima (2,57 od 2,96); popravak uzroka, granica 3,00 ostaje | tj≈mj s zalihom; balansi 1000 `--check` | L1-10 | 3 | P1 | ako uzrok traži pravilo igre |
| L1-11 | ✅ Spoji SITUACIJE → sloj1 (14.10. popravak A1 55d34f4: brane modela ruše `--check`, mjere robota se ispisuju, bez kalibracije; prije: grana `model-v2/sloj1-l111` b1bf446, 13 sukoba riješeno, 879/0; pada samo brana „udio situacija" za B Brižnog → L1-13 na toj grani, pa `--ff-only`) | 14 zajedničkih datoteka jezgre; S-4 sadržaj; dug `check-kod.mjs` za `game/tests/mentor.rs` 1670 → 1671 (G-2) | isto + tj≈mj | L1-10, L1-11b, L1-6 | 4 | P1 | — |
| L1-12 | ~~Spoji KS-3 → sloj1~~ | KS-3 ulazi sa spajanjem obnove (L1-10) | — | — | 0 | P1 | **„demo gotov" prije L1-10** |
| L1-13 | ✅ 19.10. (dijelovi 1–4, zadnji e4ec659, `CORE_VERSION` 2.6.0) Zajednički balans ([kartica](../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-l1-13-jedna-kalibracija-brojki-kvar-i-popravak-klik-75--značaj-gosti-i-popunjenost-težina-20-), odluke 17.10. [ADR-056](../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)) | Brižni ≥ 80 % sezonskog plana, pojasevi marži, F (±2, C ±1), situacije ≈ 20 % napretka slabijim učinkom svake (broj ostaje; [ADR-042](../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 15; brana 15–25 %), pravednost profila; **17.10.:** kvar u sobi −1 / dobar popravak +2, **tablica značaja situacija** (tri razine, bodovi po razini, student vidi razinu), gostinjske situacije po popunjenosti, šutnja 16 → 0 | tri balansa 1000 `--check` nakon svakog dijela; tablica prije/poslije | L1-12 | 5 → 🟨 [PROC] više (četiri dijela) | P1 | tablica mjerenja; imena razina |
| L1-13b | ✅ 19.10. (13db069, 4be1db4, bdbdebe, 7556d80; [ADR-062](../records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md)) Izmjene iz Leonova pregleda 14.10. | [ADR-042](../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 1–4, 6, 7, 10–14: kvar popravlja stanje i svi kvarovi prostora gledaju stanje, održavanje +5/mj, jedno spremanje, lik zaključi u 6 poruka, odgovori izmiješani i ujednačeni, izgled lika po rodu/dobi/podrijetlu, jednom u krugu, mjesečno 4/5/6 kao postavka igre, opis Smještaja | testovi po točki; tri balansa 1000 `--check` | L1-11, revizija temelja | 12 | P1 | odlučeno |
| L1-14 | **Sljedeće (sesija 12).** Igriva verzija za Leona | adresa + put do novoga; Leon prolazi igru | Leonov popis izmjena | L1-13 | 2 | P1 | što mijenjati |
| L1-15 | sloj1 → main | spajanje i push | CI zelen (`gh run list`) | L1-14 | 1 | P1 | **izričit OK za push** (repo javan) |
| P9-1 | Račun dobiti za profesora (P17) | USALI hotel uz hotel, GOPPAR/EBITDA; ekran i ruta su u PLATFORMI (F2-3) | zbroj = izvješće do centa | L1-15 | 6 | P3 | — |
| P9-2 | „Zašto" po odjelu (P18) + budžet/ostvareno | tri uzroka po odjelu iz jezgre; referentni hotel kao „budžet" | uzroci zbrojeni = razlika | L1-15 | 6 | P3 | pojas GOP marže 35–45 % (N9) |

Kritična staza: L1-1→2→3→4→5→6→11; O-5 → L1-10 paralelno. P1-cigle `Sloja 1` ne uključuju P9 (12 h, ide u P3).

---
### FAZA IGRA — RAZRED U JEZGRI (P2; 5 cigli, 45 h)

**Gotovo kad:** u jednoj igri igra N studenata (1, 4, 30) i **svaki hotel se obračuna kad ga njegov student zatvori**; profesor otvara
krugove (ručno ili po rasporedu), nitko ne ide iza otvorenog kruga; svatko može vidjeti tko je gotov, tko još igra i hotele kolega.
Odluka D1 (obračun po hotelu) je donesena 13.10. ([ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md)).
Preostali dio starog F1 (server, store, `app/platform`, `api/platform`, web razreda) je u [platforma.md](#roadmap--kolosijek-platforma), faza RAZRED.
Ovo je **najveća promjena jezgre** nakon Sloja 1: radi se sama (kolosijek `game` zamrznut za druge), prije LJUDI.

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| F1-7 | Jezgra: N hotela | `new_game_class(...)` uz `new_game_with` (stari ne dirati): `Vec<HotelId>` umjesto `[HotelId; 4]` (`game/command.rs:42,63`), svi `Controller::Human`, profil `profile_at` rotacijom, bez `ROBOTS` | zlatni test: stari `new_game`/`new_game_with` bajt-isti; novi: 1, 4, 30 hotela folda bez greške | L1-15 | 8 | P2 | — |
| F1-9 | Jezgra: vrata kruga | događaj „otvoren do kruga k" (profesor); naredbe studenta iza vrata odbijene; `RoundClosed` dobiva `hotel_id` (`serde(default)` = cijela igra za stare zapise); `status(m)`, `current_round` po hotelu; **zatvaranje po roku od sustava: neaktivan hotel SAM prelazi u sljedeći tjedan s istim planom kakav je imao, situacije ostaju neodgovorene** (odluka 13.10.) | student ne može zatvoriti k+1 dok profesor ne otvori; istek roka prenese prošli plan; dva studenta u različitim krugovima; demo igre foldaju isto | F1-7 | 10 | P2 | — |
| F1-10 | Jezgra: obračun po hotelu | `resolve_hotel_round(hotel, m)` umjesto zbirnog `resolve_round` (`resolve.rs:20-60`); presude čekaju samo taj hotel; ključ posla `resolve:<igra>:<krug>:<hotel>`; kraj sezone (ADR-036) po hotelu; `robot_moves` se ne zove u razrednoj igri | 3 hotela svaki u svom tempu = rezultat zbirnog obračuna istog hotela; balans `--check` stari put nepromijenjen; spor AI ne blokira druge | F1-9 | 20 | P2 | ako > 30 h: javi Leonu prije nastavka |
| M-1 | Mentor upozorava neaktivnog | mentorova rečenica kad je hotel prešao tjedan po roku s prenesenim planom (činjenice iz jezgre, tekst po ADR-023) | test teksta; bez AI-ja rezervna rečenica | F1-9 | 3 | P2 | — |
| F1-18 | Balans i determinizam razreda | `balance --class N` (bez robota); isti događaji daju iste brojke bez obzira na redoslijed zatvaranja | `--seeds 1000 --check` stari i novi put; proptest „permutacija zatvaranja = isti obračun po hotelu" | F1-10 | 4 | P2 | — |

Pazi: platforma (F0-10 predmemorija složene igre) mora biti gotova **prije** F1-10; `F1-9/F1-10` dira `command.rs`, `event.rs`,
`game.rs`, `resolve.rs`, `view.rs`, `season.rs`, `round.rs` — isti fajlovi kao svaka promjena pravila.

---
### FAZA IGRA — LJUDI (sloj 2a; P3; 12 cigli, ~64 h)

**Gotovo kad:** student ima zaposlenike s imenom, vještinama i osobinama, zapošljava, otpušta, povisuje, šalje na obuku i vidi zašto je tko
nezadovoljan; kolege u razredu vide tuđe radnike (koliko je dobar i doprinos uspjehu hotela).

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| L2a-1 | Katalog ljudi | `skills.json`, `traits.json` (7+7 osobina s učincima), banka imena (hr, regija, strana), banka rečenica | provjera sadržaja u `crates/content`; popis zamjene podrijetla | RAZRED U JEZGRI | 5 | P3 | pregled banke imena i osobina; brojke učinaka |
| L2a-2 | Zaposlenik + generator početnih ljudi | tip `Zaposlenik` (§3.3), `sljedeci_id`, generator iz profila; prosjeci točno pogode profil (C lipanj = 58 ljudi) | isti ljudi za isti profil; prosjeci = brojke profila | L2a-1 | 6 | P3 | — |
| L2a-3 | Agregati postaju izvedeni | moral, iskustvo, fluktuacija = zbroj pojedinaca (§3.8) | `equivalence.rs` + „ljudi na prosjeku = staro izvješće" na 3 profila; v1 bajt-isti (**najrizičniji komad**) | L2a-2 | 8 | P3 | — |
| L2a-4 | Motivacija, plaća, rizik odlaska | cilj motivacije = 55 + 2×(plaća/očekivana −100) − pola kazne opterećenja + voditelj + obuka + osobine; odlasci: kocka bira tko (tok `v2:ljudi`) | tablični testovi; ostali tokovi kocke bajt-isti | L2a-3 | 6 | P3 | prag povišice 8 %, razmak 4 tj, raspon plaće |
| L2a-5 | Voditelj odjela + rast vještina + obuka | uloga voditelja (±5 motivacije), odjel bez voditelja kazna; rast po danu + obuka | testovi; „samo obuka" nije dominantna | L2a-4 | 5 | P3 | kazna bez voditelja; ±5 |
| L2a-6 | Burza rada + zapošljavanje/otpuštanje | kandidati po ulozi, ista burza za sve hotele iz sjemena igre, skrivena osobina (otkriva se nakon 4 tj), otkazni rok, otpremnina, sezonac | burza ista za dva hotela; otpuštanje tablično | L2a-4 | 6 | P3 | broj kandidata, otpremnine (N11) |
| L2a-7 | Nalozi i roboti | nalozi (zaposli, otpusti, povisi, obuka, premjesti, voditelj) u planu; 4 robota (Štedljivi, Brižni obučava, Lovac, Slučajni) | plan bez naloga = staro; robot determinizam | L2a-6 | 4 | P3 | — |
| L2a-8 | Balans 2a | probe „samo početnici", „samo profesionalci", „otpusti najslabije", „svima najveća plaća", „samo obuka"; §11.1–11.5; tj≈mj | tri balansa 1000 `--check`; nijedna probe dominantna | L2a-7 | 6 | P3 | pojasevi fluktuacije |
| L2a-9 | Igra + API | događaji (Zaposlen, Otpusten, Povisica, Obuka…), `DetailDto` ljudi, rute, `openapi.json` | fold dvaput isto; `check:api` | L2a-7 | 4 | P3 | — |
| L2a-10 | Web: Ljudi + profil + burza | ekran po odjelima (`AvatarLook`, vještine, osobine zeleno/crveno/„?", plaća prema očekivanoj, rizik), profil s „zašto je motivacija pala", burza | vitest; 360 px | L2a-9 | 8 | P3 | mjesto „Ljudi" u navigaciji (A1/A2) |
| L2a-11 | UI čuvar 2a | | UI čuvar DA | L2a-10 | 2 | P3 | — |
| L2a-12 | Vidljivi tuđi radnici | izvedeno iz jezgre: kvaliteta radnika i **doprinos uspjehu hotela** („jedan od razloga zašto hotel dobro radi") za svakog radnika i voditelja odjela; skrivene osobine ostaju skrivene; javni DTO ide u platformu (F1-12) | test po polju: tuđi radnik ima kvalitetu i doprinos, nema skrivenih osobina, troškova ni odgovora na situacije | L2a-9 | 4 | P3 | **plaća tuđeg radnika: NIJE odlučeno (anketa)** |

---
### FAZA IGRA — RAZGOVORI (sloj 2c; P3; 8 cigli + 1 opcija, ~35 h)

**Gotovo kad:** student razgovara s radnikom (pregovor o plaći s upisanim brojem, upozorenje, razgovor za posao…) i vidi učinak; AI nikad ne određuje brojku.
Linija reza: samo 3 vrste (pregovor, upozorenje, za posao), bez izlaznog i motivacijskog (~5 h manje).

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| L2c-1 | Rubrike i likovi | rubrike (pregovor, upozorenje, motivacijski, za posao, izlazni), prompt `lik.v2`, rezervni odgovori, mjerilo | mjerilo AI vs Leonove ocjene | L2a-1 | 8 | P3 | ocjene mjerila (40 odgovora čeka) |
| L2c-2 | Jezgra razgovora | tipovi, okidači, formule učinka s granicama (kvaliteta 1–5), **vrijeme voditelja** | kvaliteta daje učinak samo u granicama | L2a-5 | 5 | P3 | 3 tj / 8 mj (N6) |
| L2c-3 | Pregovor o plaći | student upisuje **broj**; prihvaćanje = ponuda ≥ očekivana − tolerancija(q) | AI nikad ne zapisuje broj; bez AI-ja ista formula | L2c-2 | 3 | P3 | tolerancija(q) |
| L2c-4 | Igra + AI put | `Talk` za ljude na putu lika/presude; zakašnjela presuda = neutralno; razgovor za posao otkriva skrivenu osobinu | FakeAi: bez AI-ja isti razgovor; fold; `v1_frozen` | L2c-3 | 5 | P3 | — |
| L2c-5 | Brojač AI troška | trošak po studentu iz `AiMeta`, granica 2 $ (N7), zatim ponuđeni odgovori; granica po igri/semestru je u PLATFORMI | test praga | L2c-4 | 3 | P3 | granica 2 $ |
| L2c-6 | API | rute razgovora, vrijeme voditelja | `check:api` | L2c-4 | 2 | P3 | — |
| L2c-7 | Web: razgovori | „Razgovaraj" u profilu, ekran, preostalo vrijeme, ponuđeni odgovori | vitest; UI čuvar | L2c-6 | 6 | P3 | — |
| L2c-8 | Balans + živi test | probe „samo razgovori"; živi test 5 tipova, trošak ~0,25–0,75 $ po igri | `--check` | L2c-7 | 3 | P3 | OK za ključ |
| P-1 | Obećanja iz odgovora (opcija) | `presuda.v4` vraća obećanja iz zatvorenog popisa; primjerak `P-OBECANJE` | AI ne piše brojke | L2c-4, L2a-6 | 4 | poslije | uključiti? |

---
### FAZA IGRA — NATJECANJE ZA LJUDE (P3, rez → P4; 8 cigli, 45 h)

**Gotovo kad:** student A pošalje ponudu plaće radniku hotela B; B dobije obavijest, može dati protuponudu (povišica ili razgovor), a radnik
odluči po vlastitoj formuli; nakon svih zatvaranja ishod je isti bez obzira tko je prvi zatvorio. Pravilo: ponuda u tjednu k; vlasnik B do kraja
**svog** tjedna k protuponuda; radnik odlučuje kad B zatvori tjedan k; prelazi od k+1. Ograničenje 1–2 ponude tjedno + trošak zapošljavanja;
profesor može isključiti preotimanje za razred. Detalji: [ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md).

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| N-1 | Ponuda i stanja | događaji (ponuda, protuponuda, odluka radnika), limit 1–2 ponude/tj, trošak zapošljavanja, postavka razreda „preotimanje isključeno" | tablični testovi; stari zapisi foldaju isto | L2a-9, F1-10 | 6 | P3 | limit 1 ili 2; trošak |
| N-2 | Zadovoljstvo radnika | odluka = formula (plaća, voditelj, opterećenje, osobine) + kocka iz vlastitog toka `v2:natjecanje`; AI ne odlučuje brojke | tablični testovi; ostali tokovi bajt-isti | N-1 | 6 | P3 | težine činitelja |
| N-3 | Vremenska pravila | ponuda u k; protuponuda do kraja SVOG tjedna B; odluka pri zatvaranju k hotela B; prijelaz od k+1; nezatvoreni tjedan: pravilo neaktivnog (F1-9) | determinizam: isti ishod za sve permutacije zatvaranja | N-2, F1-18 | 8 | P3 | — |
| N-4 | Protuponuda | povišica ili razgovor (veza sa slojem razgovora, `Talk`) | razgovor daje učinak u granicama kataloga | N-3, L2c-4 | 4 | P3 | — |
| N-5 | Roboti za vježbu | Brižni čuva ljude, Lovac preotima | robot isti odgovor za isto sjeme | N-3 | 4 | P3 | — |
| N-6 | Balans natjecanja | probe „preotimaj sve najbolje", „nikad ne odgovaraj", „uvijek povisi"; tj≈mj | `--check`; nijedna strategija preotimanja dominantna | N-5 | 6 | P3 | pojasevi |
| N-7 | API + obavijest | rute; događaj za obavijest u aplikaciji (isporuka obavijesti u PLATFORMI) | `check:api` | N-3 | 4 | P3 | — |
| N-8 | Web: natjecanje | ponuda iz profila tuđeg radnika, obavijest, protuponuda, ishod | vitest; UI čuvar | N-7 | 7 | P3 | — |

---
### FAZA IGRA — JEZICI I AGENCIJA (sloj 2b; P4; 8 cigli, 29 h)

**Gotovo kad:** jezici pokrivaju goste po mjesecu i pomiču kvalitetu; student naruči radnike iz agencije (rok, naknada, smještaj) i vidi cijenu po glavi.

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| L2b-1 | Sadržaj jezika i agencija | jezični sastav po mjesecu (DZS + Kvarner, N8), po zemlji rok/naknada/dozvola, tečajevi | provjera sadržaja; izvor zabilježen | L2a-1 | 4 | P4 | pregled brojki |
| L2b-2 | Jezici → kvaliteta | pokrivenost odjela pomiče kvalitetu i teme; obuka jezika +1 | pokrivenost 100 % = staro | L2b-1, L2a-5 | 4 | P4 | veličina učinka |
| L2b-3 | Agencija: nalog | „5 radnika za domaćinstvo", dolaze nakon roka, naknada + dozvola | tablično; radnik iz istih raspodjela | L2b-1, L2a-6 | 4 | P4 | plaća ista za istu vještinu (N1) |
| L2b-4 | Smještaj radnika (17.10.: širi ga LJ-S niže) | krevet u gradu; sobe za radnike (≤ 10 %) — puni oblik čeka status sobe (S3-3, poslije pilota) | zbroj troškova = zbroj po osobama do centa | L2b-3 | 4 | P4 | N10; cijena kreveta |
| L2b-5 | Test zamjene podrijetla | ista osoba, drugo podrijetlo → razlika samo u jezicima, roku, naknadi, dozvoli, smještaju | test | L2b-4 | 2 | P4 | — |
| L2b-6 | Balans | agencija nije dominantna | tri balansa `--check` | L2b-5 | 3 | P4 | — |
| L2b-7 | Igra + API | | `check:api`; fold | L2b-6 | 3 | P4 | — |
| L2b-8 | Web: agencija, smještaj, pokrivenost | ekran agencije (narudžbe, trošak i doprinos po osobi), pokrivenost po odjelu | vitest; UI čuvar | L2b-7 | 5 | P4 | — |

---
### FAZA IGRA — VRSTE SOBA I CIJENE (P4; 6 cigli, 21,5 h)

**Gotovo kad:** hotel ima vrste soba i student na jednom ekranu Cijene postavlja cijenu po vrsti i vidi prelijevanje potražnje. Dizajn gotov
(`vrste-soba-agencije-dizajn.md` §9 u ledgeru sloj1); mijenja MODEL_V2 §3.5/§7.3 → ADR 🟨.

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| VS-S1 | Sadržaj vrsta soba | `room_types.json`, `profiles.json` (`rooms_by_type`, katovi), segmenti | zbroj soba ≠ 120 odbijen; snimak bez vrsta = isti otisak | L1-15 | 2 | P4 | B1 raspored i tlocrt 5×24 |
| VS-J1 | Jezgra vrsta | `RoomTypeSpec`, `market_by_type`, `PeriodPlan.room_prices`, prelijevanje gore/dolje, izvješće po vrsti | jedna vrsta = staro izvješće; Σ po vrstama = sažetak; probe-mutacija | VS-S1 | 7 | P4 | B3 indeksi, B4 prelijevanje (40 %/25 %), B5 gosti po sobi |
| VS-J2 | Roboti i balans vrsta | cijene po vrsti (Štediša 100 %, Brižni 105 %, Lovac 85 %, Nasumični ±10 %); probe `TypesAtReference`, `PeakTypesUp`, `FlatCut` | `--check` | VS-J1 | 3 | P4 | B2 omjeri cijena (A ima +4,6 %) |
| VS-G1 | Igra | `PlanDto.room_prices`, `DetailDto.rooms_by_type`, `{soba.vrsta}` u situacijama | fold dvaput; stara v2 isti JSON | VS-J2 | 3 | P4 | — |
| VS-A1 | API | `room_prices`, `limits.room_types`, `/prices`, `/layout` | `check:api` | VS-G1 | 1,5 | P4 | — |
| VS-W1 | Web: ekran Cijene + tlocrt | sve vrste i odjeli na jednom mjestu, „Sve ±5 %", „Vrati na preporučene" | vitest; UI čuvar; 360 px | VS-A1 | 5 | P4 | A2 mjesto ekrana |

---
### FAZA IGRA — RECENZIJE (sloj 4; P4; 7 cigli, 34 h)

**Gotovo kad:** gosti pišu recenzije (ocjena, tema, jezik, tekst), student odgovara na njih, a ocjena portala pomiče potražnju kao u v1.
Napomena: R4-2 je u nacrtu ovisio o S3-1 (soba kao entitet); S3 je poslije pilota, pa ovisi o VS-J1 — provjeriti u planu faze.

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| R4-1 | Banka rečenica | po temi/predznaku/segmentu/jeziku; imena gostiju | provjera sadržaja; test stereotipa | L2a-1 | 10 | P4 | pregled po komadima |
| R4-2 | Jezgra recenzija | boravaka = noći/4,2; ~12 % piše; ocjena = cilj iz vrste sobe, odjela, jezika + šum nultog zbroja; spremanje kao kodovi | ocjena portala = bazen s pomakom kao v1; v1 sažetak isti | VS-J1, L2b-2 | 7 | P4 | 4,2 noći, 12 %, segmenti |
| R4-3 | Balans ocjene | ocjena i potražnja ne eksplodiraju | tri balansa `--check` | R4-2 | 4 | P4 | — |
| R4-4 | Odgovor na recenziju | pristup (ispriča+mjera, zahvala, obrana, šablona), bez AI-ja, vrijeme voditelja | učinak u granicama | R4-2, L2c-2 | 3 | P4 | učinak pristupa |
| R4-5 | Istaknuta recenzija (AI) | Haiku jednom po razdoblju + rezerva iz banke | FakeAi test | R4-1 | 2 | P4 | — |
| R4-6 | Igra + API | recenzije, odgovori | fold; `check:api` | R4-4 | 3 | P4 | — |
| R4-7 | Web: Recenzije | ocjena, segment, jezik, tekst, teme, kretanje | vitest; UI čuvar | R4-6 | 5 | P4 | — |

---
### FAZA IGRA — poslije pilota (smjer; ~190 h)

Redoslijed se odlučuje nakon pilota; svaka grupa je zaseban val s jednim podizanjem `CORE_VERSION`.

| grupa | cigle | h | uvjet / napomena |
|---|---|---|---|
| SOBE | S3-1 soba kao entitet 6 · S3-2 trošenje i kvarovi 4 · S3-3 obnova kata + izvan funkcije 4 (zatvara L2b-4) · S3-4 API 2 · S3-5 web tlocrt 120 pločica 6 · S3-6 balans 3 · S3-7 rezervacije (opcija) 5 | 25 + 5 | VS gotov; §7.3 „ne radimo rezervacije po gostu" → odluka da/ne |
| HOTELI U OKOLICI | R4-8 jezgra (recenzije robota istim obračunom, MPI/ARI/RGI) 4 · R4-9 web 5 · R4-10 balans 3 | 12 | što se vidi o tuđem hotelu: radnici da (kvaliteta, doprinos), skrivene osobine, troškovi i odgovori ne |
| pošta s agencijama | AG-S1…AG-W1 | 35–40 | koristi AI brojač i vrste soba; anketa A1/A3/A4/A7/A8; Leonov pregled pisama |
| događaji | vjenčanja, MICE (EV-*), novi odjel „Događaji" | ~35 | u istom valu s poštom (jedno podizanje `CORE_VERSION`) |
| dobavljači | nabava | 15–20 | sukob s MODEL_V2 §7.3 („ne radimo nabavu") → najprije Leonova odluka; nema dizajna |
| novine / makro uvjeti | potražnja, energija, tečaj | 15–20 | dira balans; pilot pokaže treba li |
| generator gostiju | uz R4-1 | ~4 | banka imena |
| ENGLESKI (sadržaj) | prijevod situacija i banki (imena, rečenice, osobine), prompti lik/mentor/presuda (`lik.v2`, `presuda.v3`, mentor) i provjera da AI odgovara na jeziku igre; sučelje i format su u [platforma.md](#roadmap--kolosijek-platforma) | ~30 [PROC] | smjer: igra mora kasnije biti i na engleskom; mjerilo ocjena po jeziku |

Generator hotela (ideja 10) je u [platforma.md](#roadmap--kolosijek-platforma), faza poslije; od 17.10. prije njega dolazi BANKA HOTELA (niže).

---
### Nove cigle iz Leonovih odluka 17.10. ([ADR-056](../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md); mjesta 🟨 voditelja)

Leon: *„ovo što već imamo je pilot, a ovo što gradimo je već produkcijska platforma"*. Redoslijed nakon L1-13…L1-15 je
🟨 prijedlog voditelja ([ROADMAP](./ROADMAP.md) „Pilot → produkcija"): prvo PLATFORMA do najmanje igrive razredne igre, tek onda
sobe, voditelji, smještaj radnika i banka hotela. Sati su 🟨 [PROC]; gdje ih nema, procjenu daje kartica kad faza krene.

| id | cigla | što se radi | faza (🟨) | h [PROC] | Leon |
|---|---|---|---|---|---|
| ZN-1 | Tablica značaja situacija (B2) | tri razine (prijedlog: ključna / važna / svakodnevna iz `weight`), bodovi po razini, razina vidljiva studentu | **L1-13** (dio 2) | u L1-13 | imena razina |
| HN-1 | Student imenuje hotel (C4) | ime hotela prije ulaska u igru, slobodno; 🟨 najmanja brana za vulgarnosti; ime u dnevniku (`GameCreated`) i na ekranima | C (mala) | ~2–3 | — |
| TR-1 | Kartica „Tržište" (D3–D5) | na ekranu pokazatelja: graf zainteresiranih, smještenih i kapaciteta kroz sezonu (noćenja, gosti u zagradi), rastav zanimanja, samo sezonsko očekivanje ([VODITELJ_I_TRZISTE](../architecture/VODITELJ_I_TRZISTE.md) §2–3) | C ili iza | ~11 (3 + 4 + 4) | — |
| SB-1…5 | Sučelje soba (A1) | svaka soba zasebno vidljiva i obnovljiva; kvar je posljedica neodržane sobe; spaja se s grupom SOBE (S3-1…S3-6) iz „poslije pilota" | C+ (nakon platforme do razredne igre) | 3–5 cigala | izgled ekrana |
| MP-1 | Mjere potrošnje hotela (A13) | energija i voda kao pokazatelj u igri (trošak, pouka o održivosti); trošak AI-ja po igri mjeri platforma ([platforma.md](#roadmap--kolosijek-platforma)) | C+ | [PROC] ? | brojke potrošnje |
| LJ-T | Obuka pojedinačno ili skupina (A9) | student trenira jednog radnika ili skupinu u odjelu i bira **trenera iz redova radnika**; svi trenirljivi do razine voditelja (`strucnjak`); proširuje L2a-5 | LJUDI | [PROC] ? | — |
| LJ-V | Mnogo voditelja (A9, D1, D2) | voditelj kao podvrsta radnika iz banke (`strucnjak`), vidljiva crta jak / slab za svakog radnika i voditelja, voditelj vodi obuke i koordinira odjel (D2 = c); proširuje L2a-5 i L2a-10 | LJUDI | [PROC] ? | — |
| LJ-S | Smještaj radnika (A11–A12) | zasebna mehanika: ekran, trošak i kapacitet; student smještaj **kupuje, iznajmljuje ili koristi sobe svog hotela**; ne treba svaki radnik smještaj; zamjenjuje i širi L2b-4 | LJUDI | [PROC] ? | cijene najma i kupnje |
| BH-S | **BANKA HOTELA: sadržaj** | istraživanje pravih hotela i plaća (A10, A12) → 10–15 hotela 4* raznih veličina (ne uvijek 120 soba), provjera stroja, pečat u serijama koje Leon pregleda (kao banka ljudi) | nakon L1-13 (i L1-14/15), prije kraja faze C | [PROC] ? | pregled i pečat serija |
| BH-J | **BANKA HOTELA: jezgra** | ključ hotela A/B/C → tekst (`HotelSetup.profile`, [PLATFORMA §4](../architecture/PLATFORMA/04-generirani-hoteli.md)), referenca napretka **po hotelu**, balans po svakom hotelu; pravednost 80 vs 160 soba: indeksi po sobi ili udjelu, ne u eurima (§4.3) | isto | [PROC] ? | — |

Platformski dio banke hotela (profesor dodjeljuje hotel po studentu ili nasumce) je u [platforma.md](#roadmap--kolosijek-platforma) (BH-P).
Generator hotela kasnije samo **puni** banku hotela.

---
### Linije reza (ako zapne, redom)

1. Vlastite situacije profesora P3 → P4 (platforma.md, faza PROFESOR). 2. Razgovori samo 3 vrste. 3. NATJECANJE ZA LJUDE P3 → P4.
**Nikad se ne režu:** testovi, provjeravatelj (`equivalence.rs`, `v1_frozen`, probe-mutacija) i balans.

### Sažetak ([PROC])

| faza | cigli | h |
|---|---|---|
| ZAVRŠETAK SLOJA 1 | 17 | 49,5 (P9: 12 u P3) |
| RAZRED U JEZGRI | 5 | 45 |
| LJUDI | 12 | 64 |
| RAZGOVORI | 8 + 1 opcija | 35 (+4) |
| NATJECANJE ZA LJUDE | 8 | 45 |
| JEZICI I AGENCIJA | 8 | 29 |
| VRSTE SOBA I CIJENE | 6 | 21,5 |
| RECENZIJE | 7 | 34 |
| poslije pilota | grupe | ~190 |

### Rizici balansa (sažeto)

Najveći: **L2a-3** (agregati iz pojedinaca moraju na početku dati točno v1 brojke), spajanje tri stabla (L1-10…13: 14 zajedničkih datoteka, tj≈mj 296/300),
dominantne radnje (2a i natjecanje), vrste soba (ukupna potražnja pri preporučenim cijenama ±3 %), recenzije → potražnja (povratna petlja), AI trošak
(17 krugova, granica 2 $ dijele razgovori, pošta i recenzije). Prave greške prije siječnja i hitni CI: [kvaliteta.md](#roadmap--kolosijek-kvaliteta).

## ROADMAP — kolosijek PLATFORMA

**Što je ovo:** ZALIHA CIGLI po fazama za kolosijek platforme (server, `store`, `app/platform`, `api/platform`, web razreda i profesora,
hosting, GDPR). Faze iza prve su **smjer** koji se ponovno slaže nakon što profesor vidi igru; kartica cigle se razrađuje u planu faze
kad faza krene. Sati su [PROC]. Podjela je po **datotekama** ([ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)):
sve što dira srce igre (N hotela, vrata kruga, obračun po hotelu: F1-7, F1-9, F1-10, F1-18) je u [igra.md](#roadmap--kolosijek-igra), faza RAZRED U JEZGRI.
Razred = jedna igra s N hotela, studenti igraju jedni protiv drugih ([ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md)).
Susjedi: [kvaliteta.md](#roadmap--kolosijek-kvaliteta), [ROADMAP.md](./ROADMAP.md). Arhitektura platforme (nacrt za raspravu, odluke u ADR-037): [PLATFORMA.md](../architecture/PLATFORMA.md).

**Prekretnice (stupac P):** P1 kraj listopada · P2 kraj studenog (profesor na internetu stvori igru, studenti uđu kodom) · P3 sredina siječnja
(profesori vode razred i dodaju situacije) · P4 ožujak (studentski pilot) · poslije.
Platformski kod ide u NOVE module (`app/src/platform/`, `api/src/platform/`, nove migracije od 0005) da se smanje sukobi s igrom;
DTO-i platforme ne idu u `api/src/schema.rs`. `openapi.json` se uvijek regenerira poslije spajanja, nikad ručno.

---
### FAZA PLATFORMA — SPREMNO ZA INTERNET (P2, dio P3; 13 cigli, 48 h)

**Gotovo kad:** Leon može pokrenuti HotelixSim na jednoj adresi (lokalno **0 €**), a nakon njegovog izričitog OK i na Railwayu (~5–10 €/mj):
otvori adresu u privatnom prozoru i Safariju, odigra igru; ako padne baza ili poslužitelj, vidi se zašto. Izlazak na internet samo uz Leonov OK.

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| F0-1 | Tipizirana `Config` | `server/src/config.rs`: sve `env::var` u jednu strukturu; `0.0.0.0:$PORT` (`main.rs:144`); `HOTELIXSIM_CONTENT` umjesto puta iz prevođenja (`content/src/lib.rs:74-76`) | nepotpuna okolina = jasna greška; `PORT=9000` sluša na 9000 | — | 3 | P2 | — |
| F0-2 | Uredno gašenje | SIGTERM + Ctrl-C (`?` preskače `worker.abort()`, `main.rs:157`), posao u tijeku dovršen ili vraćen | `kill -TERM` usred posla: izlaz 0, posao ne ostaje `running` | F0-1 | 2 | P2 | — |
| F0-3 | HTTP zaštita | `Secure` kolačić uz https (`auth.rs:31`), `SameSite=Lax`, provjera `Origin` na POST-u, CSP/HSTS/`nosniff`, limiti tijela, `tower_governor` | tuđi `Origin` 403; tijelo > limita 413; 61. zahtjev 429 | F0-1 | 5 | P2 | — |
| F0-4 | Poslužitelj servira web | `tower-http ServeDir` za `apps/web/dist`, SPA fallback; jedna adresa | `GET /` index, `/v1/health` JSON, nepoznata ruta index | F0-1 | 3 | P2 | — |
| F0-5 | Zdravlje | `SELECT 1` + verzija + otisak pravila umjesto `COUNT(*)` (`health.rs`) | ugašena baza: 503 u 3 s | F0-1 | 2 | P2 | — |
| F0-6 | sqlx TLS i brane | `tls-rustls`; `deny.toml` zabranjuje `native-tls`/`openssl`; testovi `connect_lazy` i `migrate` na svježoj bazi | `cargo deny check` zeleno; spoj sa `sslmode=require` | — | 3 | P2 | — |
| F0-9 | `ai_usage` + granica | projekcija iz `AiMeta` (`event.rs:140-145`), cijena modela u `content/`, provjera PRIJE stavljanja AI posla u red; iznad granice rezerva (ADR-019 t. 4) | lažni AI 2 $ → idući poziv na rezervu bez mreže | — | 6 | P2 | 2 $ po studentu već odlučeno |
| F0-10 | Predmemorija složene igre | `app`: `Game` po `game_id`, nakon `decide` samo `apply` novih događaja; ključ `(game_id, last_seq)`; **preduvjet za F1-10 u igra.md** | proptest: inkrementalno = puni fold; naredba na igri s 5 000 događaja | — | 6 | P2 | — |
| F0-11 | Dockerfile + slika u CI-ju | višefazni (cargo-chef, Rust 1.98.1, `SQLX_OFFLINE=true`), `content/` i `dist` u slici; CI gradi bez objave | CI zelen; `docker run` u CI-ju odgovara na `/v1/health` | F0-1, F0-4 | 5 | P2 | — |
| F0-13 | Kopije baze (lokalno) | `scripts/backup.sh` (`pg_dump`) + `scripts/restore-test.sh` | restore ispiše jednak broj `events` i `OK` | — | 3 | P3 | — |
| F0-14 | Zapisi i Sentry | JSON zapisi, Sentry (Rust + web, EU), upozorenje na posao `failed` | bačena greška stigne u Sentry | F0-1 | 3 | P3 | račun (besplatna kvota) |
| F0-S1 | Railway Hobby: servis + Postgres | Postgres u istoj regiji, `ANTHROPIC_API_KEY` samo ondje, `migrate` prije pokretanja, `*.up.railway.app` | `curl .../v1/health` 200; kolačić `Secure`; prijava radi u Safariju | F0-1…6, F0-11 | 4 | P2 | **izričit OK + trošak ~5–10 €/mj** (D6) |
| F0-S2 | Mjerenje kašnjenja | RTT poslužitelj ↔ baza, p95 naredbe pod opterećenjem | zapisani brojevi, ne dojam | F0-S1 | 3 | P3 | — |

Što još treba za Railway (BACKLOG, sve ovdje): `Idempotency-Key` + probni apalis/pgmq ([BACKLOG.md:37](../records/BACKLOG.md)), serijski radnik ([:61](../records/BACKLOG.md)), `ctrl_c` ([:85](../records/BACKLOG.md)), deadlock expiry ([:81](../records/BACKLOG.md)); uloga baze i paralelni radnici su u PILOTU.

---
### FAZA PLATFORMA — RAZRED (P2; 21 cigla, 116 h)

**Gotovo kad:** profesor se prijavi, stvori razred, pokaže kod na projektoru, studenti se prijave svojim računom, unesu kod i uđu u čekaonicu; profesor
pritisne „Pokreni" i otvara krugove (ručno ili po rasporedu); student zatvara svoj hotel, vidi tko je gotov i hotele kolega; neaktivnom hotelu rok
sam prenese plan. **Identitet je zamjenjiv:** računi u bazi, identiteti (AAI, Google) odvojeno i vežu se kasnije na isti račun (`identities`).
D4 (kako se profesor prijavljuje za probu) je **OTVOREN**; F1-4 je preporuka (jednokratna poveznica koju izdaje admin, 0 €).

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| F1-2 | Migracija platforme | `0005`: `users` (korisničko ime, otisak lozinke, uloga), `identities` (odvojeno), `memberships`, `courses`, `classes` (kod UNIQUE, postavke JSONB, `join_open`, `game_id` NULL do starta), `class_players`, `audit_log`, `games.class_id`; `slot CHECK` 1..4 → 1..64 (`0003:18`) | `sqlx migrate` svježa i kopija; `sqlx prepare` bez razlike; slot 30 da, 65 ne | F0-6 | 4 | P2 | — |
| F1-3 | Uloge i prava | `Caller` proširen (`demo.rs:10-12`); `permissions`; student vidi svoj razred, profesor svoje (tuđe 404) | matrica dopušteno/zabranjeno po ulozi (gost, student, profesor, admin) | F1-2 | 6 | P2 | — |
| F1-4 | Ulaz za profesora | admin CLI `server admin add-professor` izdaje jednokratnu poveznicu (24 h); sesija (`auth.rs`) | poveznica radi jednom; druga 410 | F1-3 | 6 | P2 | **D4 OTVOREN** |
| ID-1 | Prijava korisničko ime + lozinka | argon2/bcrypt otisak, prijava, promjena lozinke, ograničenje pokušaja po IP-u i računu | test: lozinka nikad u dnevniku ni odgovoru; 11. pogrešan pokušaj 429 | F1-3 | 6 | P2 | — |
| ID-2 | Generator računa za studente | profesor unese popis studenata, sustav izda korisnička imena i početne lozinke (jednokratni prikaz/CSV), prisilna promjena pri prvoj prijavi; profesor može resetirati lozinku | test: tuđi razred 404; popis s duplikatom odbijen; reset ruši staru sesiju | ID-1, F1-5 | 6 | P2 | oblik korisničkog imena |
| ID-3 | Web: prijava i lozinka | ekrani prijave, promjene lozinke, profesorov popis i reset | Vitest; mobitel | ID-2 | 5 | P2 | — |
| F1-5 | Razred i kod igre | stvori kolegij/razred (Week/Month, model v2, veličina, sjeme), generator koda (6–7 znakova bez 0/O/1/I), otvori/zatvori upis; postavke se zaključavaju u startu | kod jedinstven; zatvoren upis = kod ne radi; profesor A ne vidi razred B | F1-3, F1-4 | 7 | P2 | D3 veličina (preporuka 40), D11 |
| F1-6 | Ulaz kodom igre | prijavljeni student upisuje kod igre → `class_players` (kod za ulazak može ostati; ključ za povratak više nije identitet) | isti račun na drugom uređaju vidi isti hotel; pridruživanje nakon starta odbijeno | F1-5, ID-1 | 4 | P2 | D2 |
| F1-8 | Pokretanje iz čekaonice | „Pokreni" jednom transakcijom stvara `GameCreated` s prijavljenima (jezgra: F1-7 u igra.md), `hotels` slot 1..N, `classes.game_id` | 7 prijavljenih → 7 hotela; drugi „Pokreni" ništa; pad usred ne ostavlja polovicu | F1-2, F1-6, F1-7 | 4 | P2 | D2 lobby |
| F1-11 | App naredbe i API | `open_round` (profesor), `close_round` po hotelu (`app/commands.rs:96-134`), rute u `api/src/platform/`, OpenAPI → klijent | svaka ruta s ispravnom i pogrešnom ulogom | F1-3, F1-10 (igra.md) | 8 | P2 | — |
| F1-12 | Vidljivost za kolege | `app`: ljestvica napretka, **radnici tuđih hotela (koliko je dobar, doprinos uspjehu; skrivene osobine skrivene)**, ocjene i cijene; troškovi i odgovori na situacije skriveni; DTO u zasebnoj datoteci | test po polju: „API ne vraća privatno" | F1-10, L2a-12 (igra.md) | 8 | P2 | D8 imena ili nadimci; plaća tuđeg radnika (anketa) |
| F1-13 | Odabir igre korisnika | `me/hotel`, `me/classes`: razred ili demo (`store/src/guests.rs` `latest_hotel_of`) | korisnik s demom i razredom dobiva razred | F1-8 | 3 | P2 | — |
| F1-14 | Web: kod i čekaonica | „upiši kod", čekaonica „čekamo profesora (N prijavljenih)" | Vitest + proba na mobitelu | F1-6, ID-3 | 6 | P2 | — |
| F1-15 | Web: student u razredu | stanje „čekaš otvaranje", zatvori svoj hotel, lista kolega (gotov / još igra, uvijek se može vidjeti hotele kolega), javni pogled | Vitest; Playwright | F1-11, F1-12 | 8 | P2 | — |
| F1-16 | Web: profesor | stvori razred, kod krupno (projektor), popis prijavljenih (polling), „Pokreni", „Otvori sljedeći krug" | Playwright tok profesora; mobitel i računalo (ADR-027) | F1-11 | 8 | P2 | — |
| F1-17 | E2E: razred od tri studenta | profesor + 3 studenta kroz 2 kruga, vrata, različit tempo | zelen u CI-ju s pravim Postgresom | F1-14…16 | 6 | P2 | — |
| RZ-1 | Raspored otvaranja i rokovi | posao `open:<igra>:<krug>` s `run_after` (`0004` postoji); raspored (npr. tjedno) ili ručno; 17 tjednih krugova ≈ semestar | posao u prošlosti otvori krug jednom (idempotentno) | F1-11 | 5 | P2 | potvrđeno 13.10. |
| RZ-2 | Pravilo za neaktivne | posao po roku zatvara hotelu krug naredbom sustava; hotel prelazi sam s istim planom, situacije neodgovorene (jezgra: F1-9, mentor: M-1 u igra.md) | test: rok istekao → sljedeći tjedan s prošlim planom, ostali netaknuti | RZ-1, F1-9 | 4 | P2 | odluka 13.10. |
| RZ-3 | Obavijesti u aplikaciji | zvonce: novi tjedan, ponuda za radnika (N-8), reset lozinke; bez e-pošte | Vitest; polling 5 s | F1-15 | 6 | P2 | odluka 13.10. |
| RZ-4 | AI granica po igri/semestru | zbroj `ai_usage` po razredu, profesor vidi i povisi, globalni osigurač (F2-9) | iznad granice svi na rezervu; povišenje djeluje odmah | F0-9 | 4 | P2 | da (13.10.); iznos |
| AAI-0 | AAI@EduHr: istraživanje (bez koda) | pitanja fakultetu/Srcu: tko je IdP, SAML/OIDC, atributi, rok; ovisi o fakultetu | pisani odgovor ili zapis da nema informacija | — | 2 | P2 | Leon/fakultet |

F1-1 (prepis arhitekture) je u [kvaliteta.md](#roadmap--kolosijek-kvaliteta) kao K0-7. Kritični put razreda: F1-7 → F1-9 → F1-10 (igra.md) pa F1-11.

---
### FAZA PLATFORMA — PROFESOR (P3; 16 cigli, 117 h)

**Gotovo kad:** profesor vidi razred uživo (tko je u kojem krugu, tko čeka, AI trošak), otvara pojedinog studenta u načinu samo čitanje, vidi račun dobiti
svih hotela i presude koje treba pogledati, i **piše vlastitu situaciju** kroz formu (3–5 odgovora i ocjena kvalitete, ne JSON) koju studenti dobiju.
Linija reza: vlastite situacije (F2b) P3 → P4. F2b ovisi o spojenoj grani `../hotelixsim-s1-sit` (L1-11 u igra.md).

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| F2-1 | Projekcija `hotel_progress` | tablica u istoj transakciji kao obračun (`ports.rs:107-117`); ponovno izgradiva (`rebuild`); jedna igra, N redaka po krugu | projekcija = izvedeno iz dnevnika | F1-10 (igra.md), F1-2 | 7 | P3 | — |
| F2-2 | API pregleda razreda | `GET /v1/classes/{id}/overview`; ETag po `max(events.id)`, 304 | 304 kad nema promjene; tuđi razred 404; `audit_log` | F2-1 | 6 | P3 | — |
| F2-3 | Profesorov račun dobiti | ruta + ekran svih hotela jedan uz drugoga; izvor brojki je P9-1 (igra.md) | zbroj po stavkama = izvještaj hotela; student 403 | F2-2, P9-1 | 6 | P3 | — |
| F2-4 | Detalj studenta (samo čitanje) | postojeći ekrani hotela bez tipki za mijenjanje | Playwright; pristup u `audit_log` | F2-2 | 8 | P3 | — |
| F2-5 | Web: ekran razreda | tablica studenata, krug, napredak, tko čeka, AI trošak, polling 5 s s ETagom | Playwright; 304 bez tijela | F2-2, F1-16 | 10 | P3 | — |
| F2-6 | Pregled presuda | presude s `review: true` i `VerdictLate` (ADR-019), razgovor i rubrika, samo čitanje | lista = točno neutralne/zakašnjele | F2-2 | 6 | P3 | — |
| F2b-1 | Dizajn forme situacije | ADR 🟨: forma (naslov, opis, vrsta, odjel, 3–5 odgovora, ocjena 1–5, rubrika 5 kriterija, lik) → generator slaže `Template` (`templates.rs:54`) | primjer forme → JSON koji prolazi `check` | s1-sit spojen | 4 | P3 | D10 |
| F2b-2 | Provjera i prefiks | u `content`: `validate_extra_templates(...)`, šifre `P-X-<razred>-…`, greške s putanjom polja | kolizija šifre / nepoznat odjel / rubrika ≠ 5 daju imenovanu grešku | F2b-1 | 6 | P3 | — |
| F2b-3 | Pohrana i API | `class_situations`, CRUD samo za vlasnika razreda | tuđi razred 404; nevaljan 422 s poljem; izmjena nakon starta odbijena | F2b-2, F1-5 | 7 | P3 | — |
| F2b-4 | Spoj u snimak | pri „Pokreni" predlošci idu u snimak pravila (`GameCreated.snapshot`) | igra se ponovno folda; bez dodanih bajt-isto | F2b-3, F1-8 | 5 | P3 | — |
| F2b-7 | Web: uređivač situacija | forma, popis, „pogledaj kao student", samo prije starta | Playwright | F2b-3 | 14 | P3 | D10 |
| F2b-9 | E2E vlastite situacije | profesor doda → student dobije → odgovor → presuda → obračun | e2e zelen | F2b-4, F1-17 | 4 | P3 | — |
| F2b-5 | Smještaj u krug (pin) | polje „krug/mjesec" (`serde(default)`) | situacija s pin dolazi točno u tom krugu | F2b-4 | 8 | P4 | D12 |
| F2b-6 | Broj i vrste situacija | postavke razreda: najviše po krugu, uključene `Kind` vrste | isključena vrsta se ne izvuče; balans `--check` | F1-5, F2b-4 | 6 | P4 | D11, D12 |
| F2b-8 | AI pomoć pri pisanju | Sonnet predloži odgovore i rubriku, profesor odobri; trošak u `ai_usage` | izlaz prolazi `check` ili odbijen | F2b-3, F0-9 | 10 | poslije | D12 |
| F2b-10 | Uređivač lika (ADR-021) | profesor uređuje opis/uvod lika | lik prolazi provjeru | F2b-7 | 10 | poslije | — |

F2-7, F2-10 i F2-11 su u PILOTU. Sati: P3 = 83 (F2-1…6 43 + F2b 40), P4 = 14, poslije = 20.

---
### FAZA PLATFORMA — PILOT (P4; 19 cigli, 95 h)

**Gotovo kad:** razred od ~30 studenata odigra semestar na internetu pod vlastitom domenom, s propisanim čuvanjem i brisanjem podataka, bez
gubitka događaja, i profesor može ispraviti presudu. Pravni odgovori fakulteta traju tjednima: pitanja poslati odmah (G-6).

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| F0-7 | Uloga baze bez prava nad `events` | migracija: aplikacijska uloga bez UPDATE/DELETE/TRUNCATE, zasebna uloga održavanja | `DELETE FROM events` aplikacijskom ulogom pada na pravu | F0-6 | 4 | P4 | — |
| F0-8 | Radnici i trake | N paralelnih radnika (`main.rs:123-136`), trake razgovor/presude/obračun, fix deadlocka expiry | 30 obračuna + 30 razgovora, razgovor ne čeka iza obračuna | F0-1 | 6 | P4 | — |
| F0-12 | CI dopune | akcije po SHA, `pnpm audit`, cache `sqlx-cli`, Playwright e2e s pravom bazom | e2e hvata namjerno pokvaren ekran | F0-11 | 6 | P4 | — |
| F0-15 | Alat za opterećenje | `oha`/`k6`: prijava, 30 klijenata, p50/p95 | skripta ispiše brojke | F0-10 | 3 | P4 | — |
| F0-S3 | Vlastita domena | `hotelixsim.com` (Cloudflare Registrar), CNAME `app.`, TLS od Railwaya | `https://app.hotelixsim.com/v1/health` 200 | F0-S1 | 2 | P4 | **Leon kupuje (~10 $/god), D6** |
| F0-S4 | Tok isporuke | `main` → staging; tag `v0.x.y` → produkcija uz obaveznog odobravatelja Leona (ADR-005) | tag čeka odobrenje | F0-S1, F0-12 | 3 | P4 | — |
| F0-S5 | Kopije izvan Railwaya | noćni `pg_dump` u EU spremnik, jedna stvarna proba vraćanja | datoteka < 24 h; restore zelen | F0-S1, F0-13 | 3 | P4 | trošak ~0–5 € |
| F2-7 | Ispravak presude | `VerdictOverridden` + kompenzacija u sljedećem krugu (ne ponovni obračun); po hotelu; audit | učinak točno jednom; fold determinističan | F1-10, F2-6 | 20 | P4 | — |
| F2-10 | Izvoz CSV | napredak, pet indeksa po krugu, AI trošak | sadržaj = projekcija; student 403 | F2-1 | 5 | P4 | — |
| F2-11 | Uklanjanje studenta | hotel se smrzne ili igra na zadanim potezima (neaktivni plan); roboti samo za vježbu | ostali nastavljaju; ljestvica „ispao" | F1-10 | 5 | P4 | **D7** |
| G-1 | Obavijest o privatnosti | kratka obavijest + upozorenje „ne upisuj osobne podatke" | ekran; tekst odobren | F1-14 | 3 | P3 | tekst |
| G-2 | Rok čuvanja | posao briše razred i igru nakon roka; zapis u `audit_log` | stara igra nestane, ostale nedirnute | F0-7 | 8 | P4 | **rok (pitanje fakultetu)** |
| G-3 | Brisanje studenta u zajedničkoj igri | anonimizacija (veza račun ↔ pseudonim, imena) + redakcija slobodnog teksta/AI razgovora uz `audit_log` | iz baze se ne dođe do imena/e-pošte; ostali hoteli nepromijenjeni | F1-6, F0-7 | 8 | P4 | **pravni odgovor fakulteta** |
| G-4 | Izvoz vlastitih podataka | JSON izvoz studentova hotela | samo njegovi podaci | F1-12 | 4 | P4 | — |
| G-5 | Brisanje demo gostiju | posao: demo gosti i igre starije od 7 dana (ADR-016) | stari nestanu | F0-8 | 3 | P4 | — |
| G-6 | Pravni popis | popis obrade, podizvršitelji (Railway, Sentry, Anthropic), nacrt za službenika | dokument u `docs/`; pitanja poslana | — | 3 | P4 | **šalje Leon fakultetu** |
| G-7 | Proba vraćanja kopije | mjesečna, na stvarnoj kopiji staginga | zapis s datumom i brojem događaja | F0-S5 | 3 | P4 | — |
| G-8 | Proba 30 studenata | 30 klijenata u jednoj igri na stagingu: p95, zaključavanje, AI trošak | p95 < cilj; nema `failed` poslova; D3 granica | F0-15, F1-17, F0-S1 | 4 | P4 | D3 |
| G-9 | Čuvanje IP-a | zapisi bez trajnog IP-a, 14 dana [PROC] | nakon roka nema IP-a | F0-3 | 2 | P4 | — |

G-1 je jedina P3 cigla ove faze (3 h). Prave greške koje valja riješiti uz pilot: [kvaliteta.md](#roadmap--kolosijek-kvaliteta).

---
### FAZA PLATFORMA — poslije (smjer; ~105 h)

| id | cigla | što se radi | dokaz | h | Leon |
|---|---|---|---|---|---|
| GENERATOR HOTELA | (17.10.: dolazi **nakon** BANKE HOTELA i samo je puni, niže) F3-1…8: opis hotela u snimku (`HotelSetup.profile` → `"gen:1"`), `generate(sjeme, postavke, slot)`, banka imena (bez stvarnih hotela s Kvarnera, ADR-002), referenca po hotelu, balans `--generate`, postavka raznolikosti | A/B/C bajt-isto; proptest determinizma | 56 | popis imena, prag; ovisi o vrstama soba i obnovi |
| AAI@EduHr | F5-1 generički OIDC (`openidconnect`, PKCE) · F5-2 Google/Microsoft za profesora · F5-3 Supabase Auth kao posrednik (SAML) · F5-4 povezivanje na isti `user_id` (povijest ostaje) · F5-5 TOTP za admina · **čim fakultet odobri** (AAI-0) | test s lažnim IdP-om; ista povijest nakon povezivanja | 29 | D4; ovisi o fakultetu i Srcu |
| SSE | `LISTEN/NOTIFY` + ruta, samo ako G-8/polling pokaže potrebu | G-8 | 8 | — |
| ENGLESKI (sučelje) | `packages/i18n/src/en.ts` uz `hr.ts` (627 redaka), tipiziran lokal, izbor jezika, `format.ts` bez tvrdog hr-HR ([BACKLOG.md:67](../records/BACKLOG.md)), `t()` tipizirani parametri ([:69](../records/BACKLOG.md)); sadržaj i prompti su u [igra.md](#roadmap--kolosijek-igra) | ista igra na hr i en; brojevi i datumi po lokalu | ~12 | popis jezika |

---
### Nove cigle iz Leonovih odluka 17.10. ([ADR-056](../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md); mjesta 🟨 voditelja)

Leon: *„ovo što već imamo je pilot, a ovo što gradimo je već produkcijska platforma; spremamo se za profesorov DA."* Standard
„gotovo" je **40 studenata igra tri mjeseca bez Leona uz računalo**. Redoslijed 🟨: [ROADMAP](./ROADMAP.md) „Pilot → produkcija".

| id | cigla | što se radi | dokaz | ovisi o | h [PROC] | P | Leon |
|---|---|---|---|---|---|---|---|
| AI-M | Mjerenje AI troška po igri | prije pilota: Leon odigra jednu tjednu v2 sezonu sa situacijama i razgovorima, zbroj tokena po modelu iz dnevnika × cijene (upit u [planu platforme](./2026-11-platforma.md) §5); ista projekcija kao F0-9 | zapisan trošak po igri i po studentu, ne procjena | F0-9 | ~2 | prije P4 | igra jednu sezonu |
| BH-P | **BANKA HOTELA: platforma** | profesor pri stvaranju razreda dodjeljuje hotel iz banke **po studentu ili nasumce**; dodjela se zaključava u „Pokreni" i ide u `GameCreated` | dva studenta s različitim hotelima u istoj igri; nasumična dodjela ista za isto sjeme | F1-8, BH-J ([igra.md](#roadmap--kolosijek-igra)) | [PROC] ? | 🟨 nakon razredne igre | — |
| PR-1 | Profesorovo sučelje: temelji (E6) | strukturiran izbornik AI situacija (po **odjelu hotela**), vlastite situacije, pregled razgovora studenta s gostom ili zaposlenikom, ocjena po njima, mentor uključen ili isključen, otvaranje tjedna ili mjeseca (student čeka); Leon: *„trebat će nam nekoliko sesija da inženjerski postavimo temelje"* → najprije nacrt u [PLATFORMA](../architecture/PLATFORMA.md), pa cigle u fazi PROFESOR | nacrt s raspravom (ADR-006) | F2-1 | više sesija [PROC] | P3–P4 | rasprava o nacrtu |

---
### Odluke (Leon)

Odlučeno 13.10.: D1 obračun po hotelu · raspored i rokovi · neaktivan student (hotel sam dalje, mentor upozori) · obavijesti u aplikaciji · AI granica po igri/semestru ·
generator računa prije AAI. **Otvoreno:** D4 profesor za probu · D2 lobby · D3 veličina igre (preporuka 40, mjeri G-8) · D6 domena i izlazak (OK + trošak) ·
D7 uklonjeni student · D8 imena ili nadimci · D10 sloboda profesora (forma) · D11 dani kao razdoblje (preporuka ne) · D12 AI situacije unaprijed ili u hodu · plaća tuđeg radnika (anketa).

### Sažetak ([PROC])

| faza | cigli | h | P |
|---|---|---|---|
| SPREMNO ZA INTERNET | 13 | 48 | P2 (P3: F0-13, 14, S2) |
| RAZRED | 21 | 116 | P2 |
| PROFESOR | 16 | 117 | P3 83 · P4 14 · poslije 20 |
| PILOT | 19 | 95 | P4 (G-1 P3) |
| poslije | 4 grupe | ~105 | poslije |

### Rizici

Dvije grane na istim datotekama (`api/schema.rs` + `openapi.json`, migracije: dogovoriti raspon brojeva) — zato K0-4 u kvaliteta.md prije; jedna igra s 30 pisaca
(F0-10 preduvjet, G-8 dokaz); GDPR u zajedničkoj igri (G-3, odgovor fakulteta kasni); AI je najveća stavka troška (RZ-4); F2b blokira nespojena grana s1-sit.
Najviše 2 agenta koji prevode Rust istodobno.
