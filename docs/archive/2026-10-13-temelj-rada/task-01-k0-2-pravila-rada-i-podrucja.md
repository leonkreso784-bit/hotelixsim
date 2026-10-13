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
  [ADR-038](../../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) t. 8: dodaj „i `kvaliteta`".
  Ovaj plan: `**Područje:** kvaliteta`.
- [ ] **Step 6:** `corepack pnpm check:docs; echo EXIT=$?` i `corepack pnpm test:docs; echo EXIT=$?` → oba 0.
- [ ] **Step 7: Commit** — putanje: `scripts/check-docs.mjs scripts/check-docs.test.mjs docs/workflow/AGENTI.md docs/README.md docs/records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md docs/plan/2026-10-13-temelj-rada.md`.

---

