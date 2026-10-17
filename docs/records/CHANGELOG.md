# Changelog

Isporuke (ono što radi), najnovija na vrhu. Format: [Keep a Changelog](https://keepachangelog.com/hr/1.1.0/).

## [Neobjavljeno] — Popravak temelja, faza A ([ADR-043](DECISIONS/ADR-043-popravak-temelja-ne-gradnja-ispocetka.md))

### Dodano
- `balance --check` ruši samo brane modela; šest mjera robota ispisuje se u odjeljku „Mjere robota" (A1).
- Situacije (predlošci, razgovor, prepuštanje voditelju) spojene u `model-v2/sloj1` (L1-11, A1).
- Načela igre kao provjere sadržaja pri startu servera, s popisom duga koji smije samo padati (A2).
- Zapečaćeni otisak sezone v2 vezan uz `CORE_VERSION` 2.2.0 (A3); stare v2 igre daju 409 i traže novu igru.
- Alat `corepack pnpm grane` (koliko je koja grana iza integracijske, `STARA` nakon 24 h) (A7).
- `LICENSE` (sva prava pridržana) i README s cijelom idejom igre ([ADR-044](DECISIONS/ADR-044-licenca-sva-prava-pridrzana.md)).
- Faza A gotova: igra i balans daju istu sezonu (A4), `lik.v3` i provjera mentora kruga (A5), scenariji „što student vidi" na razini API-ja (A6); `model-v2/sloj1` 024160a na GitHubu, CI zelen.
- Faza B (lokalno u `model-v2/sloj1` 39e5f25, još bez pusha): broj situacija je postavka igre, mjesečno 4/5/6 u jednom bloku (B2a, `CORE_VERSION` 2.3.0); `balance --situations L,M,H` (B2b); blok „Situacije na zadanom planu" i brane B3 s ravnalom (B3a, B3b); hitni popravak u situaciji diže stanje svog odjela, privremeno +1 bod (B1a, 2.4.0, [ADR-050](DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md)). Stare probne v2 igre daju 409.
- Faza B nastavak (lokalno u `model-v2/sloj1` 8d134a4, još bez pusha): kvar se može vezati uz stanje **drugog odjela** (`condition_of`, B1b; sadržaj ga još ne koristi); sve oznake odgovora u prvih 40 predložaka i sidrima S1–S4 prepisane tako da su unutar 1,5× najkraće (B4a, B4b). Student osjeti: odgovori su otprilike iste duljine, pa se najbolji više ne prepoznaje po duljini, zarezima ni sastojku „za poslije". Klima i praonica kvare se samo kad je Smještaj loš (ispod 65), kao bojler ([ADR-052](DECISIONS/ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md)). Balans bajt-isti osim malog pomaka težine B3 zbog uvjeta (tjedno A 42,8 → 41,9 %).
- Faza B, B4c (lokalno u `model-v2/sloj1` f7ac7cc, bez pusha): svih 55 oznaka odgovora unutar 1,5× najkraće; bravu, curenje vode i dizalo kvare se samo kad je Smještaj ispod 80 ([ADR-054](DECISIONS/ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md)); inspekcija je vanjski događaj. Težina tjedno A 41,9 → 41,6 %.
- Banka ljudi K1a (lokalno, adf5a4d): format zapisa radnika i gostiju, pravila provjere i okvir raspodjela u `crates/content` (modul `ljudi`); igra i balans netaknuti.
- Banka ljudi K1b (lokalno): 20 ručnih primjera (10 radnika, 10 gostiju) i sučelje za pregled `pregled-20.html`; serije otvorene do Leonova pregleda.
- Kartica PRAVILO (plan, bez koda): klik na ponuđeni odgovor najviše 4, prepisani odgovor vrijedi kao klik; `CORE_VERSION` 2.5.0 tek s PRAVILO-a ([ADR-055](DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md)).
- PRAVILO-a (`model-v2/sloj1` 9a275d8, na GitHubu, CI zelen; `CORE_VERSION` 2.5.0): vlastiti odgovor vrijedi više od klika — u igri uz pravilo „vlastiti odgovor" klik na ponuđeni odgovor daje najviše 4, prepisani ponuđeni odgovor vrijedi kao klik, a zalijepljena dva ili više ponuđenih odgovora daju neutralnu 3 bez učinka i bez poziva suca (parovi susjednih riječi, 70 %; B13). Mentor zna za ograničeni, prepisani i zalijepljeni odgovor. Igra modela v1 ostaje na klasičnom pravilu; stare probne v2 igre daju 409 ([ADR-047](DECISIONS/ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md), [ADR-057](DECISIONS/ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md) t. 2).
- IZGLED (a+b) (lokalno 872ff46, bez pusha): lik u situaciji i razgovoru dobiva rod, dobnu skupinu i podrijetlo (API `character.look`, `character_look`), a web crta lice iz tih podataka (`lookFor(ime, hint)`): gošća nema bradu, gost nema naušnice, stariji lik ima sijedu kosu. Lik bez podataka (sidra S1–S4, ljestvica) izgleda isto kao prije. Razgovorna tijela API-ja preseljena iz `schema.rs` u `schema_situations.rs` (dug 1770 → 1717 redaka). Balans bajt-isti.
- PRAVILO-b (lokalno ee6de80, bez pusha): balans v2 ima šesti put „klik najbolji" (najviše 4 gdje situacija ima lika), lanac B3 s četiri koraka (najbolji > klik > srednji > prepušteno > šutnja) i ispis „težina klika" (tjedno A/B/C 38,9 / 45,1 / 37,7 %, mjesečno 24,0 / 32,2 / 19,0 %; samo ispis, nije brana). Stari brojevi balansa isti; igra i `CORE_VERSION` nedirnuti.
- Dokumenti 17.10. (lokalno, bez pusha): Leonove odluke iz 43 odgovora ([ADR-056](DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)–[058](DECISIONS/ADR-058-pretpostavke-sesije-7-izgled-vanjski-platforma-k2.md)), kartica [L1-13](../plan/2026-10-14-popravak-temelja/task-14-l1-13-kalibracija.md), nacrt rizika „likovi nalik poznatim osobama" ([BANKA_LJUDI §7](../architecture/BANKA_LJUDI/07-poznate-osobe-rizik.md)) i dubinska provjera arhitekture ([E4](../architecture/PROVJERA_ARHITEKTURE_2026-10-17.md)).

### Popravljeno
- `rustls` 0.23.46 (RUSTSEC-2026-0346 i -0347) i na `main`.

## [Neobjavljeno] — Sloj 1 puta B: odjeli, račun dobiti, tjedni krug, kraj sezone

Plan: [2026-10-10-v2-sloj1.md](../plan/2026-10-10-v2-sloj1.md); pretpostavke [ADR-029](DECISIONS/ADR-029-sloj-1-pretpostavke-plana.md)
i ADR-031…036. Dokaz kraj-do-kraja: `crates/api/tests/e2e_v2.rs` (17 tjednih krugova kroz API).

### Dodano

- Obnova v2 (spojeno u sloj1, L1-10): iznos po odjelu i za zgradu u planu, plaćanje u ratama, amortizacija i EBIT u računu
  dobiti, indeks F iz GOP-a nakon ulaganja, cijena parkinga, najmovi; ekran „Kraj sezone" (pobjednik, poredak, krivulja napretka,
  kako je igrao svaki hotel, sezonski mentor `mentor-sezona.v2`); `CORE_VERSION` 2.1.0.
- Jezgra v2 `sim-hotel-v2` (`CORE_VERSION` 2.0.0): tjedni (16 × 7 dana + zadnjih 10 dana, 17 krugova) i mjesečni
  krugovi, razmjerni iznosi do centa, ekvivalentna v1 za mjesečnu igru; v1 zamrznut i pokriven zlatnim testom.
- Odjeli po USALI-ju (operativni i neraspoređeni Uprava / Prodaja i marketing / Održavanje / Energija), sezonski
  radnici i razina cijene po odjelu, računi odjela u obračunu; odlasci po odjelu (isti zbroj, razmjerno kroz sezonu);
  norma osoblja po klasi hotela i broju soba u provjeri sadržaja.
- Račun dobiti po USALI-ju (GOP, fiksne naknade, EBITDA) s budžetom i odstupanjem po retku.
- API: `/v1/hotels/{id}/departments`, `/departments/{code}`, `/pl?round=&scope=round|season`,
  `GET /v1/games/{id}/season` (poredak, krivulja, financije i odluke robota nakon zadnjeg kruga);
  `POST /v1/demo` s `{ "model": "v2", "round_kind": "week"|"month" }`; cijene odluka u granicama plana
  (`PlanLimitsBody.costs`, `DeptLimitsBody.maintenance`, `seasonal_worker_cents`, `payroll_round_cents`); novi kodovi
  `unknown_department`, `no_resolved_round`, `department_plan_invalid`, `game_outdated`, `model_v1`,
  `season_not_over`; `openapi.json` i `packages/api-client` regenerirani.
- Web: tjedni i mjesečni krugovi; ekrani Odjeli (popis s ikonom i trakom ocjene, detalj s ocjenom, slabim i jakim
  stranama i odlukama odjela) i Račun dobiti (krug i sezona do sada); cijena uz svaku odluku plana i blagajna nakon
  obnove; nespremljene izmjene plana i odjela prežive navigaciju.
- Sezonski mentor: posao `season_debrief`, događaj `SeasonDebriefRecorded`, uputa `prompts/mentor-sezona.v1.md`,
  rezerva iz brojki bez AI-ja.
- Balans v2: `balance --model v2 --round week|month`, tjedno ≈ mjesečno, Brižni drži barem 80 % sezonskog plana;
  kalibracija sadržaja; CI vrti `--seeds 1000 --check` za v1, v2 tjedno i v2 mjesečno.

### Promijenjeno

- Početni ekran nudi „Tjedna igra" (zadano) ili „Mjesečna igra" u novom modelu; stari model je pod „Za profesora"
  ([ADR-034](DECISIONS/ADR-034-zadani-model-igre-je-v2-tjedna.md)); „Nova igra" na kraju sezone vodi na isti izbor.
- Mentor (`mentor.v3`) ne pripisuje studentu odluke koje nije mijenjao i u tjednoj igri govori o tjednu; objašnjenje,
  mentor i sažetak tjedna navode odlaske po odjelima.
- Robot Brižni u tjednoj igri planira sezonce unaprijed (zapošljava prije vrhunca, otpušta prije pada).
- `PUT /v1/hotels/{id}/plans/{round}` vraća plan kakav je spremljen; naslovi grešaka `seasonal_step_too_big` i
  `no_resolved_round` govore „krug"; v1 poruka „Sezonske radnike možeš mijenjati najviše za 10 mjesečno."
- Novac i postotak imaju nelomljivi razmak prije jedinice; novi token boje `warn` (jantar, „srednje").

### Gdje se vidi

Lokalno, grana `model-v2/sloj1` (nije u `main`, nije na GitHubu do Leonova OK); igra kao u korijenskom
[README](../../README.md) → „Igra u pregledniku".

## [Neobjavljeno] — Uvod logotipa (na `main`)

- Uvodna animacija logotipa pri učitavanju (H → zgrada → vrata se otvore, toplo svjetlo), jednom po kartici; `?uvod`
  je uvijek pokaže; smanjeni pokreti = bez animacije. Kod: `apps/web/index.html` (crtež i animacija),
  `apps/web/src/splash.ts` (izlaz, test `splash.test.ts`).

## [0.5.0] — 2026-10-10 — Ekrani igrača, računalo, hotelski pokazatelji (DEMO korak 5 + put B sloj 0)

Verzija bez git taga. Planovi: [archive/2026-10-10-ekrani.md](../archive/2026-10-10-ekrani.md); sloj 0 iz
[MODEL_V2.md](../architecture/MODEL_V2.md).

### Dodano

- Igra u pregledniku: Moj hotel, situacije (razgovor s likom, slobodan odgovor s presudom, ponuđeni odgovori), plan mjeseca,
  Zatvori mjesec, Mjesec završen (indeksi, mentor, „Zašto se indeksi pomaknuli", ljestvica), Tijek, detalj svakog pokazatelja.
- Raspored za računalo: bočni izbornik od 768 px, dva stupca od 1024 px; mobitel nepromijenjen.
- Hotelski pokazatelji (`GET /v1/hotels/{id}/kpi`, ekran „Pokazatelji"): popunjenost, ADR, RevPAR, TRevPAR, GOP, GOP marža,
  GOPPAR, trošak i udio rada, izravni trošak po noći, točka pokrića — svaki s rečenicom i formulom.

### Promijenjeno

- Presuda (`presuda.v3`) ne spominje redne brojeve ponuđenih odgovora ni kriterija; jezgra takvu presudu odbija.
- Popunjenost se dijeli sa svim sobama × dani (USALI), izgubljene noći zasebno.

## [0.4.0] — 2026-10-10 — AI sloj (DEMO korak 4)

Verzija bez git taga. Plan: [archive/2026-10-09-ai-sloj.md](../archive/2026-10-09-ai-sloj.md).

### Dodano

- Razgovor s likom u S1 (gošća) i S2 (šef kuhinje), do 6 poruka; lik glumi Claude, a bez AI-ja odgovara napisana rečenica.
- Slobodan odgovor u S1 i S2: AI za svaki od pet kriterija rubrike kaže ispunjen/ne uz razlog; kvalitetu (1–5) računa
  kod, brojke dolaze iz najbližeg ponuđenog odgovora. Presuda bez valjanog oblika se ponavlja, zadnji pokušaj daje
  neutralnu presudu „na pregled profesoru".
- „Zatvori mjesec" čeka presude do 120 s, zatim neutralna; kasna presuda se zapisuje (`VerdictLate`) i ne mijenja obračun.
- Mentor nakon obračuna (Claude, ili tekst iz brojki bez AI-ja); nove stavke feeda; red poslova s odgodom.
- Crate `ai` (Claude Messages API, strukturirani izlaz, upute u `prompts/`), alat `mjerilo` (DEMO §11 t. 7).
- Rute `…/situations/{code}/messages`, `/conversation`, `/response`; ključ `ANTHROPIC_API_KEY` u `.env`.

### Gdje se vidi

Lokalno: `cargo test --workspace`, `cargo test -p ai --test live -- --ignored` (pravi ključ), `cargo run -p server`.
GitHub: `main` na `156e6df`, CI zelen. Ekrana još nema (korak 5).

## [0.3.0] — 2026-10-05 — Dnevnik, runde i red poslova (DEMO korak 3)

Verzija bez git taga. Plan: [archive/2026-10-05-dnevnik-runde.md](../archive/2026-10-05-dnevnik-runde.md).

### Dodano

- Cijela igra preko API-ja na pravom Postgresu: `POST /v1/demo` stvara gosta (kolačić `httpOnly`, bez lozinke) i igru s
  igračem i tri robota (Štediša, Brižni, Lovac na popunjenost); profili: igrač iz sjemena, roboti A/B/C.
- Lipanj: plan i odgovori na situacije; „Zatvori mjesec" vraća 202 (drugi klik 200), radnik u serveru obračuna za ~1 s;
  zatim srpanj i zatvaranje; ljestvica i feed hotela (vremenska crta kao u BitLifeu, svaka promjena nosi uzrok).
- Dnevnik događaja je nepromjenjiv (okidač u bazi, SQLSTATE 42501), jedan tok po igri, zaključavanje po igri; red
  poslova `jobs` (`SKIP LOCKED`, zakup, identitet preuzimanja `JobClaim`); radnik preživljava paniku.
- Novi crate `game` (događaji, projekcija, pravila runde, potezi robota); jezgra i dalje bez serdea.
- E2E test na Postgresu provjerava stvarne ishode; ručni prolaz curlom na pravom serveru.

### Gdje se vidi

Lokalno: `corepack pnpm check:api`, `cargo test --workspace` i `cargo run -p server` (uz `bash scripts/baza.sh start`);
CI na GitHubu zelen na `894d023`. Ekrana još nema (korak 5).

## [0.2.0] — 2026-10-04 — Jezgra igre i alat za balans

Verzija bez git taga. Plan: [archive/2026-10-04-jezgra.md](../archive/2026-10-04-jezgra.md).

### Dodano

- `engine::calc`: cjelobrojni račun s jednim zaokruživanjem (polovica od nule) i greškom umjesto prelijevanja.
- `sim-hotel`: katalog s provjerom, stanje hotela, plan mjeseca s granicama, obračun mjeseca u fazama (tržište → ljudi →
  novac → oprema i gosti) s objašnjenjem svake promjene, mehanike situacija S1–S4, pet indeksa centriranih na
  referentni hotel, relativna ocjena, četiri robota i sezona hotela.
- `content`: katalog, profili A/B/C i situacije u `content/hotel/*.json`, strogo provjereni pri učitavanju.
- `balance`: alat koji provjerava DEMO §11 (početak, ekonomija, pravednost, nema dominantne radnje, ponovljivost) i
  lekciju S3; korak u CI-ju.
- Brane determinizma: dopušteni popis ovisnosti jezgre, `forbid` clippy pravila, mutacijske probe u CI-ju.

### Gdje se vidi

Lokalno: `cargo run -p balance --release -- --seeds 1000 --check` (tablica na hrvatskom) i `cargo test --workspace`.
Ekrana za ovo još nema.

## [0.1.0] — 2026-10-04 — Kostur od kraja do kraja

Verzija bez git taga.

### Dodano

- Rust workspace: `engine` (kocka SplitMix64 sa zlatnim vektorima), `sim-hotel` (pet indeksa u stotinkama, profili A/B/C kao konstante), `app` (port `SystemStatus`), `store` (dnevnik u Postgresu, SQLx, offline `.sqlx/`), `api` (Axum + OpenAPI 3.1), `server` (`serve` | `migrate` | `openapi`).
- Lanac od API-ja do ekrana: `openapi.json` → generirani TypeScript klijent `@hotelixsim/api-client` → `packages/i18n` → jedan React ekran (`apps/web`) sa zdravljem sustava i indeksima profila.
- Brane: clippy (zabrana `HashMap`/`HashSet`/sata, float i neprovjerena aritmetika), `scripts/check-core-deps.mjs`, cargo-deny, CI (`.github/workflows/ci.yml`).
- Testovi: Rust 28, sučelje 10.
- Lokalna baza: zaseban PostgreSQL klaster na priključku 5433, `scripts/baza.sh`.

### Gdje se vidi

- Lokalno: [README](../../README.md), odjeljak „Lokalni razvoj".
- GitHub CI zelen (run 37167540215: `rust` 4m51s, `web` 1m44s).
- Plan izgradnje: [kostur](../archive/2026-10-04-kostur.md).
