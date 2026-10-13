# ROADMAP — kolosijek IGRA

**Što je ovo:** ZALIHA CIGLI po fazama za jedan kolosijek (igra: srce igre, `crates/game`, `sim-hotel-v2`, `content`, sadržaj i
ekrani igre). Faze iza ZAVRŠETKA SLOJA 1 su **smjer** koji se ponovno slaže nakon što profesor vidi igru; kartica cigle
se razrađuje u planu faze kad faza krene. Sati su [PROC] (rad agenata + pregled). Podjela kolosijeka je po **datotekama**,
ne po temi ([ADR-038](../../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)): sve što dira jezgru igre je ovdje, i razredne
cigle. Susjedi: [platforma.md](./platforma.md), [kvaliteta.md](./kvaliteta.md); pregled faza: [ROADMAP.md](../ROADMAP.md).
Razred i obračun: [ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md). Spec: [MODEL_V2.md](../../architecture/MODEL_V2.md).

**Prekretnice (stupac P):** P1 kraj listopada (cijela sezona Sloja 1 igriva) · P2 kraj studenog (profesor na internetu stvori igru,
studenti uđu kodom) · P3 sredina siječnja (verzija za profesore: ljudi, razgovori, profesor vodi razred i dodaje situacije) ·
P4 ožujak (studentski pilot) · poslije = iza pilota.

**Pravilo za SVE cigle jezgre:** `equivalence.rs` zelen, `v1_frozen` + `fixtures_sealed` zeleni, v1 balans bajt-isti, v2
`--seeds 1000 --check` (tjedan i mjesec) prije/poslije s opisanim pomakom; `CORE_VERSION` se podiže jednom po valu. Rust: najviše 2 izvođača istodobno.

---
## FAZA IGRA — ZAVRŠETAK SLOJA 1 (P1; 17 cigli, ~49,5 h)

**Gotovo kad:** Leon odigra cijelu sezonu u pregledniku (situacije, obnova, kraj sezone) i `main` ima sve tri grane spojene.

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| L1-1 | S-4 dovršetak | probe-mutacija, pregled S-4, kontrolor u `../hotelixsim-s1-sit` | mutacija pada; 686/0; v1 100 + v2 1000 `--check` | — | 3 | P1 | m7: 4 zatvorena odjela → 1 od 6300 krugova 20/21 situacija; prihod ostalih odjela nakon provizije |
| L1-2 | G-1 SituationsDrawn | događaj pri otvaranju kruga; `situations_in` čita iz dnevnika (`game.rs:483`, `talk.rs:42`, `verdict.rs:235`, `resolve.rs:36`, `ai_input.rs:209`) | fold dvaput isto; stara v2 igra bez događaja = staro; robot isti odgovor | L1-1 | 3 | P1 | — |
| L1-3 | G-2 AI ulaz | `CharacterInput`/`AssessInput` iz popunjenog primjerka; `DebriefSituation.odjel` | nijedan AI ulaz s neispunjenim `{…}`; živi test 1 predloška | L1-2 | 2 | P1 | OK za ključ |
| L1-4 | A-1 API situacije | `SituationsBody` (+department, kind, weight, uloge), `openapi.json` | `check:api`; ruta s 3–5 situacija | L1-3 | 1 | P1 | — |
| L1-5 | W-1 SituationsScreen | 3–5 kartica po važnosti, „riješeno 2 od 4", upozorenje pri „Zatvori tjedan", „prepusti voditelju" | vitest | L1-4 | 3 | P1 | — |
| L1-6 | UI čuvar situacije | mjerenja 360 px/laptop (`UI_PRAVILA.md`) | UI čuvar DA | L1-5 | 1,5 | P1 | — |
| L1-7 | O-5 dovršetak | mentor „Zašto" u `../hotelixsim-s1-obnova`; **sukob s KS-3**: jedan zajednički prompt (`mentor-sezona.v3`) za obnove i amortizaciju | pregled O-5; workspace test, clippy, core-deps, balans 1000, `check:api` | — | 3 | P1 | tekst „Obnova je odlučena…"; lokal B pada ispod početnog: prag ili 100 % pokrivanje |
| L1-8 | UI čuvar O-4+O-5 | mjerenja iz `obnova-o4-review.txt` | UI čuvar DA | L1-7 | 2 | P1 | — |
| L1-9 | KS-3 živi test | sezonski mentor s ključem (stablo spremno, 600/0) | 1 živi poziv, ocjena teksta | — | 1 | P1 | OK za ključ |
| L1-10 | Spoji OBNOVA → sloj1 | prvo obnova; sukobi u `schema.rs`, `model.rs`, `PlanScreen.tsx`; **jedno** podizanje `CORE_VERSION` | workspace zelen; v2 1000 `--check`; F pomak izmjeren | L1-7, L1-8 | 4 | P1 | potvrda da se probne v2 igre gase |
| L1-11 | Spoji SITUACIJE → sloj1 | 14 zajedničkih datoteka jezgre; S-4 sadržaj | isto + tj≈mj (296/300, tijesno) | L1-10, L1-6 | 4 | P1 | — |
| L1-12 | Spoji KS-3 → sloj1 | kraj sezone; mentor-sezona iz L1-7 | isto + KS-3 UI čuvar | L1-11, L1-9 | 2 | P1 | „demo gotov" |
| L1-13 | Zajednički balans | Brižni ≥ 80 % sezonskog plana, pojasevi marži, F (±2, C ±1), situacije ≈ ⅓ napretka, pravednost profila | tri balansa 1000 `--check`; tablica prije/poslije | L1-12 | 5 | P1 | brojke koje igrač osjeti |
| L1-14 | Igriva verzija za Leona | adresa + put do novoga; Leon prolazi igru | Leonov popis izmjena | L1-13 | 2 | P1 | što mijenjati |
| L1-15 | sloj1 → main | spajanje i push | CI zelen (`gh run list`) | L1-14 | 1 | P1 | **izričit OK za push** (repo javan) |
| P9-1 | Račun dobiti za profesora (P17) | USALI hotel uz hotel, GOPPAR/EBITDA; ekran i ruta su u PLATFORMI (F2-3) | zbroj = izvješće do centa | L1-15 | 6 | P3 | — |
| P9-2 | „Zašto" po odjelu (P18) + budžet/ostvareno | tri uzroka po odjelu iz jezgre; referentni hotel kao „budžet" | uzroci zbrojeni = razlika | L1-15 | 6 | P3 | pojas GOP marže 35–45 % (N9) |

Kritična staza: L1-1→2→3→4→5→6→11; O-5 → L1-10 paralelno. P1-cigle `Sloja 1` ne uključuju P9 (12 h, ide u P3).

---
## FAZA IGRA — RAZRED U JEZGRI (P2; 5 cigli, 45 h)

**Gotovo kad:** u jednoj igri igra N studenata (1, 4, 30) i **svaki hotel se obračuna kad ga njegov student zatvori**; profesor otvara
krugove (ručno ili po rasporedu), nitko ne ide iza otvorenog kruga; svatko može vidjeti tko je gotov, tko još igra i hotele kolega.
Odluka D1 (obračun po hotelu) je donesena 13.10. ([ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md)).
Preostali dio starog F1 (server, store, `app/platform`, `api/platform`, web razreda) je u [platforma.md](./platforma.md), faza RAZRED.
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
## FAZA IGRA — LJUDI (sloj 2a; P3; 12 cigli, ~64 h)

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
## FAZA IGRA — RAZGOVORI (sloj 2c; P3; 8 cigli + 1 opcija, ~35 h)

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
## FAZA IGRA — NATJECANJE ZA LJUDE (P3, rez → P4; 8 cigli, 45 h)

**Gotovo kad:** student A pošalje ponudu plaće radniku hotela B; B dobije obavijest, može dati protuponudu (povišica ili razgovor), a radnik
odluči po vlastitoj formuli; nakon svih zatvaranja ishod je isti bez obzira tko je prvi zatvorio. Pravilo: ponuda u tjednu k; vlasnik B do kraja
**svog** tjedna k protuponuda; radnik odlučuje kad B zatvori tjedan k; prelazi od k+1. Ograničenje 1–2 ponude tjedno + trošak zapošljavanja;
profesor može isključiti preotimanje za razred. Detalji: [ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md).

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
## FAZA IGRA — JEZICI I AGENCIJA (sloj 2b; P4; 8 cigli, 29 h)

**Gotovo kad:** jezici pokrivaju goste po mjesecu i pomiču kvalitetu; student naruči radnike iz agencije (rok, naknada, smještaj) i vidi cijenu po glavi.

| id | cigla | što se radi | dokaz | ovisi o | h | P | Leon |
|---|---|---|---|---|---|---|---|
| L2b-1 | Sadržaj jezika i agencija | jezični sastav po mjesecu (DZS + Kvarner, N8), po zemlji rok/naknada/dozvola, tečajevi | provjera sadržaja; izvor zabilježen | L2a-1 | 4 | P4 | pregled brojki |
| L2b-2 | Jezici → kvaliteta | pokrivenost odjela pomiče kvalitetu i teme; obuka jezika +1 | pokrivenost 100 % = staro | L2b-1, L2a-5 | 4 | P4 | veličina učinka |
| L2b-3 | Agencija: nalog | „5 radnika za domaćinstvo", dolaze nakon roka, naknada + dozvola | tablično; radnik iz istih raspodjela | L2b-1, L2a-6 | 4 | P4 | plaća ista za istu vještinu (N1) |
| L2b-4 | Smještaj radnika | krevet u gradu; sobe za radnike (≤ 10 %) — puni oblik čeka status sobe (S3-3, poslije pilota) | zbroj troškova = zbroj po osobama do centa | L2b-3 | 4 | P4 | N10; cijena kreveta |
| L2b-5 | Test zamjene podrijetla | ista osoba, drugo podrijetlo → razlika samo u jezicima, roku, naknadi, dozvoli, smještaju | test | L2b-4 | 2 | P4 | — |
| L2b-6 | Balans | agencija nije dominantna | tri balansa `--check` | L2b-5 | 3 | P4 | — |
| L2b-7 | Igra + API | | `check:api`; fold | L2b-6 | 3 | P4 | — |
| L2b-8 | Web: agencija, smještaj, pokrivenost | ekran agencije (narudžbe, trošak i doprinos po osobi), pokrivenost po odjelu | vitest; UI čuvar | L2b-7 | 5 | P4 | — |

---
## FAZA IGRA — VRSTE SOBA I CIJENE (P4; 6 cigli, 21,5 h)

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
## FAZA IGRA — RECENZIJE (sloj 4; P4; 7 cigli, 34 h)

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
## FAZA IGRA — poslije pilota (smjer; ~190 h)

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
| ENGLESKI (sadržaj) | prijevod situacija i banki (imena, rečenice, osobine), prompti lik/mentor/presuda (`lik.v2`, `presuda.v3`, mentor) i provjera da AI odgovara na jeziku igre; sučelje i format su u [platforma.md](./platforma.md) | ~30 [PROC] | smjer: igra mora kasnije biti i na engleskom; mjerilo ocjena po jeziku |

Generator hotela (ideja 10) je u [platforma.md](./platforma.md), faza poslije.

---
## Linije reza (ako zapne, redom)

1. Vlastite situacije profesora P3 → P4 (platforma.md, faza PROFESOR). 2. Razgovori samo 3 vrste. 3. NATJECANJE ZA LJUDE P3 → P4.
**Nikad se ne režu:** testovi, provjeravatelj (`equivalence.rs`, `v1_frozen`, probe-mutacija) i balans.

## Sažetak ([PROC])

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

## Rizici balansa (sažeto)

Najveći: **L2a-3** (agregati iz pojedinaca moraju na početku dati točno v1 brojke), spajanje tri stabla (L1-10…13: 14 zajedničkih datoteka, tj≈mj 296/300),
dominantne radnje (2a i natjecanje), vrste soba (ukupna potražnja pri preporučenim cijenama ±3 %), recenzije → potražnja (povratna petlja), AI trošak
(17 krugova, granica 2 $ dijele razgovori, pošta i recenzije). Prave greške prije siječnja i hitni CI: [kvaliteta.md](./kvaliteta.md).
