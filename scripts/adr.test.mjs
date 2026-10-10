// Test alata `adr`: razdvajanje DECISIONS.md u datoteke, generirani indeks i prepis poveznica.
// Fixture su privremene mape (bez gita — popis datoteka ide rezervnim walkom).
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { indeksAdr, prepisiPoveznice, procitajAdr, razdvoji, slug } from "./adr.mjs";

const SCRIPT = path.join(import.meta.dirname, "adr.mjs");

const STARI = `# Arhitektonske i proizvodne odluke (ADR)

Uvod koji se gubi.

---

## ADR-002 — Šuma đavola: čuvaj ćirilicu
**Datum:** 2026-10-04 · **Status:** ✅ **Odlučio Leon** (anketa) · **Vezano:** [ADR-001](#adr-001),
[MODEL](../architecture/MODEL_V2.md) §3

Tijelo druge odluke s [slikom](../x/a.png) i \`[kod](../ne.md)\`.

\`\`\`md
[u ogradi](../ostaje.md)
\`\`\`

---

## ADR-001 — Zaseban projekt, bez Sokratova koda
**Datum:** 2026-10-03 · **Status:** 🟨 ČEKA LEONOV PREGLED (Claude) · **Vezano:** [Web](https://x.hr/a.md)

Prva odluka, vidi [ARHITEKTURA](../architecture/ARHITEKTURA.md).
`;

function napraviKorijen(datoteke) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "adr-"));
  for (const [ime, sadrzaj] of Object.entries(datoteke)) {
    const abs = path.join(dir, ime);
    fs.mkdirSync(path.dirname(abs), { recursive: true });
    fs.writeFileSync(abs, sadrzaj);
  }
  return dir;
}
const cisto = (dir) => fs.rmSync(dir, { recursive: true, force: true });
const cist = () => Object.fromEntries(Object.entries(process.env).filter(([k]) => !k.startsWith("GIT_")));

const MAPA = new Map([
  ["001", "ADR-001-zaseban-projekt-bez-sokratova-koda.md"],
  ["012", "ADR-012-ime.md"],
]);

test("slug: dijakritika i ograničenje na 6 riječi", () => {
  assert.equal(slug("Šuma đavola: čuvaj ćirilicu žarko"), "suma-davola-cuvaj-cirilicu-zarko");
  assert.equal(slug("Jedan dva tri četiri pet šest sedam"), "jedan-dva-tri-cetiri-pet-sest");
});

test("razdvoji: jedan blok = jedna datoteka, # naslov, jedan ../ dublje, ograde doslovne", () => {
  const dir = napraviKorijen({ "docs/records/DECISIONS.md": STARI });
  try {
    const n = razdvoji(dir);
    assert.equal(n, 2);
    const mapa = path.join(dir, "docs/records/DECISIONS");
    assert.deepEqual(fs.readdirSync(mapa).sort(), [
      "ADR-001-zaseban-projekt-bez-sokratova-koda.md",
      "ADR-002-suma-davola-cuvaj-cirilicu.md",
    ]);
    const dva = fs.readFileSync(path.join(mapa, "ADR-002-suma-davola-cuvaj-cirilicu.md"), "utf8");
    assert.ok(dva.startsWith("# ADR-002 — Šuma đavola: čuvaj ćirilicu\n**Datum:** 2026-10-04"));
    assert.ok(dva.includes("[MODEL](../../architecture/MODEL_V2.md) §3"));
    assert.ok(dva.includes("[slikom](../../x/a.png)"));
    assert.ok(dva.includes("[ADR-001](#adr-001)"), "sidro ostaje za `poveznice`");
    assert.ok(dva.includes("`[kod](../ne.md)`"), "inline kod doslovan");
    assert.ok(dva.includes("[u ogradi](../ostaje.md)"), "ograda doslovna");
    assert.ok(!dva.includes("\n---"), "separator nije dio bloka");
    assert.ok(dva.endsWith("```\n"), "završava jednim \\n bez praznih redaka");
    const jedan = fs.readFileSync(path.join(mapa, "ADR-001-zaseban-projekt-bez-sokratova-koda.md"), "utf8");
    assert.ok(jedan.includes("[Web](https://x.hr/a.md)"), "vanjski cilj netaknut");
    assert.ok(jedan.includes("(../../architecture/ARHITEKTURA.md)"));
  } finally {
    cisto(dir);
  }
});

test("razdvoji: ## ADR u ogradi nije blok, a nema blokova je greška", () => {
  const sOgradom = `${STARI}\n\`\`\`md\n## ADR-099 — Predložak\n\`\`\`\n`;
  const dir = napraviKorijen({ "docs/records/DECISIONS.md": sOgradom });
  try {
    assert.equal(razdvoji(dir), 2);
    const prazan = napraviKorijen({ "docs/records/DECISIONS.md": "# Indeks\n" });
    try {
      assert.throws(() => razdvoji(prazan), /nema ADR blokova/);
    } finally {
      cisto(prazan);
    }
  } finally {
    cisto(dir);
  }
});

test("procitajAdr: broj, naslov, status (do · ili (, bez **), datum, datoteka", () => {
  const dir = napraviKorijen({ "docs/records/DECISIONS.md": STARI });
  try {
    razdvoji(dir);
    const adr = procitajAdr(dir);
    assert.deepEqual(
      adr.map((a) => a.broj),
      ["001", "002"],
    );
    assert.deepEqual(adr[1], {
      broj: "002",
      naslov: "Šuma đavola: čuvaj ćirilicu",
      status: "✅ Odlučio Leon",
      datum: "2026-10-04",
      datoteka: "ADR-002-suma-davola-cuvaj-cirilicu.md",
    });
    assert.equal(adr[0].status, "🟨 ČEKA LEONOV PREGLED");
  } finally {
    cisto(dir);
  }
});

test("indeksAdr: deterministički, najnoviji gore, sidro i poveznica na datoteku, bez \\r", () => {
  const dir = napraviKorijen({ "docs/records/DECISIONS.md": STARI });
  try {
    razdvoji(dir);
    const a = indeksAdr(dir);
    assert.equal(a, indeksAdr(dir));
    assert.ok(!a.includes("\r"));
    assert.ok(a.endsWith("|\n"));
    assert.ok(a.startsWith("# Arhitektonske i proizvodne odluke (ADR)\n"));
    const redci = a.split("\n").filter((r) => r.startsWith("| ") && r.includes("adr-0"));
    assert.equal(redci.length, 2);
    assert.ok(
      redci[0].includes('<a id="adr-002"></a>[ADR-002](DECISIONS/ADR-002-suma-davola-cuvaj-cirilicu.md)'),
    );
    assert.ok(redci[0].includes("Šuma đavola: čuvaj ćirilicu"));
    assert.ok(redci[0].includes("| ✅ Odlučio Leon | 2026-10-04 |"));
    assert.ok(redci[1].includes("[ADR-001](DECISIONS/ADR-001-"));
    assert.ok(a.includes("ne uređuj ručno"));
    assert.ok(a.includes("docs:adr"));
  } finally {
    cisto(dir);
  }
});

test("prepisiPoveznice: putanja iz CLAUDE.md, docs/product/ i docs/archive/x/", () => {
  const t = (p, ciljPrefiks) => prepisiPoveznice(`[a](${ciljPrefiks}DECISIONS.md#adr-012)`, p, MAPA);
  assert.equal(t("CLAUDE.md", "docs/records/"), "[a](docs/records/DECISIONS/ADR-012-ime.md)");
  assert.equal(t("docs/product/X.md", "../records/"), "[a](../records/DECISIONS/ADR-012-ime.md)");
  assert.equal(t("docs/archive/x/Y.md", "../../records/"), "[a](../../records/DECISIONS/ADR-012-ime.md)");
  assert.equal(t("docs/records/PROGRESS.md", ""), "[a](DECISIONS/ADR-012-ime.md)");
  assert.equal(t("docs/records/PROGRESS.md", "./"), "[a](DECISIONS/ADR-012-ime.md)");
  assert.equal(t("README.md", "./docs/records/"), "[a](docs/records/DECISIONS/ADR-012-ime.md)");
});

test("prepisiPoveznice: naslov poveznice, ograde i inline kod ostaju, tuđi DECISIONS.md ne", () => {
  const ulaz = [
    '[a](../records/DECISIONS.md#adr-012 "naslov") i `[b](../records/DECISIONS.md#adr-012)`',
    "```",
    "[c](../records/DECISIONS.md#adr-012)",
    "```",
    "[d](../architecture/DECISIONS.md#adr-012) [e](../records/DECISIONS.md) [f](https://x/DECISIONS.md#adr-012)",
  ].join("\n");
  const izlaz = prepisiPoveznice(ulaz, "docs/product/X.md", MAPA).split("\n");
  assert.equal(
    izlaz[0],
    '[a](../records/DECISIONS/ADR-012-ime.md "naslov") i `[b](../records/DECISIONS.md#adr-012)`',
  );
  assert.equal(izlaz[2], "[c](../records/DECISIONS.md#adr-012)");
  assert.equal(
    izlaz[4],
    "[d](../architecture/DECISIONS.md#adr-012) [e](../records/DECISIONS.md) [f](https://x/DECISIONS.md#adr-012)",
  );
});

test("prepisiPoveznice: u ADR datoteci (#adr-0NN) postaje sestrinska datoteka", () => {
  const ulaz = "Vidi [ADR-001](#adr-001) t. 4, `[k](#adr-001)` i [vani](#drugo).\n```\n[o](#adr-001)\n```\n";
  const izlaz = prepisiPoveznice(ulaz, "docs/records/DECISIONS/ADR-012-ime.md", MAPA);
  assert.equal(
    izlaz,
    "Vidi [ADR-001](ADR-001-zaseban-projekt-bez-sokratova-koda.md) t. 4, `[k](#adr-001)` i [vani](#drugo).\n```\n[o](#adr-001)\n```\n",
  );
  // izvan ADR datoteka (#adr-0NN) nije naš posao
  assert.equal(prepisiPoveznice("[a](#adr-001)", "docs/product/X.md", MAPA), "[a](#adr-001)");
});

test("prepisiPoveznice: idempotentno i CRLF-siguran", () => {
  const ulaz = "[a](../records/DECISIONS.md#adr-012)\r\n[b](#adr-001)\r\n";
  const jednom = prepisiPoveznice(ulaz, "docs/product/X.md", MAPA);
  assert.equal(prepisiPoveznice(jednom, "docs/product/X.md", MAPA), jednom);
  assert.ok(jednom.includes("\r\n"));
  const uAdr = "[b](#adr-001)\r\n";
  const adrJednom = prepisiPoveznice(uAdr, "docs/records/DECISIONS/ADR-012-ime.md", MAPA);
  assert.equal(prepisiPoveznice(adrJednom, "docs/records/DECISIONS/ADR-012-ime.md", MAPA), adrJednom);
});

test("prepisiPoveznice: nepoznat ADR se ne dira i prijavi upozorenjem", () => {
  const upozorenja = [];
  const ulaz = "[a](../records/DECISIONS.md#adr-099)";
  assert.equal(prepisiPoveznice(ulaz, "docs/product/X.md", MAPA, upozorenja), ulaz);
  assert.equal(upozorenja.length, 1);
  assert.match(upozorenja[0], /ADR-099/);
});

test("CLI: razdvoji pa indeks pa poveznice; drugi prolaz ne mijenja ništa; --provjeri", () => {
  const dir = napraviKorijen({
    "docs/records/DECISIONS.md": STARI,
    "CLAUDE.md": "Vidi [odluku](docs/records/DECISIONS.md#adr-002).\n",
    "docs/product/X.md": "[a](../records/DECISIONS.md#adr-001) i [b](../records/DECISIONS.md#adr-077)\n",
    "docs/plan/2026-10-10-v2-sloj1.md": "[a](../records/DECISIONS.md#adr-001)\n",
  });
  const pokreni = (...args) =>
    spawnSync(process.execPath, [SCRIPT, ...args, `--korijen=${dir}`], { env: cist(), encoding: "utf8" });
  try {
    assert.equal(pokreni("razdvoji").status, 0);
    assert.equal(pokreni("indeks", "--provjeri").status, 1, "indeks još nije generiran");
    assert.equal(pokreni("indeks").status, 0);
    assert.equal(pokreni("indeks", "--provjeri").status, 0);
    assert.equal(fs.readFileSync(path.join(dir, "docs/records/DECISIONS.md"), "utf8"), indeksAdr(dir));

    const p = pokreni("poveznice");
    assert.equal(p.status, 1, "ADR-077 ne postoji");
    assert.match(p.stderr, /ADR-077/);
    const cl = fs.readFileSync(path.join(dir, "CLAUDE.md"), "utf8");
    assert.equal(cl, "Vidi [odluku](docs/records/DECISIONS/ADR-002-suma-davola-cuvaj-cirilicu.md).\n");
    const x = fs.readFileSync(path.join(dir, "docs/product/X.md"), "utf8");
    assert.ok(x.startsWith("[a](../records/DECISIONS/ADR-001-zaseban-projekt-bez-sokratova-koda.md)"));
    assert.ok(x.includes("[b](../records/DECISIONS.md#adr-077)"));
    const dva = fs.readFileSync(
      path.join(dir, "docs/records/DECISIONS/ADR-002-suma-davola-cuvaj-cirilicu.md"),
      "utf8",
    );
    assert.ok(dva.includes("[ADR-001](ADR-001-zaseban-projekt-bez-sokratova-koda.md)"));
    assert.equal(
      fs.readFileSync(path.join(dir, "docs/plan/2026-10-10-v2-sloj1.md"), "utf8"),
      "[a](../records/DECISIONS.md#adr-001)\n",
      "prijelazna iznimka nedirnuta",
    );

    const prije = [cl, x, dva];
    pokreni("poveznice");
    const poslije = [
      fs.readFileSync(path.join(dir, "CLAUDE.md"), "utf8"),
      fs.readFileSync(path.join(dir, "docs/product/X.md"), "utf8"),
      fs.readFileSync(path.join(dir, "docs/records/DECISIONS/ADR-002-suma-davola-cuvaj-cirilicu.md"), "utf8"),
    ];
    assert.deepEqual(poslije, prije);
  } finally {
    cisto(dir);
  }
});
