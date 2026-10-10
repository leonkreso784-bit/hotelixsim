### Task 8: Od kraja do kraja lokalno, CI i README

**Files:**
- Create: `deny.toml`, `.github/workflows/ci.yml`
- Modify: `README.md` (odjeljak „Lokalni razvoj")

- [ ] **Step 1: Ručna provjera od kraja do kraja (Review Focus 2)**

Terminal 1 (Git Bash, iz korijena):

```bash
set -a; source .env; set +a
cargo run -q -p server -- migrate      # očekivano: "migracije primijenjene" (ili bez promjene ako su već primijenjene)
cargo run -q -p server -- serve        # u pozadini (run_in_background); očekivano: "HotelixSim API sluša na http://127.0.0.1:8080"
```

Run: `curl -s -w "\nHTTP=%{http_code}\n" http://127.0.0.1:8080/v1/health` → `{"database":"ok","events":0}`, `HTTP=200`
Run: `curl -s -w "\nHTTP=%{http_code}\n" http://127.0.0.1:8080/v1/demo/profiles/D/indices` → problem+json, `HTTP=404`

Terminal 2: `corepack pnpm --filter @hotelixsim/web dev` (u pozadini) → `http://localhost:5173`.
Run: `curl -s http://localhost:5173/v1/health` → isti JSON kroz Vite proxy.

Baza ugašena (Review Focus 2): pokreni server s krivom lozinkom u `DATABASE_URL` (npr. `…:kriva@localhost…`), pa
Run: `curl -s -w "\nHTTP=%{http_code}\n" http://127.0.0.1:8080/v1/health` → `{"database":"error","events":null}`, `HTTP=503`; proces i dalje radi.

Ugasi oba procesa (provjeri da su to tvoji procesi: port 8080 i 5173) prije idućeg koraka.

- [ ] **Step 2: `deny.toml`**

```toml
[graph]
all-features = true

[licenses]
allow = ["MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-3.0", "Zlib", "CDLA-Permissive-2.0", "MPL-2.0"]
confidence-threshold = 0.9
private = { ignore = true }

[bans]
multiple-versions = "warn"
deny = [
  { name = "openssl-sys", reason = "TLS samo preko rustlsa" },
]

[advisories]
yanked = "deny"
```

Run: `cargo install cargo-deny --locked; echo EXIT=$?` → `EXIT=0`
Run: `cargo deny check; echo EXIT=$?` → `EXIT=0`. Ako pada samo na licenci koja nije na popisu, dodaj je **uz komentar koji paket je traži**; ako pada na savjetu o ranjivosti, ne isključuj ga, nego to javi u izvještaju taska.

- [ ] **Step 3: `.github/workflows/ci.yml`**

```yaml
name: CI
on:
  push:
  pull_request:

jobs:
  rust:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: hotelixsim
          POSTGRES_PASSWORD: hotelixsim_lokalno
          POSTGRES_DB: hotelixsim
        ports: ["5432:5432"]
        options: >-
          --health-cmd "pg_isready -U hotelixsim" --health-interval 5s --health-timeout 5s --health-retries 10
    env:
      DATABASE_URL: postgres://hotelixsim:hotelixsim_lokalno@localhost:5432/hotelixsim
      SQLX_OFFLINE: "true"
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - uses: actions/setup-node@v4
        with: { node-version: 24 }
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: node scripts/check-core-deps.mjs
      - run: cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres --locked
      - run: cargo sqlx prepare --workspace --check
      - run: cargo test --workspace
      - uses: EmbarkStudios/cargo-deny-action@v2

  web:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - uses: actions/setup-node@v4
        with: { node-version: 24 }
      - run: corepack enable
      - run: pnpm install --frozen-lockfile
      - run: pnpm check:api
        env: { SQLX_OFFLINE: "true" }
      - run: pnpm lint
      - run: pnpm typecheck
      - run: pnpm test
```

Napomena: `cargo sqlx prepare --check` treba bazu s primijenjenim migracijama. Ako pada jer tablice nema, dodaj prije njega korak `- run: cargo sqlx migrate run --source crates/store/migrations`.

CI se **ne pokreće** dok Leon ne odobri push. Lokalno se isti koraci pokreću ručno (Step 5).

- [ ] **Step 4: README — odjeljak „Lokalni razvoj"** (dodaj na kraj `README.md`)

````markdown
## Lokalni razvoj

Treba: Rust 1.98 (rustup sam povuče verziju iz `rust-toolchain.toml`), Node 24, PostgreSQL 16 kao Windows servis.

1. Jednom: uloga i baza (lozinku korisnika `postgres` znaš samo ti):
   `psql -h localhost -U postgres -c "CREATE ROLE hotelixsim LOGIN PASSWORD 'hotelixsim_lokalno' CREATEDB;" -c "CREATE DATABASE hotelixsim OWNER hotelixsim;"`
2. Jednom: `cp .env.example .env` i `corepack pnpm install`.
3. API (Git Bash): `set -a; source .env; set +a; cargo run -p server -- migrate; cargo run -p server -- serve`
4. Sučelje: `corepack pnpm --filter @hotelixsim/web dev` → <http://localhost:5173>
5. Provjere prije commita: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `node scripts/check-core-deps.mjs`, `cargo test --workspace`, `corepack pnpm check:api`, `corepack pnpm lint`,
   `corepack pnpm typecheck`, `corepack pnpm test`.

Nakon promjene SQL upita: `cargo sqlx prepare --workspace` i commitaj `.sqlx/`.
Nakon promjene API-ja: `corepack pnpm gen:api` i commitaj `openapi.json` i `schema.d.ts`.
````

- [ ] **Step 5: Puni lokalni prolaz svih provjera iz README točke 5**

Pokreni svaku naredbu zasebno s `; echo EXIT=$?`. Expected: svaka `EXIT=0`. Zapiši izlaz završnih redaka (`test result:` po paketu, `Tests N passed`) u izvještaj taska. Nijedan „prošlo" bez viđenog izlaza.

- [ ] **Step 6: Commit** — `git add deny.toml .github/workflows/ci.yml README.md` · poruka: „Kostur 8/8: lokalni prolaz od kraja do kraja, CI, README"

---

