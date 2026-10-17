# ADR-048 — lik.v3: lik zna redni broj poruke i granicu razgovora
**Datum:** 2026-10-15 · **Status:** ✅ **Potvrdio Leon 17.10.** (A13 l; provedba ADR-042 t. 6; živi test 6/6 uz Leonov OK) ·
**Vezano:** [ADR-042](ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 6, [ADR-024](ADR-024-modeli-presuda-na-sonnetu-5-5.md) (Dopuna 15.10.)

**Kontekst.** Razgovor s likom ima granicu od 6 studentovih poruka (`MAX_CHAT_MESSAGES`), a prompt `lik.v2` je nije poznavao pa je
gošća i u 6. odgovoru postavljala novo pitanje.

**Pretpostavke koje sam donio (Leon pregledava):**
1. `CharacterInput` nosi `message_no` (redni broj studentove poruke, broji se iz prijepisa, ne iz brojača) i `max` (6).
   Prompt `prompts/lik.v3.md`, korak u `ai::request::conversation_stage`; `lik.v2.md` ostaje za stare zapise.
2. **5. poruka:** lik ne otvara nova pitanja. **6. poruka:** lik zaključuje bez pitanja (prihvati rješenje ili jasno kaže što ga muči).
3. Živi test na mjerilu: `character_closes_on_the_last_message` prošao 3 puta (6/6 živih testova).

**Cijena ako krivo:** lik zaključuje prerano (granica se mijenja u jednoj konstanti i promptu). Poznat ostatak: rezerva u trećem
licu na 6. poruci (C3, [BACKLOG](../BACKLOG.md)).

**Potvrdio Leon 17.10.** ([ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)): A13 l: lik zaključuje razgovor u 5. i 6. poruci.
