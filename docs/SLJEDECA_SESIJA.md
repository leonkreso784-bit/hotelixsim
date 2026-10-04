# Sljedeća sesija — plan „dnevnik, runde i red poslova" (DEMO korak 3)

> Predaja. Prepisuje se na svakom `/kraj`; povijest je u [PROGRESS.md](./records/PROGRESS.md), indeks u [README.md](./README.md).

## Gdje smo

Jezgra igre i alat za balans su gotovi, spojeni u `main` i pushani ([CHANGELOG 0.2.0](./records/CHANGELOG.md)).
Obračun mjeseca, mehanike S1–S4, pet indeksa, ocjena i roboti rade bez baze i ekrana; balans:
`cargo run -p balance --release -- --seeds 1000 --check`. **Rok demoa ~14.10.** ([ADR-012](./records/DECISIONS.md#adr-012)) —
ostaju koraci 3–6 (dnevnik i runde · AI sloj · ekrani igrača · profesorov pogled), pa svaki plan gradi samo put iz
DEMO §1.5 (lipanj i srpanj).

## Pročitaj (ciljano)

1. [ARHITEKTURA.md](./architecture/ARHITEKTURA.md) §8 (tok runde: 8.1–8.5) i §9 (dnevnik 9.1, projekcije 9.2, red poslova 9.4).
2. [DEMO.md](./product/DEMO.md) §1 (demo način: rundu zatvara gumb, gost preko poveznice, tri robota).
3. [BACKLOG.md](./records/BACKLOG.md) — stavke `[dnevnik]` (oblik tablice `events`, `api` mimo `app`, enum kodovi u OpenAPI-ju).
4. Postojeći kod: `crates/store` (tablica `events`, migracije), `crates/app` (portovi), `crates/api` (Axum + OpenAPI),
   javno sučelje jezgre `sim_hotel::{resolve_month, run_season_with, Strategy, start_indices, month_indices}`.

## Cilj

Napisati plan `docs/plan/<datum>-dnevnik-runde.md` (`superpowers:writing-plans`, zaglavlje `**Status:** 🟩 AKTIVAN`) i
graditi kroz `superpowers:subagent-driven-development` na novoj grani ([AGENTI.md](./workflow/AGENTI.md)): igra (igrač + 3 robota)
kao dnevnik događaja u Postgresu (`GameCreated` s punim snimkom kataloga, odluke, presude, `RoundClosed`), zatvaranje
runde gumbom koje zove jezgru za sve hotele, projekcija stanja i indeksa, API rute koje treba ekran, gost preko poveznice.
Situacije u ovom koraku dobivaju odgovor iz ponuđenih (AI presuda je korak 4).

**Prvi korak:** `/newsession`, zatim `corepack pnpm check:docs` i `cargo run -p balance --release -- --seeds 100 --check`
(oboje zeleno), pa ARHITEKTURA §9.1 i BACKLOG `[dnevnik]`.

## Otvoreno za Leona

U [BACKLOG.md](./records/BACKLOG.md) → „Otvoreno za Leona" i u ADR-ovima sa statusom 🟨 ([DECISIONS.md](./records/DECISIONS.md)).
Novo iz ove sesije: najviše sezonaca 50 umjesto 40, B ima samo 3 % niže fiksne troškove, A dobiva napredak od početnog
opterećenja, obnova najbolja proba za A i C, ponuđeni odgovori S3/S4 su nacrt. Ništa od toga ne blokira korak 3.
