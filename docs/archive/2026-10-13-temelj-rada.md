# Temelj rada (Faza 0) — plan izgradnje

**Status:** ✅ ISPUNJEN 2026-10-14 (Taskovi 1–3 i završni pregled 13.10.; K0-1 dokazan u CI-ju 14.10.) · **Područje:** kvaliteta · **Otvoren:** 2026-10-13 · **Grana:** `model-v2/sloj1` (glavno stablo)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Dva kolosijeka (igra, platforma) i kvaliteta mogu raditi po pravilima iz ADR-038, a brane čuvaju veličinu koda
i područja planova.

**Architecture:** Tri neovisne cigle bez dodira s pravilima igre: (1) pravila rada u `AGENTI.md` i područja planova u
`check:docs`; (2) nova brana `check:kod` (Node skripta po uzoru na `check:docs`, broji retke istom funkcijom); (3) prijenos
platformskog nacrta iz ledgera u `docs/architecture/` uz prepis po ADR-037. K0-1 (CI) je gotov u `16c4ac9` (popravak `b495c8d`), dokazan CI-jem 38446984164 na `ubuntu-24.04`.
K0-4/5/6 (rez `schema.rs` i mentora) idu tek nakon spajanja grana obnove i situacija ([kvaliteta](../plan/ROADMAP/kvaliteta.md)).

**Tech Stack:** Node 24 (ESM, `node:test`), Markdown, GitHub Actions.

**Spec:** [ADR-038](../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) ·
[ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md) ·
[ROADMAP/kvaliteta.md](../plan/ROADMAP/kvaliteta.md) (K0-2, K0-3, K0-7)

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

## Taskovi

| N | Task |
|---|---|
| 1 | [Task 1 (K0-2): Pravila rada i područja planova](2026-10-13-temelj-rada/task-01-k0-2-pravila-rada-i-podrucja.md) |
| 2 | [Task 2 (K0-3): Brana veličine koda `check:kod`](2026-10-13-temelj-rada/task-02-k0-3-brana-velicine-koda-check.md) |
| 3 | [Task 3 (K0-7): Arhitektura platforme u `docs/`](2026-10-13-temelj-rada/task-03-k0-7-arhitektura-platforme-u-docs.md) |

