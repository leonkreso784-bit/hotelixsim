// Test brane check:docs — svaka provjera mora PASTI na namjerno pokvarenom primjeru,
// inače bi brana mogla tiho prestati mjeriti a zeleno bi i dalje svijetlilo.
// Fixture je privremena mapa bez gita (skripta tada ide rezervnim walkom).
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { indeksAdr } from "./adr.mjs";

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

// ── nova pravila (Dokumentacija 7) ──────────────────────────────────

const RED = (n) => "x\n".repeat(n); // točno n redaka (kao `wc -l`)

test("1) poveznica na sliku/skriptu koje nema pada, postojeća prolazi", () => {
  assertFails(
    { "docs/product/VIZIJA.md": "# V\n\n![shema](dizajn/NEMA.png) [s](../../scripts/nema.mjs)\n" },
    "MRTVA POVEZNICA  docs/product/VIZIJA.md  ->  dizajn/NEMA.png",
  );
  const r = run({
    "docs/product/VIZIJA.md":
      "# V\n\n![shema](dizajn/a.png) [s](../../scripts/a.mjs#L3) [t](../../scripts/a%20b.mjs)\n",
    "docs/product/dizajn/a.png": "png",
    "scripts/a.mjs": "// a\n",
    "scripts/a b.mjs": "// b\n",
  });
  assert.equal(r.code, 0, r.out);
});

const PODINDEKS = {
  "docs/README.md": `${BASE["docs/README.md"]}- ARHITEKTURA.md\n`,
  "docs/product/ARHITEKTURA.md": "# Arhitektura\n\nDijelovi: DIO_A.md\n",
  "docs/product/ARHITEKTURA/DIO_A.md": "# Dio A\n",
};

test("2/8) dio u X/ naveden u X.md prolazi (ne treba biti u docs/README.md)", () => {
  const r = run(PODINDEKS);
  assert.equal(r.code, 0, r.out);
});

test("2/8) dio u X/ koji nije u X.md pada, i kad je u docs/README.md", () => {
  assertFails(
    { ...PODINDEKS, "docs/product/ARHITEKTURA.md": "# Arhitektura\n" },
    "NIJE U PODINDEKSU  docs/product/ARHITEKTURA/DIO_A.md  ->  dodaj redak u docs/product/ARHITEKTURA.md",
  );
  assertFails(
    {
      ...PODINDEKS,
      "docs/product/ARHITEKTURA.md": "# Arhitektura\n",
      "docs/README.md": `${PODINDEKS["docs/README.md"]}- DIO_A.md\n`,
    },
    "NIJE U PODINDEKSU",
  );
});

test("2/8) mapa X/ bez X.md pada", () => {
  assertFails(
    { "docs/product/ARHITEKTURA/DIO_A.md": "# Dio A\n" },
    "NEMA PODINDEKSA  docs/product/ARHITEKTURA/DIO_A.md  ->  docs/product/ARHITEKTURA.md ne postoji",
  );
});

test("2/8) X.md uz mapu mora biti u docs/README.md", () => {
  assertFails(
    { ...PODINDEKS, "docs/README.md": BASE["docs/README.md"] },
    "NIJE U INDEKSU  docs/product/ARHITEKTURA.md",
  );
});

test("2/8) dublja razina (X/Y/…) pada, ne-.md datoteke se ne broje", () => {
  assertFails(
    { ...PODINDEKS, "docs/product/ARHITEKTURA/PODMAPA/DUBOKO.md": "# Duboko\n" },
    "PREDUBOKO  docs/product/ARHITEKTURA/PODMAPA/DUBOKO.md",
  );
  const r = run({ ...PODINDEKS, "docs/product/ARHITEKTURA/shema.png": "png" });
  assert.equal(r.code, 0, r.out);
});

const PLAN_B = { "docs/README.md": `${BASE["docs/README.md"]}- [B](plan/PLAN_B.md)\n` };

test("3) dva aktivna plana u istom području padaju, u različitim prolaze", () => {
  assertFails(
    {
      ...PLAN_B,
      "docs/plan/PLAN_A.md": "# A\n\n**Status:** 🟩 AKTIVAN · **Područje:** dokumentacija\n",
      "docs/plan/PLAN_B.md": "# B\n\n**Status:** 🟩 AKTIVAN · **Područje:** dokumentacija\n",
    },
    "VISE AKTIVNIH PLANOVA u docs/plan/ (podrucje: dokumentacija)",
  );
  const r = run({
    ...PLAN_B,
    "docs/plan/PLAN_A.md": "# A\n\n**Status:** 🟩 AKTIVAN · **Područje:** kod\n",
    "docs/plan/PLAN_B.md": "# B\n\n**Status:** 🟩 AKTIVAN · **Područje:** dokumentacija\n",
  });
  assert.equal(r.code, 0, r.out);
});

test("3) plan bez retka Područje je igra; nepoznato područje pada", () => {
  assertFails(
    { ...PLAN_B, "docs/plan/PLAN_B.md": "# B\n" },
    "VISE AKTIVNIH PLANOVA u docs/plan/ (podrucje: igra)",
  );
  assertFails(
    { ...PLAN_B, "docs/plan/PLAN_B.md": "# B\n\n**Status:** 🟩 AKTIVAN · **Područje:** infrastruktura\n" },
    "NEPOZNATO PODRUCJE  docs/plan/PLAN_B.md  ->  infrastruktura",
  );
});

test("3) kod je stari naziv za igru: kod + igra aktivni padaju; igra + platforma + kvaliteta prolaze", () => {
  const plan = (p) => `# P\n\n**Status:** 🟩 AKTIVAN · **Područje:** ${p}\n`;
  const indeks = `${BASE["docs/README.md"]}- [B](plan/PLAN_B.md)\n- [C](plan/PLAN_C.md)\n`;
  assertFails(
    { "docs/README.md": indeks, "docs/plan/PLAN_A.md": plan("kod"), "docs/plan/PLAN_B.md": plan("igra") },
    "VISE AKTIVNIH PLANOVA u docs/plan/ (podrucje: igra)",
  );
  const r = run({
    "docs/README.md": indeks,
    "docs/plan/PLAN_A.md": plan("igra"),
    "docs/plan/PLAN_B.md": plan("platforma"),
    "docs/plan/PLAN_C.md": plan("kvaliteta"),
  });
  assert.equal(r.code, 0, r.out);
});

test("3) taskovi u docs/plan/X/ ne računaju se kao planovi", () => {
  const r = run({
    "docs/plan/PLAN_A.md": "# Plan A\n\n**Status:** u tijeku\n\nTaskovi: task-01.md\n",
    "docs/plan/PLAN_A/task-01.md": "### Task 1: x\n",
    "docs/plan/PLAN_A/task-02.md": "### Task 2: y\n",
  });
  assert.equal(r.code, 1, r.out); // task-02 nije u podindeksu, ali nema "VISE AKTIVNIH"
  assert.ok(!r.out.includes("VISE AKTIVNIH"), r.out);
  assert.ok(r.out.includes("NIJE U PODINDEKSU  docs/plan/PLAN_A/task-02.md"), r.out);
});

test("7) arhiva proglašena aktivnom u docs/plan/X/*.md također pada", () => {
  assertFails(
    {
      "docs/plan/PLAN_A.md": "# Plan A\n\n**Status:** u tijeku\n\ntask-01.md\n",
      "docs/plan/PLAN_A/task-01.md": "### Task 1\n\nAKTIVAN plan: docs/archive/STARI.md\n",
    },
    "ARHIVA PROGLASENA AKTIVNOM  docs/plan/PLAN_A/task-01.md:3",
  );
});

test("9) preko granice pada, točno na granici prolazi", () => {
  assertFails(
    { "docs/product/VIZIJA.md": RED(301) },
    "PREVELIKA DATOTEKA  docs/product/VIZIJA.md  301 redaka > granica 300 (zivi)",
  );
  const r = run({ "docs/product/VIZIJA.md": RED(300) });
  assert.equal(r.code, 0, r.out);
});

test("9) granice po vrsti: indeks 200, predaja 60, arhiva 1000", () => {
  assertFails(
    { "docs/README.md": `${BASE["docs/README.md"]}${RED(200)}` },
    "PREVELIKA DATOTEKA  docs/README.md",
  );
  assertFails(
    {
      "docs/SLJEDECA_SESIJA.md": RED(61),
      "docs/README.md": `${BASE["docs/README.md"]}- SLJEDECA_SESIJA.md\n`,
    },
    "docs/SLJEDECA_SESIJA.md  61 redaka > granica 60 (predaja)",
  );
  const r = run({
    "docs/SLJEDECA_SESIJA.md": RED(60),
    "docs/archive/STARO.md": RED(1000),
    "docs/README.md": `${BASE["docs/README.md"]}- SLJEDECA_SESIJA.md\n- STARO.md\n`,
  });
  assert.equal(r.code, 0, r.out);
});

test("12) tvrda granica: bilo koji .md preko 1200 pada, i izvan docs/, točno 1200 prolazi", () => {
  assertFails(
    { "crates/x/README.md": RED(1201) },
    "PREKO 1200 REDAKA  crates/x/README.md  1201 redaka > tvrda granica 1200",
  );
  assertFails(
    { "docs/archive/STARO.md": RED(1201), "docs/README.md": `${BASE["docs/README.md"]}- STARO.md\n` },
    "PREKO 1200 REDAKA  docs/archive/STARO.md",
  );
  const r = run({ "crates/x/README.md": RED(1200) });
  assert.equal(r.code, 0, r.out);
});

test("12) prijelazna iznimka ne vrijedi za tvrdu granicu (ni za dio u mapi)", () => {
  const indeks = `${BASE["docs/README.md"]}- [S](plan/2026-10-10-v2-sloj1.md)\n`;
  const plan = "# Sloj 1\n\n**Status:** ⏸️ PAUZIRAN\n\n- [T1](2026-10-10-v2-sloj1/task-01-a.md)\n";
  assertFails(
    { "docs/README.md": indeks, "docs/plan/2026-10-10-v2-sloj1.md": `${plan}${RED(1201)}` },
    "PREKO 1200 REDAKA  docs/plan/2026-10-10-v2-sloj1.md",
  );
  assertFails(
    {
      "docs/README.md": indeks,
      "docs/plan/2026-10-10-v2-sloj1.md": plan,
      "docs/plan/2026-10-10-v2-sloj1/task-01-a.md": `# T1\n${RED(1201)}`,
    },
    "PREKO 1200 REDAKA  docs/plan/2026-10-10-v2-sloj1/task-01-a.md",
  );
  // isti dio s 400 redaka: preko granice 300, ali iznimka vrijedi i za mapu
  const r = run({
    "docs/README.md": indeks,
    "docs/plan/2026-10-10-v2-sloj1.md": plan,
    "docs/plan/2026-10-10-v2-sloj1/task-01-a.md": `# T1\n${RED(400)}`,
  });
  assert.equal(r.code, 0, r.out);
});

test("9) X.md uz mapu X/ ima granicu indeksa (200)", () => {
  assertFails(
    { ...PODINDEKS, "docs/product/ARHITEKTURA.md": `# Arhitektura\nDIO_A.md\n${RED(200)}` },
    "docs/product/ARHITEKTURA.md  202 redaka > granica 200 (indeks)",
  );
});

test("9) prijelazna iznimka je izuzeta od granice", () => {
  const r = run({
    "docs/README.md": `${BASE["docs/README.md"]}- [S](plan/2026-10-10-v2-sloj1.md)\n`,
    "docs/plan/2026-10-10-v2-sloj1.md": `# Sloj 1\n\n**Status:** ⏸️ PAUZIRAN\n${RED(400)}`,
  });
  assert.equal(r.code, 0, r.out);
});

// Pravi indeks ADR-a za fixture: isti kod koji ga generira (scripts/adr.mjs).
const ADR_DATOTEKA = "# ADR-001 — Prvi\n**Datum:** 2026-10-01 · **Status:** ✅ ODOBRENO\n\nTijelo.\n";
function adrFixture(dodatno = {}) {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "check-docs-adr-"));
  try {
    fs.mkdirSync(path.join(tmp, "docs/records/DECISIONS"), { recursive: true });
    fs.writeFileSync(path.join(tmp, "docs/records/DECISIONS/ADR-001-prvi.md"), ADR_DATOTEKA);
    return {
      "docs/README.md": `${BASE["docs/README.md"]}- DECISIONS.md\n`,
      "docs/records/DECISIONS.md": indeksAdr(tmp),
      "docs/records/DECISIONS/ADR-001-prvi.md": ADR_DATOTEKA,
      ...dodatno,
    };
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

test("10) svjež generirani indeks ADR-a prolazi", () => {
  const r = run(adrFixture());
  assert.equal(r.code, 0, r.out);
});

test("10) zastario/ručno uređen indeks ADR-a pada", () => {
  const fx = adrFixture();
  assertFails(
    { ...fx, "docs/records/DECISIONS.md": `${fx["docs/records/DECISIONS.md"]}ručni redak\n` },
    "ADR INDEKS NIJE SVJEZ  docs/records/DECISIONS.md",
  );
});

test("10) samo DECISIONS.md bez mape pada; bez ičega šuti", () => {
  assertFails(
    { "docs/README.md": `${BASE["docs/README.md"]}- DECISIONS.md\n`, "docs/records/DECISIONS.md": "# ADR\n" },
    "ADR JOS NISU PO DATOTEKAMA",
  );
  assert.equal(run().code, 0);
});

test("10) loše ime ADR datoteke i dupli broj padaju", () => {
  const fx = adrFixture();
  assertFails(
    {
      ...fx,
      "docs/records/DECISIONS/ADR-2-kratki.md": "# ADR-002 — X\n",
      "docs/records/DECISIONS.md": `${fx["docs/records/DECISIONS.md"]}ADR-2-kratki.md\n`,
    },
    "ADR LOSE IME  docs/records/DECISIONS/ADR-2-kratki.md",
  );
  assertFails(
    {
      ...fx,
      "docs/records/DECISIONS/ADR-001-drugi.md": ADR_DATOTEKA,
      "docs/records/DECISIONS.md": `${fx["docs/records/DECISIONS.md"]}ADR-001-drugi.md\n`,
    },
    "ADR DUPLI BROJ  001",
  );
});

test("11) poveznica na DECISIONS.md#adr- pada; u kodu i u iznimci prolazi", () => {
  assertFails(
    {
      "docs/product/VIZIJA.md":
        "# V\n\n[odluka](../records/PROGRESS.md) [adr](../records/DECISIONS.md#adr-012)\n",
      ...adrFixture(),
    },
    "STARA ADR POVEZNICA  docs/product/VIZIJA.md  ->  ../records/DECISIONS.md#adr-012",
  );
  const r = run({ "docs/product/VIZIJA.md": "# V\n\nPiše `[a](../records/DECISIONS.md#adr-012)` u kodu.\n" });
  assert.equal(r.code, 0, r.out);
  const iznimka = run({
    ...adrFixture(),
    "docs/README.md": `${BASE["docs/README.md"]}- DECISIONS.md\n- MODEL_V2.md\n`,
    "docs/architecture/MODEL_V2.md": "# M\n\n[adr](../records/DECISIONS.md#adr-012)\n",
  });
  assert.equal(iznimka.code, 0, iznimka.out);
});
