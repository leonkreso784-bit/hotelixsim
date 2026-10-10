// Mutacijske probe brana jezgre (ADR-009): svaka mutacija MORA srušiti brane s očekivanom porukom.
// Izlaz 0 = sve brane drže; 1 = neka ne drži (proboj); 2 = proba neispravna (mutacija nema točno 1 pogodak).
// Ne mutira se ovisnost (`indexmap`): mijenja Cargo.lock i traži mrežu; ručno je dokazana u BRANE_DETERMINIZMA 6c.
import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";

const ENGINE_LIB = "crates/engine/src/lib.rs";
const HOTEL_LIB = "crates/sim-hotel/src/lib.rs";
const HOTEL_V2_LIB = "crates/sim-hotel-v2/src/lib.rs";

const CLIPPY = [
  "clippy",
  "-p",
  "engine",
  "-p",
  "sim-hotel",
  "-p",
  "sim-hotel-v2",
  "--all-targets",
  "--",
  "-D",
  "warnings",
];

const MUTACIJE = [
  {
    ime: "% umjesto checked_rem",
    datoteka: "crates/engine/src/rng.rs",
    nadi: 'span.wrapping_neg().checked_rem(span).expect("span > 0")',
    zamijeni: "span.wrapping_neg() % span",
    ocekuj: /arithmetic_side_effects|arithmetic operation/,
  },
  {
    ime: "HashMap u engine",
    datoteka: ENGINE_LIB,
    nadi: "pub mod rng;",
    zamijeni:
      "pub mod rng;\npub fn probe() -> std::collections::HashMap<u8, u8> { std::collections::HashMap::new() }",
    ocekuj: /disallowed type/,
  },
  {
    ime: "HashSet u sim-hotel",
    datoteka: HOTEL_LIB,
    nadi: "pub mod index;",
    zamijeni:
      "pub mod index;\npub fn probe() -> std::collections::HashSet<u8> { std::collections::HashSet::new() }",
    ocekuj: /disallowed type/,
  },
  {
    ime: "HashSet u sim-hotel-v2",
    datoteka: HOTEL_V2_LIB,
    nadi: "pub mod calendar;",
    zamijeni:
      "pub mod calendar;\npub fn probe() -> std::collections::HashSet<u8> { std::collections::HashSet::new() }",
    ocekuj: /disallowed type/,
  },
  {
    ime: "RandomState",
    datoteka: HOTEL_LIB,
    nadi: "pub mod index;",
    zamijeni: "pub mod index;\npub fn probe() -> std::hash::RandomState { std::hash::RandomState::new() }",
    ocekuj: /disallowed type/,
  },
  {
    ime: "sat",
    datoteka: HOTEL_LIB,
    nadi: "pub mod index;",
    zamijeni: "pub mod index;\npub fn probe() -> std::time::SystemTime { std::time::SystemTime::now() }",
    ocekuj: /disallowed method/,
  },
  {
    ime: "okolina",
    datoteka: ENGINE_LIB,
    nadi: "pub mod rng;",
    zamijeni: 'pub mod rng;\npub fn probe() -> bool { std::env::var("X").is_ok() }',
    ocekuj: /disallowed method/,
  },
  {
    ime: "f64 aritmetika",
    datoteka: HOTEL_LIB,
    nadi: "pub mod index;",
    zamijeni: "pub mod index;\npub fn probe(a: f64, b: f64) -> f64 { a + b }",
    ocekuj: /float_arithmetic|floating-point arithmetic/,
  },
  {
    ime: "f64 kao tip (bez aritmetike)",
    datoteka: HOTEL_LIB,
    nadi: "pub mod index;",
    zamijeni: "pub mod index;\npub fn probe(a: f64) -> f64 { a }",
    ocekuj: /disallowed type/,
  },
  {
    ime: "f32 kao tip (bez aritmetike)",
    datoteka: ENGINE_LIB,
    nadi: "pub mod rng;",
    zamijeni: "pub mod rng;\npub fn probe(a: f32) -> f32 { a }",
    ocekuj: /disallowed type/,
  },
  {
    ime: "as pretvorba",
    datoteka: ENGINE_LIB,
    nadi: "pub mod rng;",
    zamijeni: "pub mod rng;\npub fn probe(x: u64) -> u8 { x as u8 }",
    ocekuj: /as_conversions|using a potentially dangerous silent/,
  },
  {
    ime: "lokalni allow",
    datoteka: HOTEL_LIB,
    nadi: "pub mod index;",
    zamijeni: "pub mod index;\n#[allow(clippy::float_arithmetic)]\npub fn probe(a: u8) -> u8 { a }",
    ocekuj: /incompatible with previous forbid|E0453/,
  },
  {
    ime: "float literal sa sufiksom",
    datoteka: HOTEL_LIB,
    nadi: "pub mod index;",
    zamijeni: "pub mod index;\npub const PROBE: u8 = { let _x = 2.5f64; 0 };",
    alat: ["node", ["scripts/check-core-deps.mjs"]],
    ocekuj: /FLOAT LITERAL/,
  },
];

let neuspjeh = 0;

// Svi clippy.toml jezgrenih crateova moraju ostati bajt za bajt isti.
{
  const CLIPPY_TOML = [
    "crates/engine/clippy.toml",
    "crates/sim-hotel/clippy.toml",
    "crates/sim-hotel-v2/clippy.toml",
  ];
  const [prvi, ...ostali] = CLIPPY_TOML;
  const uzor = readFileSync(prvi, "utf8");
  const razliciti = ostali.filter((f) => readFileSync(f, "utf8") !== uzor);
  if (razliciti.length > 0) {
    console.error(`PROBOJ [clippy.toml] razlikuju se od ${prvi}: ${razliciti.join(", ")}`);
    neuspjeh += 1;
  } else {
    console.log("OK    [clippy.toml identični]");
  }
}

for (const m of MUTACIJE) {
  const original = readFileSync(m.datoteka, "utf8");
  const pogodaka = original.split(m.nadi).length - 1;
  if (pogodaka !== 1) {
    console.error(`PROBA NEISPRAVNA [${m.ime}]: ${pogodaka} pogodaka u ${m.datoteka}, treba točno 1`);
    process.exit(2);
  }
  const [naredba, args] = m.alat ?? ["cargo", CLIPPY];
  const t0 = Date.now();
  try {
    writeFileSync(
      m.datoteka,
      original.replace(m.nadi, () => m.zamijeni),
    );
    const r = spawnSync(naredba, args, {
      encoding: "utf8",
      env: { ...process.env, SQLX_OFFLINE: "true" },
    });
    const izlaz = `${r.stdout}\n${r.stderr}`;
    const pao = r.status !== 0 && m.ocekuj.test(izlaz);
    console.log(`${pao ? "OK   " : "PROBOJ"} [${m.ime}] exit=${r.status} ${Date.now() - t0}ms`);
    if (!pao) {
      neuspjeh += 1;
      console.error(izlaz.slice(0, 600));
    }
  } finally {
    writeFileSync(m.datoteka, original);
  }
}

console.log(`proboja: ${neuspjeh}`);
process.exit(neuspjeh === 0 ? 0 : 1);
