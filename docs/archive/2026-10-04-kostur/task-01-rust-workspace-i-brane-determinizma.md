### Task 1: Rust workspace i brane determinizma

**Files:**
- Create: `Cargo.toml`, `rust-toolchain.toml`, `.gitattributes`, `.env.example`, `scripts/check-core-deps.mjs`
- Create: `crates/engine/Cargo.toml`, `crates/engine/clippy.toml`, `crates/engine/src/lib.rs`
- Create: `crates/sim-hotel/Cargo.toml`, `crates/sim-hotel/clippy.toml`, `crates/sim-hotel/src/lib.rs`

**Interfaces:**
- Produces: workspace s paketima `engine` i `sim-hotel` (prazne knjižnice); naredba `node scripts/check-core-deps.mjs` (izlaz 0 = čisto, 1 = nađena zabranjena ovisnost, ispisuje svaku).

- [ ] **Step 1: Korijenski `Cargo.toml`**

```toml
[workspace]
resolver = "3"
members = ["crates/engine", "crates/sim-hotel"]

[workspace.package]
edition = "2024"
rust-version = "1.98"
publish = false

[workspace.dependencies]
engine = { path = "crates/engine" }
sim-hotel = { path = "crates/sim-hotel" }
app = { path = "crates/app" }
store = { path = "crates/store" }
api = { path = "crates/api" }
anyhow = "1"
axum = "0.8.9"
http-body-util = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.9.0", default-features = false, features = ["runtime-tokio", "postgres", "macros", "migrate"] }
thiserror = "2.0.21"
tokio = { version = "1.53", features = ["macros", "rt-multi-thread", "net", "signal"] }
tower = { version = "0.5.3", features = ["util"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
utoipa = "6.0.0"
utoipa-axum = "0.3.0"

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }

[profile.release]
overflow-checks = true
```

- [ ] **Step 2: `rust-toolchain.toml`, `.gitattributes`, `.env.example`**

`rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.98.1"
components = ["rustfmt", "clippy"]
```

`.gitattributes`:

```text
* text=auto
openapi.json text eol=lf
packages/api-client/src/schema.d.ts text eol=lf
*.rs text eol=lf
*.sql text eol=lf
```

`.env.example`:

```text
DATABASE_URL=postgres://hotelixsim:hotelixsim_lokalno@localhost:5432/hotelixsim
HOTELIXSIM_ADDR=127.0.0.1:8080
```

- [ ] **Step 3: Paket `engine` (prazan, s branama)**

`crates/engine/Cargo.toml`:

```toml
[package]
name = "engine"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[lints]
workspace = true
```

`crates/engine/clippy.toml` (clippy traži `clippy.toml` od mape paketa prema gore, pa ovo vrijedi samo za jezgru):

```toml
disallowed-types = [
  { path = "std::collections::HashMap", reason = "nasumičan redoslijed ruši determinizam (ARHITEKTURA §7.2)", replacement = "std::collections::BTreeMap" },
  { path = "std::collections::HashSet", reason = "nasumičan redoslijed ruši determinizam (ARHITEKTURA §7.2)", replacement = "std::collections::BTreeSet" },
]
disallowed-methods = [
  { path = "std::time::SystemTime::now", reason = "jezgra ne čita sat (ARHITEKTURA §7.2)" },
  { path = "std::time::Instant::now", reason = "jezgra ne čita sat (ARHITEKTURA §7.2)" },
]
```

`crates/engine/src/lib.rs`:

```rust
//! Generička jezgra: kocka, a kasnije dnevnik i životni ciklus runde. Bez I/O-a, sata i vanjske slučajnosti.
#![deny(clippy::float_arithmetic, clippy::arithmetic_side_effects)]
```

- [ ] **Step 4: Paket `sim-hotel` (prazan, iste brane)**

`crates/sim-hotel/Cargo.toml`:

```toml
[package]
name = "sim-hotel"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
engine.workspace = true

[lints]
workspace = true
```

`crates/sim-hotel/clippy.toml`: isti sadržaj kao `crates/engine/clippy.toml` iz Stepa 3 (kopirati doslovno).

`crates/sim-hotel/src/lib.rs`:

```rust
//! Hotelska domena: indeksi, profili, a kasnije stvarni sloj, mehanike i obračun (DEMO.md).
#![deny(clippy::float_arithmetic, clippy::arithmetic_side_effects)]
```

- [ ] **Step 5: Brana ovisnosti `scripts/check-core-deps.mjs`**

```js
// Brana: jezgra (engine, sim-hotel) ne smije ni tranzitivno ovisiti o I/O-u, satu ni vanjskoj slučajnosti.
// ARHITEKTURA §6.2. Izlaz 0 = čisto; 1 = nađeno (ispisuje svaku zabranjenu ovisnost).
import { execFileSync } from "node:child_process";

const CORE = ["engine", "sim-hotel"];
const FORBIDDEN = new Set([
  "tokio", "sqlx", "reqwest", "rand", "rand_core", "getrandom", "chrono", "time", "axum", "hyper",
]);

let found = 0;
for (const pkg of CORE) {
  const out = execFileSync(
    "cargo",
    // Ispravak nakon pregleda Taska 1: i build-ovisnosti, svi targeti i sve značajke; vidi stvarnu datoteku.
    ["tree", "-p", pkg, "-e", "normal,build", "--target", "all", "--all-features", "--prefix", "none", "--format", "{p}"],
    { encoding: "utf8" },
  );
  const names = new Set(out.split(/\r?\n/).filter(Boolean).map((line) => line.split(" ")[0]));
  for (const name of names) {
    if (FORBIDDEN.has(name)) {
      console.error(`ZABRANJENO: ${pkg} ovisi o ${name}`);
      found += 1;
    }
  }
  console.log(`${pkg}: provjereno ${names.size} paketa`);
}
console.log(`zabranjenih ovisnosti: ${found}`);
process.exit(found === 0 ? 0 : 1);
```

- [ ] **Step 6: Provjera da sve prolazi**

Run: `cargo fmt --all -- --check; echo EXIT=$?` → Expected: `EXIT=0`
Run: `cargo clippy --workspace --all-targets -- -D warnings; echo EXIT=$?` → Expected: `EXIT=0`
Run: `node scripts/check-core-deps.mjs; echo EXIT=$?` → Expected: `engine: provjereno 1 paketa`, `sim-hotel: provjereno 2 paketa`, `zabranjenih ovisnosti: 0`, `EXIT=0`

- [ ] **Step 7: Obrnuta provjera — brane stvarno mjere (mutacija je mjerenje)**

Commit prije mutacije nije potreban jer se vraća `git checkout`, ali datoteke iz ovog taska moraju biti spremljene na disk.

a) Dodaj u `crates/engine/src/lib.rs` redak `pub type Zabranjeno = std::collections::HashMap<u8, u8>;`
Run: `cargo clippy -p engine -- -D warnings 2>&1 | grep -c "disallowed"; echo EXIT=$?`
Expected: broj ≥ 1 (clippy javlja `disallowed_types`). Ako je 0 — **stani**, brana ne radi.
Vrati: ukloni redak (Edit alatom).

b) Dodaj u `crates/engine/Cargo.toml` odjeljak `[dependencies]` s retkom `time = "0.3"`.
Run: `node scripts/check-core-deps.mjs; echo EXIT=$?`
Expected: redak `ZABRANJENO: engine ovisi o time`, `zabranjenih ovisnosti: 1` (ili 2 jer ga povlači i `sim-hotel`), `EXIT=1`. Ako je `EXIT=0` — **stani**.
Vrati: `git checkout -- crates/engine/Cargo.toml` ako je datoteka već commitana, inače ukloni odjeljak Edit alatom; zatim ponovi Step 6 i potvrdi `EXIT=0`.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock rust-toolchain.toml .gitattributes .env.example scripts/check-core-deps.mjs crates/engine crates/sim-hotel
git status --short
git commit -F <scratchpad>/msg.txt   # poruka: "Kostur 1/8: Rust workspace i brane determinizma"
```

---

