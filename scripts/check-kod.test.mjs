import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { DUG, provjeri } from "./check-kod.mjs";

const SCHEMA_DUG = DUG["crates/api/src/schema.rs"];

const RED = (n, eol = "\n") => "x".concat(eol).repeat(n);
function uMapi(datoteke) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "check-kod-"));
  for (const [ime, sadrzaj] of Object.entries(datoteke)) {
    fs.mkdirSync(path.dirname(path.join(dir, ime)), { recursive: true });
    fs.writeFileSync(path.join(dir, ime), sadrzaj);
  }
  return dir;
}
const sMapom = (datoteke, fn) => {
  const d = uMapi(datoteke);
  try {
    return fn(provjeri(d));
  } finally {
    fs.rmSync(d, { recursive: true, force: true });
  }
};

test("1000 prolazi, 1001 pada; CRLF broji isto", () =>
  sMapom({ "a/ok.rs": RED(1000), "a/crlf.ts": RED(1000, "\r\n"), "a/velik.tsx": RED(1001) }, (r) => {
    assert.deepEqual(r.greske, ["PREKO 1000 REDAKA  a/velik.tsx  1001"]);
  }));
test("601 daje upozorenje, ne grešku", () =>
  sMapom({ "b.mjs": RED(601) }, (r) => {
    assert.deepEqual(r.greske, []);
    assert.ok(r.upozorenja.includes("IZNAD 600  b.mjs  601"));
  }));
test("ignorirane mape i generirani schema.d.ts ne ruše", () =>
  sMapom(
    {
      "node_modules/x/i.js": RED(5000),
      "target/g.rs": RED(5000),
      "apps/web/dist/a.js": RED(5000),
      "packages/api-client/src/schema.d.ts": RED(3000),
      "README.md": RED(5000),
    },
    (r) => {
      assert.deepEqual(r.greske, []);
    },
  ));
test("dug: do granice upozorava, preko pada, manji traži spuštanje, nestao je zastario unos", () => {
  sMapom({ "crates/api/src/schema.rs": RED(SCHEMA_DUG + 1) }, (r) =>
    assert.ok(r.greske.includes(`DUG NARASTAO  crates/api/src/schema.rs  ${SCHEMA_DUG + 1} > ${SCHEMA_DUG}`)),
  );
  sMapom({ "crates/api/src/schema.rs": RED(1600) }, (r) => {
    assert.deepEqual(r.greske, []);
    assert.ok(r.upozorenja.includes("SPUSTI GRANICU DUGA  crates/api/src/schema.rs  na 1600"));
    assert.ok(r.upozorenja.includes("ZASTARIO UNOS DUGA  crates/game/tests/mentor.rs"));
  });
});
