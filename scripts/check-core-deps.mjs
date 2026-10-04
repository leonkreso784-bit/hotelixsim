// Brana: jezgra (engine, sim-hotel) ne smije ni tranzitivno ovisiti o I/O-u, satu ni vanjskoj slučajnosti.
// ARHITEKTURA §6.2. Pokriva normalne i build ovisnosti, za sve targete i sa svim značajkama.
// Izlaz 0 = čisto; 1 = nađeno (ispisuje svaki prekršaj); 2 = alat nije uspio izmjeriti.
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

const CORE = ["engine", "sim-hotel"];
// Jezgra smije ovisiti SAMO o sebi. Zabranjeni popis je propuštao sve što nije nabrojano (npr. `indexmap`
// sa zadanim nasumičnim hasherom — BRANE_DETERMINIZMA 2c); dopušteni popis pada na svaku novu ovisnost.
const ALLOWED = new Set(["engine", "sim-hotel"]);

let found = 0;
for (const pkg of CORE) {
  let out;
  try {
    out = execFileSync(
      "cargo",
      [
        "tree",
        "-p",
        pkg,
        "-e",
        "normal,build",
        "--target",
        "all",
        "--all-features",
        "--prefix",
        "none",
        "--format",
        "{p}",
      ],
      { encoding: "utf8" },
    );
  } catch (err) {
    console.error(`cargo tree nije uspio za ${pkg}: ${err.message}`);
    process.exit(2);
  }
  const names = new Set(
    out
      .split(/\r?\n/)
      .filter(Boolean)
      .map((line) => line.split(" ")[0]),
  );
  if (!names.has(pkg)) {
    console.error(`cargo tree nije vratio ni sam paket ${pkg}: izlaz se ne može protumačiti`);
    process.exit(2);
  }
  for (const name of names) {
    if (!ALLOWED.has(name)) {
      console.error(`NIJE DOPUŠTENO: ${pkg} ovisi o ${name}`);
      found += 1;
    }
  }
  console.log(`${pkg}: provjereno ${names.size} paketa`);
}

// Float literal sa sufiksom (`2.5f64`, `1f32`) prolazi sve clippy brane; hvata ga samo pretraga izvora.
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
    readFileSync(f, "utf8")
      .split(/\r?\n/)
      .forEach((line, i) => {
        if (FLOAT_SUFFIX.test(line)) {
          console.error(`FLOAT LITERAL: ${f}:${i + 1}: ${line.trim()}`);
          found += 1;
        }
      });
  }
}

console.log(`prekršaja: ${found}`);
process.exit(found === 0 ? 0 : 1);
