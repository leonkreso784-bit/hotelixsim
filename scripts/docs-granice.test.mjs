// Test modula granica veličine: svaka vrsta iz tablice, iznimke i brojanje redaka.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  brojRedaka,
  granicaZa,
  IZUZECI_TVRDE_GRANICE,
  jeIznimka,
  jeIzuzetOdTvrde,
  PRIJELAZNE_IZNIMKE,
  TVRDA_GRANICA,
} from "./docs-granice.mjs";

test("brojRedaka broji kao wc -l (samo \\n)", () => {
  assert.equal(brojRedaka(""), 0);
  assert.equal(brojRedaka("a\n"), 1);
  assert.equal(brojRedaka("a\nb\n"), 2);
  assert.equal(brojRedaka("a\nb"), 1); // wc -l ne broji redak bez završnog \n
});

test("brojRedaka ne broji \\r", () => {
  assert.equal(brojRedaka("a\r\nb\r\n"), 2);
  assert.equal(brojRedaka("a\r\nb\r\n"), brojRedaka("a\nb\n"));
});

test("predaja: docs/SLJEDECA_SESIJA.md = 60", () => {
  assert.deepEqual(granicaZa("docs/SLJEDECA_SESIJA.md", false), { vrsta: "predaja", redaka: 60 });
});

test("adr: docs/records/DECISIONS/*.md = 120", () => {
  assert.deepEqual(granicaZa("docs/records/DECISIONS/ADR-015-nesto.md", false), {
    vrsta: "adr",
    redaka: 120,
  });
});

test("indeks: docs/README.md = 200", () => {
  assert.deepEqual(granicaZa("docs/README.md", false), { vrsta: "indeks", redaka: 200 });
});

test("indeks: X.md uz mapu X/ = 200, bez mape = živi 300", () => {
  assert.deepEqual(granicaZa("docs/architecture/ARHITEKTURA.md", true), { vrsta: "indeks", redaka: 200 });
  assert.deepEqual(granicaZa("docs/architecture/ARHITEKTURA.md", false), { vrsta: "zivi", redaka: 300 });
});

test("živi: sve ostalo pod docs/ = 300", () => {
  assert.deepEqual(granicaZa("docs/plan/ROADMAP.md", false), { vrsta: "zivi", redaka: 300 });
  assert.deepEqual(granicaZa("docs/plan/2026-10-10-x/task-01-y.md", false), { vrsta: "zivi", redaka: 300 });
});

test("arhiva: docs/archive/** = 1000, i kad ima mapu istog imena", () => {
  assert.deepEqual(granicaZa("docs/archive/2026-10-10-ekrani.md", false), { vrsta: "arhiva", redaka: 1000 });
  assert.deepEqual(granicaZa("docs/archive/2026-10-10-ekrani.md", true), { vrsta: "arhiva", redaka: 1000 });
  assert.deepEqual(granicaZa("docs/archive/2026-10-10-ekrani/task-01-a.md", false), {
    vrsta: "arhiva",
    redaka: 1000,
  });
});

test("izvan docs/ nema granice (null)", () => {
  assert.equal(granicaZa("README.md", false), null);
  assert.equal(granicaZa("CLAUDE.md", false), null);
  assert.equal(granicaZa("crates/x/README.md", false), null);
});

test("prijelazne iznimke: točno dvije, s razlogom", () => {
  assert.deepEqual(Object.keys(PRIJELAZNE_IZNIMKE).sort(), [
    "docs/architecture/MODEL_V2.md",
    "docs/plan/2026-10-10-v2-sloj1.md",
  ]);
  for (const razlog of Object.values(PRIJELAZNE_IZNIMKE)) assert.ok(razlog.length > 10);
  assert.equal(jeIznimka("docs/plan/2026-10-10-v2-sloj1.md"), true);
  assert.equal(jeIznimka("docs/architecture/MODEL_V2.md"), true);
  assert.equal(jeIznimka("docs/plan/ROADMAP.md"), false);
  // razdvojena iznimka: vrijedi i za dijelove u mapi istog imena, ne i za slično ime
  assert.equal(jeIznimka("docs/plan/2026-10-10-v2-sloj1/task-02-x.md"), true);
  assert.equal(jeIznimka("docs/plan/2026-10-10-v2-sloj1-b.md"), false);
});

test("tvrda granica: 1000 redaka, izuzeci samo izrijekom (danas nijedan)", () => {
  assert.equal(TVRDA_GRANICA, 1000);
  assert.deepEqual(Object.keys(IZUZECI_TVRDE_GRANICE), []);
  assert.equal(jeIzuzetOdTvrde("docs/plan/2026-10-10-v2-sloj1.md"), false);
});

test("check-docs --popis: silazno, ❌ uz prekoračenje, exit 0, korijen kao argument", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "popis-"));
  try {
    const piši = (r, redaka) => {
      fs.mkdirSync(path.dirname(path.join(dir, r)), { recursive: true });
      fs.writeFileSync(path.join(dir, r), "x\n".repeat(redaka));
    };
    piši("docs/README.md", 5);
    piši("docs/plan/VELIKI.md", 301);
    piši("docs/plan/MALI.md", 10);
    piši("docs/ARH.md", 50);
    piši("docs/ARH/dio.md", 3); // ARH.md ima mapu istog imena -> indeks 200
    piši("CLAUDE.md", 999); // izvan docs/ ne ulazi u popis
    const env = Object.fromEntries(Object.entries(process.env).filter(([k]) => !k.startsWith("GIT_")));
    const skripta = path.join(import.meta.dirname, "check-docs.mjs");
    for (const argumenti of [
      [skripta, dir, "--popis"],
      [skripta, "--popis", dir],
    ]) {
      const r = spawnSync(process.execPath, argumenti, { env, encoding: "utf8" });
      assert.equal(r.status, 0, r.stdout + r.stderr);
      const retci = r.stdout.split("\n");
      const poRedu = retci.filter((l) => /docs\//.test(l)).map((l) => l.trim().split(/\s+/).at(3));
      assert.deepEqual(poRedu, [
        "docs/plan/VELIKI.md",
        "docs/ARH.md",
        "docs/plan/MALI.md",
        "docs/README.md",
        "docs/ARH/dio.md",
      ]);
      assert.match(
        retci.find((l) => l.includes("VELIKI.md")),
        /301 +300 +zivi .*❌/,
      );
      assert.match(
        retci.find((l) => l.includes("docs/ARH.md")),
        /50 +200 +indeks/,
      );
      assert.ok(!r.stdout.includes("CLAUDE.md"));
      assert.equal(retci.filter((l) => l.includes("❌")).length, 1);
    }
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});
