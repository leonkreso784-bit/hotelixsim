# HotelixSim

Simulacija vođenja hotela za učenje — student dobije hotel, rješava situacije uz AI, a ocjenjuje se koliko je
hotel unaprijedio u odnosu na početak.

- Vizija: [docs/product/VIZIJA.md](docs/product/VIZIJA.md)
- Plan: [docs/plan/ROADMAP.md](docs/plan/ROADMAP.md)
- Arhitektura: [docs/architecture/ARHITEKTURA.md](docs/architecture/ARHITEKTURA.md)
- Demo (specifikacija): [docs/product/DEMO.md](docs/product/DEMO.md)

Privatno · autor: Leon Kreso.

## Lokalni razvoj

Treba: Rust 1.98 (rustup sam povuče verziju iz `rust-toolchain.toml`), Node 24, PostgreSQL 16 (samo programi; baza
radi kao zaseban klaster na priključku 5433 i ne dira sistemski servis na 5432).

1. Jednom: izradi lokalnu bazu. U Git Bashu, iz korijena projekta (ništa ne treba zamjenjivati, `$USERNAME` je tvoje Windows ime):
   ```bash
   PGBIN="/c/Program Files/PostgreSQL/16/bin"
   D="C:/Users/$USERNAME/AppData/Local/hotelixsim"
   mkdir -p "$D"
   echo hotelixsim_lokalno > "$D/pwfile.txt"
   "$PGBIN/initdb.exe" -D "$D/pgdata" -U hotelixsim --pwfile="$D/pwfile.txt" -A scram-sha-256 -E UTF8 --locale=C
   bash scripts/baza.sh start
   PGPASSWORD=hotelixsim_lokalno "$PGBIN/createdb.exe" -h localhost -p 5433 -U hotelixsim hotelixsim
   rm "$D/pwfile.txt"
   ```
   Svaki dan (nakon ponovnog pokretanja računala): samo `bash scripts/baza.sh start`
   (`bash scripts/baza.sh status` pokazuje radi li; `bash scripts/baza.sh stop` je gasi).
2. Jednom: `cp .env.example .env` i `corepack pnpm install`.
3. API (Git Bash, prvi terminal): `set -a; source .env; set +a; cargo run -p server -- migrate && cargo run -p server -- serve`
   `serve` radi u prvom planu i drži terminal zauzetim dok ga ne zaustaviš (Ctrl+C). Pri startu učitava i provjerava
   `content/hotel` (ili mapu iz `HOTELIXSIM_CONTENT`) i pokreće radnika koji obračunava zatvorene mjesece; migracije se
   ne pokreću same — uvijek `migrate` prije `serve`.
4. Sučelje (drugi terminal): `corepack pnpm --filter @hotelixsim/web dev` → <http://localhost:5173>
5. Provjere prije commita: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `node scripts/check-core-deps.mjs`, `cargo test --workspace`, `corepack pnpm check:api`, `corepack pnpm lint`,
   `corepack pnpm typecheck`, `corepack pnpm test`.

Gradnja koristi spremljene podatke iz `.sqlx/` (`SQLX_OFFLINE=true` u `.env`), pa ne treba živu bazu. Nakon promjene SQL upita (baza mora raditi): `cargo sqlx prepare --workspace` i commitaj `.sqlx/`; alat se instalira jednom: `cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres --locked`.
Nakon promjene API-ja: `corepack pnpm gen:api` i commitaj `openapi.json` i `schema.d.ts`.
