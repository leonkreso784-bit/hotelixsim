### Task 1: Brane determinizma (ADR-009)

**Files:**
- Modify: `scripts/check-core-deps.mjs` (FORBIDDEN → dopušteni popis; + grep float sufiksa)
- Modify: `crates/engine/src/lib.rs:2`, `crates/sim-hotel/src/lib.rs:2` (`deny` → `forbid` + nove brane)
- Modify: `crates/engine/clippy.toml`, `crates/sim-hotel/clippy.toml` (moraju ostati identični)
- Create: `scripts/probe-mutacija.mjs`
- Modify: `.github/workflows/ci.yml` (posao `rust`, iza `check-core-deps`)
- Modify: `.gitattributes` (LF za `Cargo.lock`, `Cargo.toml`, `*.mjs`, `.sqlx/*.json`)

**Interfaces:**
- Produces: `node scripts/probe-mutacija.mjs` → izlaz 0 samo ako SVAKA mutacija padne s očekivanom porukom; 1 proboj; 2 proba neispravna.

- [ ] **Step 1: Dopušteni popis u `check-core-deps.mjs`.** Zamijeni `FORBIDDEN` s:

```js
// Jezgra smije ovisiti SAMO o sebi. Zabranjeni popis je propuštao sve što nije nabrojano (npr. `indexmap`
// sa zadanim nasumičnim hasherom — BRANE_DETERMINIZMA 2c); dopušteni popis pada na svaku novu ovisnost.
const ALLOWED = new Set(["engine", "sim-hotel"]);
```
i u petlji `if (!ALLOWED.has(name)) { console.error(`NIJE DOPUŠTENO: ${pkg} ovisi o ${name}`); found += 1; }`.
Na kraju dodaj grep za float sufikse (clippy `2.5f64` ne hvata — BRANE 4b):

```js
// Float literal sa sufiksom (`2.5f64`, `1f32`) prolazi sve clippy brane; hvata ga samo pretraga izvora.
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
const FLOAT_SUFFIX = /\b\d[\d_]*(\.\d[\d_]*)?(e[+-]?\d+)?_?f(32|64)\b/;
function walkRs(dir, out = []) {
  for (const n of readdirSync(dir)) {
    const p = join(dir, n);
    if (statSync(p).isDirectory()) walkRs(p, out);
    else if (n.endsWith(".rs")) out.push(p);
  }
  return out;
}
for (const crate of ["crates/engine", "crates/sim-hotel"]) {
  for (const f of walkRs(join(crate, "src"))) {
    readFileSync(f, "utf8").split(/\r?\n/).forEach((line, i) => {
      if (FLOAT_SUFFIX.test(line)) { console.error(`FLOAT LITERAL: ${f}:${i + 1}: ${line.trim()}`); found += 1; }
    });
  }
}
```
(Pomakni `console.log(\`zabranjenih ovisnosti…\`)` i `process.exit` iza ove petlje; poruku promijeni u „prekršaja".)

- [ ] **Step 2: Pokreni** `node scripts/check-core-deps.mjs; echo EXIT=$?` → `engine: provjereno 1 paketa`,
  `sim-hotel: provjereno 2 paketa`, `prekršaja: 0`, `EXIT=0`.

- [ ] **Step 3: `lib.rs` oba cratea** — zamijeni redak 2 s:

```rust
#![forbid(
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::default_numeric_fallback
)]
```
`forbid` (ne `deny`) jer `deny` dopušta lokalni `#[allow]` (BACKLOG, Task 1 #3 kostura). Radi samo pod `cargo clippy`.

- [ ] **Step 4: Oba `clippy.toml`** (isti sadržaj, bajt za bajt):

```toml
disallowed-types = [
  { path = "std::collections::HashMap", reason = "nasumičan redoslijed ruši determinizam (ARHITEKTURA §7.2)", replacement = "std::collections::BTreeMap" },
  { path = "std::collections::HashSet", reason = "nasumičan redoslijed ruši determinizam (ARHITEKTURA §7.2)", replacement = "std::collections::BTreeSet" },
  { path = "std::hash::RandomState", reason = "nasumično sjeme hashera (ARHITEKTURA §7.2)" },
  { path = "std::hash::DefaultHasher", reason = "izlaz nije stabilan među verzijama Rusta" },
  { path = "f32", reason = "jezgra računa cijelim brojevima (DEMO §12)" },
  { path = "f64", reason = "jezgra računa cijelim brojevima (DEMO §12)" },
]
disallowed-methods = [
  { path = "std::time::SystemTime::now", reason = "jezgra ne čita sat (ARHITEKTURA §7.2)" },
  { path = "std::time::Instant::now", reason = "jezgra ne čita sat (ARHITEKTURA §7.2)" },
  { path = "std::env::var", reason = "jezgra ne čita okolinu" },
  { path = "std::env::var_os", reason = "jezgra ne čita okolinu" },
]
```

- [ ] **Step 5: Pokreni** `cargo clippy --workspace --all-targets -- -D warnings; echo EXIT=$?` → `EXIT=0`. Ako
  `default_numeric_fallback` ili `as_conversions` prijave postojeći kod (`rng.rs`, `index.rs`, `profile.rs`), popravi
  izvor (tipiziraj literal: `0_u64`; `u64::from(x)`), ne branu.

- [ ] **Step 6: `scripts/probe-mutacija.mjs`** — po skici iz BRANE_DETERMINIZMA (§„Skica skripte"), s ovim mutacijama
  (svaka: datoteka, točno jedan pogodak `nadi`, `zamijeni`, regex očekivane poruke):

| ime | datoteka | umetni / zamijeni | očekuj (regex) |
|---|---|---|---|
| `%` umjesto `checked_rem` | `crates/engine/src/rng.rs` | `span.wrapping_neg().checked_rem(span).expect("span > 0")` → `span.wrapping_neg() % span` | `arithmetic_side_effects\|arithmetic operation` |
| HashMap u engine | `crates/engine/src/lib.rs` | iza `pub mod rng;`: `pub fn probe() -> std::collections::HashMap<u8, u8> { std::collections::HashMap::new() }` | `disallowed type` |
| HashSet u sim-hotel | `crates/sim-hotel/src/lib.rs` | iza `pub mod index;`: `pub fn probe() -> std::collections::HashSet<u8> { std::collections::HashSet::new() }` | `disallowed type` |
| RandomState | `crates/sim-hotel/src/lib.rs` | `pub fn probe() -> std::hash::RandomState { std::hash::RandomState::new() }` | `disallowed type` |
| sat | `crates/sim-hotel/src/lib.rs` | `pub fn probe() -> std::time::SystemTime { std::time::SystemTime::now() }` | `disallowed method` |
| okolina | `crates/engine/src/lib.rs` | `pub fn probe() -> bool { std::env::var("X").is_ok() }` | `disallowed method` |
| f64 aritmetika | `crates/sim-hotel/src/lib.rs` | `pub fn probe(a: f64, b: f64) -> f64 { a + b }` | `disallowed type\|float` |
| `as` pretvorba | `crates/engine/src/lib.rs` | `pub fn probe(x: u64) -> u8 { x as u8 }` | `as_conversions\|using a potentially dangerous silent` |
| lokalni allow | `crates/sim-hotel/src/lib.rs` | `#[allow(clippy::float_arithmetic)] pub fn probe(a: u8) -> u8 { a }` | `incompatible with previous forbid\|E0453` |

  Uz clippy mutacije, dvije za `check-core-deps.mjs` (pokreće `node scripts/check-core-deps.mjs` umjesto clippyja):
  `crates/sim-hotel/src/lib.rs` umetni `pub const PROBE: u8 = { let _x = 2.5f64; 0 };` → očekuj `FLOAT LITERAL`; i
  provjera da su oba `clippy.toml` identična (`readFileSync` usporedba; razlika = izlaz 1 s porukom
  `clippy.toml se razlikuju`). Ovisnost (`indexmap`) se **ne** mutira u CI-ju (mijenja `Cargo.lock` i traži mrežu) —
  ručno je dokazana u BRANE 6c; zapiši to u zaglavlju skripte.
  Pravila skripte: mutacija koja nema točno 1 pogodak → izlaz 2 (zamka 8 u `windows-git-bash-zamke`); datoteka se
  vraća u `finally`; zamjena preko `s.replace(a, () => b)`; clippy se zove `cargo clippy -p engine -p sim-hotel
  --all-targets -- -D warnings`.

- [ ] **Step 7: Pokreni** `node scripts/probe-mutacija.mjs; echo EXIT=$?` → svaki redak `OK`, `proboja: 0`, `EXIT=0`;
  zatim `git diff --stat -- crates/` → prazno (datoteke vraćene).

- [ ] **Step 8: CI i `.gitattributes`.** U `ci.yml`, posao `rust`, iza `node scripts/check-core-deps.mjs`:
  `- run: node scripts/probe-mutacija.mjs` i `- run: git diff --exit-code`. U `.gitattributes` dodaj
  `Cargo.lock text eol=lf`, `Cargo.toml text eol=lf`, `*.mjs text eol=lf`, `.sqlx/*.json text eol=lf` (ako već nisu
  pokriveni širim pravilom — provjeri `git check-attr eol -- Cargo.lock`).

- [ ] **Step 9: Commit** — `scripts/check-core-deps.mjs scripts/probe-mutacija.mjs crates/engine/src/lib.rs
  crates/sim-hotel/src/lib.rs crates/engine/clippy.toml crates/sim-hotel/clippy.toml .github/workflows/ci.yml
  .gitattributes`, poruka „Jezgra 1/12: brane determinizma — dopušteni popis, forbid, mutacijske probe u CI-ju".

---

