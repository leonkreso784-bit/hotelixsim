# Backlog

Ovo je JEDINO mjesto za ono što nije riješeno: odgođeni nalazi pregleda, otvorena pitanja i tehnički dug. Plan ne nosi tuđe stavke — plan uzima stavke odavde po oznaci; kad se stavka riješi, briše se (povijest je u gitu i `PROGRESS.md`).

Oznake: `[jezgra]` plan jezgre i balansa · `[dnevnik]` plan dnevnik/runde/red poslova (uključuje API dio, ARHITEKTURA §20.2) · `[korak 4]` uzima plan koraka 4 (AI, sadržaj) · `[ekrani]` ekrani igrača · `[prije Railwaya]` prije prvog deploya · `[drugi val]` opseg poslije demoa · `[možda nikad]` · `[ci]` CI i alati · `[leon]` čeka Leonov odgovor.
Izvor „L37" = redak u [ledgeru kostura](../archive/2026-10-04-kostur-ledger.md); „Task N #k" = nalaz pregleda tog taska.

## Odgođeni nalazi — [jezgra]

Nalazi pregleda taskova jezgre koji nisu popravljeni na grani `gradnja/jezgra`; svaki je mali.

**Brane i kocka (`engine`)**
- [jezgra] **`div_round` javlja `Overflow` za |a| > i128::MAX/2** iako rezultat stane (sigurno, izvan raspona igre); dopisati doc o rasponu i testove (veliki a koji taman stane, `add(magnitude, b)`). (Task 2)
- [jezgra] **Probe mutacija:** nema baznog prolaza prije mutacija (CI ga pokriva clippyjem); neprobane stavke `DefaultHasher`, `Instant::now`, `env::var_os`, `default_numeric_fallback`; float-sufiks grep ne gleda `tests/`; nema obrade Ctrl-C (lokalno može ostaviti mutiran `lib.rs`). (Task 1)

**Domena (`sim-hotel`, `content`)**
- [jezgra] **Nema testa da `Index::position()` odgovara `Index::ALL`** — postoji samo test kodova. (Task 3 #1, L47)
- [jezgra] **Provjera kataloga:** poruke granica imenuju samo jedno polje (`price_min/max`, `wage_min/max`); `validate` ne pokriva `marketing_max_cents`, `seasonal_step/max`, `marketing_max_bp`, `interest_bp`, `seasonal_plan`, `load` — dodati kad postanu nazivnici; nema testa za `OutOfRange` ni za referentni profil krive kategorije. (Task 3)
- [jezgra] **Test fluktuacije nema slučaj polovice** (npr. moral 5 990 → 302). (Task 3)
- [jezgra] **Nedostajuća kategorija javlja se kao `CalcError::BadDivisor`** (`market.rs`, `mechanic.rs`) — nema bolje varijante. (Task 4, Task 6)
- [jezgra] **`plan::default_for` ograničava na `seasonal_max` nakon ±koraka;** `market` ne provjerava plan (negativan marketing → `BadDivisor`), pozivatelj mora zvati `validate` — dokumentirati. (Task 4)
- [jezgra] **Kazna opterećenja duplicirana** (`indices.rs`, `resolve.rs`) — izdvojiti `load_penalty()`; `indices.rs` `try_into` daje pogrešnu varijantu greške ako `SEASON` promijeni duljinu (nedostižno). (Task 7)
- [jezgra] **Retci `Staff/Turnover` nose +departures** (zamijenjeni, broj ljudi se ne mijenja) pa zbroj redaka ≠ promjena broja — razmotriti zaseban `Cause` ili znak za ekran profesora. (Task 5)
- [jezgra] **Mehanike:** granica ocjene bez testa (nijedna mehanika ne mijenja ocjenu), oprema bez granice; nema testa granice morala, `BadGroup` ni greške u drugoj od više situacija; TDD crveni korak preskočen u Tasku 6. (Task 6)
- [jezgra] **Učitavanje sadržaja:** duplicirani ključevi u mapama = zadnji pobjeđuje. (Task 8; privremene mape sada brišu `Drop` čuvari, a prazni odgovori i duplicirane šifre se odbijaju)

**Alat za balans (`balance`)**
- [jezgra] **`margin_bp` tiho vraća 0 kad je prihod ≤ 0;** testovi pokrivaju samo granu §11.1 u `failures`, nema testa izlaznog koda `--check`; tolerancija §11.1 tvrdo u kodu, `_cat` neiskorišten. (Task 10)
- [jezgra] **Linije objašnjenja nisu zbrojive za ekran profesora:** moral miješa cilj i stvarno, nema `Cause::Bound` retka za granice (clamp), komponente cilja zadovoljstva nedostaju. Riješiti prije plana ekrana. (Završni pregled)
- [jezgra] **`RefTable` ne bilježi svoju kategoriju** — pozivatelj mora paziti da je tablica prave kategorije. (Završni pregled)
- [jezgra] **`wage_cents % 100` nije provjereno** — `div_floor` tiho gubi cente. (Završni pregled)
- [jezgra] **`price_min_pct > 0` nije provjereno** u katalogu. (Završni pregled)
- [jezgra] **Provjera pravednosti je zapravo jedan robot** dok ne postoje okidači iz §9: 3 od 4 strategije su determinističke. (Završni pregled)

## Odgođeni nalazi — [dnevnik]

- [dnevnik] → korak 4: zaglavlje `Idempotency-Key` (ARHITEKTURA §11); ponovni pokušaj s odgodom; probni test apalis/pgmq (ARHITEKTURA §9.4); zakašnjela AI presuda za zatvorenu rundu (§8.3).
- [dnevnik] **Siroče gost:** `start_demo` upisuje gosta pa igru u dvije zasebne radnje — ako `create_game` padne, ostaje gost bez igre; počistiti uz brisanje gostiju nakon 7 dana. (`app/src/demo.rs:41`)
- [dnevnik] **`ErrorCode::ALL` i `as_str()` se ne koriste** (žičani kod daje serde `ProblemCode`) — dva izvora istine; test `ProblemCode::from(c)` = `c.as_str()` za sve ili brisanje. (`app/src/error.rs:42,84`; M1)
- [dnevnik] **`parse_hotel`/`parse_game`/`parse_month` idu prije `caller.require()`** — anonimac s pokvarenim uuid-om dobiva 404 umjesto 401; bezopasno. (`api/src/hotels.rs:54,107`, `games.rs:222,247,274`; M2)
- [dnevnik] **`check:api` pada na staged promjenama** (`git status --porcelain` vidi i staged) — gledati samo `??` i nestageano ili zapisati u README. (`scripts/check-clean.mjs:10`, bez try/catch; M5, T11)
- [dnevnik] **`?` nakon `axum::serve` preskače `worker.abort()`.** (`server/src/main.rs`; T11)
- [dnevnik] **Neodgovorene u odgovoru na „Zatvori mjesec"** (`unanswered: 0` jer runda nije `open`) — ekran mora upozoriti PRIJE klika. (`app/src/views.rs:187`; M4) Uz to `unanswered` je izveden na dva mjesta (`RoundView`, `schema::SituationsBody`) → jedno polje u `SituationsView`. (T10)
- [dnevnik] **Ručni „ponovi obračun" za `failed` posao** (Leon/profesor); `COALESCE(last_error, …)` kod isteka na zadnjem pokušaju čuva stariju poruku. (`store/src/jobs.rs`; T6, T8)
- [dnevnik] **`game` pomoćnici:** dupli blok čitanja `RoundResolved` u `state_at`/`latest_state` i 5× lookup `HotelResolution` → helper (T3, T4); `apply` ne provjerava poznatu situaciju/indeks odgovora/jedinstvenost slota u `created` i kod duplog odgovora prepiše prije greške (fold ionako pada); `robot_moves` ne provjerava da je mjesec `Open`. (T3)
- [dnevnik] **`set_plan`/`answer` zovu `game_of_hotel` prije prava** (404 vs 403 otkriva postojanje uuid-a v4 — zanemarivo); `corrupt()` log bez id-a hotela; mapiranje strategija string-matchom paralelno s `game`. (T6, T9)
- [dnevnik] **Testovi koji fale:** (a) `INDEX_ORDER` prati `Index::ALL` i JSON oblik `Controller` + `GameId::stream()` = `game:<uuid>`; (b) A/B/C samo za seed 42, determinizam `new_game`, `seasonal_min`, `renovation_available=false`; (c) feed „samo moje" bez simetričnog slučaja, srpanj samo brojem događaja, `private_hotel.now` nakon kraja igre, S3 linija neodgovorenog; (d) `Memory` paritet (duplikat `create_game`/`create_guest` tiho prepisuje, Postgres pada), semantika memory porta, zatvorena/završena igra, `token_hash`, potpunost `ErrorCode::ALL`; (e) grana dvostrukog pada (`fail` sam padne) i otpuštanje advisory locka nakon odbijanja, duple igre samo `is_err`; (f) konkurentnost `claim` s 2+ posla i N preuzimača; (g) kolačići (31/33 znaka, ne-hex, nepoznat token), `me/hotel` bez igre, popis ključeva tuđeg hotela, loša tijela 422 po polju, feed `by`/mjesec/redoslijed, `FeedItemBody` bez `discriminator` u OpenAPI-ju; (h) e2e: replay samo lipanj, pretpostavka jedne neodgovorene po mjesecu, robot provjere samo postojanje događaja; dva preklapajuća roundtrip testa u `content/tests/snapshot.rs`. (T1–T12)
- [dnevnik] **Test T4 `resolution_uses_snapshot_not_files` ručno gradi `created`** — dodati `created_from(&Snapshot, seed)` u `tests/common`. (T4)


## Odgođeni nalazi — [korak 4]

- [korak 4] **PRIJE prve promjene content DTO-a: otisak pravila nad kanonskim `serde_json::Value`** spremljenog snimka (ne nad tipiziranim ponovnim zapisom) i nova DTO polja samo uz `#[serde(default)]`; inače svaka postojeća igra postaje `Corrupt`/500. Vidi [ADR-015](DECISIONS.md#adr-015) t. 3. (`game/src/game.rs:70`, `content/src/lib.rs` `Snapshot`; završni pregled I2)
- [korak 4] **`core_version` se zapisuje, ali se nikad ne provjerava** — promjena jezgre usred igre obračuna srpanj novim kodom (replay lipnja više nije bajt-isti). (`game/src/game.rs:69`; M3)
- [korak 4] **Svako čitanje radi puni fold** (`Snapshot::build` klonira cijeli snimak, SHA-256, `RefTable::compute`) — zanemarivo za 4 hotela × 2 mjeseca; riješiti uz projekcije. (M6, T1)

## Odgođeni nalazi — [ekrani]

- [ekrani] **`openapi-fetch` i `application/problem+json`** — provjeriti parsiranje 404 tijela u prvom stvarnom pozivu. (Task 6 #3, L71)
- [ekrani] **i18n bez lokala i zajedničkog tipa za en rječnik** — `format.ts` ima tvrdo „hr-HR". (Task 7 #1, L77)
- [ekrani] **`retry` 3 u QueryClientu daje ~7 s „Učitavam…"** kad API ne radi. (Task 7 #2, L78)
- [ekrani] **Testovi ne pokrivaju pending, 500 na health, skriveni red događaja kad je events null** — lažni fetch pretpostavlja `Request`. (Task 7 #3, L79)
- [ekrani] **Health redak bez `role="status"`/`aria-live`.** (Task 7 #4, L80)
- [ekrani] **`t()` parametri nisu tipizirani po poruci** — krivo ime daje vidljiv `{placeholder}`. (Task 7 #5, L82)
- [ekrani] **`vite build` u CI-ju.** (Final minori, L96)
- [ekrani] Posao obračuna u statusu `failed` ostavlja rundu u „closing" zauvijek — ekran i profesor trebaju to vidjeti; SSE „runda je obračunata" umjesto ispitivanja (ARHITEKTURA §11).

## Odgođeni nalazi — [prije Railwaya]

- [prije Railwaya] **Red poslova:** mogući deadlock u expiry `UPDATE` bez reda/`SKIP LOCKED` tek uz više radnika; seq overflow kao `sqlx::Error::Protocol`; dvostruke dev-ovisnosti u `store`. (T7, T8)
- [prije Railwaya] **Nema testa za `connect_lazy` na nedostupnu bazu (~3 s) ni `migrate` na svježoj bazi.** (Task 4 #3, L55)
- [prije Railwaya] **sqlx bez TLS značajke** — dodati `tls-rustls` (nikad native-tls/openssl). (Task 4 #4, L56)
- [prije Railwaya] **`COUNT(*)` u zdravlju** je skup na velikoj tablici. (Final minori, L96)
- [prije Railwaya] **Greška `ctrl_c` je ignorirana** — server bi se odmah ugasio. (Task 5 #3, L63)
- [prije Railwaya] **Base URL klijenta/API-ja.** (Final minori, L96)
- [prije Railwaya] **SIGTERM (graceful shutdown).** (Final minori, L96)
- [prije Railwaya] **Typed config** umjesto sirovih env varijabli. (Final minori, L96)
- [prije Railwaya] **CI bez filtra, akcije po tagu a ne po SHA** — CI radi na push i pull_request; `permissions: contents: read` već postoji. (Task 8 #2, L87)
- [prije Railwaya] Uloga baze bez prava UPDATE/DELETE nad `events` (okidač ostaje); kolačić `Secure` uz HTTPS; `content::repo_dir()` je put iz vremena prevođenja — na Railwayu `HOTELIXSIM_CONTENT`; zaseban način `work`.
- [prije Railwaya] **`deny.toml` zabranjuje samo `openssl-sys`** — ne native-tls/openssl. (Task 8 #4, L89)
- [prije Railwaya] **E2E „baza ne radi" dokazan krivom lozinkom, ne ugašenom bazom** — 3 s `acquire_timeout` nije izmjeren. (Task 8 #5, L90)

## Odgođeni nalazi — [možda nikad]

- [možda nikad] **`Problem.status` je slobodan u16; `to_vec(..).unwrap_or_default`.** (Task 5 #5, L65)
- [možda nikad] **`pnpm.mjs` pretpostavlja da je `npm_execpath` JS datoteka.** (Task 6 #1, L69)
- [možda nikad] **`gen-openapi.mjs` piše relativno na cwd.** (Task 6 #4, L72)
- [možda nikad] **`Cargo.lock` ima neaktivne sqlx-sqlite/libsqlite3-sys unose** — `cargo deny` s all-features ih može vidjeti (lokalno prolazi). (Task 4 note, L58)
- [možda nikad] **README: uputa za `sqlx-cli`.** (Final minori, L96)

## Otvoreno za Leona

Pitanja o domeni ne blokiraju kod: brojke su podaci u katalogu, gradi se s pretpostavkom i mijenja kad odgovori.

- [leon] **Ostali prihod (hrana, piće, usluge) = 30 % prihoda soba** — je li realno? Odgovor: [DEMO](../product/DEMO.md) „Za Leonovu provjeru", točka 2. (SLJEDECA_SESIJA §5.2)
- [leon] **Trošak zaposlenog 2 200 €/mj i omjer 30 stalnih + 20 sezonaca** (120 soba, 4*) — realno? Odgovor: [DEMO](../product/DEMO.md) „Za Leonovu provjeru", točka 3.
- [leon] **S3: 25 % popusta agenciji za grupu u kolovozu** — realna ponuda? Odgovor: [DEMO](../product/DEMO.md) „Za Leonovu provjeru", točka 4.
- [leon] **Kriteriji rubrika u S1–S4** — Leon je mjerilo; blokira sadržaj i AI presude, ne kod jezgre. Odgovor: [DEMO](../product/DEMO.md) „Za Leonovu provjeru", točka 5.
- [leon] **Priče profila A i B (§5)** — prepoznaje li takve hotele? Odgovor: [DEMO](../product/DEMO.md) „Za Leonovu provjeru", točka 6.
- [leon] **AI ključ za demo** — koje besplatne ključeve ima (pružatelj i model); ne blokira jezgru. Odgovor: [ROADMAP](../plan/ROADMAP.md).
- [leon] **Pitanja za profesore** — Leon ih šalje tijekom tjedna 5.–11.10.; pregled prije slanja. Odgovor: [PITANJA_ZA_PROFESORE](../product/PITANJA_ZA_PROFESORE.md).
- [leon] **Inkubator** — prijava do **16.11.**: nosi li se Sokrat ili simulacija. Odgovor: [ROADMAP](../plan/ROADMAP.md).
- [leon] **Potvrda ARHITEKTURE v2** — ROADMAP kaže „Leon ih je pregledao", a ARHITEKTURA kaže „prijedlog dok Leon ne potvrdi"; potvrditi ili ostaviti raspravu otvorenom, pa uskladiti oba. Odgovor: [ARHITEKTURA](../architecture/ARHITEKTURA.md).
- [leon] **Sezonska marža 35,8 % blizu donje granice (35 %)**, lipanj 22,5 %, rujan 20,3 % (rujan nosi 34 sezonca jer plan smije pustiti najviše 10 mjesečno). Odgovor: [ADR-013](DECISIONS.md#adr-013) t. 5 i [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md) „Za Leonovu provjeru", točka 2.
- [leon] **B ima fiksne troškove samo 3 % niže od C (145 000 prema 150 000 €)** — §5 kaže „niži fiksni troškovi", ali uz likvidnost bez granice (ADR-014) svaki euro razlike ulazi u blagajnu i F profila B raste sam od sebe: s 112 500 € B je najbolji pod svim strategijama (razlika medijana 8,69), sa 140 000 € tijesno prolazi (4,24). Jasno niži fiksni troškovi traže promjenu formule F ili ocjene napretka (nova odluka). Odgovor: [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md) „Za Leonovu provjeru", točka 3.
- [leon] **A: ugled od opreme i iskustva, a premalo ljudi** — A sad ima opremu 75, iskustvo 70, 28 stalnih i početno opterećenje 2,20 (u lipnju ≈ 1,6, pa kazna u O nestane sama). Sa starim podacima A pada na medijan −12,96 (razlika profila 10,4). Prepoznaje li Leon takav hotel? Odgovor: [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md) „Za Leonovu provjeru", točka 4.
- [leon] **Obnova je i dalje najbolja proba za A i C, ali nije dominantna** — sad košta 6–7 bodova F (prije je F rastao), a napredak je A −0,39 → +5,02, B −3,03 → +2,03, C −0,90 → +6,21; za B je bolja „plaća 115". Treba li obnova od 150 000 € biti dobra odluka već u prvoj sezoni? Odgovor: [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md) „Za Leonovu provjeru", točka 5.
- [leon] **Najviše sezonaca 50, ne 40** (DEMO §6.1 promijenjen u kalibraciji) — rasprodan kolovoz traži 44 sezonca da marža ostane ≤ 50 %. Realno za 4* od 120 soba? Odgovor: `content/hotel/catalog.json` (`seasonal_max`), DEMO §6.1.
- [leon] **Ponuđeni odgovori S3 i S4** u `content/hotel/situations.json` su nacrt (ADR-013 t. 7) — provjeriti uz rubrike. Odgovor: `content/hotel/situations.json`, [ADR-013](DECISIONS.md#adr-013) t. 7.
- [leon] **Ishod neodgovorene situacije** (kvaliteta 1 + mehanike `unanswered` za S1–S4; S3 ignorirana = ponuda propada bez kazne) — realno? Odgovor: `content/hotel/situations.json` (Task 1 plana), [ADR-015](DECISIONS.md#adr-015) t. 2.
- [leon] **Pretpostavke ADR-017** (odluke voditelja gradnje: identitet preuzimanja posla, 500 za nepoznat profil, provjere pristupa prije tijela, odgovor bez mjeseca u putanji, javne funkcije jezgre, zabrane u `game`) čekaju pregled. Odgovor: [ADR-017](DECISIONS.md#adr-017).
- [leon] **Pretpostavke ADR-015** (crate `game`, zaključavanje igre po naredbi, vlastiti red poslova, server ne migrira sam) čekaju pregled. Odgovor: [ADR-015](DECISIONS.md#adr-015).
- [leon] **Pretpostavke ADR-013** (centrirani indeksi, početni F, sezonci po planu, redoslijed obračuna, cilj marže 35–45 %) čekaju pregled. Odgovor: [ADR-013](DECISIONS.md#adr-013).

## Drugi val (poslije demoa)

Opseg koji [ADR-013](DECISIONS.md#adr-013) t. 8 odgađa iza demoa; spec je u [DEMO.md](../product/DEMO.md).

- [drugi val] **Splet okolnosti i tragovi (§9)** — odluke ostavljaju tragove koji se kasnije vraćaju.
- [drugi val] **Rubrike i ponuđeni odgovori za S1b i S5–S8** — za robote/balans (§11.3) faltaju; task sadržaja. Nacrt za Leonovu provjeru (rubrike su njegove): [SITUACIJE](../product/SITUACIJE_S1b_S5-S8.md). (DEMO §7, §11.3)
- [drugi val] **Naplata otvorenih tragova na kraju sezone.**
- [drugi val] Limit 3 demo igre po IP-u na dan i brisanje gosta i igre nakon 7 dana (DEMO §1.4; Leon 05.10.).
- [drugi val] **Različit učinak S4 po stanju hotela.**
- [drugi val] **Tablice za 3* i 5*** — danas postoji samo 4*.
- [drugi val] **JSON Schema kataloga** — danas samo provjera u kodu.

## Rupe u specu DEMO.md

Presuđene pretpostavkama u [ADR-013](DECISIONS.md#adr-013) (profil C na 50, sezonska marža, pravila zaokruživanja P1–P32, nedefinirano manje); pretpostavke čekaju Leonov pregled (vidi „Otvoreno za Leona").

## CI i alati

- [ci] **Node 20 akcije zastarjele** — podići `actions/checkout@v4` i `actions/setup-node@v4` na novije verzije u nekom tasku CI-ja. (prvi CI 04.10., run 37167540215)
- [ci] **`ubuntu-latest` prelazi na Ubuntu 26 od 19.10.** — provjeriti da poslovi `rust` i `web` prolaze, po potrebi pinirati sliku. (isto upozorenje CI-ja)
- [ci] **CI prevodi `sqlx-cli` svaki put bez cachea (~5–10 min)** — bitno kad CI dobije mutacijski posao. (Task 8 #1, L86)
- [ci] **`cargo-deny-action@v2` koristi vlastiti Rust, a repo `rust-toolchain` 1.98.1** — provjeriti na prvom CI-u. (Task 8 #3, L88)
