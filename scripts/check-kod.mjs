#!/usr/bin/env node
// ===== check:kod — brana veličine koda =====
// Postoji jer datoteke koda tiho rastu do tisuća redaka (teško za čitanje i pregled).
// Broji redaka .rs .ts .tsx .mjs .js (kao `wc -l`):
//   - iznad UPOZORENJE (600) daje upozorenje, iznad TVRDA (1000) grešku (exit 1);
//   - DUG: postojeće velike datoteke, izuzete od tvrde granice dok ne prerastu upisani maksimum
//     (rast = greška; pad = upozorenje da se unos spusti; nestala datoteka = zastario unos);
//   - IZUZECI: generirano, ne broji se uopće.
// Pokreni: pnpm check:kod   |   korijen: node scripts/check-kod.mjs <dir> ili env CHECK_KOD_ROOT.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { brojRedaka } from "./docs-granice.mjs";

export const UPOZORENJE = 600;
export const TVRDA = 1000;

// Putanja → najviše redaka. Spajanje grane koja legitimno podiže dug diže unos u istom commitu.
export const DUG = {
  "crates/api/src/schema.rs": 1717,
  "crates/game/tests/mentor.rs": 1687,
  "crates/game/src/mentor.rs": 1399,
};

// Putanja → razlog. Ne broji se uopće.
export const IZUZECI = {
  "packages/api-client/src/schema.d.ts": "generirano (gen:api)",
};

const nastavci = new Set([".rs", ".ts", ".tsx", ".mjs", ".js"]);
const zanemareneMape = new Set([
  "node_modules",
  "target",
  "dist",
  ".git",
  ".superpowers",
  ".playwright-mcp",
  ".claude",
]);

function skupi(korijen, rel, izlaz) {
  const unosi = fs.readdirSync(path.join(korijen, rel), { withFileTypes: true });
  for (const u of unosi) {
    const put = rel ? `${rel}/${u.name}` : u.name;
    if (u.isDirectory()) {
      if (!zanemareneMape.has(u.name)) skupi(korijen, put, izlaz);
    } else if (u.isFile() && nastavci.has(path.extname(u.name))) {
      izlaz.push(put);
    }
  }
  return izlaz;
}

export function provjeri(korijen) {
  const greske = [];
  const upozorenja = [];
  const vidjeno = new Set();
  for (const put of skupi(korijen, "", []).sort()) {
    vidjeno.add(put);
    if (Object.hasOwn(IZUZECI, put)) continue;
    const n = brojRedaka(fs.readFileSync(path.join(korijen, put), "utf8"));
    if (Object.hasOwn(DUG, put)) {
      const max = DUG[put];
      if (n > max) greske.push(`DUG NARASTAO  ${put}  ${n} > ${max}`);
      else if (n < max) upozorenja.push(`SPUSTI GRANICU DUGA  ${put}  na ${n}`);
      continue;
    }
    if (n > TVRDA) greske.push(`PREKO ${TVRDA} REDAKA  ${put}  ${n}`);
    else if (n > UPOZORENJE) upozorenja.push(`IZNAD ${UPOZORENJE}  ${put}  ${n}`);
  }
  for (const put of Object.keys(DUG)) {
    if (!vidjeno.has(put)) upozorenja.push(`ZASTARIO UNOS DUGA  ${put}`);
  }
  return { greske, upozorenja };
}

// Ulazna točka: izvrši se uvijek kad je ovo pokrenuta skripta (ne kad je uvezena u test).
// Usporedba putanja ne smije šutjeti preko junctiona/poveznice/subst pogona: realpath s obje strane,
// a ako i to zakaže, prepoznaje se po imenu datoteke.
function jePokrenuta() {
  const arg = process.argv[1];
  if (!arg) return false;
  const ja = fileURLToPath(import.meta.url);
  try {
    if (fs.realpathSync.native(arg) === fs.realpathSync.native(ja)) return true;
  } catch {}
  return path.basename(arg) === path.basename(ja);
}

if (jePokrenuta()) {
  const korijen =
    process.argv[2] ||
    process.env.CHECK_KOD_ROOT ||
    path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  const { greske, upozorenja } = provjeri(korijen);
  console.log("=== check:kod ===");
  for (const u of upozorenja) console.log(`⚠️  ${u}`);
  for (const g of greske) console.log(`❌ ${g}`);
  if (greske.length === 0) console.log("✅ kod je unutar granica");
  process.exit(greske.length > 0 ? 1 : 0);
}
