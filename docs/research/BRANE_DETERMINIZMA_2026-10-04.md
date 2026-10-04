# Brane determinizma — pokus izvedivosti (04.10.)

> Ulaz za **Task 1** plana „jezgra i alat za balans" (SLJEDECA_SESIJA §3.1). Pokus je rađen u zasebnom worktreeu,
> ništa nije ostalo u kodu. Svaka tvrdnja je izmjerena viđenim izlazom, osim 6d (iz znanja).

## Rezultati

| # | Tvrdnja | Rezultat | Dokaz |
|---|---|---|---|
| 1 | `#![forbid(clippy::float_arithmetic, …)]` u `lib.rs` sprječava `#[allow]` u modulu | RADI, **samo pod `cargo clippy`** | `error[E0453]: allow(clippy::float_arithmetic) incompatible with previous forbid`; `cargo check` na istoj mutaciji prolazi → CI mora vrtjeti clippy |
| 2a | `disallowed-types` za `std::hash::RandomState` | RADI | jedan zapis pokriva i `std::collections::hash_map::RandomState` (isti item, re-export) |
| 2b | `std::hash::DefaultHasher` | RADI | `use of a disallowed type std::hash::DefaultHasher` |
| 2c | `IndexMap::new()` sa zadanim hasherom | **NE RADI** | tip se zaključuje, ne piše u izvoru → brani samo dopušteni popis ovisnosti (6) |
| 3 | `as_conversions`, `cast_possible_truncation`, `cast_sign_loss`, `cast_lossless`, `cast_possible_wrap` | 0 upozorenja | izvor nema nijedan `as` → uključivanje je besplatno (hvatanje mutacijom još nije dokazano) |
| 4 | Zabrana `f32`/`f64` kao tipova (`disallowed-types` s `path = "f64"`) | RADI djelomično | hvata parametar, povratni tip, polje, `x as f64`, `parse::<f64>()` |
| 4b | Što promašuje | | `let y = 1.5;` hvata `default_numeric_fallback`; `2.5f64` ne hvata ništa → grep-brana `\d(f32\|f64)\b` |
| 5 | `std::env::var` kao `disallowed-methods` | RADI | `use of a disallowed method std::env::var` (`var_os` dodati, nije mjereno) |
| 6a | Dopušteni popis u `check-core-deps.mjs` | izvedivo | `FORBIDDEN` → `ALLOWED = {engine, sim-hotel}`, izlaz 1 za sve ostalo |
| 6b | cargo-deny `[bans] allow` samo za jezgru | RADI preko zasebnog configa | `allow` je globalan za graf → `cargo deny --manifest-path crates/<c>/Cargo.toml --config deny-core.toml check bans`, po crateu; cijeli workspace istim configom pada (177 grešaka) |
| 6c | Negativni test za 6b | RADI | s `indexmap = "2"`: `error[not-allowed]: crate 'indexmap = 2.14.2' is not explicitly allowed` (i `hashbrown`, `equivalent`, tranzitivno za sim-hotel), EXIT=2 |
| 6d | `wrappers` | ne treba | iz znanja; allowlist po crateu je jači |
| 7 | Skripta mutacijskih proba | RADI | 3 mutacije, 2,4 s ukupno, sve padaju (`exit=101`), „proboja: 0" |

## Preporučeni oblik Task 1 (po prioritetu)

1. Dopušteni popis ovisnosti kao 6b (`deny-core.toml` s `allow = ["engine", "sim-hotel"]`, poziv po crateu) ili 6a.
   To je jedina obrana od 2c (tranzitivni hasher).
2. Oba `lib.rs`: `deny` → `forbid` za `float_arithmetic` i `arithmetic_side_effects`.
3. Oba `clippy.toml`: `disallowed-types` + `std::hash::RandomState`, `std::hash::DefaultHasher`, `f32`, `f64`;
   `disallowed-methods` + `std::env::var`, `std::env::var_os`.
4. `clippy::as_conversions` i `default_numeric_fallback` kao `forbid`; grep-brana za sufiks-literale.
5. Skripta mutacijskih proba u CI (~2 s) + `git diff --exit-code` iza nje (dokaz da je datoteka vraćena).

Napomena: sandbox u worktreeu odbija `git`/`cargo` u petljama i s varijablama — takve naredbe ide u `.sh` datoteku.

## Skica skripte (`scripts/probe-mutacija.mjs`, nije u repou)

```js
// Mutacijske probe brana jezgre: svaka mutacija MORA srušiti clippy. Izlaz 0 = sve brane drže; 1 = neka ne drži; 2 = proba neispravna.
import { readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const MUTACIJE = [
  { ime: "dijeljenje operatorom %", datoteka: "crates/engine/src/rng.rs",
    nadi: 'let threshold = span.wrapping_neg().checked_rem(span).expect("span > 0");',
    zamijeni: "let threshold = span.wrapping_neg() % span;", ocekuj: /arithmetic_side_effects|clippy::arithmetic/ },
  { ime: "HashMap u jezgri", datoteka: "crates/engine/src/lib.rs",
    nadi: "pub mod rng;",
    zamijeni: "pub mod rng;\npub fn probe() -> std::collections::HashMap<u8, u8> { std::collections::HashMap::new() }",
    ocekuj: /disallowed type/ },
  { ime: "f64 aritmetika", datoteka: "crates/sim-hotel/src/lib.rs",
    nadi: "pub mod index;",
    zamijeni: "pub mod index;\npub fn probe(a: f64, b: f64) -> f64 { a + b }", ocekuj: /float_arithmetic|floating-point arithmetic/ },
];

let neuspjeh = 0;
for (const m of MUTACIJE) {
  const original = readFileSync(m.datoteka, "utf8");
  const pogodaka = original.split(m.nadi).length - 1;
  if (pogodaka !== 1) {
    console.error(`PROBA NEISPRAVNA [${m.ime}]: ${pogodaka} pogodaka u ${m.datoteka}, treba točno 1`);
    process.exit(2);
  }
  const t0 = Date.now();
  try {
    writeFileSync(m.datoteka, original.replace(m.nadi, () => m.zamijeni));
    const r = spawnSync("cargo", ["clippy", "-p", "engine", "-p", "sim-hotel", "--all-targets"],
      { encoding: "utf8", env: { ...process.env, SQLX_OFFLINE: "true" } });
    const izlaz = `${r.stdout}\n${r.stderr}`;
    const pao = r.status !== 0 && m.ocekuj.test(izlaz);
    console.log(`${pao ? "OK   " : "PROBOJ"} [${m.ime}] exit=${r.status} ${Date.now() - t0}ms`);
    if (!pao) { neuspjeh += 1; if (r.status !== 0) console.error(izlaz.slice(0, 600)); }
  } finally {
    writeFileSync(m.datoteka, original);
  }
}
console.log(`proboja: ${neuspjeh}`);
process.exit(neuspjeh === 0 ? 0 : 1);
```
