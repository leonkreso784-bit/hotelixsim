# Sljedeća sesija — plan „jezgra i alat za balans"

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

## Pročitaj (ciljano)

1. [DEMO.md](./product/DEMO.md) §2–§12 — **spec ovog plana**.
2. [BACKLOG.md](./records/BACKLOG.md) — stavke s oznakom `[jezgra]` (odgođeni nalazi kostura + rupe u specu).
3. [BRANE_DETERMINIZMA_2026-10-04.md](./research/BRANE_DETERMINIZMA_2026-10-04.md) — što brane danas drže, a što ne
   (ulaz za Task 1). [KALIBRACIJA_2026-10-04.md](./research/KALIBRACIJA_2026-10-04.md) — opcije za početne indekse.
4. [ARHITEKTURA.md](./architecture/ARHITEKTURA.md) samo §0, §6.2, §7, §9.5 i §20 kad zatrebaju.

## Cilj

Napisati plan `docs/plan/<datum>-jezgra.md` (`superpowers:writing-plans`, zaglavlje `**Status:** 🟩 AKTIVAN`) i
krenuti u gradnju kroz `superpowers:subagent-driven-development` na novoj grani ([AGENTI.md](./workflow/AGENTI.md)).

- **Task 1 = ojačati brane determinizma** prije ijedne linije jezgre ([ADR-009](./records/DECISIONS.md#adr-009)):
  dopušteni popis ovisnosti u `check-core-deps.mjs`, `RandomState` u oba `clippy.toml`, `forbid` + zabrana
  `as`/float u `lib.rs`, i **skripta mutacija u CI-ju** (svaka mutacija mora pasti).
- **Prije pisanja taskova jezgre** presudi rupe u specu (BACKLOG → „Rupe u specu"): početne vrijednosti profila C,
  s čime se uspoređuje marža, pravila zaokruživanja (DEMO §12, P1–P32). Odluku zapiši kao pretpostavku (ADR 🟨).
- API dio (ruta profila mimo `app`, enum kodovi u OpenAPI-ju) je izvan opsega jezgre → zaseban task na kraju plana.

**Prvi korak:** `corepack pnpm check:docs` (mora biti zeleno), zatim pročitaj DEMO §12 i BACKLOG `[jezgra]`.

## Otvoreno za Leona

Pitanja žive u [BACKLOG.md](./records/BACKLOG.md) → „Otvoreno za Leona" i u ADR-ovima sa statusom 🟨
([DECISIONS.md](./records/DECISIONS.md)). Na početku sesije pitaj samo ono što blokira posao; ništa od toga ne
blokira Task 1.
