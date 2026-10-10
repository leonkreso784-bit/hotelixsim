# ADR-003 — Arhitektura: Rust (jezgra + backend) + TypeScript (sučelje), v2
**Datum:** 2026-10-04 · **Status:** 🟨 NACRT v2, čeka Leonovu potvrdu · **Vezano:** [ADR-006](ADR-006-arhitektura-kroz-dokument-i-raspravu-ne.md),
[ADR-004](ADR-004-sve-se-gradi-i-pokazuje-lokalno.md), [ADR-009](ADR-009-ojacavanje-brana-determinizma-je-task-1.md) · **Dokument:** [ARHITEKTURA.md](../../architecture/ARHITEKTURA.md),
[RASPRAVA_ARHITEKTURE.md](../../architecture/RASPRAVA_ARHITEKTURE.md)

**Kontekst.** Nacrt v1 pisan je po zahtjevima sustava (determinizam, ponovljivost, pravednost profila), ne po Leonovu
znanju ([ADR-006](ADR-006-arhitektura-kroz-dokument-i-raspravu-ne.md)). Krug 1 rasprave ugrađen je u v2: ispravka presude, puni snimak pravila, pravednost profila
i AI-ja, red poslova, što tko vidi, slobodan potez, obrazloženje za Rust.
**Odluka.** Jezgra simulacije i backend u **Rustu**, sučelje u **TypeScriptu**. Domena i jezici dogovoreni su 04.10.,
ostalo je prijedlog.
**Posljedice.** **Status je izrijekom nepotvrđen:** `ARHITEKTURA.md` (zaglavlje) kaže „prijedlog dok Leon ne potvrdi", dok
`ROADMAP.md` navodi da ju je Leon pregledao; do izričite Leonove potvrde vrijedi stroža formulacija (nacrt). Kostur
(Rust workspace, API, jedan ekran) već je izgrađen na toj osnovi, pa je promjena smjera skupa. Brane determinizma: [ADR-009](ADR-009-ojacavanje-brana-determinizma-je-task-1.md).
