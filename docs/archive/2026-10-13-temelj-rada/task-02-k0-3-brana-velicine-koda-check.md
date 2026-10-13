### Task 2 (K0-3): Brana veličine koda `check:kod`

**Files:**
- Create: `scripts/check-kod.mjs`, `scripts/check-kod.test.mjs`
- Modify: `package.json` (`"check:kod": "node scripts/check-kod.mjs"`), `.github/workflows/ci.yml` (posao `web`, korak
  `- run: pnpm check:kod` odmah iza `pnpm test:docs`), `docs/records/BACKLOG.md` (stavka [alati] „kod preko 1000" → riješeno, obriši)

**Interfaces:**
- Consumes: `brojRedaka(text)` iz `scripts/docs-granice.mjs`.
- Produces: `export const UPOZORENJE = 600; export const TVRDA = 1000; export const DUG = { "<put>": <najviše redaka> };`
  `export const IZUZECI = { "<put>": "<razlog>" };` i `export function provjeri(korijen) → { greske: string[], upozorenja: string[] }`.
  Korijen: argument ili env `CHECK_KOD_ROOT`, inače mapa iznad skripte. Izlaz 1 ako `greske.length > 0`.

Pravila: broje se `.rs .ts .tsx .mjs .js` izvan `node_modules target dist .git .superpowers .playwright-mcp`;
`DUG` danas = `crates/api/src/schema.rs: 1687`, `crates/game/tests/mentor.rs: 1670`, `crates/game/src/mentor.rs: 1367`;
`IZUZECI` = `packages/api-client/src/schema.d.ts: "generirano (gen:api)"`. Poruke: `PREKO 1000 REDAKA  <put>  <n>`,
`DUG NARASTAO  <put>  <n> > <max>`, upozorenja `IZNAD 600  <put>  <n>`, `SPUSTI GRANICU DUGA  <put>  na <n>`,
`ZASTARIO UNOS DUGA  <put>`. Spajanje grane koja legitimno podiže dug (npr. obnova: `schema.rs` 1889) diže unos u
istom commitu, s razlogom u poruci commita.

- [ ] **Step 1: Testovi (čuvar testova)** — `scripts/check-kod.test.mjs`:

```js
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { provjeri } from "./check-kod.mjs";

const RED = (n, eol = "\n") => "x".concat(eol).repeat(n);
function uMapi(datoteke) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "check-kod-"));
  for (const [ime, sadrzaj] of Object.entries(datoteke)) {
    fs.mkdirSync(path.dirname(path.join(dir, ime)), { recursive: true });
    fs.writeFileSync(path.join(dir, ime), sadrzaj);
  }
  return dir;
}
const sMapom = (datoteke, fn) => { const d = uMapi(datoteke); try { return fn(provjeri(d)); } finally { fs.rmSync(d, { recursive: true, force: true }); } };

test("1000 prolazi, 1001 pada; CRLF broji isto", () => sMapom(
  { "a/ok.rs": RED(1000), "a/crlf.ts": RED(1000, "\r\n"), "a/velik.tsx": RED(1001) },
  (r) => { assert.deepEqual(r.greske, ["PREKO 1000 REDAKA  a/velik.tsx  1001"]); },
));
test("601 daje upozorenje, ne grešku", () => sMapom({ "b.mjs": RED(601) }, (r) => {
  assert.deepEqual(r.greske, []); assert.deepEqual(r.upozorenja, ["IZNAD 600  b.mjs  601"]);
}));
test("ignorirane mape i generirani schema.d.ts ne ruše", () => sMapom({
  "node_modules/x/i.js": RED(5000), "target/g.rs": RED(5000), "apps/web/dist/a.js": RED(5000),
  "packages/api-client/src/schema.d.ts": RED(3000), "README.md": RED(5000),
}, (r) => { assert.deepEqual(r.greske, []); }));
test("dug: do granice upozorava, preko pada, manji traži spuštanje, nestao je zastario unos", () => {
  sMapom({ "crates/api/src/schema.rs": RED(1688) }, (r) =>
    assert.ok(r.greske.includes("DUG NARASTAO  crates/api/src/schema.rs  1688 > 1687")));
  sMapom({ "crates/api/src/schema.rs": RED(1600) }, (r) => {
    assert.deepEqual(r.greske, []);
    assert.ok(r.upozorenja.includes("SPUSTI GRANICU DUGA  crates/api/src/schema.rs  na 1600"));
    assert.ok(r.upozorenja.includes("ZASTARIO UNOS DUGA  crates/game/tests/mentor.rs"));
  });
});
```

- [ ] **Step 2:** `corepack pnpm test:docs; echo EXIT=$?` → FAIL („Cannot find module ./check-kod.mjs").
- [ ] **Step 3: Kod (graditelj)** — `scripts/check-kod.mjs` po sučelju gore: rekurzivni obilazak (`fs.readdirSync` s
  `withFileTypes`), putanje s `/`, sortirano; `brojRedaka`; `DUG` se uspoređuje točno; glavni dio se izvršava samo kad je
  skripta pokrenuta izravno (`import.meta.url === pathToFileURL(process.argv[1]).href`), ispisuje `=== check:kod ===`,
  upozorenja s `⚠️`, greške s `❌`, inače `✅ kod je unutar granica`.
- [ ] **Step 4:** `corepack pnpm test:docs; echo EXIT=$?` → PASS. Zatim na pravom repou `corepack pnpm check:kod; echo EXIT=$?`
  → 0 (tri unosa duga, upozorenja za datoteke 600–1000 su očekivana; zapiši ih u izvještaj).
- [ ] **Step 5: Commit** — `scripts/check-kod.mjs scripts/check-kod.test.mjs package.json .github/workflows/ci.yml docs/records/BACKLOG.md`.

---

