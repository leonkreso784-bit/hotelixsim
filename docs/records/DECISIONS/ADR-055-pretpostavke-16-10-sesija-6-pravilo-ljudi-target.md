# ADR-055 — Pretpostavke 16.10. (sesija 6): kartica PRAVILO, voditelj kao čuvar, banka ljudi, target po stablu
**Datum:** 2026-10-16 · **Status:** ✅ **Potvrdio Leon 17.10., osim t. 6 🟨** (t. 1 A5 i A6; t. 2, 4, 5 F1; t. 3 C1, C2, C6) · **Vezano:** [ADR-053](ADR-053-pretpostavke-16-10-oznake-testovi-b4.md),
[ADR-054](ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md), [ADR-047](ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md),
[ADR-042](ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 8, [plan popravka](../../plan/2026-10-14-popravak-temelja.md)

Pretpostavke voditelja bez pitanja (ADR-005); svaka ima cijenu ako je krivo.
1. **Kartica PRAVILO** ([task-12](../../plan/2026-10-14-popravak-temelja/task-12-pravilo-vlastiti-odgovor.md), 224 r.): postavka `AnswerRule { Classic, OwnWords }`,
   zadano **OwnWords za nove v2 igre**; klik na ponuđeni odgovor najviše 4; prepisani ponuđeni odgovor vrijedi kao klik, **prag 70 % riječi** (skup riječi, obje strane);
   AI ispad → **4 bez troška**; roboti **neograničeni** (paritet A4 i mjere robota isti); prompti isti (`presuda.v3`, `mentor.v4`).
   **`CORE_VERSION` 2.4.0 → 2.5.0** s pečatom A3 (nacrt je tvrdio da se ne diže); Leonove probne v2 igre opet daju 409. Podjela na PRAVILO-a (igra), -b (balans), -c (napomena).
   Ako krivo: prag je jedna konstanta, roboti i AI ispad jedna grana koda; krivo zadano pravilo košta jednu postavku, ne novu jezgru.
2. **Voditelj kao čuvar za jednoredne popravke testova** (D7 u K1a: primjer recepcionera bez engleskog rušio je i D8; prag 6500 → 8000 u T9 i B1b):
   RED dokazuje graditelj, redak mijenja voditelj uz commit s obrazloženjem. Ako krivo: isti redak, drugi izvođač.
3. **Banka ljudi K1a/K1b:** (a) test S6 "stereotip je slijep na uloge po podrijetlu" (I1 pregleda K1a) piše čuvar K1b; (b) **I2, rod po ulozi, ide Leonu** u `pregled-20`
   (svih 10 radnika prati većinski rod iz 🟨 `rod_po_ulozi`; ADR-042 t. 8 je pravilo "25 %" odbio) — odluka je njegova; (c) gradovi u tekstu u hrvatskom obliku (Trst, Pečuh, Krakov);
   (d) serije-000 ostaju **OTVORENE** (nezapečaćene) do Leonova pregleda 20 zapisa. Ako krivo: jedan zapis mijenja rod ili grad prije pečata.
4. **Zajednički `CARGO_TARGET_DIR` među stablima ukida se** ([BUGS](../BUGS.md) B12): svako stablo `target/<stablo>` u glavnom `target/`, briše se sa stablom.
   Cijena: više diska (~2,5 GB po stablu); zauzvrat nema lažno zelenog iz tuđe biblioteke.
5. **Mjerenje inačica prije ankete o igri:** jedan agent, jedan krug (B4c prag: 7 inačica izmjereno prije nego što je Leon pitan). Ako krivo: ~1 h mjerenja viška.
6. **B4c nadskup:** granica "najviše 2/18 čistih" za pravilo "najbolji je nadskup slabijeg" (rezultat 0 čistih + 3 granična: P-POSEBNA-PREHRANA kruh, P-LEZALJKE, P-SPA-TERMIN).
   Ako krivo: prag u testu, ili tri oznake još jednom.

**Potvrdio Leon 17.10.** ([ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)): t. 1 (A5 „ok svih pet"; granica zalijepljenih bigramima A6), t. 2, 4, 5 (F1), t. 3 (C1 pečat svih 120, C2 rod po ulozi privremeno, C6 gradovi). t. 6 ostaje 🟨 do Leonova slijepog pregleda tekstova B4 (B1, B2).
