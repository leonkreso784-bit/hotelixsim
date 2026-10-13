# Backlog

Ovo je JEDINO mjesto za ono što nije riješeno: odgođeni nalazi pregleda, otvorena pitanja i tehnički dug. Plan ne nosi tuđe stavke — plan uzima stavke odavde po oznaci; kad se stavka riješi, briše se (povijest je u gitu i `PROGRESS.md`).

Oznake: `[jezgra]` plan jezgre i balansa · `[dnevnik]` plan dnevnik/runde/red poslova (uključuje API dio, ARHITEKTURA §20.2) · `[korak 4]` uzima plan koraka 4 (AI, sadržaj) · `[ekrani]` ekrani igrača · `[prije Railwaya]` prije prvog deploya · `[drugi val]` opseg poslije demoa · `[možda nikad]` · `[ci]` CI i alati · `[leon]` čeka Leonov odgovor · `[sloj1]` put B, sloj 1 (grana `model-v2/sloj1`) · `[sloj2]` ulaz za plan sloja 2.
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

- [dnevnik] → korak 4: zaglavlje `Idempotency-Key` (ARHITEKTURA §11); probni test apalis/pgmq (ARHITEKTURA §9.4) — odgođeno, ADR-019.
- [dnevnik] **Siroče gost:** `start_demo` upisuje gosta pa igru u dvije zasebne radnje — ako `create_game` padne, ostaje gost bez igre; počistiti uz brisanje gostiju nakon 7 dana. (`app/src/demo.rs:41`)
- [dnevnik] **`ErrorCode::ALL` i `as_str()` se ne koriste** (žičani kod daje serde `ProblemCode`) — dva izvora istine; test `ProblemCode::from(c)` = `c.as_str()` za sve ili brisanje. (`app/src/error.rs:42,84`; M1)
- [dnevnik] **`parse_hotel`/`parse_game`/`parse_month` idu prije `caller.require()`** — anonimac s pokvarenim uuid-om dobiva 404 umjesto 401; bezopasno. (`api/src/hotels.rs:54,107`, `games.rs:222,247,274`; M2)
- [dnevnik] **`check:api` pada na staged promjenama** (`git status --porcelain` vidi i staged) — gledati samo `??` i nestageano ili zapisati u README. (`scripts/check-clean.mjs:10`, bez try/catch; M5, T11)
- [dnevnik] **`?` nakon `axum::serve` preskače `worker.abort()`.** (`server/src/main.rs`; T11)
- [dnevnik] **`unanswered` je izveden na dva mjesta (`RoundView`, `schema::SituationsBody`) → jedno polje u `SituationsView`. (T10)
- [dnevnik] **Ručni „ponovi obračun" za `failed` posao** (Leon/profesor); `COALESCE(last_error, …)` kod isteka na zadnjem pokušaju čuva stariju poruku. (`store/src/jobs.rs`; T6, T8)
- [dnevnik] **`game` pomoćnici:** dupli blok čitanja `RoundResolved` u `state_at`/`latest_state` i 5× lookup `HotelResolution` → helper (T3, T4); `apply` ne provjerava poznatu situaciju/indeks odgovora/jedinstvenost slota u `created` i kod duplog odgovora prepiše prije greške (fold ionako pada); `robot_moves` ne provjerava da je mjesec `Open`. (T3)
- [dnevnik] **`set_plan`/`answer` zovu `game_of_hotel` prije prava** (404 vs 403 otkriva postojanje uuid-a v4 — zanemarivo); `corrupt()` log bez id-a hotela; mapiranje strategija string-matchom paralelno s `game`. (T6, T9)
- [dnevnik] **Testovi koji fale:** (a) `INDEX_ORDER` prati `Index::ALL` i JSON oblik `Controller` + `GameId::stream()` = `game:<uuid>`; (b) A/B/C samo za seed 42, determinizam `new_game`, `seasonal_min`, `renovation_available=false`; (c) feed „samo moje" bez simetričnog slučaja, srpanj samo brojem događaja, `private_hotel.now` nakon kraja igre, S3 linija neodgovorenog; (d) `Memory` paritet (duplikat `create_game`/`create_guest` tiho prepisuje, Postgres pada), semantika memory porta, zatvorena/završena igra, `token_hash`, potpunost `ErrorCode::ALL`; (e) grana dvostrukog pada (`fail` sam padne) i otpuštanje advisory locka nakon odbijanja, duple igre samo `is_err`; (f) konkurentnost `claim` s 2+ posla i N preuzimača; (g) kolačići (31/33 znaka, ne-hex, nepoznat token), `me/hotel` bez igre, popis ključeva tuđeg hotela, loša tijela 422 po polju, feed `by`/mjesec/redoslijed, `FeedItemBody` bez `discriminator` u OpenAPI-ju; (h) e2e: replay samo lipanj, pretpostavka jedne neodgovorene po mjesecu, robot provjere samo postojanje događaja; dva preklapajuća roundtrip testa u `content/tests/snapshot.rs`. (T1–T12)
- [dnevnik] **Test T4 `resolution_uses_snapshot_not_files` ručno gradi `created`** — dodati `created_from(&Snapshot, seed)` u `tests/common`. (T4)


## Odgođeni nalazi — [korak 4]

- [korak 4] **Svako čitanje radi puni fold** (`Snapshot::build` klonira cijeli snimak, SHA-256, `RefTable::compute`) — zanemarivo za 4 hotela × 2 mjeseca; riješiti uz projekcije. (M6, T1)

## Odgođeni nalazi — [ai-sloj]

- [ai-sloj] **Pravi Postgres test „rok prošao dok procjena još čeka“** (posao procjene u odgodi/`running`, obračun ostario > 120 s, neutralna na roku, zakašnjela presuda poslije) — e2e dokazuje samo put preko petog pokušaja. (`crates/api/tests/e2e.rs`)
- [ai-sloj] **`DebriefSituation` treba zastavicu `answered`** — mentor ne smije kritizirati odgovor koji student nije dao (rezerva i ponuđeni q1 se ne razlikuju). (`crates/game/src/ai_input.rs`)
- [ai-sloj] **Testovi koji nedostaju:** fold odbija pokvaren dnevnik razgovora (7. poruka, dvije studentove zaredom, krivi `reply_to`, `AnswerChosen` nakon `ResponseSubmitted`), negativni test privatnosti feeda (tuđi hotel ne vidi Response/Verdict/Mentor, late/neutral stavka), 404 za nove rute, točne odgode 2/4/8/16/60 s. (`crates/game/tests/`, `crates/api/tests/`, `crates/store`)
- [ai-sloj] **`AssessInput` se gradi na dva mjesta** — `ai/benchmark.rs` i `game/ai_input.rs`; izvući jedno. (`crates/ai/src/benchmark.rs`)
- [ai-sloj] **Serijski radnik:** odgovor lika čeka iza dugog posla presude (jedan radnik); za pilot više radnika ili odvojeni redovi. (`crates/server/src/main.rs`)
- [ai-sloj] **Latencija Opusa (effort high) prema roku 120 s** — izmjeriti mjerilom (`cargo run -p ai --bin mjerilo`, ispisuje trajanje po pozivu) prije 14.10. (`crates/ai/src/benchmark.rs`)
- [ai-sloj] **Sitnice:** `legacy_fingerprint_differs` je slab test i `from_value` klonira `Value` pri svakom foldu (`content`); `validate_verdict` vraća `NotFreeText` i za nepoznatu šifru (`game/ai_input.rs`); `unwrap_or_default` pri serijalizaciji `DebriefInput` (`ai/request.rs`); nema testa UTF-8 rezanja tijela odgovora (`ai/response.rs`); `aws-lc-sys` traži C alate — provjeriti prvi CI; SQL odgode ima niz razmaka u nastavcima retka (`store`); odgoda se ne provjerava prije UPDATE roka i `tick` resetira i `running` (e2e).

## Odgođeni nalazi — [ekrani]

- [ekrani] **i18n bez lokala i zajedničkog tipa za en rječnik** — `format.ts` ima tvrdo „hr-HR". (Task 7 #1, L77)
- [ekrani] **Testovi ne pokrivaju pending, 500 na health, skriveni red događaja kad je events null** — lažni fetch pretpostavlja `Request`. (Task 7 #3, L79)
- [ekrani] **`t()` parametri nisu tipizirani po poruci** — krivo ime daje vidljiv `{placeholder}`. (Task 7 #5, L82)
- [ekrani] Posao obračuna u statusu `failed` ostavlja rundu u „closing" zauvijek — ekran sad kaže „traje dulje", ali ne razlikuje pad od sporosti, a profesor ne vidi ništa; SSE „runda je obračunata" umjesto ispitivanja (ARHITEKTURA §11).
- [ekrani] **E2e u CI-ju** — Postgres i poslužitelj u poslu weba (danas e2e pokrećeš samo lokalno).
- [ekrani] **Ilustracija obale** na ekranu hotela.
- [ekrani] **409 „Runda nije otvorena" za neobračunat mjesec na ruti objašnjenja** — treba poseban kod `month_not_resolved`.
- [ekrani] **Moral u `resolve.rs` (~178, ~364)** još tvrdo kodira granicu 180 i formulu kazne (nije na `load_penalty`; vidi [jezgra] „Kazna opterećenja duplicirana").
- [ekrani] **Popis modela s poslužiteljskim rezervnim modelom tvrdo kodiran** (`supports_server_fallback`).
- [ekrani] **„0 kad runda nije otvorena" napisano dvaput** u `crates/app/src/views.rs`.
- [ekrani] **Nema HTTP testa za `/situations.unanswered`.**

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

## Odgođeni nalazi — [sloj1]

Svjesno odgođeno u sloju 1 (pretpostavke: [ADR-029](DECISIONS/ADR-029-sloj-1-pretpostavke-plana.md) i ADR-031…036) i
nalazi pregleda koji nisu popravljeni.

**Opseg odgođen planom**
- [sloj1] **Situacija s tjednom u mjesecu** (P13): situacija mjeseca dolazi u krugu koji sadrži 1. dan mjeseca (t01, t05, t09, t14); polje „tjedan u mjesecu" iz MODEL_V2 §4.3 nije napravljeno. Uvjet: mjesec situacije mora prvo postati podatak na ishodu (danas ga jezgra izvodi iz razdoblja, `Period::month_starting_in`).
- [sloj1] **Zbirni mjesečni mentor u tjednoj igri** (P3): AI mentor gleda činjenice samo zadnjeg kruga mjeseca, ne zbroj mjeseca; ostali tjedni dobivaju „tri uzroka" bez AI-ja.
- [sloj1] **Profesorov račun dobiti** — svi hoteli igre jedan uz drugoga (P17, MODEL_V2 §7.2); student vidi samo svoj hotel. Procjena ~4–6 h (ruta za profesora + ekran).
- [sloj1] **„Zašto" po odjelu** (P18): detalj odjela pokazuje razlaganje brojkama, ne retke uzroka po odjelu.
- [sloj1] **Naknada za upravljanje i informatika (I&T) nisu modelirani** (P10); „Fiksne naknade" = osiguranje, najam, porez na imovinu.
- [sloj1] **Povišice iz situacija** (`extra_wage_cents`) idu u trošak rada Smještaja (P12); po odjelu u sloju 2.
- [sloj1] **Probne v2 igre:** igre stvorene usred sloja 1 nisu zaštićene verzijom jezgre (stari v2 dnevnici postaju `Corrupt` pri podizanju `CORE_VERSION`; obračun bez `accounts` čita se kao `Corrupt`; krugovi prije Taska 23 nemaju odlaske po odjelu pa kartica „Ljudi" kaže „nitko"). Prije izdanja obrisati lokalne razvojne v2 igre ili ih ponovno obračunati.

**Nalazi pregleda**
- [sloj1] **Equipment/Maintenance redak** nosi i −30 % promjene Smještaja (miješanje 70/30 opreme sažetka; Task 12).
- [sloj1] **`department_plan_invalid`** ima jednu poruku za svih šest vrsta greške (poruka po vrsti traži prošireni `Problem` s `detail`); neispravan `scope` u `/pl` daje axumov 400, ne problem+json. (Task 15)
- [sloj1] **`hr.ts` nema `problem.game_outdated` ni `problem.invalid_settings`** — ekran pokazuje opću poruku greške. (Task 6)
- [sloj1] **Prag opterećenja u webu:** `insights.ts`/`score.ts` drže 180 (granica kazne, v1 `LOAD_LIMIT_HUNDREDTHS`) i 240 (puni kapacitet) kao svoje konstante; API ne šalje normu odjela (`capacity_tenths_per_staff_day`). (Task 19, OD)
- [sloj1] **Balans:** indeks likvidnosti u tjednu sa zapošljavanjem skalira jednokratni trošak ×D₀/7. (Task 16/17)
- [sloj1] **Sadržaj v2:** jezgra za Smještaj ne čita `seasonal_plan`, `wear_pts_per_month` ni `maintenance` iz `departments.json` (uzima kategoriju hotela, [ADR-032](DECISIONS/ADR-032-sloj-1-odjeli-u-obracunu-i.md) t. 1) — maknuti polja ili ih početi čitati; nema provjere da je plan sezonaca kategorije ≥ zbroj ostalih odjela po profilu i mjesecu. (Task 11)
- [sloj1] **[PROC] bez obrazloženja:** `content/hotel/v2/profiles.json` — stalni dio Održavanja profila A (`fixed_monthly_cents` 12.400 €/mj., uz B 7.200 € i C 4.800 €) je oko 2.000 € viši nego što obrazloženje Taska 17 pokriva; treba izvor ili ispravak. (Task 23)
- [sloj1] **Testovi i čuvari:** `crates/balance/tests/robots_v2.rs` ima `.min(50)` tvrdo (čitati `seasonal_max`); zastario komentar u `crates/game/tests/model_v2_catalog.rs:101`; `dispatch_lives_in_one_place` hvata samo doslovne nizove (dodati `matches!(…, Model::V1)` i `== Model::V1`); v1 tijelo s `departments` uz nepoznat krug daje `unknown_month` (prije `invalid_body`). (kontrolor ZP)
- [sloj1] **UI čuvar:** rezervni tekst mentora lomi „7.|lipnja"; „Ta situacija nije u ovom tjednu." i „Ovaj tjedan još nije zatvoren." su bez `h1`. (kontrolor ZP)
- [sloj1] **Rečenice strategije robota** su u Rustu (`strategy_words`, za sezonskog mentora), a ekran kraja sezone ih nosi u `hr.ts` — dva mjesta istog teksta; test koji ih drži jednakima ili jedan izvor (API šalje rečenicu). Ekran treba i isto pravilo zaokruživanja napretka kao mentor (cijeli bodovi, desetinke samo kad bi se različiti napretci poklopili). (KS-2)
- [sloj1] **Mentor kruga sprema nepotpun AI tekst kao `ai`:** prazan ili odrezan tekst (`run_debrief`) ostaje zauvijek zapisan kao `source: ai`; ista provjera kao sezonski mentor (četiri dijela iz `mentor.v3`) pa rezerva `fallback_debrief`. (KS-2)
- [sloj1] **Razmak prije „%" u tekstovima iz Rusta** (objašnjenje kruga, `crates/game/src/mentor.rs` „{pct} % potražnje") je običan, a web ga ima nelomljivog. Ne dirati dok v1 mora ostati bajt-ist (ili samo u v2 grani teksta). (OD-7)
- [sloj1] **Mentor i sezonski ugovori:** mentor krajem kolovoza upozori „rujan pada, planiraj kraj sezonskih ugovora"; u v2 redak troškova „fiksni troškovi" (stalni dio zajedničkih službi) treba drugo ime, blizu Računa dobiti — tekst je zajednički s v1, pa objašnjenje mora nositi model (ili `LineDto` drugi uzrok); redak blagajne već se zove „fiksni troškovi zgrade (najam, osiguranje, porez)". (ZP)

## Za plan sloja 2 — [sloj2]

Leonove želje 12.10.; brojke idu kroz Leonovu anketu pri planu sloja 2. Ideje nakon profesorova pregleda: [IDEJE.md](../product/IDEJE.md).

- [sloj2] **Odnosi među zaposlenima kao u BitLifeu** (Leon: „treba biti"): prijateljstva, sukobi, obitelj/kolege — odnos dvoje konkretnih ljudi (prijateljice odlaze zajedno, kuhar i konobar u sukobu). MODEL_V2 §3.4 danas ima samo osobine koje djeluju na cijeli odjel. U plan sloja 2 (2a ili zaseban dio).
- [sloj2] **Obećanja iz odgovora koja se prate** (S1 kuhar): student obeća konkretne radnje (slobodan dan, jelovnik, zamjenik), a igra ih danas ne provodi niti provjerava. Prijedlog za 2c i plan situacija: presuda izvuče obećane radnje u strukturirani popis, igra ih pretvori u zadatke („zaposli zamjenika u kuhinji do t04") i prati — ispunjeno → motivacija lika +, neispunjeno → lik se vrati („obećali ste mi zamjenika"). Uz burzu rada (2a): zamjenik je stvarna osoba s burze.
- [sloj2] **Plaća po radniku i pravednost plaća:** plaća se bira za svakog radnika (MODEL_V2 §3.3 već ima plaću po osobi). Novo pravilo: ako netko ima neopravdano višu plaću od kolega iste uloge (bez veće vještine ili staža), motivacija ostalih u odjelu pada — član „nepravednost u odjelu" u formuli motivacije.
- [sloj2] **Dugotrajna i kratkotrajna oprema, realna amortizacija** (Leon: *„parking … jednom ga obnoviš i traje godinama"*): dugotrajna imovina (zgrada, parking, instalacije — sporo trošenje, skupa obnova) i kratkotrajna (posteljina, namještaj, sitna oprema); amortizacija ispod GOP-a (USALI). Ide u obnovu v2 (grana `model-v2/sloj1-obnova`, još nije spojena).
- [sloj2] **Sezonski ugovor do datuma:** odlazak na kraju ugovora ne troši tjedni korak ±3. (ZP)

## Otvoreno za Leona

Pitanja o domeni ne blokiraju kod: brojke su podaci u katalogu, gradi se s pretpostavkom i mijenja kad odgovori.

- [leon] **Ostali prihod (hrana, piće, usluge) = 30 % prihoda soba** — je li realno? Odgovor: [DEMO](../product/DEMO.md) „Za Leonovu provjeru", točka 2. (SLJEDECA_SESIJA §5.2)
- [leon] **Trošak zaposlenog 2 200 €/mj i omjer 30 stalnih + 20 sezonaca** (120 soba, 4*) — realno? Odgovor: [DEMO](../product/DEMO.md) „Za Leonovu provjeru", točka 3.
- [leon] **Brana veličine koda `check:kod`: upozorenje 600, granica 1000, popis duga (3 datoteke)** — potvrda ili druge brojke (za `.md` si rekao 1200). Odgovor: [ADR-039](DECISIONS/ADR-039-brana-velicine-koda-upozorenje-600-granica.md).
- [leon] **S3: 25 % popusta agenciji za grupu u kolovozu** — realna ponuda? Odgovor: [DEMO](../product/DEMO.md) „Za Leonovu provjeru", točka 4.
- [leon] **Kriteriji rubrika u S1–S4** — Leon je mjerilo; blokira sadržaj i AI presude, ne kod jezgre. Odgovor: [DEMO](../product/DEMO.md) „Za Leonovu provjeru", točka 5.
- [leon] **Priče profila A i B (§5)** — prepoznaje li takve hotele? Odgovor: [DEMO](../product/DEMO.md) „Za Leonovu provjeru", točka 6.
- [leon] **Pitanja za profesore** — Leon ih šalje tijekom tjedna 5.–11.10.; pregled prije slanja. Odgovor: [PITANJA_ZA_PROFESORE](../product/PITANJA_ZA_PROFESORE.md).
- [leon] **Inkubator** — prijava do **16.11.**: nosi li se Sokrat ili simulacija. Odgovor: [ROADMAP](../plan/ROADMAP.md).
- [leon] **Potvrda ARHITEKTURE v2** — ROADMAP kaže „Leon ih je pregledao", a ARHITEKTURA kaže „prijedlog dok Leon ne potvrdi"; potvrditi ili ostaviti raspravu otvorenom, pa uskladiti oba. Odgovor: [ARHITEKTURA](../architecture/ARHITEKTURA.md).
- [leon] **Sezonska marža 35,8 % blizu donje granice (35 %)**, lipanj 22,5 %, rujan 20,3 % (rujan nosi 34 sezonca jer plan smije pustiti najviše 10 mjesečno). Odgovor: [ADR-013](DECISIONS/ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md) t. 5 i [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md) „Za Leonovu provjeru", točka 2.
- [leon] **B ima fiksne troškove samo 3 % niže od C (145 000 prema 150 000 €)** — §5 kaže „niži fiksni troškovi", ali uz likvidnost bez granice (ADR-014) svaki euro razlike ulazi u blagajnu i F profila B raste sam od sebe: s 112 500 € B je najbolji pod svim strategijama (razlika medijana 8,69), sa 140 000 € tijesno prolazi (4,24). Jasno niži fiksni troškovi traže promjenu formule F ili ocjene napretka (nova odluka). Odgovor: [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md) „Za Leonovu provjeru", točka 3.
- [leon] **A: ugled od opreme i iskustva, a premalo ljudi** — A sad ima opremu 75, iskustvo 70, 28 stalnih i početno opterećenje 2,20 (u lipnju ≈ 1,6, pa kazna u O nestane sama). Sa starim podacima A pada na medijan −12,96 (razlika profila 10,4). Prepoznaje li Leon takav hotel? Odgovor: [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md) „Za Leonovu provjeru", točka 4.
- [leon] **Obnova je i dalje najbolja proba za A i C, ali nije dominantna** — sad košta 6–7 bodova F (prije je F rastao), a napredak je A −0,39 → +5,02, B −3,03 → +2,03, C −0,90 → +6,21; za B je bolja „plaća 115". Treba li obnova od 150 000 € biti dobra odluka već u prvoj sezoni? Odgovor: [KALIBRACIJA](../research/KALIBRACIJA_2026-10-04.md) „Za Leonovu provjeru", točka 5.
- [leon] **Najviše sezonaca 50, ne 40** (DEMO §6.1 promijenjen u kalibraciji) — rasprodan kolovoz traži 44 sezonca da marža ostane ≤ 50 %. Realno za 4* od 120 soba? Odgovor: `content/hotel/catalog.json` (`seasonal_max`), DEMO §6.1.
- [leon] **Ponuđeni odgovori S3 i S4** u `content/hotel/situations.json` su nacrt (ADR-013 t. 7) — provjeriti uz rubrike. Odgovor: `content/hotel/situations.json`, [ADR-013](DECISIONS/ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md) t. 7.
- [leon] **Ishod neodgovorene situacije** (kvaliteta 1 + mehanike `unanswered` za S1–S4; S3 ignorirana = ponuda propada bez kazne) — realno? Odgovor: `content/hotel/situations.json` (Task 1 plana), [ADR-015](DECISIONS/ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md) t. 2.
- [leon] **Ocjene za mjerilo AI-ja** — upisati `leon_quality` (1–5) za 40 nacrta odgovora, pa `cargo run -p ai --bin mjerilo -- S1 S2` (prag 80 % unutar ±1). Odgovor: `content/hotel/mjerilo/S1.json`, `S2.json`, DEMO §11 t. 7.
- [leon] **Koncept ekrana prema igri** — ocjena na portalima do 10 ili do 5; „1/4" mjeseca ili demo s 2 mjeseca; kratki nazivi kriterija rubrike; avatari likova. Odgovor: [DIZAJN.md](../product/DIZAJN.md) §3.
- [leon] **Sadržaj situacija:** rezerva lika „Gošća je previše ljuta da bi nastavila razgovor" zvuči kao ocjena studenta — preformulirati neutralno; **PRIJE DEMOA:** S3 nema opisa situacije (samo naslov; DEMO §1.5 obećaje „ponuda agencije (brojke)", a osnovna ponuda nigdje nije rečena — opis traži i polje u API-ju jer je jedini „opis" danas uvodna rečenica lika) — „(nacrt)" je maknut iz naslova S3 i S4 10.10.; `znacenje` za „Zaposleni" je predugo i tehničko za studenta. Odgovor: `content/hotel/situations.json`, `content/hotel/catalog.json`.
- [leon] **Put B — 12 odluka prije gradnje** (isti ljudi po profilu, odjeli po profilu, stare igre na v1, otpuštanje, broj mjeseci, RevPAR nad svim sobama, GOP po USALI-ju, odgovor na recenziju, vidljivost drugih hotela, cijena po tipu sobe, imena, voditelj odjela) — svaka s preporukom. Odgovor: [MODEL_V2.md](../architecture/MODEL_V2.md) §9.
- [leon] **Hosting za prvi izlazak (13.10.)** — potvrda preporuke: Railway Hobby, servis i Postgres na Railwayu u istoj regiji, ~5–10 €/mj [PROC]; Supabase samo kasnije za prijavu. Izlazak tek uz profesorov OK i tvoj izričit OK (ADR-004, ADR-005); do tada 0 €. Odgovor: [PLATFORMA dio 06](../architecture/PLATFORMA/06-produkcija-i-trosak.md).
- [leon] **Sesije umjesto JWT-a (L10)** — potvrda; nakon nje ispravak ARHITEKTURA §12 i zasebni ADR. Odgovor: [PLATFORMA dio 03](../architecture/PLATFORMA/03-prijava.md).
- [web] **Bočni izbornik na računalu:** razmak od vrha `md:top-28` tvrdo zadan ispod zaglavlja — kod velikog teksta ili dugog čipa može se preklopiti; staviti izbornik i zaglavlje u isti stupac. Odgovor: `apps/web/src/ui/Layout.tsx`.
- [leon] **Vizija za razvoj cijele aplikacije (Leon 10.10.)** — ništa od ovoga ne ulazi u demo, za demo je dovoljno što imamo:
  - više situacija i više od dva mjeseca;
  - novi likovi: Ivan Gudeljević (zaštitar koji se često opija), Dominik Smurjević („prati sve što je prljavo"), Nikola Magdić, Dr. Tin Šljivović, Leona Krešo — uloge Nikole, Tina i Leone još nisu zadane; pitanje pristanka za prava imena;
  - profesor pri kreiranju igre bira razdoblje (dani / tjedni / mjeseci), broj AI-situacija, dodaje vlastite situacije, bira vrste situacija i „branšu". Otvoreno: (1) je li branša odjel hotela ili vrsta objekta, (2) AI situacije unaprijed uz profesorovo odobrenje ili u hodu, (3) bira li se duljina igre ili ritam kruga. Ide kroz dizajnerski dokument (arhitektura);
  - studenti ulaze u igru kodom, kao u Kahootu;
  - trošak tokena smanjiti „na drugačiji način" — izračunat ćemo kasnije; mjerilo (40 odgovora) za usporedbu modela presude;
  - uređivač lika ([ADR-021](DECISIONS/ADR-021-koncept-ekrana-v2-i-odluke-o.md)).
- [leon] **Pretpostavke ADR-019** (AI kroz red poslova, rok presude 120 s, rezerve na zadnjem pokušaju, likovi i nacrti mjerila) čekaju pregled. Odgovor: [ADR-019](DECISIONS/ADR-019-ai-sloj-arhitektonske-pretpostavke-koraka-4.md).
- [leon] **Pretpostavke ADR-017** (odluke voditelja gradnje: identitet preuzimanja posla, 500 za nepoznat profil, provjere pristupa prije tijela, odgovor bez mjeseca u putanji, javne funkcije jezgre, zabrane u `game`) čekaju pregled. Odgovor: [ADR-017](DECISIONS/ADR-017-odluke-voditelja-gradnje-u-koraku-3.md).
- [leon] **Pretpostavke ADR-015** (crate `game`, zaključavanje igre po naredbi, vlastiti red poslova, server ne migrira sam) čekaju pregled. Odgovor: [ADR-015](DECISIONS/ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md).
- [leon] **Pretpostavke ADR-013** (centrirani indeksi, početni F, sezonci po planu, redoslijed obračuna, cilj marže 35–45 %) čekaju pregled. Odgovor: [ADR-013](DECISIONS/ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md).

- [leon] **Pretpostavke Sloja 1** (P5–P18, pravila tjednog kruga i odjela, kalibracija, sučelje Odjela, kraj sezone)
  čekaju pregled; posebno **norma osoblja 4\*** (Smještaj 17–25, Uprava 5–7, Prodaja 1,5–3, Održavanje 3–6 ljudi na
  100 soba [PROC]). Odgovor: [ADR-029](DECISIONS/ADR-029-sloj-1-pretpostavke-plana.md), ADR-031…036
  ([ADR-032](DECISIONS/ADR-032-sloj-1-odjeli-u-obracunu-i.md) t. 9 za normu).

## Drugi val (poslije demoa)

Opseg koji [ADR-013](DECISIONS/ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md) t. 8 odgađa iza demoa; spec je u [DEMO.md](../product/DEMO.md).

- [drugi val] **Splet okolnosti i tragovi (§9)** — odluke ostavljaju tragove koji se kasnije vraćaju.
- [drugi val] **Rubrike i ponuđeni odgovori za S1b i S5–S8** — za robote/balans (§11.3) faltaju; task sadržaja. Nacrt za Leonovu provjeru (rubrike su njegove): [SITUACIJE](../product/SITUACIJE_S1b_S5-S8.md). (DEMO §7, §11.3)
- [drugi val] **Naplata otvorenih tragova na kraju sezone.**
- [drugi val] Limit 3 demo igre po IP-u na dan i brisanje gosta i igre nakon 7 dana (DEMO §1.4; Leon 05.10.).
- [drugi val] **Različit učinak S4 po stanju hotela.**
- [drugi val] **Tablice za 3* i 5*** — danas postoji samo 4*.
- [drugi val] **JSON Schema kataloga** — danas samo provjera u kodu.

## Rupe u specu DEMO.md

Presuđene pretpostavkama u [ADR-013](DECISIONS/ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md) (profil C na 50, sezonska marža, pravila zaokruživanja P1–P32, nedefinirano manje); pretpostavke čekaju Leonov pregled (vidi „Otvoreno za Leona").

## CI i alati

- [alati] **Sokratis `docs` mjeri staro pravilo (2026-10-13)** — 0/100, 124 nalaza: 122 „not-indexed" (traži svaki dio u
  `docs/README.md`; naše pravilo 2 dopušta podindeks `X.md`), 1 „multiple-active-plans" (taskove u mapi plana broji kao
  planove), 1 „docs-lag" (točno). Uz to mjeri tekuće stablo i kad mu se zada druga mapa. Prijedlog: Sokratis za
  HotelixSim pokreće `corepack pnpm check:docs` repoa (jedan izvor pravila) — posao u repou Sokratisa, Leon odlučuje.

- [ci] **Prelazak CI-ja na Ubuntu 26 svjesno, ne usput** — 13.10. slika je pribijena na `ubuntu-24.04` (a `checkout`/`setup-node` na v5) jer `ubuntu-latest` od 19.10. postaje Ubuntu 26; prijeći kao zasebnu ciglu kad se provjeri Postgres servis, `sqlx-cli` i Rust 1.98.1 na novoj slici. (cigla K0-1, [ROADMAP/kvaliteta](../plan/ROADMAP/kvaliteta.md))
- [ci] **CI prevodi `sqlx-cli` svaki put bez cachea (~5–10 min)** — bitno kad CI dobije mutacijski posao. (Task 8 #1, L86)
- [ci] **`cargo-deny-action@v2` koristi vlastiti Rust, a repo `rust-toolchain` 1.98.1** — provjeriti na prvom CI-u. (Task 8 #3, L88)
- [alati] **Brana i alati dokumentacije — sitni nalazi (2026-10-10)** (završni pregled; danas ništa ne puca):
  1. skener poveznica ne zna ciljeve `<…>`, naslove `'…'`/`(…)`, escape `\[a](x)` ni reference-style `[x]: url` (danas 0 pojava; pravila 1 i 11 ih ne vide);
  2. ograde u listama/citatima (uvlaka ≥4) i inline kod preko više redaka nisu prepoznati;
  3. `razdvoji-plan` prepisuje izvornik prije provjere s diska (bolje: usporedba u memoriji prije pisanja); nema testa da izmjena tablice ne mijenja spoj;
  4. `adr.mjs razdvoji` pri ponovnom pokretanju prepisuje ADR datoteke bez upozorenja kad se razlikuju; `procitajAdr` gleda samo 5 redaka zaglavlja;
  5. brana: Područje se čita iz prvih 10 redaka; nema testa za skidanje `?upita`; sidra (`#granice-veličine`) se ne provjeravaju.
