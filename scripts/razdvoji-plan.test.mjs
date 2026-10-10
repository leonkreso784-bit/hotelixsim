// Test alata razdvoji-plan: razdvajanje velikog plana na pregled + datoteku po tasku,
// uz dokaz da spajanje vraća izvornik bajt po bajt.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { razdvoji, spoji } from "./razdvoji-plan.mjs";

const SCRIPT = path.join(import.meta.dirname, "razdvoji-plan.mjs");

const PLAN = [
  "# Plan X",
  "",
  "**Goal:** nešto.",
  "",
  "### Task 1: Prvi korak, i još nešto",
  "",
  "Tekst prvog.",
  "",
  "### Task 2: Šđčćž — drugi",
  "",
  "Tekst drugog.",
  "",
  "## Ukupno",
  "",
  "Kraj plana.",
  "",
].join("\n");

function vrati(text, ime = "plan") {
  const { pregled, taskovi } = razdvoji(text, ime);
  return spoji(pregled, taskovi);
}

test("razdvoji: pregled, tablica i imena datoteka", () => {
  const { pregled, taskovi } = razdvoji(PLAN, "plan");
  assert.deepEqual(
    taskovi.map((t) => t.datoteka),
    ["task-01-prvi-korak-i-jos-nesto.md", "task-02-sdccz-drugi.md"],
  );
  assert.ok(taskovi[0].tekst.startsWith("### Task 1: Prvi korak, i još nešto\n"));
  assert.ok(pregled.startsWith("# Plan X\n\n**Goal:** nešto.\n\n## Taskovi\n\n| N | Task |\n|---|---|\n"));
  assert.ok(
    pregled.includes("| 1 | [Task 1: Prvi korak, i još nešto](plan/task-01-prvi-korak-i-jos-nesto.md) |\n"),
  );
  assert.ok(pregled.includes("| 2 | [Task 2: Šđčćž — drugi](plan/task-02-sdccz-drugi.md) |\n"));
  assert.ok(pregled.endsWith("## Ukupno\n\nKraj plana.\n"));
  assert.ok(!pregled.includes("Tekst prvog"));
});

test("razdvoji + spoji: izvornik bajt po bajt", () => {
  assert.equal(vrati(PLAN), PLAN);
});

test("ograda: ### Task unutar ograde ne reže, ## unutar ograde od 4 znaka ne zatvara zadnji task", () => {
  const text = [
    "# P",
    "",
    "### Task 1: A",
    "",
    "```md",
    "### Task 9: predložak",
    "```",
    "",
    "### Task 2: B",
    "",
    "````md",
    "```",
    "## Lokalni razvoj",
    "```",
    "## Lokalni razvoj",
    "````",
    "",
    "još B",
    "",
    "## Ukupno",
    "kraj",
    "",
  ].join("\n");
  const { pregled, taskovi } = razdvoji(text, "p");
  assert.equal(taskovi.length, 2);
  assert.ok(taskovi[0].tekst.includes("### Task 9: predložak"));
  assert.ok(taskovi[1].tekst.includes("još B"));
  assert.ok(!pregled.includes("Lokalni razvoj"));
  assert.ok(pregled.endsWith("## Ukupno\nkraj\n"));
  assert.equal(spoji(pregled, taskovi), text);
});

test("zadnji task završava na prvom ## naslovu, ostatak natrag u pregled", () => {
  const { pregled, taskovi } = razdvoji(PLAN, "plan");
  assert.ok(!taskovi[1].tekst.includes("Ukupno"));
  assert.ok(taskovi[1].tekst.endsWith("Tekst drugog.\n\n"));
  assert.ok(pregled.includes("## Ukupno"));
});

test("bez ## iza zadnjeg taska: zadnji task ide do kraja datoteke", () => {
  const text = "# P\n\n### Task 1: A\n\na\n\n### Task 2: B\n\nb\n";
  const { pregled, taskovi } = razdvoji(text, "p");
  assert.equal(taskovi[1].tekst, "### Task 2: B\n\nb\n");
  assert.ok(pregled.endsWith("|\n\n"));
  assert.equal(spoji(pregled, taskovi), text);
});

test("datoteka bez završnog novog retka", () => {
  const text = "# P\n\n### Task 1: A\n\nbez kraja";
  assert.equal(vrati(text), text);
  const t2 = "# P\n\n### Task 1: A\n\na\n## Kraj\nbez kraja";
  assert.equal(vrati(t2), t2);
});

test("CRLF: bajt po bajt", () => {
  const crlf = PLAN.replaceAll("\n", "\r\n");
  const { pregled, taskovi } = razdvoji(crlf, "plan");
  assert.equal(taskovi.length, 2);
  assert.ok(pregled.includes("## Taskovi\r\n\r\n| N | Task |\r\n"));
  assert.equal(spoji(pregled, taskovi), crlf);
});

test("poveznice: ../x.md postaje ../../x.md, sidra, http i kod netaknuti", () => {
  const text = [
    "# P",
    "",
    "### Task 1: A",
    "",
    "Vidi [x](../x.md), [y](y.md#s), [a](#b), [h](https://e.com/a.md), ![s](slike/s.png) i `[k](k.md)`.",
    "",
    "```",
    "[o](o.md)",
    "```",
    "",
  ].join("\n");
  const { pregled, taskovi } = razdvoji(text, "p");
  const t = taskovi[0].tekst;
  assert.ok(t.includes("[x](../../x.md)"));
  assert.ok(t.includes("[y](../y.md#s)"));
  assert.ok(t.includes("[a](#b)"));
  assert.ok(t.includes("[h](https://e.com/a.md)"));
  assert.ok(t.includes("![s](../slike/s.png)"));
  assert.ok(t.includes("`[k](k.md)`"));
  assert.ok(t.includes("[o](o.md)"));
  assert.equal(spoji(pregled, taskovi), text);
});

test("poveznice u pregledu (izvan taskova) se ne diraju", () => {
  const text = "# P\n\n[z](../z.md)\n\n### Task 1: A\n\na\n\n## Kraj\n\n[w](w.md)\n";
  const { pregled } = razdvoji(text, "p");
  assert.ok(pregled.includes("[z](../z.md)"));
  assert.ok(pregled.includes("[w](w.md)"));
});

test("broj taska ≥ 100 bez skraćivanja, naslov razine ## i prazan slug", () => {
  const text = "### Task 100: Sto\n\na\n\n## Task 101\n\nb\n";
  const { taskovi } = razdvoji(text, "p");
  assert.deepEqual(
    taskovi.map((t) => t.datoteka),
    ["task-100-sto.md", "task-101.md"],
  );
});

test("slug skida Task N: i najviše šest riječi", () => {
  const { taskovi } = razdvoji("### Task 3: a b c d e f g h\n\nx\n", "p");
  assert.equal(taskovi[0].datoteka, "task-03-a-b-c-d-e-f.md");
});

test("dva taska s istim imenom datoteke bacaju grešku", () => {
  assert.throws(() => razdvoji("### Task 1: A\n\n### Task 1: A\n", "p"), /isto ime/);
});

test("bez taskova: pregled je izvornik", () => {
  const text = "# P\n\nnema taskova\n";
  const { pregled, taskovi } = razdvoji(text, "p");
  assert.equal(pregled, text);
  assert.deepEqual(taskovi, []);
  assert.equal(spoji(pregled, taskovi), text);
});

test("pipe u naslovu ne lomi tablicu i spajanje ostaje točno", () => {
  const text = "### Task 1: a | b\n\nx\n";
  const { pregled, taskovi } = razdvoji(text, "p");
  assert.ok(pregled.includes("a \\| b"));
  assert.equal(spoji(pregled, taskovi), text);
});

// ----- CLI -----

function cli(args) {
  return spawnSync(process.execPath, [SCRIPT, ...args], { encoding: "utf8" });
}

function privremena() {
  return fs.mkdtempSync(path.join(os.tmpdir(), "razdvoji-"));
}

test("CLI: razdvaja, zapisuje mapu, a spoji s diska daje izvornik; ponovno pokretanje odbija", () => {
  const dir = privremena();
  try {
    const plan = path.join(dir, "plan.md");
    fs.writeFileSync(plan, PLAN);
    const r = cli([plan]);
    assert.equal(r.status, 0, r.stderr);
    const datoteke = fs.readdirSync(path.join(dir, "plan")).sort();
    assert.deepEqual(datoteke, ["task-01-prvi-korak-i-jos-nesto.md", "task-02-sdccz-drugi.md"]);
    const pregled = fs.readFileSync(plan, "utf8");
    assert.ok(pregled.includes("## Taskovi"));
    const taskovi = datoteke.map((d) => ({
      datoteka: d,
      tekst: fs.readFileSync(path.join(dir, "plan", d), "utf8"),
    }));
    assert.equal(spoji(pregled, taskovi), PLAN);

    const ponovno = cli([plan]);
    assert.equal(ponovno.status, 1);
    assert.equal(fs.readFileSync(plan, "utf8"), pregled);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test("CLI --provjeri: ne mijenja ništa u mapi i prolazi", () => {
  const dir = privremena();
  try {
    const plan = path.join(dir, "plan.md");
    fs.writeFileSync(plan, PLAN);
    const r = cli([plan, "--provjeri"]);
    assert.equal(r.status, 0, r.stderr);
    assert.equal(fs.readFileSync(plan, "utf8"), PLAN);
    assert.deepEqual(fs.readdirSync(dir), ["plan.md"]);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

test("CLI: nepostojeća datoteka i prazna naredba daju exit 1", () => {
  assert.equal(cli([]).status, 1);
  assert.equal(cli([path.join(os.tmpdir(), "nema-me-ovdje.md")]).status, 1);
});

test("CLI: neprazna mapa odbija i ne dira ništa", () => {
  const dir = privremena();
  try {
    const plan = path.join(dir, "plan.md");
    fs.writeFileSync(plan, PLAN);
    fs.mkdirSync(path.join(dir, "plan"));
    fs.writeFileSync(path.join(dir, "plan", "x.md"), "x");
    const r = cli([plan]);
    assert.equal(r.status, 1);
    assert.equal(fs.readFileSync(plan, "utf8"), PLAN);
    assert.deepEqual(fs.readdirSync(path.join(dir, "plan")), ["x.md"]);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});
