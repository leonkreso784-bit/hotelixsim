# ADR-031 — Sloj 1: tjedni krug u jezgri v2
**Datum:** 2026-10-11 · **Status:** ✅ **Potvrdio Leon 17.10.** (t. 2 i 4 A13, t. 1, 3 i 5 F1) ·
**Vezano:** [ADR-029](ADR-029-sloj-1-pretpostavke-plana.md) P13, P16, [plan Sloja 1](../../plan/2026-10-10-v2-sloj1.md)
„Pravila razdoblja", [MODEL_V2.md](../../architecture/MODEL_V2.md) §4

**Kontekst.** Tjedni krug (7 dana, zadnji 10) daje male brojeve: pravila koja su u mjesecu davala cijeli korak u tjednu
se zaokružuju na nulu. Pregledi Taska 3, Taska 6 i Ojačanja H1 našli su četiri takva mjesta. Mjesečna v2 igra mora i
dalje dati točno v1 brojke (ekvivalencija), pa svako pravilo za cijeli mjesec mora biti identitet.

**Odluka (pretpostavke).**
1. **Odlasci osoblja u krugu = razmjerni dio mjesečnih odlazaka:** `prorate(div_round(staff·turnover_bp, 10_000), p)`,
   a ne zaokruživanje po tjednu. Inače se na ~56 zaposlenih i 3 % fluktuacije svaki tjedan zaokruži na 0 (test
   `weekly_departures_are_the_prorated_monthly_departures`). Odlasci ostaju hotelsko pravilo; raspodjela po odjelima
   je u [ADR-032](ADR-032-sloj-1-odjeli-u-obracunu-i.md) t. 8. (Pregled Taska 3; Task 12 je svjesno odstupio od
   briefa koji je tražio odlaske po odjelu s `D₀`.)
2. **Najmanji korak prema cilju:** pomak morala, zadovoljstva i ocjene je `round(Δ·step_bp/10 000)`; u krugu koji
   **nije cijeli kalendarski mjesec**, kad je Δ ≠ 0 a zaokruženi korak 0, korak je **±1 jedinica prema cilju** (nikad
   preko cilja). Bez toga bi se uz tjedne koeficijente razmak do 6 (moral), 3 (zadovoljstvo) i 7 (ocjena) zauvijek
   zaokruživao na 0 i stanje bi stalo ispod cilja. Mjesečni krug nepromijenjen. Jedna funkcija:
   `resolve::step_toward` (`crates/sim-hotel-v2/src/resolve/mod.rs`). (Ojačanje H1, korak 4)
3. **Mjesec situacije je mjesec čiji je 1. dan u krugu** (`Period::month_starting_in()`, isto pravilo kao
   `situations_for`, P13): `effects(o, mjesec situacije, …)`. t05 (29. 6. – 5. 7.) obračunava srpanjske situacije sa
   srpnjem; lipanjska grupa u srpanjskoj situaciji je `GroupInPast`; noćenja srpanjske grupe u t05 idu u srpanjske
   dane (test `july_situation_in_week_five_books_its_group_into_july_days`). Time je riješen nalaz pregleda Taska 3
   (grupa u tjednu preko granice mjeseca gubi dio bloka) za provjeru prošlosti. (Ojačanje H1, korak 2)
4. **Prenošenje plana:** krug bez `PlanSet` uzima plan **najbližeg ranijeg kruga koji ima `PlanSet`** (studentov
   izvorni plan), preko `carry_into` u granice novog kruga — ne lanac „prenesenog iz prenesenog". Razlika se vidi samo
   kad se cijena ili sezonci stisnu u granicama nekog međukruga. v1 i dalje ne prenosi (vrijedi zadani plan).
   (Task 6, `Game::effective_plan`)
5. **Verzija jezgre:** `CORE_VERSION` v2 (2.0.0) podiže se pri izdanju sloja, ne po svakom ojačanju (tjedni brojevi
   su se promijenili u H1, ali nijedna v2 igra još nije objavljena). Probne v2 igre: [BACKLOG](../BACKLOG.md) `[sloj1]`.

**Cijena ako krivo:** t. 1–3 su po jedna funkcija u `sim-hotel-v2` uz ponovnu kalibraciju tjednih koeficijenata
(balans); t. 4 je jedna funkcija u `game`.

**Potvrdio Leon 17.10.** ([ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)): t. 2 i 4 (A13 h), t. 1, 3 i 5 (F1).
