# ADR-018 — Leonove proizvodne odluke za AI sloj: gdje slobodan tekst, odakle brojke, razgovor, pružatelj
**Datum:** 2026-10-09 · **Status:** ✅ ODLUČIO LEON (anketa 09.10.) · **Vezano:** [ADR-019](ADR-019-ai-sloj-arhitektonske-pretpostavke-koraka-4.md), DEMO §1.5, §7, §11 t. 7

**Odluka.**
1. **Slobodan tekst samo u S1 i S2**, uz ponuđene odgovore; S3 (agencija, brojke) ostaje biranje.
2. **Brojke slobodnog odgovora iz najbližeg ponuđenog odgovora:** AI ocijeni kvalitetu po rubrici i izabere najbliži
   ponuđeni odgovor; trošak i ostale brojke dolaze od njega. AI nikad ne izmišlja novac.
3. **Razgovor s likom: najviše 6 poruka studenta po situaciji;** presuditelj vidi cijeli razgovor uz konačni odgovor.
4. **Pružatelj: Anthropic (Claude)** — jači model za presude, jeftiniji za lika i mentora; ključ u `.env`.
   *Zamijenjeno [ADR-024](ADR-024-modeli-presuda-na-sonnetu-5-5.md): presuda Sonnet 5.5, lik Haiku 5.5, mentor Sonnet 5.5.*
