# Backlog

Ovo je JEDINO mjesto za ono što nije riješeno: odgođeni nalazi pregleda, otvorena pitanja i tehnički dug. Plan ne nosi tuđe stavke — plan uzima stavke odavde po oznaci; kad se stavka riješi, briše se (povijest je u gitu i `PROGRESS.md`).

Oznake: `[jezgra]` plan jezgre i balansa · `[dnevnik]` plan dnevnik/runde/red poslova (uključuje API dio, ARHITEKTURA §20.2) · `[ekrani]` ekrani igrača · `[prije Railwaya]` prije prvog deploya · `[možda nikad]` · `[ci]` CI i alati · `[leon]` čeka Leonov odgovor.
Izvor „L37" = redak u [ledgeru kostura](../archive/2026-10-04-kostur-ledger.md); „Task N #k" = nalaz pregleda tog taska.

## Odgođeni nalazi — [jezgra]

- [jezgra] **`deny` umjesto `forbid` za clippy brane** — `#![deny]` u `lib.rs` lokalni `allow` zaobilazi. (Task 1 #3, L37)
- [jezgra] **Mutacijski dokaz brana nepotpun** — mutacijom je provjeren samo `HashMap` u `engine`; faltaju `disallowed-methods`, HashSet, sat, float/arith i `clippy.toml` za `sim-hotel`. (Task 1 #4, L38)
- [jezgra] **Ojačati brane determinizma (Final #3)** — allowlist ovisnosti (indexmap), `RandomState` u oba `clippy.toml`, forbid + `as_conversions`, zabrana f32/f64, `env::var`, skripta mutacija u CI-u. **Obavezan Task 1 plana, prije ijedne linije jezgre.** (L95, [SLJEDECA_SESIJA](../SLJEDECA_SESIJA.md) §3.1)
- [jezgra] **Test `derive_streams_are_independent` je gotovo prazan** — dodati `assert_ne` za T1≠T2, redoslijed oznaka, drugo sjeme. (Task 2 #1, L42)
- [jezgra] **Grana odbacivanja u `range` se ne izvršava** — mali rasponi u testovima je ne pogađaju. (Task 2 #2, L43)
- [jezgra] **Doc za `derive(seed, &[])` vs `from_seed`** — različiti prostori stanja, treba zapisati. (Task 2 #3, L44)
- [jezgra] **Nema testa da `position()` odgovara `Index::ALL`.** (Task 3 #1, L47)
- [jezgra] **Profili A i B testirani samo točkasto** — pinati pune vektore; uskladiti s rupom „profil C ne kreće s 50". (Task 3 #2, L48)
- [jezgra] **Test odbijanja u `from_hundredths`** provjerava samo zadnje mjesto. (Task 3 #3, L49)
- [jezgra] **Imenovana i ograničavajuća izgradnja `IndexSet`** — danas pozicijski `[IndexValue; 5]`. (Final minori, L96)
- [jezgra] **Profili u `content/hotel/` kao testno proročanstvo** — kategorija hotela kao podatak. (Final minori, L96)
- [jezgra] **`.gitattributes`: LF za `Cargo.lock`, `Cargo.toml`, `*.mjs`** — danas nisu pinirani. (Task 1 #5, L39)
- [jezgra] **`.sqlx/*.json` nije piniran na LF.** (Task 4 #5, L57)
- [jezgra] **CI prevodi `sqlx-cli` svaki put bez cachea (~5–10 min)** — bitno kad CI dobije mutacijski posao. (Task 8 #1, L86)
- [jezgra] **`cargo-deny-action@v2` koristi vlastiti Rust, a repo `rust-toolchain` 1.98.1** — provjeriti na prvom CI-u. (Task 8 #3, L88)

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

## Rupe u specu DEMO.md (ulaz za plan jezgre)

Revizija 04.10., račun ponovljen neovisno, nalazi potvrđeni. Rješavaju se u planu jezgre prije prvog testa; pretpostavke zapisati u dokument.

- [jezgra] **Profil C ne kreće s 50** — iz DEMO §2.1 i §4 za početak lipnja izlazi G=70, R=50, Z≈54, O=70, F≈55 (zbroj ≈299, ne 250). Samo je F relativan; G, Z, O su apsolutni, iako §2.0 kaže „prema referentnom hotelu". Odlučiti: centrirati formule na referentni hotel ili promijeniti početne vrijednosti. Analiza opcija: [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md) (preporuka `50 + (x − x_ref)`; prijedlog, provjeriti skriptu prije preuzimanja brojki). (DEMO §5, §2.1)
- [jezgra] **Kalibracija bez redovnog održavanja 15 000 €** — s njim: lipanj 33,6 %, srpanj 50,4 %, kolovoz 55,5 %, rujan 34,3 %, sezona ≈45 %. Cilj 30–40 % je godišnji, a demo ima četiri sezonska mjeseca; odlučiti s čime se uspoređuje (KALIBRACIJA predlaže sezonski VI–IX 35–45 %). ([KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md))
- [jezgra] **Cjelobrojni račun bez pravila zaokruživanja** — kazna opterećenja, potražnja i f-faktori, fluktuacija, nazivnik GOPPAR-a, jedinice pohrane, „p. b." vs postotak. Zlatne vrijednosti ovise o tome. Nacrt pravila P1–P32 u [DEMO](../product/DEMO.md) §12; najrizičnije P21 (stepenica odlazaka), P28/P29 (nazivnik GOPPAR-a), P17 (linearna kazna opterećenja). (DEMO §12)
- [jezgra] **Nedefinirano manje** — početni F (GOPPAR traži mjesec), „mješavina" iskustva, cijena automatske zamjene, trošak sezonca, obuka „moral +2", dijeljenje s nulom u g_i, tvrdo 120 soba u §3.1. (DEMO §3.1)
- [jezgra] **Rubrike i ponuđeni odgovori za S1b i S5–S8** — za robote/balans (§11.3) faltaju; task sadržaja. Nacrt za Leonovu provjeru (rubrike su njegove): [SITUACIJE](../product/SITUACIJE_S1b_S5-S8.md). (DEMO §7, §11.3)

## CI i alati

- [ci] **Node 20 akcije zastarjele** — podići `actions/checkout@v4` i `actions/setup-node@v4` na novije verzije u nekom tasku CI-ja. (prvi CI 04.10., run 37167540215)
- [ci] **`ubuntu-latest` prelazi na Ubuntu 26 od 19.10.** — provjeriti da poslovi `rust` i `web` prolaze, po potrebi pinirati sliku. (isto upozorenje CI-ja)
