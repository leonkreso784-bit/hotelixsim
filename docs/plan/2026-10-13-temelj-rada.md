# Temelj rada (Faza 0) — plan izgradnje

**Status:** 🟩 AKTIVAN · **Područje:** dokumentacija · **Otvoren:** 2026-10-13 · **Grana:** `model-v2/sloj1` (glavno stablo)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Dva kolosijeka (igra, platforma) i kvaliteta mogu raditi po pravilima iz ADR-038, a brane čuvaju veličinu koda
i područja planova.

**Architecture:** Tri neovisne cigle bez dodira s pravilima igre: (1) pravila rada u `AGENTI.md` i područja planova u
`check:docs`; (2) nova brana `check:kod` (Node skripta po uzoru na `check:docs`, broji retke istom funkcijom); (3) prijenos
platformskog nacrta iz ledgera u `docs/architecture/` uz prepis po ADR-037. K0-1 (CI) je gotov u `16c4ac9`, a čeka push.
K0-4/5/6 (rez `schema.rs` i mentora) idu tek nakon spajanja grana obnove i situacija ([kvaliteta](./ROADMAP/kvaliteta.md)).

**Tech Stack:** Node 24 (ESM, `node:test`), Markdown, GitHub Actions.

**Spec:** [ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) ·
[ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md) ·
[ROADMAP/kvaliteta.md](./ROADMAP/kvaliteta.md) (K0-2, K0-3, K0-7)

## Global Constraints

- Bash alat resetira cwd: svaka naredba počinje s `cd /c/Users/leonk/Documents/hotelixsim && …`.
- Provjera: gola naredba pa `; echo EXIT=$?`, nikad `| tail`. Za testove skripti `corepack pnpm test:docs` (čitaj „ℹ pass N / ℹ fail 0").
- Datoteke s hrvatskim znakovima piši alatom Write/Edit, ne heredocom.
- Commit: poruka u datoteci u scratchpadu, `git commit -F`, izričite putanje (nikad `git add -A`), oblik
  „Temelj rada K0-N: …", završetak `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Push samo uz Leonov OK.
- Leonov demo radi iz ovog stabla (8080, 5173): ne gasiti procese, ne dirati `apps/`, `crates/`.
- Svaki `.md` unutar granice (`node scripts/check-docs.mjs --popis`); kod: ništa preko 1000 redaka osim dosadašnjeg duga.
- Čuvar testova piše test **prije** koda i pokaže da pada; graditelj ne mijenja testove (ADR-038 t. 3).

## Review Focus

1. Plan s `**Područje:** kod` i drugi s `**Područje:** igra`, oba aktivna → mora pasti (alias, ne dva područja). Test u Tasku 1.
2. Datoteka koda s CRLF završecima broji isto kao LF (1000 redaka CRLF prolazi). Test u Tasku 2.
3. Velika datoteka u `node_modules/`, `target/`, `dist/` ili generirani `schema.d.ts` ne ruši branu. Test u Tasku 2.
4. Unos u popisu duga za datoteku koje više nema → upozorenje „zastario unos", ne pad. Test u Tasku 2.
5. Dug koji se smanjio → upozorenje „spusti granicu na N" (ratchet se ne zaboravlja). Test u Tasku 2.

---

### Task 1 (K0-2): Pravila rada i područja planova

**Files:**
- Modify: `scripts/check-docs.mjs:176-206` (pravilo 3), `scripts/check-docs.test.mjs:201-230`
- Modify: `docs/workflow/AGENTI.md` (§1, §3, §4), `docs/README.md` (pravilo 3, odlomak „Planovi, područja i ADR-ovi")
- Modify: ovaj plan, redak `**Područje:**` → `kvaliteta`

**Interfaces:** Produces: područja `igra | platforma | kvaliteta | dokumentacija`; bez retka = `igra`; `kod` = zastarjeli
naziv za `igra` (isti brojač).

- [ ] **Step 1: Testovi (čuvar testova)** — u `check-docs.test.mjs` zamijeni očekivanje `(podrucje: kod)` s `(podrucje: igra)` i dodaj:

```js
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
```

- [ ] **Step 2:** `corepack pnpm test:docs; echo EXIT=$?` → očekivano FAIL (novi test i promijenjeno očekivanje).
- [ ] **Step 3: Kod (graditelj)** — u `check-docs.mjs`:

```js
const PODRUCJA = ["igra", "platforma", "kvaliteta", "dokumentacija"];
const ALIAS_PODRUCJA = { kod: "igra" }; // Sloj 1 i stariji planovi (ADR-038 t. 8)
// …
  const procitano = /\*\*Područje:\*\*\s*([^\s·]+)/.exec(zaglavlje)?.[1] ?? "igra";
  const podrucje = ALIAS_PODRUCJA[procitano] ?? procitano;
```

i poruku `NEPOZNATO PODRUCJE` s „(bez retka = igra; kod = igra)". Ažuriraj komentar pravila 3 (redci 176-181) i komentar na vrhu (redak 13).
- [ ] **Step 4:** `corepack pnpm test:docs; echo EXIT=$?` → PASS, `ℹ fail 0`.
- [ ] **Step 5: Dokumenti** — `AGENTI.md`: §1 tok rada → kartica cigle i koraci voditelj → čuvar testova → graditelj →
  provjeravatelj → čuvar dokumentacije (prepiši ADR-038 t. 3–7 kao pravila, s poveznicom na ADR); §2 „Dvije sesije"
  i „Pretinac (faza A)" → zamijeni: jedna sesija, kolosijeci po datotekama, pretinac samo za nespojene grane; §3 tablica
  modela po ulozi; §4 dodaj „najviše 2 agenta prevode Rust, zbrojeno preko kolosijeka". `docs/README.md` pravilo 3 i
  odlomak o područjima: `igra | platforma | kvaliteta | dokumentacija`, bez retka = `igra`, `kod` = stari naziv.
  [ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) t. 8: dodaj „i `kvaliteta`".
  Ovaj plan: `**Područje:** kvaliteta`.
- [ ] **Step 6:** `corepack pnpm check:docs; echo EXIT=$?` i `corepack pnpm test:docs; echo EXIT=$?` → oba 0.
- [ ] **Step 7: Commit** — putanje: `scripts/check-docs.mjs scripts/check-docs.test.mjs docs/workflow/AGENTI.md docs/README.md docs/records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md docs/plan/2026-10-13-temelj-rada.md`.

---

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

### Task 3 (K0-7): Arhitektura platforme u `docs/`

**Files:**
- Read: `.superpowers/sdd/2026-10-10-v2-sloj1/platforma-arhitektura.md` (720 redaka, nije u gitu)
- Create: `docs/architecture/PLATFORMA.md` (indeks ≤ 200) + `docs/architecture/PLATFORMA/NN-tema.md` (svaki ≤ 300)
- Modify: `docs/README.md` (redak u `architecture/`), `docs/product/IDEJE.md` (stupac „Gdje" za ideje 10–13 → `PLATFORMA.md`)

**Interfaces:** Consumes: ADR-037 t. 1–8, dopune i ISPRAVAK na kraju nacrta. Produces: `PLATFORMA.md` s tablicom „§ → datoteka".

- [ ] **Step 1: Provjera prije (čuvar testova)** — zapiši naredbu koja mora na kraju vratiti 0 pogodaka:
  `grep -rniE "partij[ae] po studentu|i mod 4|3 robota u razredu|zasebn(e|ih) partij|zatvara krug za sve|Supabase kao baza" docs/architecture/PLATFORMA*; echo EXIT=$?`
  (očekivano na kraju: EXIT=1, tj. nema pogodaka).
- [ ] **Step 2: Prijenos i prepis (graditelj)** — podijeli po smislu: 01 pojmovi i odgovor, 02 uloge i razred, 03 prijava,
  04 generirani hoteli, 05 igrini slojevi, 06 produkcija i trošak, 07 GDPR i sigurnost, 08 faze i pitanja. Prepiši
  zastarjelo po ADR-037: razred = jedna igra s N hotela; obračun po hotelu i vrata kruga; lista kolega; vidljivi radnici
  i natjecanje za ljude; sat i semestar, neaktivni, obavijesti; računi studenata (korisničko ime + lozinka) pa AAI;
  hosting po dopunama 13.10. (Railway Hobby + Postgres na Railwayu, ~5–10 €/mj, Supabase samo kao IdP kasnije);
  faze F0–F5 → imena faza iz [ROADMAP](./ROADMAP.md). Odluke se ne prepisuju iz ADR-a: poveznica na ADR-037. Brojke
  troška ostaju s oznakom [PROC]. Na vrhu indeksa: „nacrt za raspravu (ADR-006); odluke su u ADR-037".
- [ ] **Step 3:** naredba iz Step 1 → EXIT=1; `corepack pnpm check:docs; echo EXIT=$?` → 0; `node scripts/check-docs.mjs --popis`
  pokazuje sve nove dijelove ispod granice.
- [ ] **Step 4: Commit** — `docs/architecture/PLATFORMA.md docs/architecture/PLATFORMA/ docs/README.md docs/product/IDEJE.md`.

---

### Završetak faze

- [ ] Provjeravatelj (svjež agent) pregleda sva tri commita naspram ovog plana i ADR-038; nalazi idu graditelju.
- [ ] Voditelj: plan u `docs/archive/` kad su Taskovi 1–3 gotovi i K0-1 potvrđen u CI-ju (nakon Leonova OK-a za push);
  K0-4/5/6 ostaju u [ROADMAP/kvaliteta](./ROADMAP/kvaliteta.md) do spajanja grana; K0-8 je Leonova sjednica.
