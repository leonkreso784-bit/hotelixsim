# ADR-004 — Sve se gradi i pokazuje lokalno; Railway tek za pilot
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-003](ADR-003-arhitektura-rust-jezgra-backend-typescript-sucelje.md), [ADR-007](ADR-007-lokalna-baza-zaseban-postgresql-16-klaster.md) ·
**Izvor:** [ARHITEKTURA](../../architecture/ARHITEKTURA.md) §14

**Kontekst.** Demo se pokazuje profesoru uskoro; hosting nije potreban da bi se dokazala igra.
**Odluka.** Leon: *„za sada radimo sve lokalno, Railway je za kraj"*. Razvoj i demo idu lokalno; Railway (jedan servis +
Postgres) dolazi u PILOTU.
**Posljedice.** Ništa u kodu ne smije ovisiti o Railwayu: postavke dolaze iz varijabli okoline, a isporuka je zaseban
završni korak. Deploy i dalje samo uz izričit OK ([ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)). Vidi [ROADMAP](../../plan/ROADMAP.md).

**Izmjena 17.10. (dopuna; Leonova odluka 16.10., [ADR-057](ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md) t. 8).**
Platforma se gradi od studenog 2026., usporedo s fazom C, i dalje lokalno. Plan platforme predlaže izlazak na internet već za P3 (profesori,
prije pilota), uz Leonov izričit OK ([ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)) — to je 🟨
[ADR-058](ADR-058-pretpostavke-sesije-7-izgled-vanjski-platforma-k2.md) t. 9. Pružatelj nije odlučen: plan predlaže Railway Hobby (~5–10 €/mj [PROC]),
a Leon 17.10. (E3, [ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)): *„hosting je najmanje problem, mislim da ćemo koristiti
Cloudflare, još nije sigurno"*; Supabase mu je privlačan, ali ga je strah latencije. Naslov „Railway tek za pilot" ostaje kao povijest odluke.
