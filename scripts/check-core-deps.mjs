// Brana: jezgra (engine, sim-hotel) ne smije ni tranzitivno ovisiti o I/O-u, satu ni vanjskoj slučajnosti.
// ARHITEKTURA §6.2. Pokriva normalne i build ovisnosti, za sve targete i sa svim značajkama.
// Izlaz 0 = čisto; 1 = nađeno (ispisuje svaku zabranjenu ovisnost); 2 = alat nije uspio izmjeriti.
import { execFileSync } from "node:child_process";

const CORE = ["engine", "sim-hotel"];
const FORBIDDEN = new Set([
  "tokio",
  "sqlx",
  "reqwest",
  "rand",
  "rand_core",
  "getrandom",
  "chrono",
  "time",
  "axum",
  "hyper",
]);

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
    if (FORBIDDEN.has(name)) {
      console.error(`ZABRANJENO: ${pkg} ovisi o ${name}`);
      found += 1;
    }
  }
  console.log(`${pkg}: provjereno ${names.size} paketa`);
}
console.log(`zabranjenih ovisnosti: ${found}`);
process.exit(found === 0 ? 0 : 1);
