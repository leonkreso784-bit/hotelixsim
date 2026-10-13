# ADR-040 — Leonove odluke iz pregleda situacija (S-4): granica po odjelu, osnovica ostalih odjela, tjedan ≈ mjesec
**Datum:** 2026-10-13 · **Status:** ✅ **Leonove odluke 13.10.** (anketa) · **Vezano:** [ADR-033](ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md),
[ADR-038](ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)

**Kontekst.** Provjeravatelj cigle S-4 (situacije v2, grana `model-v2/sloj1-situacije`) potvrdio je sve provjere i
ocijenio tri brige iz predaje. Leon je o njima odlučio anketom.

**Odluka.**
1. **Stroga granica situacija po odjelu (m7) ostaje.** Kad su četiri odjela zatvorena od kolovoza, otprilike jedan od
   6 300 krugova dobije jednu situaciju manje (20 umjesto 21 u mjesecu). Ponašanje je namjerno, a student to ne primjećuje.
   Ako ikad zatreba pun broj, manjak se prenosi u sljedeći tjedan, a granica po odjelu se ne ublažava.
2. **Osnovica za prihod ostalih odjela je prihod soba prije provizije (bruto).** Gost u restoranu ne ovisi o tome koliko je
   agencija uzela. To je ekonomski ispravnije i u skladu s USALI-jem. Provizija ostaje u trošku Smještaja. Danas nema
   učinka u igri, jer model „udio prihoda soba" koristi samo testni katalog. Cigla na grani situacija, uz test.
3. **Tjedna ≈ mjesečna igra: najprije dijagnoza, granica ostaje 3,00.** Zaliha je 0,04 boda (2,96 od 3,00), a 2,57 od te
   razlike postoji i bez situacija: Brižni robot u profilu B igra drukčije po tjednima i po mjesecima. Prije spajanja grana
   obnove i situacija u `sloj1` agent pronalazi i popravlja uzrok. Granica se ne podiže.

**Posljedice.** Dvije nove cigle u fazi ZAVRŠETAK SLOJA 1 ([ROADMAP/igra.md](../../plan/ROADMAP/igra.md)): osnovica bruto
i dijagnoza tjedan ≈ mjesec. Dijagnoza se radi prije spajanja (L1-10, L1-11). Pitanje m7 nestaje iz „Otvoreno za Leona".
