#!/usr/bin/env node
// ===== grane — koliko je koja lokalna grana iza integracijske i koliko dugo živi =====
// Alat, ne brana: izlazni kod je uvijek 0. Pravilo (AGENTI.md §1): grana cigle ne živi dulje od ~1 dana.
// Pokreni: corepack pnpm grane   |   node scripts/grane.mjs [--integracija <grana>]

import { spawnSync } from "node:child_process";

export const ZADANA_INTEGRACIJA = "model-v2/sloj1";
export const GRANICA_MS = 24 * 60 * 60 * 1000;

// Čista funkcija: grane = [{ ime, ispred, iza, prviNespojen }] → retci sa stanjem.
// Stanje: "spojena" (ispred 0), "STARA" (nespojen rad strogo stariji od 24 h), inače "u radu".
export function procijeni(grane, sada) {
  return grane.map((g) => {
    let stanje = "u radu";
    if (g.ispred === 0) stanje = "spojena";
    else if (g.prviNespojen && sada - Date.parse(g.prviNespojen) > GRANICA_MS) stanje = "STARA";
    return { ...g, stanje };
  });
}

function git(args) {
  const r = spawnSync("git", args, { encoding: "utf8" });
  if (r.status !== 0) throw new Error(`git ${args.join(" ")}: ${r.stderr.trim()}`);
  return r.stdout.trim();
}

export function procitajGrane(integracija) {
  const imena = git(["for-each-ref", "refs/heads", "--format=%(refname:short)"])
    .split("\n")
    .filter(Boolean)
    .filter((ime) => ime !== integracija);
  return imena.map((ime) => {
    const [ispred, iza] = git(["rev-list", "--left-right", "--count", `${ime}...${integracija}`])
      .split(/\s+/)
      .map(Number);
    const prvi =
      ispred > 0 ? git(["log", "--reverse", "--format=%cI", `${integracija}..${ime}`]).split("\n")[0] : null;
    return { ime, ispred, iza, prviNespojen: prvi || null };
  });
}

function ispisi(reci, integracija) {
  const zaglavlje = ["grana", "ispred", "iza", "prvi nespojen", "stanje"];
  const retci = reci.map((r) => [
    r.ime,
    String(r.ispred),
    String(r.iza),
    r.prviNespojen ? r.prviNespojen.slice(0, 16).replace("T", " ") : "-",
    r.stanje === "STARA" ? "STARA (nespojen rad stariji od 24 h)" : r.stanje,
  ]);
  const sirine = zaglavlje.map((z, i) => Math.max(z.length, ...retci.map((r) => r[i].length)));
  const redak = (r) =>
    r
      .map((c, i) => c.padEnd(sirine[i]))
      .join("  ")
      .trimEnd();
  console.log(`Integracijska grana: ${integracija}`);
  console.log(redak(zaglavlje));
  for (const r of retci) console.log(redak(r));
  const stare = reci.filter((r) => r.stanje === "STARA").length;
  const spojene = reci.filter((r) => r.stanje === "spojena").length;
  console.log(`\n${reci.length} grana: ${spojene} spojenih, ${stare} STARIH.`);
}

if (process.argv[1]?.endsWith("grane.mjs")) {
  const i = process.argv.indexOf("--integracija");
  const integracija = i > -1 ? process.argv[i + 1] : ZADANA_INTEGRACIJA;
  try {
    ispisi(procijeni(procitajGrane(integracija), Date.now()), integracija);
  } catch (e) {
    console.error(String(e.message ?? e));
  }
}
