# Popravak temelja — faze A, B i C — plan izgradnje

**Status:** 🟩 AKTIVAN · **Područje:** igra · **Otvoren:** 2026-10-14 · **Grana:** integracijska `model-v2/sloj1`;
svaka cigla na kratkoj grani `popravak/<cigla>` u radnom stablu `../hotelixsim-<cigla>`, spaja se isti dan.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Igra stoji na temelju koji mjeri pravu stvar: provjere ruše kad pukne model ili načelo igre (a ne kad se
robotu promijeni plan), v2 situacije više ne posuđuju v1 mehaniku, balans se kalibrira jednom, a igra koja traje
semestar preživi popravak jezgre.

**Architecture:** Ništa se ne prepisuje ([ADR-043](../records/DECISIONS/ADR-043-popravak-temelja-ne-gradnja-ispocetka.md)).
Faza A dodaje brane i testove oko postojećeg koda (bez promjene brojki, osim svjesnog dizanja `CORE_VERSION`). Faza B
zamjenjuje v1 kalup ondje gdje već puca i završava jednom kalibracijom. Faza C priprema igru za semestar i razred.

**Tech Stack:** Rust 2024 (1.98) · `engine::calc` (cijeli brojevi) · serde/serde_json izvan jezgre · axum 0.8 + utoipa 6 ·
React 19 · Vitest · `node --test` za skripte.

**Spec:** [REVIZIJA_TEMELJA.md](../architecture/REVIZIJA_TEMELJA.md) §4 (nalazi #1–#15) i §6 (faze) · puni izvještaji
revizora `.superpowers/sdd/2026-10-14-revizija/01…06-*.md` · [ADR-042](../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md)
· [BUGS.md](../records/BUGS.md) B1–B10 · tok rada [AGENTI.md](../workflow/AGENTI.md). Ledger:
`.superpowers/sdd/2026-10-14-popravak-temelja/progress.md`. Preuzima L1-11 i L1-13…L1-15 iz pauziranog
[plana Sloja 1](./2026-10-10-v2-sloj1.md).

## Pretpostavke 🟨 (čekaju Leonov pregled)

| # | pretpostavka 🟨 | ako Leon kaže drukčije, mijenja se |
|---|---|---|
| P1 | Dominantna proba (§11.4) ostaje **brana modela** (čvrst plan, bez strategije robota) | A1: jedan poziv seli u `robot_measures_v2` |
| P2 | v1 balans u CI-ju ostaje na **100 sjemena** (v1 je zamrznut i čuvan SHA testom); ispravlja se ADR-033 t. 8, ne CI | A1 korak K |
| P3 | Načelo duljine: **sve oznake odgovora unutar 1,5×** najkraće (danas 0 od 55; srednji omjer 4,3×) — **potvrdio Leon 15.10.** | A2 (konstanta), B4 (prepisivanje) |
| P4 | Načelo „šutnja nije bolja od najlošijeg odgovora" gleda **bodove i trošak** (16 predložaka u dugu); samo bodovi → dug 0 | A2 |
| P5 | `{odjel}` je dopušten iza „odjel*" **i u zagradi** „({odjel})" (5 naslova, ispravan hrvatski) | A2 |
| P6 | Pet predložaka dobiva `external_cause: true` (oluja, toplinski val, nestanak struje, dobavljač, terminal, po ADR-042 t. 7); P-INSPEKCIJA ostaje u dugu | A2, B4 |
| P7 | `CORE_VERSION` se diže na **2.2.0** prije pečaćenja (J-3, G-1, L1-11 promijenili su obračun bez dizanja). **Leonove probne v2 igre tada daju 409** i treba nova igra | A3 |
| P8 | Igra i balans: „**igra je istina**"; ako se razilaze, popravlja se strana balansa | A4 korak 5 |

## Global Constraints

Vrijede `.superpowers/sdd/2026-10-10-v2-sloj1/global-constraints.md` i `implementer-common.md`. Uz to:
- **v1 se ne mijenja ni u bajtu:** `cargo run -q -p balance --release -- --seeds 100 --check` i `--seeds 1000` `cmp`
  prema osnovici, zlatni testovi v1 (`crates/game/tests/v1_frozen.rs`) zeleni.
- **Svaka promjena koja mijenja brojke v2 igre diže `CORE_VERSION`** (`crates/sim-hotel-v2/Cargo.toml`) i obnavlja otisak
  iz A3. Vrijedi i za `content::instance`, `game::drawn` i robote u `game`, ne samo za `sim-hotel-v2`.
- **Svaka cigla prije predaje vrti cijeli popis provjera iz CI-ja** (`.github/workflows/ci.yml`: fmt, clippy, core-deps,
  testovi, balansi, check:docs, test:docs, check:kod, check:api, lint, typecheck, test, web build), ne samo kartične (B12).
- **Tekstovi poruka brana balansa ostaju doslovno isti** (usporedba s ~40 povijesnih izvještaja).
- **Najviše 2 agenta istodobno prevode Rust.** Sva stabla dijele `CARGO_TARGET_DIR=C:/Users/leonk/Documents/hotelixsim/target`;
  samo osnovica (B8) ima privremeni target koji se briše odmah. Stablo koje pokreće poslužitelj ima svoju bazu (B9).
  Disk ostaje iznad 100 GB slobodno; stablo cigle se briše isti dan kad se spoji (B11).
- **Grana cigle živi najviše ~1 dan**; nakon svakog spajanja balans v2 tjedno i mjesečno s 1000 sjemena i `--check`.
- **Brojke i pravila koje student osjeti** idu Leonu kroz anketu prije gradnje (memorija „odluke o igri pitaj Leona").
- **Živi AI test i push** samo uz Leonov izričit OK za tu radnju.

## Review Focus

Ulazi i stanja koje nijedan test faze A ne pokriva, a najvjerojatnije bi ugrizli studenta ili Leona:
1. **Igra u tijeku nakon popravka jezgre** (P7, A3 diže verziju): student dobije „Nešto nije u redu", bez izlaza
   (REVIZIJA #6). Vlasnik: **C1**. Do tada voditelj Leonu uz svako dizanje verzije kaže da su probne v2 igre zastarjele.
2. **Student prepusti sve dopuštene situacije voditelju** (ocjena 2): tu sezonu balans nikad nije vrtio (A4). Vlasnik:
   **B3**, nova mjera udjela mora uključiti put prepuštanja i šutnje.
3. **AI padne na 6. poruci:** razgovor završi rezervnom rečenicom u trećem licu (A5). Vlasnik: **C3** (podrijetlo retka
   razgovora), u A5 samo zabilježeno.
4. **Tjedan preko granice mjeseca (t05)**: tjedni sažetak, mjesečni mentor i plaćanje obnove u istom krugu; nijedan
   scenarij A6 ga ne prolazi. Vlasnik: provjeravatelj A6 dodaje scenarij ako stane u karticu, inače **B** (scenariji uz B1).
5. **Sidra S1–S4** (v1 sadržaj u svakoj v2 igri) nisu pod načelima A2, a najbolji odgovor im je prvi i najdulji.
   Vlasnik: **B4** (prepisivanje oznaka uključuje sidra; prikaz miješa redoslijed po ADR-042 t. 10).

## Graf ovisnosti i valovi

```text
A0 kopija ✅ ─► A1 brane + spajanje situacija ─┬─► val 2: A2 načela ∥ A3 otisak v2
                                              ├─► val 3: A4 igra = balans ∥ A5 granice AI-ja
                                              ├─► val 4: A6 scenariji
                                              └─► A7 način rada (ne prevodi Rust, uz bilo koji val)
kraj faze A: kopija, Leon igra mjesečnu v2 igru na jednoj adresi ─► faza B
```

Faza B, redom (Leon odobrio okvir 15.10.; ne paralelno, jer B2a/B1a dižu `CORE_VERSION` i pišu `v2-golden.json`, a B2b/B3a diraju `crates/balance/src/v2.rs`):

```text
B2a ─► B2b ─► B3a ─► [anketa Leonu s brojkama B3a] ─► B3b ─► B1a ─► B1b ─► B4a ─► B4b ─► B4c ─► L1-13 ─► L1-13b ─► L1-14 ─► L1-15
                       pravilo „vlastiti odgovor vrijedi više" (nacrt + anketa) mora sletjeti prije L1-13
```
Razlozi: B3 mjeri tek nakon B2a (B2a mijenja mjesečni udio situacija); B1a ne prije B3b (brane u `failures_v2`, I1);
B4 nakon B1b (uvjeti kvarova trebaju `condition_of`). Ako krivo: kasniji B4 (~1 dan).
Valovi poštuju „najviše 2 agenta prevode Rust". A1 i A4 diraju `crates/balance`, pa nikad zajedno.

## Faza A — učvrsti (~25–35 h)

**Gotovo kad:** crveni `--check` znači da je pukao model; server odbija novi predložak koji krši načelo igre; promjena
obračuna v2 bez dizanja verzije padne u CI-ju; igra i balans daju istu sezonu robota; lik zna da je razgovor na kraju;
pet scenarija „što student vidi" je zeleno; Leon odigra mjesečnu v2 igru sa situacijama na jednoj adresi.

| task | kartica | veličina |
|---|---|---|
| A1 | [Razdvoji brane balansa i spoji situacije u sloj1](2026-10-14-popravak-temelja/task-01-a1-razdvoji-brane-i-spoji-situacije.md) | S–M, ~3 h |
| A2 | [Načela igre kao provjere sadržaja](2026-10-14-popravak-temelja/task-02-a2-nacela-igre-kao-provjere-sadrzaja.md) | M, 6–8 h |
| A3 | [Zapečaćeni otisak sezone v2](2026-10-14-popravak-temelja/task-03-a3-zapecaceni-otisak-sezone-v2.md) | S, 2–3 h |
| A4 | [Igra i balans daju isto](2026-10-14-popravak-temelja/task-04-a4-igra-i-balans-daju-isto.md) | S, 2–3 h (+3) |
| A5 | [Granice igre u ulazu AI-ja](2026-10-14-popravak-temelja/task-05-a5-granice-igre-u-ulazu-ai.md) | M, 4–6 h |
| A6 | [Scenariji „što student vidi"](2026-10-14-popravak-temelja/task-06-a6-scenariji-sto-student-vidi.md) | M, 4–5 h |
| A7 | [Kratke grane i način rada](2026-10-14-popravak-temelja/task-07-a7-kratke-grane-i-nacin-rada.md) | S, 2–3 h |

## Faza B — zamijeni v1 kalup gdje puca (~35–50 h) · kartice napisane 15.10.

| cigla | što | ovisi o | Leonov dio |
|---|---|---|---|
| B2a ✅ 15.10. (143f593) | [Broj situacija u postavkama](2026-10-14-popravak-temelja/task-09-b2-broj-situacija-u-postavkama.md): jezgra, dnevnik (`GameCreated.situation_counts`) i pečat; razmak i prilika u sadržaj; mjesečno 4/5/6; **`CORE_VERSION` 2.2.0 → 2.3.0**; priprema `Case` s postavkama i `common::v2_game_with` · ADR-042 t. 12–14, REVIZIJA #13 · 7–8 h | A1, A3 | — (odlučeno) |
| B2b ✅ 15.10. (635e647) | [Kartica B2b](2026-10-14-popravak-temelja/task-09b-b2b-balans-prima-broj-situacija.md): balans `--situations` i paritet s igrom, brojke bez promjene · 2–3 h | B2a | — |
| B3a ✅ 15.10. (76271d1) | [Mjera udjela situacija](2026-10-14-popravak-temelja/task-10-b3-mjera-udjela-situacija.md): mjerač 5 putova (najbolji / srednji / prepušteno / šutnja / bez) na zadanom planu, ispis i paritet s igrom; bez `CORE_VERSION` | B2b, A1, A4 | **anketa „~20 % od čega" nakon brojki B3a** |
| B3b ✅ 15.10. (b94264a) | [Isto](2026-10-14-popravak-temelja/task-10-b3-mjera-udjela-situacija.md): brane u `failures_v2` (I1); pojas težine 5–50 % samo ako B3a (mjereno nakon B2a) leži unutar njega s marginom, inače samo smjer (Δ > 0) | B3a, anketa | — |
| B1a ✅ 15.10. (39e5f25) | [v2 učinak situacije u prostor odjela](2026-10-14-popravak-temelja/task-08-b1-v2-mehanike-situacija.md): popravak diže stanje **tog** odjela (+10, i uz ocjenu 1), načelo šutnje na v2 (I2), pečat A3 + **slučajevi I3** (slobodni odgovor s presudom, plan čovjeka); **`CORE_VERSION` 2.3.0 → 2.4.0** · ADR-042 t. 1, 7, REVIZIJA #2 | B3b, B2a, A2, A3 | — (odgovoreno: popravak i uz ocjenu 1, Leon 15.10.) |
| B1b ✅ 16.10. (cc4a0f2) | [Kartica B1b](2026-10-14-popravak-temelja/task-08b-b1b-uvjet-condition-of.md): `condition_of` (uvjet stanja „čijeg odjela"), balans bajt-isti, bez dizanja verzije · ≈ 85 redaka | B1a | — |
| B4a ✅ 16.10. (6e8ff2e) | [Prepis oznaka i uvjeta](2026-10-14-popravak-temelja/task-11-b4-prepis-oznaka-i-uvjeta.md): alat, testovi, sidra S1–S4 (prepis u **zajedničkoj** datoteci), uvjeti klime i praonice, 15 predložaka (rooms, guest_facing) · ~6 h | B1b, A2 | pregled tekstova (slijepi); odgovoreno: 1,5×, zajednička datoteka. **Prije spajanja vrti `--check` 1000 tjedno i mjesečno** (tjedna težina B je 48,9 % uz granicu 50 %, margina 1,1 boda; bonus 200 bi je digao na 57,6 %) |
| B4b ✅ 16.10. (8d134a4) | [Isto](2026-10-14-popravak-temelja/task-11-b4-prepis-oznaka-i-uvjeta.md): 22 predloška (hotel, staffed, admin, sales, maintenance, parking) · ~4 h | B4a | pregled tekstova |
| B4c 🔄 napola (c69f022: oznake; uvjeti kvarova čekaju Leonov prag, [ADR-052](../records/DECISIONS/ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md)) | [Isto](2026-10-14-popravak-temelja/task-11-b4-prepis-oznaka-i-uvjeta.md): 18 predložaka (restoran, bar, spa, rooftop, lobby) + uvjeti kvarova P-BRAVE, P-CURENJE-VODE, P-DIZALO (preko `condition_of`); P-INSPEKCIJA = vanjski događaj (`external_cause`, Leon 15.10.); „dug pada na 0" = oznake i uvjeti kvarova, **ne** šutnja (`SilenceNotBetter` 16 → L1-13) · ~4 h | B4b | pregled tekstova |
| PRAVILO | **Vlastiti odgovor vrijedi više** (Leon 15.10.: student koji sam odgovori gostu može dobiti više bodova od onoga koji samo klikne najbolje rješenje). Odlučeno: [nacrt](../architecture/BODOVANJE_VLASTITOG_ODGOVORA.md), opcija A, [ADR-047](../records/DECISIONS/ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md) — klik najviše 4, vlastite riječi do 5. Lanac B3 postaje vlastiti > klik najbolji > srednji > prepušteno > šutnja. Mora sletjeti **prije L1-13**; ~10–14 h. **B3b je spojen (15.10.) — kartica se piše sada** (gradi na `GameSettings` iz B2a i mjeraču putova iz B3a) | B2a, B3b | — (odlučeno) |
| L1-13 | **Jedna** kalibracija: situacije ~20 % po novoj mjeri, ostale brane modela; preuzima dug `SilenceNotBetter` (16, P4). **„Oko +10" za popravak je Leonova odluka** ([ADR-050](../records/DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md); privremeno +1, uvjet C5), uz čestoću kvarova i težinu ~20 % ([ADR-049](../records/DECISIONS/ADR-049-situacije-dvadeset-posto-mjeri-se-od-ravnala.md): pojas 5–50 % → 15–25 %). Dug redoslijeda je već otplaćen u B1a (`ORDER_DEBT` prazan): brišu se još `ORDER_DEBT_FLOOR`, test `order_debt` i grana `clean(Month)` | B4c, PRAVILO | brojke koje student osjeti |
| L1-13b | Ostatak ADR-042: održavanje +5, jedno spremanje, izmiješani odgovori, izgled lika po rodu (rod u `RoleFill`), opis Smještaja | L1-13 | — (odlučeno) |
| L1-14 | Igriva verzija: Leon igra tjednu i mjesečnu sezonu | L1-13b | popis izmjena |
| L1-15 | `sloj1` → `main` | L1-14 | **izričit OK za push** |

## Faza C — igra za semestar i razred (~50–90 h) · obris

| cigla | što | REVIZIJA |
|---|---|---|
| C1 | Stara igra se čita i nakon promjene jezgre (provjera verzije iz `fold` u obračun), početno stanje i referentna tablica u dnevnik, događaj „pravila nadograđena od kruga k"; web poruka i gumb „Nova igra" za 409 | #6, 01 #3 |
| C2 | Više radnika za poslove, zaseban red za lika, neuspjeli poslovi vidljivi u zdravlju sustava i ponovno pokretanje | #9 |
| C3 | `presuda.v4` (opis situacije, uloga lika, cijena odgovora), mjerilo AI-ja na v2 predlošcima preko `Game::assess_input` + Leonove ocjene, podrijetlo rezervnog retka razgovora | #11, #12 |
| C4 | Predmemorija složene igre (ključ: zadnji `seq`) | 05 #3 |
| C5 | Pravila iz weba u API: ocjena odjela, „Obvezna", brojke u tekstovima, kodovi problema u ugovoru | #15 |
| C6 | Razdvojiti `api/src/schema.rs` i `game/src/mentor.rs` (preduvjet dva kolosijeka, dug `check:kod` pada) | 06 #6 |

Profesorov pregled i ispravka presude, obračun po hotelu, prijava i osobe **nisu popravak**: to su planirane faze
(PROFESOR, RAZRED U JEZGRI, RAZRED, LJUDI) iz [ROADMAP-a](./ROADMAP.md) i kreću nakon faze C.

## Ukupno

Faza A ~25–35 h, B ~35–50 h (+ L1-13…L1-15 ~20 h iz plana Sloja 1), C ~50–90 h [PROC]. Svaka faza završava kopijom
(ADR-043 t. 6), Leonovim igranjem i završnim pregledom najjačim modelom.
