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
[plana Sloja 1](../archive/2026-10-10-v2-sloj1.md).

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
- **Najviše 2 agenta istodobno prevode Rust.** Svako stablo ima svoj `CARGO_TARGET_DIR=C:/Users/leonk/Documents/hotelixsim/target/<stablo>` (zajednički daje zastarjelu biblioteku, BUGS B12, AGENTI t. 6);
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
IZGLED (izgled lika iz roda, dobi i podrijetla) ide usporedo s PRAVILO-om, izvan ovog reda: dira samo app, API i web (Leon 16.10., sesija 7).
Razlozi: B3 mjeri tek nakon B2a (B2a mijenja mjesečni udio situacija); B1a ne prije B3b (brane u `failures_v2`, I1);
B4 nakon B1b (uvjeti kvarova trebaju `condition_of`). Ako krivo: kasniji B4 (~1 dan).
Valovi poštuju „najviše 2 agenta prevode Rust". A1 i A4 diraju `crates/balance`, pa nikad zajedno.

## Faza A — učvrsti (~25–35 h)

**Gotovo kad:** crveni `--check` znači da je pukao model; server odbija novi predložak koji krši načelo igre; promjena
obračuna v2 bez dizanja verzije padne u CI-ju; igra i balans daju istu sezonu robota; lik zna da je razgovor na kraju;
pet scenarija „što student vidi" je zeleno; Leon odigra mjesečnu v2 igru sa situacijama na jednoj adresi.

| task | kartica | veličina |
|---|---|---|
| A1 | [Razdvoji brane balansa i spoji situacije u sloj1](../archive/2026-10-14-popravak-temelja-kartice/dio-1.md#task-a1-razdvoji-brane-balansa-brane-modela--mjere-robota-i-spoji-situacije-u-sloj1) | S–M, ~3 h |
| A2 | [Načela igre kao provjere sadržaja](../archive/2026-10-14-popravak-temelja-kartice/dio-1.md#task-a2-načela-igre-kao-provjere-sadržaja-pri-učitavanju-s-popisom-duga-koji-smije-samo-padati) | M, 6–8 h |
| A3 | [Zapečaćeni otisak sezone v2](../archive/2026-10-14-popravak-temelja-kartice/dio-1.md#task-a3-zapečaćeni-otisak-sezone-v2-vezan-uz-core_version) | S, 2–3 h |
| A4 | [Igra i balans daju isto](../archive/2026-10-14-popravak-temelja-kartice/dio-2.md#task-a4-jedna-istina-o-sezoni-robota--igra-i-balans-daju-isto) | S, 2–3 h (+3) |
| A5 | [Granice igre u ulazu AI-ja](../archive/2026-10-14-popravak-temelja-kartice/dio-2.md#task-a5-granice-igre-u-ulazu-ai-ja-likv3-i-provjera-izlaza-mentora-kruga) | M, 4–6 h |
| A6 | [Scenariji „što student vidi"](../archive/2026-10-14-popravak-temelja-kartice/dio-2.md#task-a6-scenariji-što-student-vidi-na-razini-api-ja) | M, 4–5 h |
| A7 | [Kratke grane i način rada](../archive/2026-10-14-popravak-temelja-kartice/dio-2.md#task-a7-kratke-grane-jedna-integracija-i-ledger-kao-stanje) | S, 2–3 h |

## Faza B — zamijeni v1 kalup gdje puca (~35–50 h) · kartice napisane 15.10.

| cigla | što | ovisi o | Leonov dio |
|---|---|---|---|
| B2a ✅ 15.10. (143f593) | [Broj situacija u postavkama](../archive/2026-10-14-popravak-temelja-kartice/dio-3.md#task-b2a-broj-situacija-u-postavkama-igre-mjesečno-456-isti-predložak-najviše-jednom-u-krugu): jezgra, dnevnik (`GameCreated.situation_counts`) i pečat; razmak i prilika u sadržaj; mjesečno 4/5/6; **`CORE_VERSION` 2.2.0 → 2.3.0**; priprema `Case` s postavkama i `common::v2_game_with` · ADR-042 t. 12–14, REVIZIJA #13 · 7–8 h | A1, A3 | — (odlučeno) |
| B2b ✅ 15.10. (635e647) | [Kartica B2b](../archive/2026-10-14-popravak-temelja-kartice/dio-3.md#task-b2b-balans-prima-broj-situacija-kao-ulaz-paritet-s-postavkama): balans `--situations` i paritet s igrom, brojke bez promjene · 2–3 h | B2a | — |
| B3a ✅ 15.10. (76271d1) | [Mjera udjela situacija](../archive/2026-10-14-popravak-temelja-kartice/dio-4.md#task-b3-mjera-udio-situacija-na-zadanom-planu--b3a-mjerač-i-ispis-b3b-brane-modela): mjerač 5 putova (najbolji / srednji / prepušteno / šutnja / bez) na zadanom planu, ispis i paritet s igrom; bez `CORE_VERSION` | B2b, A1, A4 | **anketa „~20 % od čega" nakon brojki B3a** |
| B3b ✅ 15.10. (b94264a) | [Isto](../archive/2026-10-14-popravak-temelja-kartice/dio-4.md#task-b3-mjera-udio-situacija-na-zadanom-planu--b3a-mjerač-i-ispis-b3b-brane-modela): brane u `failures_v2` (I1); pojas težine 5–50 % samo ako B3a (mjereno nakon B2a) leži unutar njega s marginom, inače samo smjer (Δ > 0) | B3a, anketa | — |
| B1a ✅ 15.10. (39e5f25) | [v2 učinak situacije u prostor odjela](../archive/2026-10-14-popravak-temelja-kartice/dio-3.md#task-b1-vlastite-v2-mehanike-situacija-s-ciljem-odjela--podijeljeno-na-b1a-učinak-na-prostor-odjela-i-b1b-uvjet-čijeg-prostora-condition_of): popravak diže stanje **tog** odjela (+10, i uz ocjenu 1), načelo šutnje na v2 (I2), pečat A3 + **slučajevi I3** (slobodni odgovor s presudom, plan čovjeka); **`CORE_VERSION` 2.3.0 → 2.4.0** · ADR-042 t. 1, 7, REVIZIJA #2 | B3b, B2a, A2, A3 | — (odgovoreno: popravak i uz ocjenu 1, Leon 15.10.) |
| B1b ✅ 16.10. (cc4a0f2) | [Kartica B1b](../archive/2026-10-14-popravak-temelja-kartice/dio-3.md#b1b-uvjet-stanja-čijeg-prostora-condition_of): `condition_of` (uvjet stanja „čijeg odjela"), balans bajt-isti, bez dizanja verzije · ≈ 85 redaka | B1a | — |
| B4a ✅ 16.10. (6e8ff2e) | [Prepis oznaka i uvjeta](../archive/2026-10-14-popravak-temelja-kartice/dio-4.md#task-b4-prepis-oznaka-odgovora-55-predložaka--sidra-s1s4-i-uvjeta-kvarova--podijeljeno-b4a--b4b--b4c): alat, testovi, sidra S1–S4 (prepis u **zajedničkoj** datoteci), uvjeti klime i praonice, 15 predložaka (rooms, guest_facing) · ~6 h | B1b, A2 | pregled tekstova (slijepi); odgovoreno: 1,5×, zajednička datoteka. **Prije spajanja vrti `--check` 1000 tjedno i mjesečno** (tjedna težina B je 48,9 % uz granicu 50 %, margina 1,1 boda; bonus 200 bi je digao na 57,6 %) |
| B4b ✅ 16.10. (8d134a4) | [Isto](../archive/2026-10-14-popravak-temelja-kartice/dio-4.md#task-b4-prepis-oznaka-odgovora-55-predložaka--sidra-s1s4-i-uvjeta-kvarova--podijeljeno-b4a--b4b--b4c): 22 predloška (hotel, staffed, admin, sales, maintenance, parking) · ~4 h | B4a | pregled tekstova |
| B4c ✅ 16.10. (f7ac7cc; oznake 71 ≤ 1,39×, uvjeti kvarova Smještaj ispod **80**, [ADR-054](../records/DECISIONS/ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md)) | [Isto](../archive/2026-10-14-popravak-temelja-kartice/dio-4.md#task-b4-prepis-oznaka-odgovora-55-predložaka--sidra-s1s4-i-uvjeta-kvarova--podijeljeno-b4a--b4b--b4c): 18 predložaka (restoran, bar, spa, rooftop, lobby) + uvjeti kvarova P-BRAVE, P-CURENJE-VODE, P-DIZALO (preko `condition_of`); P-INSPEKCIJA = vanjski događaj (`external_cause`, Leon 15.10.); „dug pada na 0" = oznake i uvjeti kvarova, **ne** šutnja (`SilenceNotBetter` 16 → L1-13) · ~4 h | B4b | pregled tekstova |
| PRAVILO-a | [Vlastiti odgovor vrijedi više](../archive/2026-10-14-popravak-temelja-kartice/dio-4.md#task-pravilo-vlastiti-odgovor-vrijedi-više-od-klika-klik-najviše-4-vlastite-riječi-do-5) (Leon 15.10., [ADR-047](../records/DECISIONS/ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md), [nacrt](../architecture/BODOVANJE_VLASTITOG_ODGOVORA.md)): pravilo u igri — postavka `AnswerRule` u dnevniku, klik najviše 4, prepisani ponuđeni odgovor vrijedi kao klik, AI ne stigne → 4, mentor zna za ograničenje; **`CORE_VERSION` 2.4.0 → 2.5.0** i pečat A3 (🟨, kartica 16.10.) · 5–6 h | B2a, B3b | — (odlučeno; 🟨 [ADR-055](../records/DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md); **prije koda anketa o pravilu za zalijepljene odgovore**, [BUGS](../records/BUGS.md) B13) |
| PRAVILO-b | [Isto](../archive/2026-10-14-popravak-temelja-kartice/dio-4.md#task-pravilo-vlastiti-odgovor-vrijedi-više-od-klika-klik-najviše-4-vlastite-riječi-do-5): balans — put „klik najbolji", lanac vlastiti > klik > srednji > prepušteno > šutnja, paritet s igrom; težina B3 ostaje na vlastitom putu · 3–4 h | PRAVILO-a | — |
| PRAVILO-c | [Isto](../archive/2026-10-14-popravak-temelja-kartice/dio-4.md#task-pravilo-vlastiti-odgovor-vrijedi-više-od-klika-klik-najviše-4-vlastite-riječi-do-5): napomena studentu („Klikom dobivaš najviše 4 od 5…") kroz app, API i web · 3 h; usporedo s b | PRAVILO-a | — |
| IZGLED-a/b | [Izgled lika iz roda, dobi i podrijetla](../archive/2026-10-14-popravak-temelja-kartice/dio-4.md#task-izgled-izgled-lika-iz-roda-dobi-i-podrijetla) (ADR-042 t. 11, [nacrt](../architecture/IZGLED_LIKA.md) mogućnost A): a — app i API šalju uz lika `look` (rod iz popisa imena, dob, podrijetlo); b — web `lookFor(ime, look)`; jezgra, balans, pečat i `CORE_VERSION` netaknuti · ~5 h | — (usporedo s PRAVILO-om) | pregled lica u pregledniku; 🟨 podrijetlo i dob do K5 |
| IZGLED-c | [Uniforme vanjskih](../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-izgled-c-uniforma-iz-uloge-vanjskog-lika) (ADR-056 C10, Leon 18.10.): uniforma iz uloge — policajac (šapka), vatrogasac (kaciga), djelatnik hitne (crvena, zvijezda života, nikad crveni križ), nova uloga `lijecnik` (kuta, stetoskop); app, API, web; jezgra netaknuta · 🟨 [PROC] | PRAVILO-c | red „Službe" na /likovi |
| L1-13 ✅ 19.10. (e4ec659, `CORE_VERSION` 2.6.0) | [Jedna kalibracija](../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-l1-13-jedna-kalibracija-brojki-kvar-i-popravak-klik-75--značaj-gosti-i-popunjenost-težina-20-) (pregled; puna kartica 18.10.) po Leonovim odlukama 17.10. ([ADR-056](../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)) i 18.10. (klik ~75 %, razine, „zadovoljan gost" ≥ 4); pet dijelova redom 1 → 1b → 2 → 3 → 4, nikad usporedo; dijelovi 1–3 iza ključa sadržaja (balans i pečat bajt-isti), `CORE_VERSION` 2.6.0 jednom u dijelu 4 · 🟨 [PROC] 21–25 h | B4c, PRAVILO-a/b/c | tablica A2 i prije/poslije prije spajanja dijela 4 |
| L1-13 · 1 ✅ 18.10. (7f22802) | [Kvar i popravak](../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-l1-13-dio-1--kvar-i-popravak-a1--l113-1-kvar) (A1): kvar prostora −1, hitni popravak uz ocjenu ≥ 4 +2, ključevi u `economy` (u repou 0) | PRAVILO-c | — |
| L1-13 · 1b ✅ 18.10. (9813b8a) | [Klik nosi ~75 %](../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-l1-13-dio-1b--klik-nosi-75--leon-1810-t-1--l113-1b-klik) (Leon 18.10.): klik gubi dio dobiti (`click_gain_bp`), brana „B3 klik" ≤ 75 % težine vlastitog | L1-13 · 1 | — |
| L1-13 · 2 ✅ 18.10. (fb3a563) | [Tri razine značaja](../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-l1-13-dio-2--tri-razine-značaja-b2-leon-1810-t-2--l113-2-znacaj) (B2): ključna / važna / svakodnevna iz `weight`, faktor bodova, oznaka na ekranu | L1-13 · 1b | — |
| L1-13 · 3 ✅ 18.10. (c1ef169) | [Situacije s gostima i popunjenost](../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-l1-13-dio-3--situacije-s-gostima-rastu-s-popunjenošću-a7--l113-3-gosti) (A7): mjesta iznad niske razine samo gostima | L1-13 · 2 | — |
| L1-13 · 4 ✅ 19.10. (e4ec659) | [Kalibracija i 2.6.0](../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-l1-13-dio-4--kalibracija-a2-a3-a4-a8-i-260--l113-4-kalibracija) (A2, A3, A4, A8): pojas 15–25 %, čestoća kvarova, dug 0, pečat | L1-13 · 3 | tablica A2 |
| L1-13b ✅ 19.10. (dio 2 13db069, 3 4be1db4, 4 bdbdebe, 1 7556d80; [ADR-062](../records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md)) | [Ostatak ADR-042 u valovima](../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-l1-13b-izmiješani-odgovori-pojačano-održavanje-5-jedan-gumb-spremi-plan-opis-smještaja) (Leon 19.10.: cijela prije pilota): izmiješani odgovori, jedan gumb „Spremi plan", opis Smještaja, održavanje +5 (izgled lika je već IZGLED) · 🟨 [PROC] 11–17 h | L1-13 | gdje gumb, što je opis Smještaja, anketa održavanja |
| L1-14 (sljedeće, sesija 12) | Igriva verzija: Leon igra tjednu i mjesečnu sezonu lokalno (izmiješani odgovori, „Spremi plan", opis Smještaja, održavanje +5) | L1-13b | popis izmjena |
| L1-15 | `sloj1` → `main` (popravci iz Leonove igre, pa spajanje) | L1-14 | **izričit OK za push** |
| Pilot | [Priprema pilota](#task-pilot-priprema-pilota-na-railwayu): kolačić 7 → 120 dana prvo, jedna lozinka, AI strop i 3 igre po gostu, Rust poslužuje web, Dockerfile, CI deploy, kopije; Railway Hobby ([ADR-062](../records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md) t. 9) · ~20–27 h | L1-15 | plan prije gradnje; **deploy samo uz izričit OK** |

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

Kartica **Pilot** iz [plana popravka temelja](./2026-10-14-popravak-temelja.md) (redak „Pilot": „kartica piše sesija 12"). Cilj pilota:
1 profesor i 20–40 studenata igraju ovu igru na internetu nekoliko tjedana, na jednoj adresi, bez prijave, uz zaštićen AI trošak.
Izvor posla i brojki: prijedlog hostinga `.superpowers/sdd/2026-10-14-popravak-temelja/pilot-hosting-prijedlog.md` (19.10., §1.1, §3, §4);
kod provjeren 19.10. na `26089c3` (samo čitanje, bez prevođenja). Ništa od ovoga se ne gradi dok Leon ne vidi plan (ADR-062, „Što se mijenja").

### Task Pilot: Priprema pilota na Railwayu

**Veličina:** 🟨 [PROC] ~20–27 h rada agenata (prijedlog §4.1) + ~1 h Leonova posla · **Ovisi o:** L1-15 za spajanje u `main` i deploy;
gradnja dijelova 1–8 počinje usporedo s L1-14 (Leon 19.10.), jer nijedan dio ne dira jezgru · **Model:** čuvar i graditelj sonnet,
provjeravatelj **opus** za dijelove 2 i 3 (ulaz i novac), ostali sonnet · **Prevodi Rust:** dijelovi 1–4, **nikad dva usporedo**
(jedan Rust slot po sesiji, drugi drži sesija banke).

## Što je odlučeno, što nije

- ✅ **Odlučeno** ([ADR-062](../records/DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md) t. 9): Railway Hobby, Amsterdam; **jedna lozinka
  za cijelu stranicu** (HTTP Basic Auth); strop AI **5 $/dan i 30 $/mj u aplikaciji**; **40 $/mj u Anthropic konzoli** (zaseban radni prostor);
  **najviše 3 igre po gostu**.
- ✅ Deploy je odobren kao pilot na kraju faze B, nakon Leonova OK-a za `model-v2/sloj1` → `main`, i to **samo taj jedan**
  ([ADR-060](../records/DECISIONS/ADR-060-leonove-odluke-18-10-i-pretpostavke-sesije-9.md) t. 12); ADR-062 t. 9: „deploy i dalje tek uz izričit OK".
  Ništa u kodu ne ovisi o Railwayu, sve iz varijabli okoline ([ADR-004](../records/DECISIONS/ADR-004-sve-se-gradi-i-pokazuje-lokalno.md)).
- ✅ **Leon 19.10. (anketa sesije 12):** kolačić gosta **120 dana s obnovom** pri otvaranju igre (klizni rok); besplatna adresa
  **`*.up.railway.app`**; **bez** ograničenja po IP adresi; gradnja dijelova 1–8 **smije odmah, usporedo s L1-14** (potvrđuje ADR-062 t. 18).
- **Za trajanje pilota jezgra se zamrzava:** svaka promjena `CORE_VERSION` daje 409 postojećim igrama (prijedlog §4.1, [BACKLOG](../records/BACKLOG.md)
  „[pilot]"). Nijedan dio niže ne dira `game`, `sim-hotel*`, `content` ni balans.
- **Leon sam** otvara Railway račun (kartica, DPA) i Anthropic radni prostor s ključem i stropom; ključ upisuje izravno u varijable Railwaya.

## Što je u kodu danas (provjereno)

| stvar | gdje | posljedica |
|---|---|---|
| kolačić `HttpOnly; SameSite=Lax; Path=/; Max-Age=604800`, bez `Secure` | `crates/api/src/auth.rs:32-37` | 7 dana, pa student gubi hotel |
| kolačić se šalje samo novom gostu | `crates/api/src/demo.rs:73-76`; test `crates/api/tests/hotels.rs:5-30` (pribija `Max-Age=604800` i „postojeći gost ne dobiva novi kolačić") | nema obnove |
| gost u bazi nema roka | `crates/store/migrations/0003_platform.sql:2-6` | rok drži samo preglednik, baza ne treba promjenu |
| `POST /v1/demo` bez ikakve provjere; postojeći gost dobiva novu igru, gost bez kolačića novi račun | `crates/app/src/demo.rs:62-117`; igra pamti `created_by` (`0003_platform.sql:11`) | brojanje igara po gostu je jednostavno, ali gost bez kolačića ga zaobilazi |
| AI iz `ANTHROPIC_API_KEY`, bez stropa | `crates/server/src/main.rs:52-66` | omotač ide ovdje |
| `AiError::Disabled` se ne ponavlja i ide na rezervu | `crates/app/src/jobs.rs:121-124`, `crates/app/src/ports.rs:153-162` | strop ne ruši igru |
| tokeni se čitaju samo `input_tokens`/`output_tokens` | `crates/ai/src/response.rs:28-34`; predmemorija upita uključena `crates/ai/src/request.rs:107` | upis u predmemoriju se ne broji |
| odrezan odgovor (`max_tokens`) postaje `BadOutput` **bez** tokena, a ponavlja se do 5 puta | `crates/ai/src/response.rs:62`, `ports.rs:103`, presuda `max_tokens: 16000` `request.rs:106` | zbroj iz `events` promaši baš najskuplji slučaj |
| sav promet koji nije `/v1` dobiva 404 Problem | `crates/api/src/lib.rs:62-69` (`fallback(unknown_route)`) | posluživanje weba mijenja ovo mjesto |
| web zove istu adresu | `apps/web/src/main.tsx:16` (`window.location.origin`), Vite proxy `apps/web/vite.config.ts:10` | jedna adresa bez CORS-a |
| gašenje samo na Ctrl-C, radnik se prekida `abort()` | `crates/server/src/main.rs:155-160`; zakup posla 180 s `crates/app/src/jobs.rs:23` | SIGTERM s Railwaya: posao se ne gubi, kasni do 3 min |
| zdravlje broji `COUNT(*)` nad `events` | `crates/store/src/lib.rs:36` | skupo i odaje broj događaja |
| `tower-http` nije naša ovisnost (samo posredno) | `Cargo.toml:30` (samo `tower`), `Cargo.lock:2124` | nova ovisnost; `deny.toml:12` je provjerava |
| `tokio` već ima `signal` | `Cargo.toml:29` | SIGTERM bez nove ovisnosti |
| Dockerfile i `.dockerignore` ne postoje; **Docker nije instaliran lokalno** | korijen repoa; `docker --version` → nema naredbe | slika se može provjeriti samo u CI-ju (push) ili uz Docker Desktop (pitanje 5) |
| CI na svaki push i PR, bez isporuke | `.github/workflows/ci.yml:2-4`, web build `:62` | tok isporuke je nov |

## Dijelovi

| dio | što | dira | Rust | sati | čuvar pribija |
|---|---|---|---|---|---|
| **1 · kolačić** | trajanje 120 dana kao **jedna konstanta** (`10368000` s); **klizni rok**: `GET /v1/me/hotel` poznatom gostu ponovo šalje isti token s punim trajanjem; `Secure` kad je uključen produkcijski način | `crates/api/src/auth.rs`, `hotels.rs` (`me_hotel` 27-34), `demo.rs`, `lib.rs` (`AppState` nosi postavku), `crates/api/tests/hotels.rs`, `tests/common/mod.rs`, opis zaglavlja u `openapi.json` | da (`api`) | 1–1,5 | vidi niže |
| **2 · vrata poslužitelja** | (a) **Basic Auth**: lozinka iz varijable, usporedba u stalnom vremenu, bez lozinke u varijabli nema vrata (lokalno); (b) **Rust poslužuje `apps/web/dist`**: nepoznat `/v1/…` i dalje 404 Problem, ostali putovi `index.html`; (c) **SIGTERM** uz Ctrl-C; (d) zdravlje na `SELECT 1`; (e) **tipizirane postavke** jednim mjestom | `crates/server/src/main.rs` (+ `config.rs`), `crates/api/src/lib.rs`, `health.rs`, `crates/store/src/lib.rs:36`, `Cargo.toml` (`tower-http` s `fs`), `openapi.json` ako se mijenja `HealthBody` | da (`api`, `server`, `store`) | 4–5,5 | lozinka: bez nje 401 s `WWW-Authenticate`, s krivom 401, s dobrom 200; `/v1/health` bez lozinke 200; `/v1/nepostoji` 404 `application/problem+json`; `/hotel/x` vraća `index.html`; bez varijable lozinke ponašanje kao danas |
| **3 · strop AI** | omotač oko `AiPort` u `server`: prije poziva usporedi današnji i mjesečni trošak sa stropom, iznad vrati `AiError::Disabled` (rezerve, ne pad); trošak = tokeni × cijena modela (Sonnet 2/10 $, Haiku 0,10/0,50 $ po milijunu, prijedlog §3.1); zapis upozorenja u log | `crates/server/src/main.rs` (`ai_port`), novi modul u `server` ili `app`, `crates/ai/src/response.rs` (brojanje i odrezanih odgovora i predmemorije), 🟨 nova tablica troška (migracija) | da (`ai`, `app`, `server`, `store`) | 3–5 | ispod stropa poziv prolazi; preko dnevnog ili mjesečnog `Disabled` i posao ne ide u ponavljanje; novi dan otvara dnevni, ali ne mjesečni; odrezan odgovor se broji; strop iz varijable |
| **4 · 3 igre po gostu** | `start_demo` broji igre gosta (`games.created_by`); četvrta → nova greška (uzor `ChatLimit`, 409, `crates/app/src/error.rs:105-106`); poruka u webu | `crates/app/src/demo.rs`, `ports.rs` (`Guests`), `crates/store/src/` (+ `Memory`), `crates/app/src/error.rs`, `crates/api/src/problem.rs`, `openapi.json`, `packages/i18n/src/hr.ts`, početni ekran | da (`app`, `store`, `api`) + web | 2–3 | 3 igre prolaze, 4. odbijena i **ne** stvara igru ni gosta; drugi gost neovisan; broj iz varijable; web pokaže poruku |
| **5 · Dockerfile** | više faza: Rust `server` uz `SQLX_OFFLINE=true`, Node za `apps/web/dist`, mala završna slika s `server`, `content/`, `dist/`; `HOTELIXSIM_ADDR=0.0.0.0:8080`, `HOTELIXSIM_CONTENT` (`main.rs:35-38`, `:144`) | `Dockerfile`, `.dockerignore` (bez `.env`, `target/`, `node_modules/`) | ne lokalno (gradi CI) | 2–3 | slika se izgradi i `server openapi` u njoj radi (CI korak) |
| **6 · isporuka** | CI gradi sliku na PR-u bez objave; tok isporuke samo ručno (`workflow_dispatch`) uz GitHub okolinu „pilot" kojoj je **Leon obavezni odobravatelj**; `server migrate` prije pokretanja | `.github/workflows/ci.yml`, novi `deploy-pilot.yml` | ne | 2 | dry-run toka; okolina bez Leona ne pokreće |
| **7 · kopije** | provjeriti Railway kopije na Hobby planu (prijedlog §2a: nepotvrđeno) + skripta tjednog `pg_dump` na Leonovo računalo + **jedna proba vraćanja** u zasebnu lokalnu bazu (ne u razvojnu) | `scripts/pilot-kopija.sh`, kratko u `README.md` | ne | 2 | proba vraćanja: broj igara i događaja isti |
| **8 · privatnost** | kratka obavijest na početnom ekranu: nadimci, ne prava imena; tekst odgovora ide AI-ju (Anthropic, SAD); kontakt (prijedlog §4.1 t. 12) | `apps/web/src/screens/` početni ekran, `packages/i18n/src/hr.ts` | ne | 1–2 | vitest: tekst vidljiv na mobitelu i računalu |
| **9 · postava i proba** | usluga + Postgres u Amsterdamu, varijable (nova lozinka baze iz Railwaya, nikad lokalna), upozorenje potrošnje Railwaya i tvrdi strop, vanjski ping na `/v1/health`; proba: Safari i Chrome na mobitelu, privatni prozor, krug s razgovorom, presudom i mentorom, **namjerno nizak strop → rezerva** | Railway sučelje, varijable | ne | 2,5–4 | ručna lista u ledgeru |
| **10 · zapis** | ADR: hosting, stropovi, lozinka, kolačić; dopuna ADR-004 | `docs/records/DECISIONS/` | ne | 0,5 | `check:docs` |

**Ukupno ~20–28,5 h**, u granicama prijedloga (§4.1: 20–27 h); razlika je tablica troška u dijelu 3.

### Dio 1 pobliže (prvi, najmanji, samostalan)

- **Trajanje:** jedna konstanta u `auth.rs` (120 dana = `Max-Age=10368000`, Leon 19.10.). Ako Leon kaže drugo, mijenja se samo ona i jedan test.
- **Obnova: da, klizni rok** (Leon 19.10., ~0,5 h više): rok teče od zadnjeg otvaranja igre, ne od prvog. Mjesto je `GET /v1/me/hotel`
  jer web njime počinje (`apps/web/src/api/queries.ts:56`; graditelj potvrđuje da se zove pri svakom otvaranju). `POST /v1/demo` ostaje
  kakav je (postojeći gost ne dobiva novi kolačić — test `hotels.rs:26-29` ostaje). Bez obnove: student koji igra dulje od 120 dana opet
  gubi hotel; s obnovom samo onaj koji 120 dana ne otvori igru.
- **Produkcija:** `Secure` dolazi s varijablom produkcijskog načina (dio 2 e; do tada jedna varijabla samo za kolačić). `SameSite=Lax` ostaje:
  web i API su na istoj adresi pa kolačić nije „tuđi" (prijedlog §1.3). Lokalno (http) bez `Secure`, inače preglednik ne bi spremio kolačić.
- **Čuvar pribija:** (1) novi gost dobiva `Max-Age=10368000`, `HttpOnly`, `SameSite=Lax`, `Path=/`; (2) bez produkcijskog načina **nema** `Secure`,
  s njim **ima**; (3) `GET /v1/me/hotel` s poznatim tokenom vraća `Set-Cookie` s **istim** tokenom i punim trajanjem; (4) nepoznat ili pokvaren
  token ne dobiva kolačić; (5) postojeći gost na `POST /v1/demo` i dalje bez novog kolačića. Testovi idu u `crates/api/tests/hotels.rs`.

### Otvorene tehničke odluke (voditelj, 🟨 u ADR na `/kraj`)

1. **Basic Auth:** provjerava se samo lozinka, korisničko ime je bilo koje (student upiše bilo što); `/v1/health` je bez lozinke (ping i
   provjera Railwaya), a nakon dijela 2 d ne odaje ništa. Nalaz za probu: mobilni Safari može ponovo pitati lozinku nakon zatvaranja preglednika.
2. **Produkcijski način** (`HOTELIXSIM_PRODUKCIJA=1` ili slično): poslužitelj **odbija start** ako nedostaje lozinka, stropovi AI-ja ili mapa weba,
   i tada uključuje `Secure`. Time zaboravljena varijabla ne otvara stranicu svima. Cijena ~0,5 h; zatvara BACKLOG „Typed config".
3. **Gdje se broji trošak:** (A) zbroj `meta` iz `events` po `recorded_at` — bez migracije, ali promaši odrezane odgovore (do 5 × ~0,16 $ po
   presudi) i upis u predmemoriju; (B) **preporuka:** brojač vidi svaki odgovor Anthropica (i odrezan) i upisuje u vlastitu tablicu; točan,
   preživi restart, ~1–2 h više. Migracija bi bila `0005`, a plan platforme tim brojem zove F1-2 ([plan platforme](./2026-11-platforma.md)
   faza 2, red 1) — platforma tada prima sljedeći broj.
4. **Dan za dnevni strop:** UTC (jednostavno; u Hrvatskoj se dan otvara u 1 ili 2 h noću).
5. **Što student vidi kad je strop dosegnut:** ništa posebno, igra radi na rezervama (neutralna presuda, kratak odgovor lika, mentor iz brojki).
   Poruka „AI je danas potrošen" nije u opsegu.
6. **Gost bez kolačića zaobilazi 3 igre** (svaki put novi gost, `app/src/demo.rs:67-73`). Pravi čuvar je lozinka (dio 2) i strop (dio 3);
   ograničenje po IP-u Leon 19.10. **ne želi** za pilot (cijeli razred na fakultetskom Wi-Fiju dijeli IP).
7. **`tls-rustls` za `sqlx` ne treba** ako baza ide preko unutarnje mreže Railwaya (prijedlog §1.1: treba samo za bazu preko interneta) —
   🟨 provjeriti pri postavi (dio 9).

## Valovi

```text
val 1 (odmah po Leonovu OK-u za plan):  dio 1 kolačić (Rust slot)  ∥  dio 7 kopije (skripta)  ∥  dio 8 privatnost (web, tekst čeka Leona)
val 2 (kad je dio 1 spojen):            dio 2 vrata poslužitelja (Rust slot)  ∥  dio 5 Dockerfile (bez Rusta lokalno)
val 3 (kad je dio 2 spojen):            dio 3 strop AI (Rust slot)  ∥  dio 6 isporuka (yml)
val 4 (kad je dio 3 spojen):            dio 4 tri igre po gostu (Rust slot)
val 5 (nakon L1-15 i Leonova OK-a):     Leonovi računi → dio 9 postava i proba → deploy → dio 10 zapis
spajanje redom 1 → 2 → 3 → 4 (Rust), web i skripte kad su gotove; svaki uz cijeli CI i cmp balansa (mora biti bajt-isti)
```

- **Zašto redom:** dijelovi 1, 2 i 4 diraju `crates/api` (dio 1 i 2 oba `lib.rs`/`AppState`), a dijelovi 2 i 3 `crates/server/src/main.rs`; jedan
  Rust slot ionako ne dopušta usporedno. Dio 2 prije 3 jer strop čita postavke iz tipiziranih postavki. Kolačić prvi jer je najmanji i jedini
  koji štiti studentov hotel bez obzira na hosting.
- **Svaki dio:** svoje stablo `../hotelixsim-pilot-N` (apsolutni put), svoj `CARGO_TARGET_DIR=…/hotelixsim/target/pilot-N` (briše se odmah
  nakon spajanja), tok čuvar → graditelj → provjeravatelj ([ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)).
  Grana iz `model-v2/sloj1`. Gradnja samo na punjaču; ispod 50 GB slobodno ne gradi.
- **Dijelovi 5 i 6** se mogu potpuno provjeriti tek u GitHub Actions (nema Dockera lokalno), a to traži push grane — **push je objava**, uz
  Leonov OK (pitanje 5).

## Varijable okoline na Railwayu (dio 9)

`DATABASE_URL` (nova lozinka iz Railwaya) · `HOTELIXSIM_ADDR=0.0.0.0:8080` · `HOTELIXSIM_CONTENT` · `ANTHROPIC_API_KEY` (ključ pilot radnog
prostora, upisuje Leon) · lozinka stranice · stropovi (5 $/dan, 30 $/mj) · broj igara po gostu (3) · mapa weba · produkcijski način.
Imena varijabli određuje dio 2 e; nijedna vrijednost ne ide u git ni u `.env.example` (osim imena bez vrijednosti).

## Čeka Leona prije gradnje

Odlučeno 19.10.: kolačić 120 dana s obnovom, `*.up.railway.app`, bez granice po IP-u, gradnja odmah (bivša pitanja 1–3 i 8).

4. **Spustiti `max_tokens` presude 16 000 → 4 000** (najgori poziv 0,16 $ → ~0,04 $)? Dira presudu, pa samo uz mjerilo; preporuka: ne za pilot.
5. **Provjera Docker slike:** push grane pilota na GitHub da je CI izgradi (push je objava; repo je javan, tajni nema) ili Docker Desktop lokalno (disk).
6. **Railway:** upozorenje potrošnje 7 $ i tvrdi strop 15 $ (prijedlog §2a); tjedni `pg_dump` na tvoje računalo — tko ga pokreće.
7. **Tekst obavijesti o privatnosti** (dio 8) — voditelj daje nacrt, Leon odobrava.
9. **Leonov posao (~1 h, prijedlog §4.2):** Railway račun, Hobby, kartica, DPA; Anthropic radni prostor „HotelixSim pilot", novi ključ, strop 40 $/mj;
   GitHub okolina „pilot" s tobom kao odobravateljem i Railway token u tajnama; odabrati lozinku i reći je profesoru; OK za `sloj1` → `main`, pa za deploy.

## Trošak (prijedlog §2a, §3.3, §4.3; 1 $ ≈ 0,87 €, bez PDV-a)

| stavka | €/mj |
|---|---|
| Railway Hobby (poslužitelj + baza) | ~4,5–6 |
| AI (40 studenata, tjedna sezona) | ~15–30 u mjesecu igranja, inače ~0; tvrdo ograničeno na 30 $ (~26 €) u aplikaciji i 40 $ u konzoli |
| domena, nadzor, CI | 0 |
| **ukupno** | **~20–36 € u mjesecu igranja, ~5 € u mirovanju** |

**Najjeftinija varijanta koja čuva temelj** je ova (Railway Hobby + besplatna adresa): ista cijena kao OVH VPS, ali bez stroja koji netko
mora čuvati; selidba kasnije je `pg_dump`/`pg_restore` i nova `DATABASE_URL` (prijedlog §2, preporuka).

## Što se NE radi u pilotu

Prijava i računi (ID-1…3, F1-2…F1-4), razred i ulaz kodom (F1-5, F1-6, polje „kod razreda"), AAI@EduHr, više lozinki ili lozinka po razredu,
Vercel ili web i API na dvije adrese, Cloudflare kontejneri, više radnika reda (C2/F0-8; jedan radnik je za 40 studenata podnošljiv, prijedlog §1.4),
zaseban način `work`, uloga baze bez UPDATE/DELETE, i **bilo kakva promjena jezgre ili `CORE_VERSION`** dok pilot traje.
