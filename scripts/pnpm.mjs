// Pokreće isti pnpm koji je pokrenuo skriptu (npm_execpath), pa skripte rade i kad pnpm nije na PATH-u
// (npr. `corepack pnpm …` bez `corepack enable`) i kad jest (CI).
import { spawnSync } from "node:child_process";

const execpath = process.env.npm_execpath;
if (!execpath) {
  console.error("npm_execpath nije postavljen: pokreni kroz pnpm (npr. `pnpm gen:api`).");
  process.exit(1);
}
const result = spawnSync(process.execPath, [execpath, ...process.argv.slice(2)], { stdio: "inherit" });
process.exit(result.status ?? 1);
