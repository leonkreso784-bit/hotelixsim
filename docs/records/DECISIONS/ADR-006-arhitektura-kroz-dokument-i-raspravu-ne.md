# ADR-006 — Arhitektura kroz dokument i raspravu, ne kroz ankete; ankete samo za jednostavne proizvodne odluke
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-003](ADR-003-arhitektura-rust-jezgra-backend-typescript-sucelje.md), [ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md) ·
**Izvor:** [RASPRAVA_ARHITEKTURE](../../architecture/RASPRAVA_ARHITEKTURE.md)

**Kontekst.** Na anketu o obliku sustava Leon je odgovorio ljutito (*„ovo je ozbiljno, ovo nije igraonica"*), a jedna od
ponuđenih opcija bila je lažna, ubačena radi kontrasta. Isti dan je odbio da se stack bira prema onome što on zna:
*„ne razmišljaj o tome što ja znam, razmišljaj široko… Rust, drugi jezici, budi otvoren i kreativan"*.
**Odluka.** Arhitektura se radi u [ARHITEKTURA.md](../../architecture/ARHITEKTURA.md): za svaku odluku razlog, razmotrene
alternative i cijena. Leon čita, raspravlja, potvrđuje (*„sada ćemo raspravljati o arhitekturi, to još nije to"*).
Kandidati se biraju po zahtjevima sustava, ne po Leonovom iskustvu; nikad lažna opcija. Ankete (AskUserQuestion) ostaju
za jednostavne proizvodne odluke.
**Posljedice.** Dokument je polazište rasprave, nije gotov dok Leon ne kaže. Cilj je razina seniora („arhitektura 10/10").
