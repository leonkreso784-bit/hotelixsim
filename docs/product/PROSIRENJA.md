# Proširenja — katalog dodataka nakon demoa („put B")

> **Što je ovo:** popis svih dodataka koje igra dobiva nakon demoa — za svaki: što student vidi, što time uči (koji
> kolegij FMTU-a), koje brojke mijenja i u kojem je sloju. **Kako** se gradi piše u
> [MODEL_V2.md](../architecture/MODEL_V2.md); **zašto** ovim redom u [ADR-026](../records/DECISIONS.md#adr-026).
> Otvorena pitanja su u [BACKLOG.md](../records/BACKLOG.md), ne ovdje.

## Leonova zamisao

*„Na mjestu gdje su zaposlenici trebaju se vidjeti profili zaposlenika; mjesto gdje su sve sobe i kad se stisne vidi se
stanje svake sobe; posebna sekcija za dijelove hotela — spa, restoran, kafić, rooftop ako ih hotel ima — unutar njih sve
potrebne informacije: zaposlenici tamo, zarada, stanje održavanja; sekcija gdje se čitaju recenzije; mora se vidjeti ADR.
Kao BitLife — opsežno do bola."* Cilj: prezentacija profesoru koja bude „wow".

## Jedno pravilo za svaki dodatak

**Detalj ulazi u igru samo ako mijenja brojke i student vidi zašto.** Profil konobara koji ništa ne mijenja je ukras;
konobar čija pala motivacija uspori uslugu u restoranu, pa gosti pišu lošije recenzije o doručku, pa pada prihod
restorana i ocjena hotela — to je lekcija. Svaki uzrok objašnjava jezgra (isti izvor kao mentor i kartica „Zašto"), ne
ekran.

## Što profesor prepoznaje (gradivo FMTU-a, Hospitality Management)

| Kolegij | Pojmovi | Gdje u igri |
|---|---|---|
| Accounting (USALI, analiza izvještaja, budžet) | odjeli Rooms / F&B / ostali odjeli, odjelni i neraspoređeni troškovi, GOP, GOPPAR | Sloj 1 — odjeli |
| Economics in Hospitality (pokazatelji, cijene, troškovi, investicije) | ADR, RevPAR, popunjenost, fiksni/varijabilni trošak, točka pokrića, isplativost | Sloj 0 i 1 |
| Management (HRM, motivacija, vođenje) | motivacija, fluktuacija, obuka, opterećenje, timovi | Sloj 2 — zaposlenici |
| E-Business (PMS, kanali, OTA) | pregled soba i stanja, kanali prodaje i provizije | Sloj 3 — sobe |
| Marketing (segmenti, ponašanje potrošača) | tko su gosti, što pišu, odgovor na recenziju | Sloj 4 — recenzije |

## Katalog

Oznake: **sloj** = redoslijed gradnje (svaki se gradi, dokaže i pregleda zasebno); **status** ⬜ nije počelo ·
🟦 u izradi · ✅ gotovo.

| # | Dodatak | Što student vidi i radi | Što uči | Koje brojke mijenja | Sloj | Status |
|---|---|---|---|---|---|---|
| P1 | **Raspored za računalo** | Na laptopu bočni izbornik i dva stupca; mobitel ostaje isti | — (profesor gleda na projektoru) | nijedne | prije demoa ([ADR-027](../records/DECISIONS.md#adr-027)) | 🟦 |
| P2 | **Hotelski pokazatelji** | ADR, RevPAR, popunjenost, GOPPAR po mjesecima, s objašnjenjem riječima | Economics in Hospitality, Accounting | nijedne nove — prikaz onoga što jezgra već računa ili lako izvede | 0 | ⬜ |
| P3 | **Dijelovi hotela (odjeli)** | Sobe, restoran, kafić, spa, rooftop — samo oni koje hotel ima; u svakom: zaposlenici, prihod, troškovi, stanje održavanja, rezultat odjela | Accounting (USALI), Economics (troškovi, investicije) | „ostali prihod" (danas ravnih 30 % prihoda od soba) postaje zbroj stvarnih odjela; održavanje po odjelu | 1 | ⬜ |
| P4 | **Račun dobiti po USALI-ju** | Izvještaj mjeseca: prihod i trošak po odjelima, neraspoređeni troškovi, GOP i GOPPAR | Accounting | nijedne nove — prikaz sloja 1 | 1 | ⬜ |
| P5 | **Profili zaposlenika** | Popis ljudi po odjelima: ime, lik, uloga, iskustvo, motivacija, plaća, opterećenje, rizik odlaska | Management (HRM, motivacija) | moral, iskustvo i fluktuacija hotela postaju zbroj pojedinaca | 2 | ⬜ |
| P6 | **Upravljanje zaposlenicima** | Zaposli, otpusti, povisi plaću, pošalji na obuku, premjesti u drugi odjel | Management (HRM, vođenje) | trošak rada, opterećenje, kvaliteta usluge odjela | 2 | ⬜ |
| P7 | **Sobe i njihovo stanje** | Tlocrt ili popis svih soba (npr. 120): tip, stanje opreme, izvan funkcije, obnovljena; klik → povijest sobe | E-Business (PMS), Economics (investicije) | raspoložive sobe, popunjenost, zadovoljstvo gostiju, ADR po tipu sobe | 3 | ⬜ |
| P8 | **Recenzije s tekstom** | Čitanje recenzija gostiju (ocjena, tekst, na što se žale ili što hvale), odgovor na recenziju | Marketing, upravljanje ugledom | ocjena na portalima, potražnja | 4 | ⬜ |
| P10 | **Ljudi „kao Sims"** | Svaki zaposlenik ima osobnost, dobre i loše strane, trake vještina, jezike i razinu znanja; raste obukom i iskustvom | Management (HRM, motivacija) | kvaliteta usluge odjela, plaća koju traži, rizik odlaska | 2 | ⬜ |
| P11 | **Burza rada** | Kandidati s cijenom i vještinama: jeftin početnik koji raste ili skup profesionalac; izvrstan radnik s vremenom traži povišicu | Management (HRM, zapošljavanje) | trošak rada, kvaliteta, fluktuacija | 2 | ⬜ |
| P12 | **Agencija za strane radnike** | Narudžba skupine radnika (npr. 5), smještaj koji hotel plaća, jezici; pregled koliko donose i koliko koštaju | Management, Economics (trošak rada) | trošak rada i smještaja, kapacitet osoblja, kvaliteta | 2 | ⬜ |
| P13 | **Tjedni ili mjesečni krug** | Profesor pri stvaranju igre bira krug: tjedan ili mjesec | — (tempo nastave) | duljina razdoblja u svim formulama | 1 (temelj) | ⬜ |
| P14 | **AI razgovori sa zaposlenicima i gostima** | Slobodan razgovor u situacijama (povišica, upozorenje, žalba gosta); AI ocjenjuje po rubrici, brojke računaju pravila | Management (vođenje), komunikacija | ishod situacije kroz postojeće mehanike | 2 i 4 | ⬜ |
| P15 | **„Booking" drugih hotela** | Stranica konkurencije s ocjenama i recenzijama drugih hotela iste igre | Marketing, E-Business | nijedne (prikaz) | 4 | ⬜ |
| P9 | **Profesorov pregled v2** | Usporedba hotela: USALI izvještaj, pokazatelji prema konkurenciji (MPI / ARI / RGI) | sve gore | nijedne | uz slojeve | ⬜ |

## Odluke i pretpostavke

Leonove odluke su u [ADR-028](../records/DECISIONS.md#adr-028) (ljudi kao u Simsima, burza rada, agencija i smještaj,
tjedni ili mjesečni krug, AI razgovori, „Booking" drugih hotela, voditelj odjela). Pretpostavke koje čekaju pregled
(🟨) su u [ADR-026](../records/DECISIONS.md#adr-026) i [ADR-028](../records/DECISIONS.md#adr-028) — među njima
**pravilo: nacionalnost određuje samo činjenice (jezici, trošak agencije, smještaj, dozvola), nikad vještinu ni
osobnost**.

## Vezano

- Vizija cijele aplikacije (profesor kreira igru, ulaz kodom kao u Kahootu, više situacija i mjeseci, novi likovi):
  [BACKLOG.md](../records/BACKLOG.md) → „Otvoreno za Leona".
- Postojeće brojke i formule: [DEMO.md](./DEMO.md), istraživanje [HOTELSKI_POKAZATELJI.md](../research/HOTELSKI_POKAZATELJI.md).
