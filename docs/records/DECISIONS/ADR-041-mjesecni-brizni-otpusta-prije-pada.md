# ADR-041 — Mjesečni Brižni otpušta sezonce prije pada (tjedan ≈ mjesec)
**Datum:** 2026-10-13 · **Status:** ✅ **Leonova odluka 13.10.** (anketa) · **Vezano:** [ADR-040](ADR-040-leonove-odluke-iz-pregleda-situacija-s.md) t. 3,
[ADR-029](ADR-029-sloj-1-pretpostavke-plana.md) (dopuna 11.10., ZP-6)

**Kontekst.** Brana balansa „tjedan ≈ mjesec" (granica 3,00) imala je zalihu 0,04: robot Brižni u profilu B na 1000
sjemena daje medijan napretka 7,86 tjedno i 4,90 mjesečno (razlika 2,96; 2,57 i bez situacija). Dijagnoza (ADR-040 t. 3,
pokus u potrošnom stablu) našla je uzrok: po pravilu ZP-6 tjedni Brižni planira sezonce u oba smjera i otpušta ih prije
pada gostiju, a mjesečni je ostao na v1 pravilu („koliko treba sada"). Mjesečni u kolovozu drži 41 sezonca i korak od 10
ga u rujnu zaustavi na 31, tjedni stigne na 28; indeks Financija gleda zadnji krug, pa tjedni dobije GOPPAR +7,9 % prema
referentnom hotelu, a mjesečni +1,9 %. Ocjena po cijelom zadnjem mjesecu (isprobano) razliku povećava (4,50), pa nije rješenje.

**Odluka.** U mjesečnoj igri Brižni **otpušta prije pada** kao tjedni (gornja granica iz ZP-6), ali **ne zapošljava unaprijed**
(to ostaje samo tjedno, zbog v1 ekvivalencije mjesečne igre s osnovnim katalogom, MODEL_V2 §2 t. 6).

**Posljedice.** Pokus na 1000 sjemena: B Brižni 7,86 / 8,04 (razlika 0,18); sve brane prolaze; testovi sim-hotel-v2, balance
i game 343 / 0. Najveća preostala razlika je B Štediša (2,02 na grani situacija; nakon spajanja s obnovom izmjeriti). Student
osjeti samo robota-protivnika Brižnog u mjesečnoj igri (u kolovozu ~38 umjesto 41 sezonca); pravila i brojke studentova hotela
se ne mijenjaju. Brana „Brižni sezonci prema planu" mjesečno C pada s 84,3 % na 80,0 % (granica 80 %, bez zalihe). Cigla RB-1
na grani situacija (`crates/sim-hotel-v2/src/robots.rs`, `caring_target`), uz test.
