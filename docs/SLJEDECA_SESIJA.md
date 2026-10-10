# Sljedeća sesija — završni pregled grane „Ekrani igrača" (DEMO korak 5)

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

## Prompt za početak (Leon zalijepi nakon `/clear`)

```text
/newsession

Nastavljamo DEMO korak 5 na grani gradnja/ekrani (lokalno, nije pushana). Ledger je
.superpowers/sdd/2026-10-10-ekrani/progress.md — vjeruj ledgeru i git logu, ne pamćenju.
Gotovo i pregledano: Task 1–19 osim završnog pregleda (13b preglednik i prolaz s pravim AI-jem su obavljeni).

Ostalo: završni pregled cijele grane na najjačem modelu (opus), s popisom odgođenih nalaza iz ledgera; jedan val
popravaka (obavezno: presuda piše „ponuđenom odgovoru 0" — interni broj u tekstu za studenta); ponovni pregled; pa
spajanje u main. Push samo uz moj OK. Testovi i CI nikad ne zovu Claude; preglednik bez ključa.
```

## Cilj

Grana `gradnja/ekrani` spremna za `main` i demo (~14.10.): završni pregled, popravci, spajanje, push uz Leonov OK.

**Prvi korak:** `/newsession`, `corepack pnpm check:docs`, CI, baza `bash scripts/baza.sh start`; zatim ledger — sve odluke
koje je Claude donio umjesto Leona su retci `Ruling 1–23`; odgođeni nalazi su retci `minor (deferred)`.

## Za znati

- **Modeli (ADR-024 ✅):** presuda Sonnet 5.5, lik Haiku 5.5 (bez poslužiteljskog rezervnog modela), mentor Sonnet 5.5.
  Stvarni trošak 10.10.: cijela igra ≈ 0,05 $ (presuda ≈ 0,008 $, mentor ≈ 0,015 $ po mjesecu, lik ≈ 0,0001 $ po odgovoru).
- **Okolina:** u `../hotelixsim-igra` je odvojeno radno stablo (detached, `4db99d9`) iz kojeg je poslužitelj vrtio preglednik —
  može se obrisati (`git worktree remove ../hotelixsim-igra`); `../hotelixsim-mentor` (grana `gradnja/mentor-ljudski`, spojena)
  također. Poslužitelj na kraju sesije možda još radi **s ključem** iz `../hotelixsim-igra` — ugasiti ili pokrenuti bez ključa.
  Pokretanje igre: [README.md](../README.md) → „Igra u pregledniku".
- **Ledger** `.superpowers/sdd/2026-10-10-ekrani/` se briše tek kad je završni pregled čist i grana spojena.

## Otvoreno za Leona

- Pregled ADR-022, ADR-023 (mentor u ljudskim jedinicama) i ADR-019; ocjene 40 odgovora za mjerilo
  (`content/hotel/mjerilo/*.json`) — tada usporedba presude Sonnet/Opus brojkama.
- Vizija za razvoj cijele aplikacije i novi likovi (uloge, pristanak za prava imena): [BACKLOG.md](./records/BACKLOG.md) →
  „Otvoreno za Leona".
