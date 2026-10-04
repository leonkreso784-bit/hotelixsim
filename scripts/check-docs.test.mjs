// Test brane check:docs — svaka provjera mora PASTI na namjerno pokvarenom primjeru,
// inače bi brana mogla tiho prestati mjeriti a zeleno bi i dalje svijetlilo.
// Fixture je privremena mapa bez gita (skripta tada ide rezervnim walkom).
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

const SCRIPT = path.join(import.meta.dirname, "check-docs.mjs");

// Ispravan minimalni raspored; testovi ga mijenjaju preko `over` (null = obriši datoteku).
const BASE = {
  "CLAUDE.md": "# Pravila\n\nKratko.\n",
  "README.md": "# HotelixSim\n\nVidi [dokumentaciju](docs/README.md).\n",
  "docs/README.md":
    "# Indeks\n\n- [Vizija](product/VIZIJA.md)\n- [Roadmap](plan/ROADMAP.md)\n- [Plan](plan/PLAN_A.md)\n- [Povijest](records/)\n- PROGRESS.md\n",
  "docs/product/VIZIJA.md": "# Vizija\n\nDefinicija, 2026-10-04.\n",
  "docs/plan/ROADMAP.md": "# Roadmap\n",
  "docs/plan/PLAN_A.md": "# Plan A\n\n**Status:** u tijeku\n",
  "docs/records/PROGRESS.md": "# Progress\n",
};

function run(over = {}) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "check-docs-"));
  try {
    for (const [name, content] of Object.entries({ ...BASE, ...over })) {
      if (content === null) continue;
      const abs = path.join(dir, name);
      fs.mkdirSync(path.dirname(abs), { recursive: true });
      fs.writeFileSync(abs, content);
    }
    // bez GIT_* varijabli: GIT_DIR iz kuke inače bi pokvario pravi repo
    const env = Object.fromEntries(Object.entries(process.env).filter(([k]) => !k.startsWith("GIT_")));
    const r = spawnSync(process.execPath, [SCRIPT], {
      env: { ...env, CHECK_DOCS_ROOT: dir },
      encoding: "utf8",
    });
    return { code: r.status, out: r.stdout + r.stderr };
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

function assertFails(over, needle) {
  const r = run(over);
  assert.equal(r.code, 1, r.out);
  assert.ok(r.out.includes("❌"), r.out);
  assert.ok(r.out.includes(needle), `očekivano "${needle}" u:\n${r.out}`);
}

test("ispravan minimalni primjer je zelen", () => {
  const r = run();
  assert.equal(r.code, 0, r.out);
  assert.ok(r.out.includes("=== check:docs ==="));
  assert.ok(r.out.includes("✅"));
});

test("1) mrtva .md poveznica pada", () => {
  assertFails(
    { "docs/product/VIZIJA.md": "# V\n\n[x](NEMA.md#sidro)\n" },
    "MRTVA POVEZNICA  docs/product/VIZIJA.md  ->  NEMA.md",
  );
});

test("1) mrtva poveznica na mapu pada, postojeća ne", () => {
  assertFails({ "docs/product/VIZIJA.md": "# V\n\n[x](../nema-mape/)\n" }, "MRTVA POVEZNICA");
  const r = run({ "docs/product/VIZIJA.md": "# V\n\n[x](../records/) [y](../plan/ROADMAP.md#a)\n" });
  assert.equal(r.code, 0, r.out);
});

test("1) poveznica u kodu i vanjski URL se ne broje", () => {
  const r = run({
    "docs/product/VIZIJA.md": "# V\n\n`[x](NEMA.md)`\n\n```\n[x](NEMA2.md)\n```\n[g](https://a.b/c.md)\n",
  });
  assert.equal(r.code, 0, r.out);
});

test("2) .md pod docs/ koji nije u indeksu pada", () => {
  assertFails({ "docs/research/SIROCE.md": "# Siroče\n" }, "NIJE U INDEKSU  docs/research/SIROCE.md");
});

test("2) bez docs/README.md pada", () => {
  assertFails({ "docs/README.md": null, "README.md": "# H\n" }, "NEMA INDEKSA");
});

test("3) dva aktivna plana padaju, pauziran se ne broji", () => {
  const idx = `${BASE["docs/README.md"]}- [B](plan/PLAN_B.md)\n`;
  assertFails(
    { "docs/README.md": idx, "docs/plan/PLAN_B.md": "# B\n" },
    "VISE AKTIVNIH PLANOVA u docs/plan/",
  );
  const r = run({ "docs/README.md": idx, "docs/plan/PLAN_B.md": "# B\n\n**Status:** ⏸️ PAUZIRAN\n" });
  assert.equal(r.code, 0, r.out);
});

test("4) više od 3 datuma u product/ pada, točno 3 prolazi", () => {
  assertFails(
    { "docs/product/VIZIJA.md": "# V\n2026-01-01 2026-01-02 2026-01-03 2026-01-04\n" },
    "DNEVNIK U DEFINICIJI  docs/product/VIZIJA.md  (4 datuma)",
  );
  const r = run({ "docs/product/VIZIJA.md": "# V\n2026-01-01 2026-01-02 2026-01-03\n" });
  assert.equal(r.code, 0, r.out);
});

test("5) CLAUDE.md preko budžeta pada (CR se ne broji)", () => {
  assertFails({ "CLAUDE.md": "a".repeat(12001) }, "CLAUDE.md PREKO BUDZETA  12001 > 12000");
  const crlf = `${"a".repeat(99)}\r\n`.repeat(120); // 12000 znakova bez CR, 12120 s njima
  const r = run({ "CLAUDE.md": crlf });
  assert.equal(r.code, 0, r.out);
});

test("6) ćirilica izvan koda pada, u kodu prolazi", () => {
  const cyr = String.fromCharCode(0x0430); // „a" koje izgleda kao latinično
  assertFails({ "docs/product/VIZIJA.md": `# V\n\nkat${cyr}log\n` }, "CIRILICA  docs/product/VIZIJA.md");
  const r = run({ "docs/product/VIZIJA.md": `# V\n\nraspon \`${cyr}\`\n\n\`\`\`\n${cyr}\n\`\`\`\n` });
  assert.equal(r.code, 0, r.out);
});

test("7) plan u archive/ proglašen aktivnim pada", () => {
  assertFails(
    { "docs/plan/ROADMAP.md": "# Roadmap\n\nAKTIVAN plan: docs/archive/STARI.md\n" },
    "ARHIVA PROGLASENA AKTIVNOM  docs/plan/ROADMAP.md:3",
  );
  const r = run({ "docs/plan/ROADMAP.md": "# Roadmap\n\nZavršeno, vidi docs/archive/STARI.md\n" });
  assert.equal(r.code, 0, r.out);
});
