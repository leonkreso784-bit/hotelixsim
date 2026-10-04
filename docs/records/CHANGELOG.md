# Changelog

Isporuke (ono što radi), najnovija na vrhu. Format: [Keep a Changelog](https://keepachangelog.com/hr/1.1.0/).

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
