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
2. Jednom: `cp .env.example .env` i `corepack pnpm install`. Claude ključ upiši u `.env` (`ANTHROPIC_API_KEY=…`); bez njega igra radi na rezervama.
3. API (Git Bash, prvi terminal): `set -a; source .env; set +a; cargo run -p server -- migrate && cargo run -p server -- serve`
   `serve` radi u prvom planu i drži terminal zauzetim dok ga ne zaustaviš (Ctrl+C). Pri startu učitava i provjerava
   `content/hotel` (ili mapu iz `HOTELIXSIM_CONTENT`) i pokreće radnika koji obračunava zatvorene mjesece; migracije se
   ne pokreću same — uvijek `migrate` prije `serve`.
4. Sučelje (drugi terminal): `corepack pnpm --filter @hotelixsim/web dev` → <http://localhost:5173>
5. Provjere prije commita: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `node scripts/check-core-deps.mjs`, `cargo test --workspace`, `corepack pnpm check:api`, `corepack pnpm lint`,
   `corepack pnpm typecheck`, `corepack pnpm test`, `corepack pnpm check:kod` (brana veličine koda, [ADR-039](docs/records/DECISIONS/ADR-039-brana-velicine-koda-upozorenje-600-granica.md)); za dokumentaciju `corepack pnpm check:docs` i `corepack pnpm test:docs`.
6. Dokumentacija ([pravila i granice](docs/README.md)): popis ADR-ova
   nakon novog ADR-a `corepack pnpm docs:adr`; razdvajanje plana `corepack pnpm docs:plan <plan.md>`; veličine svih dokumenata
   uz granice `node scripts/check-docs.mjs --popis`.
7. Grane: `corepack pnpm grane` pokazuje koliko je koja lokalna grana iza integracijske i je li `STARA` (nespojen rad stariji od 24 h).

### Igra u pregledniku

Baza (`bash scripts/baza.sh start`), API i sučelje kao u koracima 3 i 4 gore; zatim otvori <http://localhost:5173/demo>.

- **Izbor igre:** početni ekran (bez igre, ili `/nova`) nudi novi model s odjelima i računom dobiti — „Tjedna igra"
  (17 krugova, zadano) ili „Mjesečna igra" (4 kruga); stari model (v1 demo lipanj–srpanj) je pod sklopivim „Za
  profesora: postavke nove igre" → „Stari model (usporedba)" ([ADR-034](docs/records/DECISIONS/ADR-034-zadani-model-igre-je-v2-tjedna.md)).
  Kroz API: `POST /v1/demo` s tijelom `{ "model": "v2" }` (tjedno) ili `{ "model": "v2", "round_kind": "month" }`;
  bez tijela ostaje v1 demo.
- **Bez AI-ja** (rezerve, bez troška): prije `cargo run -p server -- serve` upiši `unset ANTHROPIC_API_KEY`.
- **Na mobitelu u istoj mreži:** `corepack pnpm --filter @hotelixsim/web dev --host`. Pazi: tada svatko u istoj mreži vidi igru.
- **Modeli:** presuda Sonnet 5.5, lik Haiku 5.5, mentor Sonnet 5.5 ([ADR-024](docs/records/DECISIONS/ADR-024-modeli-presuda-na-sonnetu-5-5.md));
  mijenjaju se varijablama `HOTELIXSIM_AI_MODEL_PRESUDA`, `_LIK` i `_MENTOR` (vidi `.env.example`).

Gradnja koristi spremljene podatke iz `.sqlx/` (`SQLX_OFFLINE=true` u `.env`), pa ne treba živu bazu. Nakon promjene SQL upita (baza mora raditi): `cargo sqlx prepare --workspace` i commitaj `.sqlx/`; alat se instalira jednom: `cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres --locked`.
Nakon promjene API-ja: `corepack pnpm gen:api` i commitaj `openapi.json` i `schema.d.ts`.
Mjerilo AI-ja (DEMO §11 t. 7): u `content/hotel/mjerilo/*.json` upiši `leon_quality`, pa `set -a; source .env; set +a; cargo run -p ai --bin mjerilo -- S1 S2`.
