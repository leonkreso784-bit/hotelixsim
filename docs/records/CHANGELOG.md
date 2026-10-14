# Changelog

Isporuke (ono što radi), najnovija na vrhu. Format: [Keep a Changelog](https://keepachangelog.com/hr/1.1.0/).

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
