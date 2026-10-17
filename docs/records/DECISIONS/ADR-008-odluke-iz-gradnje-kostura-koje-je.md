# ADR-008 — Odluke iz gradnje kostura koje je Claude donio umjesto Leona
**Datum:** 2026-10-04 · **Status:** ✅ **Potvrdio Leon 17.10.** (F1) · **Vezano:** [ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md), [ADR-007](ADR-007-lokalna-baza-zaseban-postgresql-16-klaster.md),
[ADR-009](ADR-009-ojacavanje-brana-determinizma-je-task-1.md) · **Izvor:** [SLJEDECA_SESIJA.md](../../SLJEDECA_SESIJA.md) §4

**Kontekst.** Po pravilu „radi bez pitanja" ([ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)) Claude je tijekom gradnje kostura sam odlučivao sitnije
stvari. Sve su u gitu i mogu se vratiti. (Točka 2 je zaseban [ADR-007](ADR-007-lokalna-baza-zaseban-postgresql-16-klaster.md), točka 8 je [ADR-009](ADR-009-ojacavanje-brana-determinizma-je-task-1.md).)
**Odluka (pretpostavke, s cijenom ako su krive):**
1. Gradnja na grani `gradnja/kostur` u istom direktoriju, ne u worktreeu. Cijena: nikakva.
2. Brana ovisnosti mjeri i build-ovisnosti, sve targete i značajke. Cijena: sporija brana.
3. `cargo fmt` nad kodom iz plana (plan je pisan preširoko). Cijena: jedan commit formatiranja.
4. `scripts/pnpm.mjs` jer `pnpm` nije na PATH-u (radi lokalno i u CI-ju). Cijena: zamijeniti ako se uvede samostalni pnpm.
5. Ekran pokazuje grešku indeksa; ključevi i interpolacija u i18n su provjereni (popravak nakon pregleda).
6. README i `.gitattributes` popravljeni za svježi Windows stroj (LF svugdje, `SQLX_OFFLINE`).
7. Sitni nalazi pregleda (~40) zapisani kao ulaz za sljedeće planove, ne popravljeni odmah ([BACKLOG](../BACKLOG.md)).
**Posljedice.** Leon pregledava; ono što odbije, vraća se iz gita. Do tada vrijede kao radna pretpostavka.

**Potvrdio Leon 17.10.** ([ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)): F1: svih sedam točaka vrijedi kako su zapisane.
