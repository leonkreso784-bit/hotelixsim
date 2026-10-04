# Backlog

Ovo je JEDINO mjesto za ono što nije riješeno: odgođeni nalazi pregleda, otvorena pitanja i tehnički dug. Plan ne nosi tuđe stavke — plan uzima stavke odavde po oznaci; kad se stavka riješi, briše se (povijest je u gitu i `PROGRESS.md`).

Oznake: `[jezgra]` plan jezgre i balansa · `[dnevnik]` plan dnevnik/runde/red poslova (uključuje API dio, ARHITEKTURA §20.2) · `[ekrani]` ekrani igrača · `[prije Railwaya]` prije prvog deploya · `[drugi val]` opseg poslije demoa · `[možda nikad]` · `[ci]` CI i alati · `[leon]` čeka Leonov odgovor.
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
- [jezgra] **Učitavanje sadržaja:** privremene mape u testovima se ne brišu ako test padne (nema `Drop` čuvara); nema provjere dupliciranih kodova situacija ni praznih odgovora, a duplicirani ključevi u mapama = zadnji pobjeđuje. (Task 8)

**Alat za balans (`balance`)**
- [jezgra] **`margin_bp` tiho vraća 0 kad je prihod ≤ 0;** `--seeds 0` se prihvaća; testovi pokrivaju samo granu §11.1 u `failures`, nema testa izlaznog koda `--check`; tolerancija §11.1 tvrdo u kodu, `_cat` neiskorišten. (Task 10)

## Odgođeni nalazi — [dnevnik]

- [dnevnik] **`stream_sequence_is_unique` je preslab** — provjerava samo `is_err`, ne SQLSTATE 23505, ni da isti seq u drugom toku prolazi. (Task 4 #2, L54)
- [dnevnik] **Oblik tablice `events`** — globalni redoslijed, tip `actor`, indeks na `game_id`; mora biti gotovo prije Railwaya. (Final minori, L96; SLJEDECA_SESIJA §3)
- [dnevnik] **`HealthBody.events` nije u `required`** — TS tip je `events?: number|null`. (Task 5 #1, L61)
- [dnevnik] **OpenAPI `info` bez description i license.name.** (Task 5 #2, L62)
- [dnevnik] **Nema fallback rute** — nepoznata ruta daje prazan 404, ne problem+json. (Task 5 #4, L64)
- [dnevnik] **Test OpenAPI-ja ne tvrdi 503 ni problem+json za 404** — 503 je testiran samo lažnim portom. (Task 5 #6, L66)
- [dnevnik] **`api` zove `sim-hotel` mimo `app`** (`api/src/demo.rs`) — zaseban označen task na kraju plana jezgre ili plan dnevnika. (Final minori, L96; SLJEDECA_SESIJA §3.2)
- [dnevnik] **Enum kodovi u OpenAPI-ju** — danas `String`. (Final minori, L96)
- [dnevnik] **`check:api` ne hvata nepraćenu generiranu datoteku** — dodati `git status --porcelain`. (Task 6 #2, L70)

## Odgođeni nalazi — [ekrani]

- [ekrani] **`openapi-fetch` i `application/problem+json`** — provjeriti parsiranje 404 tijela u prvom stvarnom pozivu. (Task 6 #3, L71)
- [ekrani] **i18n bez lokala i zajedničkog tipa za en rječnik** — `format.ts` ima tvrdo „hr-HR". (Task 7 #1, L77)
- [ekrani] **`retry` 3 u QueryClientu daje ~7 s „Učitavam…"** kad API ne radi. (Task 7 #2, L78)
- [ekrani] **Testovi ne pokrivaju pending, 500 na health, skriveni red događaja kad je events null** — lažni fetch pretpostavlja `Request`. (Task 7 #3, L79)
- [ekrani] **Health redak bez `role="status"`/`aria-live`.** (Task 7 #4, L80)
- [ekrani] **`t()` parametri nisu tipizirani po poruci** — krivo ime daje vidljiv `{placeholder}`. (Task 7 #5, L82)
- [ekrani] **`vite build` u CI-ju.** (Final minori, L96)

## Odgođeni nalazi — [prije Railwaya]

- [prije Railwaya] **Nema testa za `connect_lazy` na nedostupnu bazu (~3 s) ni `migrate` na svježoj bazi.** (Task 4 #3, L55)
- [prije Railwaya] **sqlx bez TLS značajke** — dodati `tls-rustls` (nikad native-tls/openssl). (Task 4 #4, L56)
- [prije Railwaya] **`COUNT(*)` u zdravlju** je skup na velikoj tablici. (Final minori, L96)
- [prije Railwaya] **Greška `ctrl_c` je ignorirana** — server bi se odmah ugasio. (Task 5 #3, L63)
- [prije Railwaya] **Base URL klijenta/API-ja.** (Final minori, L96)
- [prije Railwaya] **SIGTERM (graceful shutdown).** (Final minori, L96)
- [prije Railwaya] **Typed config** umjesto sirovih env varijabli. (Final minori, L96)
- [prije Railwaya] **CI bez filtra, akcije po tagu a ne po SHA** — CI radi na push i pull_request; `permissions: contents: read` već postoji. (Task 8 #2, L87)
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
- [leon] **S3: lekcija je OBRNUTA, ne samo tanka** — prihvaćanje grupe (175 soba-noći, −25 %) donosi GOP u kolovozu više nego u lipnju: C +25 598 € (kolovoz) naspram +19 184 € (lipanj), A +8 336 € u kolovozu, B +26 009 € (bez istisnutih gostiju); istiskivanje prolazi na ≈ 2 soba-noći. DEMO §7 („dobra u lipnju, loša u kolovozu") danas ne vrijedi. Poluge (odluka je Leonova): viša popunjenost kolovoza (96 % ≈ 20 soba-noći), manji popust u ponudi, niži ostali prihod / dodatni trošak grupe, kolovoz s tijesnim kapacitetom. Odgovor: [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md) „Za Leonovu provjeru", točka 4.
- [leon] **Sezonska marža 35,9 % blizu donje granice (35 %)**, lipanj i rujan ≈ 22 %. Odgovor: [ADR-013](DECISIONS.md#adr-013) t. 5 i [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md).
- [leon] **Obnova je praktički besplatna** — „obnova u lipnju" je najbolja proba za A i C (A +7,63, C +10,12). Mehanizam: do rujna likvidnost je zaglavljena na 100, a capex i kamate su izvan GOP-a pa GOPPAR ne vidi 150 000 €; F čak raste (A 42,72 → 43,62, B 61,26 → 62,26, C 50,00 → 50,95), napredak A −1,05 → +7,63, B −7,21 → +1,49, C −0,26 → +10,12. Poluga je formula likvidnosti/F (pitanje za ADR), ne cijena. Odgovor: [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md) „Za Leonovu provjeru", točka 5.
- [leon] **Ponuđeni odgovori S3 i S4** u `content/hotel/situations.json` su nacrt (ADR-013 t. 7) — provjeriti uz rubrike. Odgovor: `content/hotel/situations.json`, [ADR-013](DECISIONS.md#adr-013) t. 7.
- [leon] **Pretpostavke ADR-013** (centrirani indeksi, početni F, sezonci po planu, redoslijed obračuna, cilj marže 35–45 %) čekaju pregled. Odgovor: [ADR-013](DECISIONS.md#adr-013).

## Drugi val (poslije demoa)

Opseg koji [ADR-013](DECISIONS.md#adr-013) t. 8 odgađa iza demoa; spec je u [DEMO.md](../product/DEMO.md).

- [drugi val] **Splet okolnosti i tragovi (§9)** — odluke ostavljaju tragove koji se kasnije vraćaju.
- [drugi val] **Rubrike i ponuđeni odgovori za S1b i S5–S8** — za robote/balans (§11.3) faltaju; task sadržaja. Nacrt za Leonovu provjeru (rubrike su njegove): [SITUACIJE](../product/SITUACIJE_S1b_S5-S8.md). (DEMO §7, §11.3)
- [drugi val] **Naplata otvorenih tragova na kraju sezone.**
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
