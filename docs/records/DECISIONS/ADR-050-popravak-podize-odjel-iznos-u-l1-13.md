# ADR-050 — Popravak u situaciji diže stanje svog odjela; iznos „mehanizam sad, iznos u L1-13"
**Datum:** 2026-10-15 · **Status:** ✅ **Leonova odluka 15.10.** (anketa B1a; izmjena 17.10. dolje, ADR-056 A1) · **Vezano:** [ADR-042](ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 1,
[ADR-046](ADR-046-leonove-odluke-15-10-za-fazu-b.md) t. 1, [ADR-049](ADR-049-situacije-dvadeset-posto-mjeri-se-od-ravnala.md),
[plan popravka](../../plan/2026-10-14-popravak-temelja.md) (B1a, L1-13)

**Što se dogodilo:** B1a je gradila mehanizam s bonusom +10 bez faktora kruga; balans je pao (BLOCKED): „B3 težina" tjedno A/B/C
54,0 / 72,4 / 58,7 % i mjesečno B 65,8 % (granica 50 %, cilj ~20 %). Uzrok: 12 predložaka ima hitni popravak (10 i u srednjem
odgovoru), a tjedna igra ima ~65 situacija po sezoni.

**Leon:** *„Mehanizam sad, iznos u L1-13"* (anketa B1a, 15.10.).

**Odluka:**
1. **Mehanizam ostaje:** hitni popravak diže stanje **odjela svoje situacije** (bez faktora kruga; i uz ocjenu 1, ADR-046 t. 1).
2. **Iznos je privremeno +1 bod** (`economy.repair_urgent_condition_pts` = 100): najveći cijeli bod s kojim OBA `--check`
   (tjedno i mjesečno, 1000 sjemena) prolaze. Mjereno 100–500: 100 prolazi; 200 ruši težinu (tjedno A 50,2 %, B 57,6 %); 300 i 400 padaju.
   Težina uz 100: tjedno 42,8 / 48,9 / 41,5 %, mjesečno 25,9 / 33,8 / 20,3 %.
3. **„Oko +10" (ADR-042 t. 1) odlučuje Leon u L1-13**, zajedno s čestoćom kvarova i težinom situacija (cilj ~20 % je njegova odluka, ne samo broj).
4. **Uvjet prije dizanja bonusa je C5:** ekran odjela popravak iz situacije prikazuje kao „održavanje" (uz +1 nevidljivo, uz +10 kriva pouka).
5. Voditeljev dodatak: uz bonus +1 dug redoslijeda iz ADR-049 se otplaćuje (`ORDER_DEBT` prazan); brisanje stropa i stare logike duga je L1-13.

**Cijena ako krivo:** bonus je jedna vrijednost u sadržaju; mehanizam (prostor odjela, `situation_effect`) ostaje. Stari snimci bez ključa = 0.

**Izmjena 17.10. (Leon, A1; [ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)).** Leon: *„Hitni popravak ne rješava stanje
tog odjela, već samo privremeno popravlja klimu. Pokvarena klima je rezultat neodržanog odjela; ukoliko je odjel održan, nema problema s
klimom."* Kvar u sobi je −1 bod, a popravak odmah uz gosta koji je kasnije zadovoljan +2 (neto +1); anketa: bodovi su **napredak odjela
0–100** (stanje Smještaja). Posljedice:
1. **Mehanizam t. 1 ukida se u L1-13:** hitni popravak više ne diže stanje odjela sam po sebi (ni uz ocjenu 1). Umjesto toga kvar spušta stanje
   Smještaja za 1 bod, a dobro riješen kvar ga diže za 2 (što je „dobro riješen", 🟨 [ADR-058](ADR-058-pretpostavke-sesije-7-izgled-vanjski-platforma-k2.md) t. 19).
2. **„Oko +10" (t. 3) više nije cilj.** Stanje odjela trajno diže obnova i održavanje odjela.
3. **Privremeni +1 (t. 2) ostaje do L1-13**; jezgra se zbog ove izmjene ne dira prije L1-13.
4. **Sučelje soba** (svaka soba vidljiva i obnovljiva zasebno) je nova mehanika i ekran, faza C ili kasnije, ne L1-13 ([BACKLOG](../BACKLOG.md)).
5. Uvjet t. 4 (ekran ne smije popravak iz situacije prikazati kao „održavanje") ostaje — tumačenje voditelja: student mora vidjeti −1 i +2 kao ishod situacije.
