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
    ["tree", "-p", pkg, "-e", "normal", "--prefix", "none", "--format", "{p}"],
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
