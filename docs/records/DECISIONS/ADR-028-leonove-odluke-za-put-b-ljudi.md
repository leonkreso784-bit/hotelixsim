# ADR-028 — Leonove odluke za put B: ljudi kao u Simsima, burza rada, tjedni krug, AI razgovori
**Datum:** 2026-10-10 · **Status:** ✅ **Leonova odluka 10.10.** (osim t. 9 i pravila o nacionalnosti — vidi dolje) ·
**Vezano:** [ADR-026](ADR-026-put-b-hotel-od-brojeva-postaje.md), [MODEL_V2.md](../../architecture/MODEL_V2.md) §9, [PROSIRENJA.md](../../product/PROSIRENJA.md)

**Kontekst.** Arhitekt je u MODEL_V2 §9 postavio 12 pitanja s preporukama; Leon je odgovorio 10.10. i dodao nove
mehanike.
**Odluka (Leon).**
1. Isti ljudi za isti profil u svakoj igri — ali **svi ljudi razvijeni „kao Sims svijet"**: osobnost, dobre i loše
   strane, vidljiva traka vještine.
2. Odjeljak **„Odjeli"**; unutra wellness, restoran, kafić… što hotel ima; klik → stanje prostora, prihod, zadovoljstvo
   gostiju.
3. Stare demo igre igraju se dalje na v1.
4. Otpuštanje dopušteno (otkazni rok, otpremnina), **motivacija kolega pada samo u izvanrednim slučajevima**. Novo:
   **burza rada** s kandidatima; početnik je jeftin i slab, obukom raste, kad postane izvrstan traži veću plaću;
   iskusni profesionalac košta više; **jezici** koje osoba govori i koliko dobro; **agencija za strane radnike**
   (npr. 5 radnika) kojima hotel plaća **smještaj** — student vidi koliko donose, koliko kvalitete dodaju ili oduzimaju
   i koliko košta smještaj.
5. Prezentacija v2 na četiri mjeseca; **igra po tjednu ili po mjesecu — bira profesor pri stvaranju igre**.
6. Popunjenost, RevPAR i GOPPAR nad svim sobama. 7. GOP po USALI-ju (osiguranje, najam, porez ispod GOP-a).
8. Odgovor na recenziju izborom bez AI-ja; **razgovori sa zaposlenicima i gostima u situacijama s AI-jem gotovo
   uvijek** (slobodan tekst).
9. Student **vidi recenzije drugih hotela** („kao Booking drugih hotela").
10. Jedna osnovna cijena + doplate po tipu sobe. 12. **Voditelj odjela** — svakako.
**Pravilo voditelja (🟨 ČEKA LEONOV PREGLED), uz t. 4 i 11:** nacionalnost određuje samo **činjenice** — jezike, trošak
agencije, potrebu za smještajem, dozvolu i rok dolaska; **nikad** vještinu, pouzdanost ni osobnost — one su osobne i
izvlače se iz istih raspodjela za sve. Razlog: igra je nastavni materijal fakulteta; student uči procijeniti čovjeka,
ne zastavu. Ljude drugih hotela student ne vidi (pretpostavka; t. 9 govori samo o recenzijama).
**Odgovori na 11 novih pitanja arhitekta (Leon 10.10., MODEL_V2 §9):** (1) strani radnici: **ista plaća za istu
vještinu** — agencija daje dostupnost uz naknadu i smještaj; (2) **zajednička burza rada** — kandidata kojeg zaposli
jedan hotel drugi više ne mogu (suprotno preporuci); (3) tjedni kalendar **16 × 7 dana + završnih 10 dana**; (4) nova v2
igra je **tjedna** po zadanom; (5) mentor tjedno kratko bez AI-ja, **AI mentor na kraju mjeseca**; (6) broj AI razgovora
**bira profesor** pri stvaranju igre; (7) granica AI troška **2 $ po studentu po igri**, zatim ponuđeni odgovori;
(8) mješavinu jezika gostiju **istražuje agent** (DZS, Kvarner) — rezultat u [JEZICI_GOSTIJU.md](../../research/JEZICI_GOSTIJU.md): njemački 35–42 %, engleski 17–20 %, hrvatski 9–14 % po mjesecu; procjena iz godišnjih udjela 🟨; (9) ciljna GOP marža **35–45 %**, profesor potvrđuje;
(10) sobe hotela kao smještaj osoblja **do 10 % soba**; (11) **jedna skrivena osobina** kandidata, otkriva je razgovor
za posao ili 4 tjedna rada.
**Cijena ako krivo:** tjedni krug i bogati ljudi povećavaju posao sloja 1 i 2 — nova procjena u MODEL_V2 §5.
