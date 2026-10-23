# ADR-064 — Plan je jedna datoteka do 1000 redaka
**Datum:** 2026-10-23 · **Status:** ✅ **Leonova odluka 23.10.** (t. 1), 🟨 t. 2–7 ČEKA LEONOV PREGLED ·
**Vezano:** [ADR-038](ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) (način rada), [ADR-043](ADR-043-popravak-temelja-ne-gradnja-ispocetka.md),
[docs/README.md](../../README.md#granice-veličine) (granice), plan dokumentacije [2026-10-10-dokumentacija](../../plan/2026-10-10-dokumentacija.md) (Task 2 alat `razdvoji-plan`)

**Povod.** Leon 23.10.: „imamo previše datoteka s planom … za plan je pravilo drukčije i to se treba promijeniti". Plan je
imao granicu živog dokumenta (300 redaka), pa se svaki veći plan razdvajao u pregled + mapu s jednim taskom po datoteci.
Rezultat 23.10.: **60 datoteka** u `docs/plan/` (4 mape, ~9 200 redaka), plan se nije mogao pročitati na jednom mjestu, a
gotovi taskovi ostajali su među živima. Tvrda granica (1200 redaka za svaki `.md`) nije bila uzrok.

## Odluka
1. **Plan je jedna datoteka izravno u `docs/plan/`, do 1000 redaka.** Taskovi su odjeljci (`### Task …`), ne zasebne datoteke.
   Tvrda granica 1200 i dalje vrijedi za sve.
2. 🟨 **Brana:** nova vrsta `plan` = 1000 redaka (`scripts/docs-granice.mjs`); pravilo 13 u `check:docs` pada na svaku `.md`
   u podmapi `docs/plan/` („PLAN U MAPI"). Testovi u `scripts/check-docs.test.mjs` i `docs-granice.test.mjs`.
3. 🟨 **Gotove kartice sele u arhivu**, redom i bez izmjene sadržaja: `docs/archive/<plan>-kartice.md` (kratak indeks) +
   `docs/archive/<plan>-kartice/dio-N.md` (svaki do 1000 redaka). U planu ostaje redak tablice s poveznicom na karticu.
4. 🟨 **Preuređenje 23.10.:** `ROADMAP` (3 dijela) i `leonov-pregled` (5 dijelova) spojeni u po jednu datoteku (589 i 530 r.);
   plan popravka temelja zadržava samo otvoreno (L1-15, kartica Pilot, faza C; 303 r.), a 23 gotove kartice idu u
   `archive/2026-10-14-popravak-temelja-kartice/` (5 dijelova); pauzirani plan Sloja 1 i njegove 22 kartice u arhivu (6 dijelova).
   `docs/plan/` sada ima **5 datoteka**. Poveznice (i u ADR-ovima, zapisnicima i `.superpowers` bilješkama) prepisane skriptom na
   nova mjesta sa sidrom; kopija bilješki prije prepisivanja: `Documents/hotelixsim-kopije/2026-10-23/`.
5. 🟨 **Alat `razdvoji-plan` (`corepack pnpm docs:plan`) uklonjen** — po t. 1 više nema što razdvajati; `/kraj`, AGENTI i
   korijenski README opisuju novo pravilo. Brief izvođaču = putanja plana + naslov taska.
6. 🟨 Prijelazna iznimka za plan Sloja 1 uklonjena (plan je u arhivi); ostaje samo `MODEL_V2.md`.
7. 🟨 Arhivski planovi stariji od 23.10. (kostur, jezgra, dnevnik, AI sloj, ekrani, temelj rada) **ostaju** kao indeks + mapa —
   zamrznuti su i ne smetaju; spajaju se samo ako Leon želi.

## Što ovo mijenja u radu
- Sljedeća sesija razrađuje novi plan (L1-15, pilot dijelovi 8–10, faza C) **u jednu datoteku**; kad prijeđe ~900 redaka,
  gotove kartice sele u arhivu po t. 3.
- Cijena: veći plan znači više tokena kad ga agent čita cijelog — zato brief daje naslov taska, a izvođač čita samo svoj odjeljak.
