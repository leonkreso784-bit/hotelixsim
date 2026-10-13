# ADR-004 — Sve se gradi i pokazuje lokalno; Railway tek za pilot
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-003](ADR-003-arhitektura-rust-jezgra-backend-typescript-sucelje.md), [ADR-007](ADR-007-lokalna-baza-zaseban-postgresql-16-klaster.md) ·
**Izvor:** [ARHITEKTURA](../../architecture/ARHITEKTURA.md) §14

**Kontekst.** Demo se pokazuje profesoru uskoro; hosting nije potreban da bi se dokazala igra.
**Odluka.** Leon: *„za sada radimo sve lokalno, Railway je za kraj"*. Razvoj i demo idu lokalno; Railway (jedan servis +
Postgres) dolazi u PILOTU.
**Posljedice.** Ništa u kodu ne smije ovisiti o Railwayu: postavke dolaze iz varijabli okoline, a isporuka je zaseban
završni korak. Deploy i dalje samo uz izričit OK ([ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)). Vidi [ROADMAP](../../plan/ROADMAP.md).
