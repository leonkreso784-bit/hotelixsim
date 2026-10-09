# Changelog

Isporuke (ono što radi), najnovija na vrhu. Format: [Keep a Changelog](https://keepachangelog.com/hr/1.1.0/).

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
