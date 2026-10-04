// Generira openapi.json iz Rust opisa API-ja. ARHITEKTURA §11.
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";

const json = execFileSync("cargo", ["run", "-q", "-p", "server", "--", "openapi"], { encoding: "utf8" });
if (!json.startsWith("{") || !json.includes('"openapi": "3.1.0"')) {
  console.error("Neočekivan izlaz servera (nije OpenAPI 3.1):", json.slice(0, 200));
  process.exit(1);
}
writeFileSync("openapi.json", json);
console.log(`openapi.json: ${json.length} znakova`);
