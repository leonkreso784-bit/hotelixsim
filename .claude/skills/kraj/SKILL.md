---
name: kraj
description: Kraj HotelixSim sesije — zapiši što je rađeno (PROGRESS, CHANGELOG, ADR-ovi, plan, BACKLOG), prepiši CLAUDE.md "Stanje" i predaju docs/SLJEDECA_SESIJA.md, provjeri docs branom i commitaj lokalno. Koristi se kad Leon završava sesiju.
disable-model-invocation: true
---

# Kraj sesije — zapiši, provjeri, predaj

Leon završava sesiju. Tvoj posao je da sljedeća sesija (`/newsession`) zna sve što znaš ti, a da se nijedna činjenica
ne zapiše na dva mjesta. Radi redom; radi bez pitanja za tok rada, **push samo uz izričit OK**.

## 1 · Pregled sesije

`git log` od početka sesije (`git log --since=<početak> --oneline`), `git status -sb` i što je u razgovoru
odlučeno ili ostalo otvoreno. Necommitane tuđe izmjene ne diraj — javi ih.

## 2 · Dnevnik i isporuke

- `docs/records/PROGRESS.md` — novi unos **na vrh**: `## YYYY-MM-DD — naslov`, 3–8 natuknica: što je napravljeno,
  čime je provjereno i gdje se vidi (lokalno / GitHub / preview), što je ostalo otvoreno.
- Ako je isporučeno nešto što radi → redak u `docs/records/CHANGELOG.md`.

## 3 · Odluke

Svaka odluka iz sesije → ADR na vrh `docs/records/DECISIONS.md` (`## ADR-00N — naslov`, sljedeći broj; Leonov
citat doslovno ako postoji; status, razlog, cijena ako je krivo). Pravilo „radi bez pitanja": pretpostavke koje
si donio umjesto Leona → ADR sa statusom `🟨 ČEKA LEONOV PREGLED`.

## 4 · Plan i backlog

- Ažuriraj aktivni plan u `docs/plan/` (zaglavlje `**Status:** 🟩 AKTIVAN`; najviše jedan).
- Ispunjen plan → `git mv` u `docs/archive/` s pečatom `✅ ISPUNJEN datum` u zaglavlju, **istog dana**.
- Ledger subagenata iz `.superpowers/sdd/` (gitignoriran) kopiraj u `docs/archive/<datum>-<ime>-ledger.md`.
- Novo neriješeno → `docs/records/BACKLOG.md`; riješeno iz backloga → **obriši** redak.

## 5 · CLAUDE.md „Stanje"

Prepiši odjeljak na **trenutno** stanje (ne dopisuj povijest — povijest ide u PROGRESS). Budžet datoteke: 12 000 znakova.

## 6 · Predaja

Prepiši **cijeli** `docs/SLJEDECA_SESIJA.md` (≤60 redaka): cilj sljedeće sesije, prvi korak, otvorena pitanja
za Leona, što pročitati. Bez povijesti i bez odluka — one su u PROGRESS i DECISIONS.

## 7 · Indeks

Svaka nova datoteka u `docs/` → redak u `docs/README.md` (jedini indeks).

## 8 · Brana

```bash
corepack pnpm check:docs; echo EXIT=$?
```

Mora biti `EXIT=0`. Bez `| tail` (cijev maskira exit kod). Crveno → popravi uzrok, ne zaobilazi branu.

## 9 · Commit (lokalno)

Poruku napiši u datoteku (scratchpad) i `git commit -F <datoteka>`, na hrvatskom, uz propisani `Co-Authored-By`.
Dodaj **izričitim putanjama** (`git add docs/records/PROGRESS.md …`), nikad `git add -A`.
**Push ne radi sam** — pitaj Leona na kraju.

## 10 · Javi Leonu (hrvatski, ≤8 redaka)

Što je zapisano i gdje (datoteke), gdje se vidi, rezultat brane (`EXIT=0`), što čeka njegov OK (push, ADR-ovi
`ČEKA LEONOV PREGLED`) i pitanje „Pushati?".

## Zašto

Sokrat kraj radi ručno pa mu predaja zastari i činjenice se razmnože po dokumentima. Ovdje je kraj jedna naredba,
a pravilo je **jedna činjenica jedno mjesto**: povijest u PROGRESS, odluke u DECISIONS, neriješeno u BACKLOG,
trenutno stanje u CLAUDE.md, a predaja samo kaže što raditi sljedeće.
