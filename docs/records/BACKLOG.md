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
- [sloj1] **Faza C, objašnjenje tjednog kruga** ([ADR-045](DECISIONS/ADR-045-trosak-odmah-ugled-kasnije-tjedni-krug.md)): rečenica na ekranu kruga i u mentoru „trošak se vidi odmah, učinak na ugled stiže idućih tjedana".
- [sloj1] **Moral: vidljivost uzroka** (Leon u igri 15.10.: moral stalno pada, nije povezao plaće na Planu 90–115 % s moralom; cilj = 55 + 2 po % plaće iznad 100 − preopterećenje/2 + obuka + situacije − otpuštanje − grupe). Kandidat za fazu C — **pitati Leona** na kraju igre.
- [sloj1] **Leonove želje 15.10.:** pojedinačno vidljivi svi ljudi u odjelu ([BANKA_LJUDI](../architecture/BANKA_LJUDI.md), nakon popravka); više načina zarade (dodatni prihodi) — idea, „o tom potom". Izmiješati ponuđene odgovore je već L1-13b (A6 (a) tada bira po izvornom broju najboljeg).
- [sloj1] **Mentor kruga bez provjere 180 riječi / 4 brojke / roda** (provjerava se samo četiri dijela, kao sezonski); strogi podnaslovi bez testa za dvotočku/podebljano; u dnevniku pratiti `source: fallback`. (A5)
- [sloj1] **Faza C:** nalaz 9 (duljina odgovora lika, `max_tokens`); nalaz 10 (rezerva vraćena AI-ju kao replika lika, treće lice na 6. poruci; ~2 h, C3); ulaz suca (nalazi 3, 4) ide s `presuda.v4`. (A5)
- [sloj1] **Završni pregled faze A, sitno:** `balance/src/main.rs:60` čita `read_v2` bez načela → `read_v2_checked` (B4/L1-13, m1); završni redak „Sve provjere prolaze." ispod mjera robota zbunjuje → „Sve brane modela prolaze." (B3, m2); brane J-3 broje izvlačenja iz sezona robota (B1/B4, m3); `names_unique` preskače duplikat unutar popisa (B4, m4); `DeptPlaceholder` u `character.*` bez testa (B4); test 4 A3 tvrdo 650 bp (L1-13); `GAME_ROBOTS` ručna kopija privatnog `ROBOTS` i `s.drawn` neprovjeren kao neprazan (B2, I3).
- [sloj1] **Mentor-poll vitest testovi** (`SeasonScreen.test.tsx` „ne javi se…") nestabilni pod opterećenjem (pravi sat) → povremeni lažni pad `pnpm test`; prebaciti na lažni sat. (B2a/B2b; izolirano 30/30)
- [sloj1] **C5, uvjet prije dizanja bonusa popravka:** ekran odjela (`apps/web` `helpers.ts` `shownChanges`) pripisuje popravak iz situacije „održavanju"; uz +1 nevidljivo, uz +10 kriva pouka. ([ADR-050](DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md); B1a)
- [sloj1] **Mjere robota nakon B1a (za L1-13):** tjedno §11.3 pravednost 560 > 499, J-3 Brižni 43,5 %, „tjedno ≈ mjesečno" B brižni; uz `--situations` ispod 4/5/6 mjesečno J-3 raznolikost P-POHVALA 25 % > 15 % (B2b). Ne ruše `--check`.
- [sloj1] **Odgođeni minori B2a–B1a** (detalj: [ledger](../archive/2026-10-15-popravak-temelja-faza-b-ledger.md), redci „minor (deferred)"): B2a: test 5 pečata djelomično tautologija, `draw.rs` 556 redaka, `model.rs:1423` opis „zadanim"→„izričitim"; B2b: `--situations` uz v1 tiho zanemaren, `run_v2` n-torka, bez rubnih testova; B3a: put „bez situacija" se vrti 1000× bez potrebe, zaglavlje „kao dio raspona plana" ≠ |Δ|/(|Δ|+R), `has_drawn` suvišan, `pct` kružni uvoz v2↔v2_answers; B3b: `unwrap_or(("?","?"))` i oprost tiho ignoriraju dug s indeksom ≥ 3, doc `ORDER_DEBT` > 120 znakova; B1a: zastario komentar `ORDER_DEBT_FLOOR` i mrtav mehanizam duga (briše L1-13), pečat ne pokriva bonus > 0, plan čovjeka u pečatu uzak.
- [sloj1] **P1–P8 🟨 iz plana popravka** pokazati Leonu najkasnije prije B4. (Pečat sa slobodnim odgovorom i planom čovjeka dodan u B1a.)
- [sloj1] **Razmak prije „%" u tekstovima iz Rusta** (objašnjenje kruga, `crates/game/src/mentor.rs` „{pct} % potražnje") je običan, a web ga ima nelomljivog. Ne dirati dok v1 mora ostati bajt-ist (ili samo u v2 grani teksta). (OD-7)
- [sloj1] **Mentor i sezonski ugovori:** mentor krajem kolovoza upozori „rujan pada, planiraj kraj sezonskih ugovora"; u v2 redak troškova „fiksni troškovi" (stalni dio zajedničkih službi) treba drugo ime, blizu Računa dobiti — tekst je zajednički s v1, pa objašnjenje mora nositi model (ili `LineDto` drugi uzrok); redak blagajne već se zove „fiksni troškovi zgrade (najam, osiguranje, porez)". (ZP)
- [sloj1] **Tijek nema stavku za prepuštenu situaciju** (odgovorena „izabrano…", neodgovorena „bez odgovora (1/5)"; prepuštena se vidi samo kod mentora) → „prepušteno voditelju (ocjena 2/5)" u `feed.rs`. (W-1 UI čuvar)
- [sloj1] **Uputa AI mentora ne zna ključ `prepustena_voditelju`** — uz 2/5 može prepuštenu nazvati loše riješenom; pri sljedećoj verziji `mentor.*`. (A-1 pregled)
- [sloj1] **Situacije nakon 409:** poruka greške nestane čim se ekran osvježi; dok je potvrda prepuštanja otvorena, slobodni odgovor i razgovor s likom ostaju aktivni; 409 pri izboru ponuđenog odgovora ne osvježava situacije. (W-1 pregledi)

## Za plan sloja 2 — [sloj2]

Leonove želje 12.10.; brojke idu kroz Leonovu anketu pri planu sloja 2. Ideje nakon profesorova pregleda: [IDEJE.md](../product/IDEJE.md).

- [sloj2] **Odnosi među zaposlenima kao u BitLifeu** (Leon: „treba biti"): prijateljstva, sukobi, obitelj/kolege — odnos dvoje konkretnih ljudi (prijateljice odlaze zajedno, kuhar i konobar u sukobu). MODEL_V2 §3.4 danas ima samo osobine koje djeluju na cijeli odjel. U plan sloja 2 (2a ili zaseban dio).
- [sloj2] **Obećanja iz odgovora koja se prate** (S1 kuhar): student obeća konkretne radnje (slobodan dan, jelovnik, zamjenik), a igra ih danas ne provodi niti provjerava. Prijedlog za 2c i plan situacija: presuda izvuče obećane radnje u strukturirani popis, igra ih pretvori u zadatke („zaposli zamjenika u kuhinji do t04") i prati — ispunjeno → motivacija lika +, neispunjeno → lik se vrati („obećali ste mi zamjenika"). Uz burzu rada (2a): zamjenik je stvarna osoba s burze.
- [sloj2] **Plaća po radniku i pravednost plaća:** plaća se bira za svakog radnika (MODEL_V2 §3.3 već ima plaću po osobi). Novo pravilo: ako netko ima neopravdano višu plaću od kolega iste uloge (bez veće vještine ili staža), motivacija ostalih u odjelu pada — član „nepravednost u odjelu" u formuli motivacije.
- [sloj2] **Dugotrajna i kratkotrajna oprema, realna amortizacija** (Leon: *„parking … jednom ga obnoviš i traje godinama"*): dugotrajna imovina (zgrada, parking, instalacije — sporo trošenje, skupa obnova) i kratkotrajna (posteljina, namještaj, sitna oprema); amortizacija ispod GOP-a (USALI). Ide u obnovu v2 (grana `model-v2/sloj1-obnova`, još nije spojena).
- [sloj2] **Sezonski ugovor do datuma:** odlazak na kraju ugovora ne troši tjedni korak ±3. (ZP)

## Otvoreno za Leona

Pitanja o domeni ne blokiraju kod: brojke su podaci u katalogu, gradi se s pretpostavkom i mijenja kad odgovori. Leon je 17.10. odgovorio na
43 pitanja paketa ([ADR-056](DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)); stare stavke v1 (brojke prve igre, A12) i
riješene stavke (F2) su zatvorene. Pretpostavke koje još čekaju: [ADR-058](DECISIONS/ADR-058-pretpostavke-sesije-7-izgled-vanjski-platforma-k2.md).

- [leon] **Slijepi pregled tekstova B4a, B4b i B4c** (B1 „ok", još nije napravljen) — `.superpowers/sdd/2026-10-14-popravak-temelja/b4a-pregled-tekstova.md`, `b4b-pregled-tekstova.md`, `b4c-pregled-tekstova.md` (uz „Napomene voditelja"); blokira L1-15, ne spajanje. Nakon njega B4-ispravci i napomene a–e iz B2 ([ADR-055](DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md) t. 6).
- [leon] **Ocjene za mjerilo AI-ja** (B4 „ok") — upisati `leon_quality` (1–5) za 40 nacrta odgovora, pa `cargo run -p ai --bin mjerilo -- S1 S2` (prag 80 % unutar ±1); prije C3. Odgovor: `content/hotel/mjerilo/S1.json`, `S2.json`, DEMO §11 t. 7.
- [leon] **Rubrike S1 i S2** (B5) — Leon pročita kriterije (~10 min); nacrt opisa S3 i rubrika S3 i S4 piše agent, Leon potvrđuje (B3 c).
- [leon] **Poznate osobe s izmijenjenim imenom (C3)** — Leonova ideja „kao BitLife"; ništa se ne gradi dok ne pročita [nacrt rizika](../architecture/BANKA_LJUDI/07-poznate-osobe-rizik.md); deset nesigurnih imena banke čeka isti nacrt.
- [leon] **Pet pitanja plana platforme** (§7; E3 je odgovorio dio o profesorovu računu) — prijava za P3, Railway ili Cloudflare za P3 (Leon 17.10. „nije sigurno"; Supabase uz strah od latencije), ritam kolosijeka, redoslijed IGRE od studenog, vlastite situacije profesora. Odgovor: [plan platforme](../plan/2026-11-platforma.md).
- [leon] **Inkubator** (E5 „ok") — prijaviti HotelixSim (ne Sokrat Study) do **16.11.**
- [leon] **Pitanja za profesore** (E2) — poslano, odgovora još nema; sastanak sljedeći tjedan. Odgovor: [PITANJA_ZA_PROFESORE](../product/PITANJA_ZA_PROFESORE.md).
- [kvaliteta] **Duboka provjera arhitekture (E4)** — Leon: „ti provjeri duboko i reci mi je li ok"; dubinski pregled (najjači model) → nalaz Leonu → tek tada ✅ [ADR-003](DECISIONS/ADR-003-arhitektura-rust-jezgra-backend-typescript-sucelje.md) i usklađivanje ARHITEKTURA/ROADMAP.
- [prije L1-13] **Istražiti prave hotele i prave plaće** — broj soba i ljudi po odjelu u stvarnim hotelima raznih veličina (A10, norma [ADR-032](DECISIONS/ADR-032-sloj-1-odjeli-u-obracunu-i.md) t. 9) i plaće po ulozi u hrvatskom hotelijerstvu (A12: 2.200 € ima rijetko tko osim direktora i nekih voditelja).
- [L1-13] **Tablica značaja situacija** (B2) — tri razine s boljim imenima (danas `major`/`minor`, student ih ne vidi); značajnije situacije i pametnije odluke nose više bodova.
- [L1-13] **Slabiji odgovor s troškom** (A4 „ok") — jednom slabijem odgovoru dati trošak (npr. pretjeranu naknadu, kao P-KLIMA[2]), da „biraj jedini koji gostu nešto daje" ne pogađa najbolji. Izvor: `b4b-pregled-tekstova.md`.
- [L1-14] **Priče hotela A i B i obnova od 150.000 €** (A12) — Leon provjerava pri igranju sezone („prepoznajem li ovaj hotel?").
- [faza C+] **Sučelje soba** (A1) — svaka soba vidljiva i obnovljiva zasebno; nova mehanika i ekran, 3–5 cigala, ne u L1-13 ([ADR-050](DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md) Izmjena 17.10.).
- [faza C+] **Mjere potrošnje hotela** (A13) — energija i voda kao pokazatelj u igri (nova mehanika).
- [LJUDI] **Smještaj radnika kao zasebna mehanika** (A11) — student smještaj kupuje, iznajmljuje ili koristi svoj hotel; ekran, trošak i kapacitet; tada se smještaj izdvaja iz 2.500 € po radniku Smještaja.
- [LJUDI] **Treniranje i voditelji** (A9, C1, D1) — trening pojedinačno ili skupine u odjelu, trener iz redova radnika; podvrste voditelji, studenti i radnici, svi trenirljivi do razine voditelja; vidljiva crta snage i slabosti svakog radnika i voditelja; puno različitih voditelja.
- [igra] **Student imenuje svoj hotel prije igre** (C4) — slobodno ime; najmanja brana vulgarnosti 🟨 (ADR-058 t. 18).
- [igra] **Banka hotela** (Leon 17.10.) — profesor dodjeljuje studentima gotove, pregledane hotele raznih veličina; mjesto u rasporedu 🟨 (ADR-058 t. 21), vidi [ROADMAP](../plan/ROADMAP.md).
- [platforma] **Profesorovo sučelje** (E6) — strukturiran izbornik AI situacija po odjelu hotela, vlastite situacije, uvid u studentove razgovore, ocjene, AI mentor uključen ili isključen, profesor otvara tjedan ili mjesec; temelji kroz više sesija.
- [platforma] **ARHITEKTURA §12 uskladiti sa sesijama** (E1 „ok": sesije, ne JWT; L10) — ispravak teksta. Vidi [PLATFORMA dio 08](../architecture/PLATFORMA/08-faze-i-pitanja.md).
- [prije pilota] **Izmjeriti AI trošak po igri** (A13) — Leon odigra lokalno jednu tjednu v2 sezonu, pa SQL iz [plana platforme](../plan/2026-11-platforma.md) §5 i cijene iz ADR-024; danas je samo procjena 10–20 $ po razredu.
- [web] **Bočni izbornik na računalu** (F1: voditelj popravi bez pitanja): razmak od vrha `md:top-28` tvrdo zadan ispod zaglavlja — kod velikog teksta ili dugog čipa može se preklopiti; staviti izbornik i zaglavlje u isti stupac. Odgovor: `apps/web/src/ui/Layout.tsx`.
- [leon] **Vizija za razvoj cijele aplikacije (Leon 10.10.)** — što bira profesor odlučeno 17.10. (E6, gore); ostaje:
  - novi likovi: Ivan Gudeljević (zaštitar koji se često opija), Dominik Smurjević („prati sve što je prljavo"), Nikola Magdić, Dr. Tin Šljivović, Leona Krešo — uloge Nikole, Tina i Leone još nisu zadane; pitanje pristanka za prava imena (veže se uz C3);
  - studenti ulaze u igru kodom, kao u Kahootu;
  - trošak tokena smanjiti „na drugačiji način" — mjerilo (40 odgovora) za usporedbu modela presude;
  - uređivač lika ([ADR-021](DECISIONS/ADR-021-koncept-ekrana-v2-i-odluke-o.md)).
- [banka ljudi] **Pečat serija 000 i 001** (C1: „super su", zapečatiti svih 120) — pečat radi glavna sesija ili sesija banke; rod po ulozi privremeno, Eurostat izmjeriti prije sljedeće serije (C2).
- [banka ljudi] **S6 ćelija ≥ 40 do M1 vidi samo sobare** (I1 pregleda K2) — neizravna standardizacija za male uloge u K3.
- [sloj1] **N3: „središnji" odgovor pogađa najbolji** — pravilo „odgovor s najviše zajedničkih riječi" pogađa najbolji u 8 od 19 jedinica B4a (plus 5 izjednačenih); nakon B4c mjereno: B4c 9/16 → 5/17, svih 59 jedinica 25/56 → 21/57; unija pravila 44 % prema ~27 % slučajnih (M4 pregleda B4c) — odlučiti u L1-13b (miješanje odgovora).
- [banka ljudi] **Odgođeni minori pregleda K1a/K1b za K2:** nema pravila "Azija hrvatski 0–1" u kodu (pravilo postoji samo u nacrtu); test S6 ne razlikuje "slijep unutar uloge" od "slijep skupno" (čuvar K2); P1 staje na prvoj nedostajućoj datoteci; ostali minori u `k1a-review.md` i `k1b-review.md` (M2, M5 sadržaj; M4, M6–M8).
- [sloj1] **Odgođeni minori pregleda B4c:** N3 unija (gore); nadskup "granični ×3" (P-POSEBNA-PREHRANA kruh, P-LEZALJKE, P-SPA-TERMIN razlika samo isprika); M1 prag 80 nije u kartici B4 (stoji u ADR-054); M6 poruke commita bez dijakritika (povijest, ne mijenja se).
- [sloj1] **Sadržaj situacija** (B3 „ok": voditelj popravlja sam) — P-RANI-DOLAZAK ima neusklađen broj (opis „jedna gošća", oznake „Stigli su"); rezerva lika „Gošća je previše ljuta da bi nastavila razgovor" → „Gošća je prekinula razgovor."; `znacenje` za „Zaposleni" je predugo i tehničko; S3 nema opisa, a S3 i S4 rubrike (nacrt piše agent). Slabiji odgovori P-PRISTUPACNOST[2], P-KRIZA[2], P-BOJLER[2] nose radnje iz rubrike bez kriterija (N1 B4a) — u B4-ispravcima nakon slijepog pregleda.
- [sloj1] **Minor odgođeni iz B4:** `wears` (B1b, `templates_principles`) vraća true kad nema podataka; popis riječi testa T4b zaobilazi sinonim.

## Drugi val (poslije demoa)

Opseg koji [ADR-013](DECISIONS/ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md) t. 8 odgađa iza demoa; spec je u [DEMO.md](../product/DEMO.md).

- [drugi val] **Splet okolnosti i tragovi (§9)** — odluke ostavljaju tragove koji se kasnije vraćaju.
- [drugi val] **Rubrike i ponuđeni odgovori za S1b i S5–S8** — **zatvoreno 17.10.** (B5: v1 zamrznut, v2 ima svoje predloške); nacrt ostaje kao povijest: [SITUACIJE](../product/SITUACIJE_S1b_S5-S8.md).
- [drugi val] **Naplata otvorenih tragova na kraju sezone.**
- [drugi val] Limit 3 demo igre po IP-u na dan i brisanje gosta i igre nakon 7 dana (DEMO §1.4; Leon 05.10.).
- [drugi val] **Različit učinak S4 po stanju hotela.**
- [drugi val] **Tablice za 3* i 5*** — danas postoji samo 4*.
- [drugi val] **JSON Schema kataloga** — danas samo provjera u kodu.

## Rupe u specu DEMO.md

Presuđene pretpostavkama u [ADR-013](DECISIONS/ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md) (profil C na 50, sezonska marža, pravila zaokruživanja P1–P32, nedefinirano manje); Leon ih je potvrdio 17.10. (A13, F1; [ADR-056](DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)).

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
