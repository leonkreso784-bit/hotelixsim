# Kalibracija početnih indeksa i marže (demo)

> **Prijedlog za Leonovu provjeru, ništa nije odlučeno.**
> Računa skripta `kalibracija.py` (scratchpad sesije, pokretanje `PYTHONUTF8=1 python kalibracija.py`); model je DEMO.md §2.1–§4, profili A, B, C, četiri mjeseca (VI–IX) bez ijedne odluke.
> Oznake: **[IZR]** izračunato modelom iz DEMO.md · **[PRET]** pretpostavka ovog dokumenta · **[IZVOR]** iz [HOTELSKI_POKAZATELJI.md](./HOTELSKI_POKAZATELJI.md).

## 0 · Što je potvrđeno

**[IZR]** Reprodukcija s fiksnim stanjem (bez drifta) daje točno brojke iz zadatka: marža VI 33,6 %, VII 50,4 %, VIII 55,5 %, IX 34,3 %.
S driftom stanja (gost, moral, ocjena se mijenjaju iz mjeseca u mjesec) je 33,6 / 50,4 / 55,4 / 34,2 %, sezona 45,2 %. Trošak zapošljavanja zamjena (§3.2, 1 500 € po odlasku) skida još ≈ 0,5 boda.

**Početni indeksi, apsolutne formule [IZR]:**

| profil | G | R | Z | O | F | zbroj |
|---|---|---|---|---|---|---|
| C | 70,0 | 50,0 | 54,25 | 70,0 | 54,6 | **298,8** (ne 250) |
| A [PRET] | 78,0 | 75,0 | 27,2 | 70,2 | 44,1 | 294,5 |
| B [PRET] | 66,0 | 35,7 | 58,3 | 59,5 | 63,8 | 283,3 |

A i B u §5 nemaju zadane stanje, samo smjer. Njihove stvarne vrijednosti u skripti su moje pretpostavke (A: 47 zaposlenih, moral 35, oprema 65, iskustvo 62, fiksni 135 000 €; B: moral 58, zadovoljstvo 66, oprema 40, iskustvo 55, plaća 97 %). Zato su brojke za A i B ilustrativne, a za C točne.

**Usputni nalaz [IZR]:** profil C **bez odluka** nije miran. Do kolovoza G pada 70 → 66, Z 54 → 49, O 70 → 66 (opterećenje 1,87 u srpnju i 1,99 u kolovozu, iznad praga 1,8), a F raste 54,6 → 65 jer se skuplja blagajna. Pasivan hotel dakle gubi na tri indeksa, a dobiva na jednom. Ocjena napretka mora to uračunati (npr. mjeriti prema pasivnom C).

---

## 1 · Centriranje G, Z, O na referentni hotel kategorije

`indeks = 50 + k × (x − x_ref)`, `x_ref` = vrijednost referentnog 4* hotela (C iz §2.1: G 70, Z 54,25, O 70), k = 1. R je već centriran (8,4 → 50), F je već relativan.

| početni indeksi [IZR], k = 1 | G | R | Z | O | F | zbroj | cilj iz §5 |
|---|---|---|---|---|---|---|---|
| C | 50 | 50 | 50 | 50 | 54,6 | 254,6 | 50 u svemu |
| A [PRET] | 58 | 75 | 23 | 50 | 44 | 250,2 | 60 / 75 / 30 / 50 / 35 |
| B [PRET] | 46 | 36 | 54 | 40 | 64 | 239,1 | 45 / 35 / 55 / 40 / 75 |

- **Prednosti:** C postaje 50 bez diranja stanja i ekonomije. A i B padaju vrlo blizu §5 (G 58 ≈ 60, O 50 = 50, Z 23 ≈ 30, B: G 46 ≈ 45, Z 54 ≈ 55, O 39,5 ≈ 40) — što upućuje da je tablica u §5 i bila pisana kao odstupanje od prosjeka. Ista formula radi za 3* i 5* (tablica `x_ref` po kategoriji), što je i načelo §2.0.
- **Zbroj:** G, R, Z, O su gotovi. Ostaje F: C ima 54,6 jer je likvidnost 65 (blagajna 300 000 €). F = 50 daje blagajna C **230 000 €** [IZR: F = 0,7·50 + 0,3·L, L = blagajna / 460 000 · 100]. Tada je C = 250. B ostaje na zbroju ≈ 239 jer mu F 64 (cilj 75) treba niže fiksne troškove (≈ 71 000 €/mj, F 77,5) [IZR]; A treba fiksne ≈ 170 000 € da F padne na 35 [IZR]. To su parametri za alat za balans, ne za formulu.
- **Osjetljivost na k** [IZR]: k = 0,5 daje A 259,6 / B 244,3; k = 1 daje 250,2 / 239,1; k = 1,5 daje 240,8 / 233,8. Veći k povećava rasipanje indeksa unutar profila (A-ov Z pada do 10), ali ne mijenja redoslijed.
- **Cijena:** nova konstanta po kategoriji (`x_ref` za G, Z, O) i formula u §4 (podatak + jedna linija koda). Indeksi više nisu apsolutni, pa je „G = 78" za igrača manje čitljivo (odmah ga se može objasniti: „8 bodova bolje od prosječnog 4*"). Ograničenje 0–100 se i dalje primjenjuje.

## 2 · Mijenjati početne vrijednosti C da indeksi budu 50

Apsolutne formule ostaju. Treba G = 50 (zadovoljstvo 50), Z = 50 (moral ≈ 51,8, fluktuacija 4,2 %), O = 50 (npr. oprema 35 i iskustvo 30, ili oprema 45 i iskustvo 22,5).

| C [IZR] | G | R | Z | O | marža VI / VII / VIII / IX | ocjena u IX | sobe izvan funkcije |
|---|---|---|---|---|---|---|---|
| §2.1 (sada) | 66,6 | 47,0 | 50,9 | 70,7 | 33,6 / 50,4 / 55,4 / 34,2 | 8,29 | 0 |
| 2b: zad 50, moral 51,8, oprema 35, iskustvo 30 | 53,9 | **33,5** | 50,4 | 51,8 | 33,6 / 50,0 / 54,9 / 33,2 | 7,94 | **1,25 → 1,75** |
| 2c: oprema 45, iskustvo 22,5 | 54,6 | 34,1 | 50,4 | 52,7 | 33,6 / 50,0 / 54,9 / 33,2 | 7,95 | 0 |

- **Što to radi potražnji i marži [IZR]:** malo. Ocjena portala se pomiče prema 5,6 + 0,04 × zadovoljstvo, pa pada s 8,4 na ≈ 7,95, a ugled mijenja potražnju za ≈ −1,4 % u kolovozu (3 037 prema 3 082 soba-noći). Marža pada 0–1 bod, sezonski prihod −1,1 %. Problem je drugdje: R se sruši s 50 na 34 do rujna (nije više „prosječan"), a „prosječni" 4* hotel s opremom 35 je ispod praga 40 gdje se sobe gase.
- **Prednosti:** formule i dokument §4 ostaju netaknuti.
- **Cijena:** C više nije prosječan 4* hotel (oprema 35–45, iskustvo 22–30 su loši, ne prosječni), brojke su [PRET] pa nema izvora za njih, §2.1 se mijenja, a A i B treba **ponovno izvesti** jer ovise o istom okviru. Uz to ne rješava G, Z, O za 3* i 5*: za svaku kategoriju treba ponoviti ručno traženje. Isto za F: treba i blagajna 230 000 €.

## 3 · Kalibracija marže

### 3.1 Je li cilj trebao biti sezonski

- **[IZVOR]:** u HOTELSKI_POKAZATELJI.md **nema** sezonske ni godišnje GOP marže za hrvatske hotele (izričito: „Nema javnih podataka: … GOP marža HR hotela"). Postoji samo (a) HotStats Europa, lipanj 2019: GOP marža ≈ 44,6 %, plaće 27,5 % prihoda, dakle **jedan vršni mjesec**, ne sezona i ne Jadran; (b) LRH 2024: prilagođena EBITDA marža 23,0 % (EBITDA nije GOP, GOP je viši), trošak osoblja ≈ 35 % prihoda; (c) udio troška rada 37,3 % (H1 2025) i 38,8 % (H1 2026) za 13 listanih turističkih tvrtki.
- Cilj „30–40 %" u DEMO:163 je [PRET] bez izvora, i to **godišnji**, a demo ima samo 4 vršna mjeseca. Godišnje bi trebali zatvoreni mjeseci nositi fiksne troškove: pretpostavimo da osam mjeseci van sezone nosi udio x od 120 000 €/mj i nema prihoda [PRET]: x = 0 → 45,2 %, x = 0,1 → 40,9 %, **x = 0,25 → 34,5 %**, x = 0,5 → 23,7 % [IZR]. Znači: današnji model uz skromne troškove izvan sezone već daje „godišnjih" 30–40 %. Ali demo to ne simulira, pa se cilj ne može provjeriti.
- **Najjači pokazatelj da je model nerealan nije marža nego udio rada:** model ima **19,7 %** (VI 24,2 %, VIII 15,7 %), a izvori 35–39 %. LRH ima 0,4 zaposlena po sobi **prosječno godišnje**, a u vrhuncu >1 000. Naš model ima 50 zaposlenih konstantno (0,42/sobu) i u vrhuncu, uz opterećenje 1,99.

### 3.2 Parametri (samo C, sezona VI–IX, [IZR])

| promjena | sezonska marža | marža po mjesecima VI / VII / VIII / IX | udio rada |
|---|---|---|---|
| sada | 45,2 % | 33,6 / 50,4 / 55,4 / 34,2 | 19,7 % |
| fiksni 120 → 150 000 € | ≈ 40 % (tražen 149 081 €) | ≈ 27,0 / 45,6 / 51,1 / 27,7 | 19,7 % |
| fiksni 180 000 € | 34,5 % | 20,4 / 40,7 / 46,8 / 21,1 | 19,7 % |
| trošak zaposlenog 2 200 → 2 600 € | 41,6 % | 29,2 / 47,1 / 52,5 / 29,8 | ≈ 23 % |
| konstantno 70 zaposlenih (40 sezonaca) | 37,4 % | 23,9 / 43,3 / 49,1 / 24,7 | 27,5 % |
| konstantno 89 zaposlenih (udio rada 35 %) | 29,9 % | 14,7 / 36,6 / 43,2 / 15,6 | 35,0 % |
| zaposleni prate potražnju 58/63/67/59 (opterećenje ≈ 1,5) | 40,6 % | 29,7 / 45,8 / 50,1 / 29,9 | 24,3 % |
| ostali prihod 30 → 20 % | 40,6 % | 28,1 / 46,2 / 51,7 / 28,7 | 19,7 % |
| CPOR 22 → 30 € | 41,2 % | 29,0 / 46,6 / 51,9 / 29,6 | 19,7 % |
| trošak ostalih odjela = 44 % ostalog prihoda [PRET] | 35 % | — | 19,7 % |
| **zaposleni 58/63/67/59 + fiksni 130 000 €** | **38,8 %** | **27,5 / 44,1 / 48,6 / 27,8** | 24,3 % |
| zaposleni 58/63/67/59 + ostali 25 % + fiksni 130 000 € | 36,4 % | 24,6 / 41,9 / 46,6 / 24,9 | 25,2 % |

Za jedan parametar [IZR]: sezonska marža 40 % traži fiksne 149 000 €/mj ili trošak zaposlenog 2 782 € ili trošak ostalih odjela 23 % ostalog prihoda; 35 % traži 177 000 € / 3 341 € / 44 %; 30 % traži 205 000 € / 3 900 € / 66 %.

**Nalazi:**
1. **Raspon po mjesecima (VI/IX ≈ 25–30 %, VIII ≈ 50 %) ne smanjuje nijedan parametar** jer ga drži sezonski hod cijene i popunjenosti (175 € × 83 % prema 135 € × 72 %) uz stalne troškove. Realan je, uz uvjet da osoblje prati potražnju (to ga malo sužava).
2. **Ostali prihod (30 %) nema vlastitog troška** (hrana i piće, DEMO §3.2) — dolazi gotovo besplatno. Uz to LRH ADR 132 € **uključuje pansion** [IZVOR], pa 30 % povrh toga može biti dvostruko brojanje. Ovo je najveći strukturni izvor previsoke marže, ali traži promjenu formule (novi trošak) i izvor za udio troška hrane — kojeg nemamo.
3. **Sezonci:** u sezonskom hodu (58 → 67 → 59) opterećenje pada s 1,87–1,99 na ≈ 1,5. To makne i nuspojavu da prosječni C u srpnju i kolovozu pretrpi kaznu opterećenja (O i Z padaju 5 bodova), što je u §3.3 vjerojatno nenamjerno.

## Preporuka (jedna)

**Opcija 1 (centriranje) + u §2.1 sezonski hod sezonaca i fiksnih troškova, cilj marže prepisan kao sezonski.**

1. **Indeksi:** G, Z, O = `50 + (x − x_ref(kategorija))`; blagajna C 230 000 € da F bude 50. Čuva C kao prosječan hotel, A i B padaju blizu §5, bez nove formule za kategorije, pa je zbroj C točno 250.
2. **Marža:** (podaci, ne kod) sezonci tako da zaposleni 58 / 63 / 67 / 59 (opterećenje ≈ 1,5) i fiksni 130 000 € → sezonska marža ≈ 39 % (VI/IX ≈ 28 %, VII 44 %, VIII 49 %), blizu HotStats vršnog mjeseca 44,6 % [IZVOR] i ispod ranijih 55 %.
3. **Cilj:** napisati kao **„sezonska (VI–IX) GOP marža 35–45 %, nijedan mjesec iznad ≈ 50 %"** [PRET, bez izvora za Jadran]; godišnji cilj 30–40 % maknuti dok demo ne simulira zatvorene mjesece. Leonu je to pitanje za provjeru stručnjaka (profesori ili STR/HUT podaci), jer u istraživanju nema izvora.

Otvoreno za Leona: (a) je li 30 % ostalog prihoda previsok i treba li trošak hrane i pića; (b) udio rada 24 % je i dalje ispod 35 % iz izvora — za 35 % trebalo bi ≈ 89 zaposlenih konstantno (marža ≈ 30 %, opterećenje ≈ 1), što mijenja značenje praga 1,8.

---

## Rezultat kalibracije (alat za balans, jezgra 11/12)

> Mijenjani su samo podaci u `content/hotel/catalog.json` i `content/hotel/profiles.json`; formule, testni katalog i
> `scripts/oracle-jezgra.mjs` nisu dirani. Mjereno: `cargo run -q -p balance --release -- --seeds 1000 --check` → sve
> provjere DEMO §11 prolaze (izlaz 0).

### Što je promijenjeno i zašto

| podatak | prije → poslije | zašto |
|---|---|---|
| plan sezonaca VI/VII/VIII/IX | 20/20/20/20 → **28/34/40/30** | osoblje prati potražnju (preporuka §3.2); 40 je najviše što plan dopušta, a skok VIII→IX je točno jedan korak (10) |
| bazna popunjenost VIII | 83 % → **95,5 %** | bez toga grupa iz S3 ne istiskuje nikoga (vidi „Za Leona" ispod) |
| fiksni troškovi C | 120 000 → **150 000 €** | spušta sezonsku maržu u 35–45 % i drži kolovoz ispod 50 % |
| sezonci C (početak) | 20 → **28** | = plan lipnja, inače lipanj plaća zapošljavanje (ADR-013) |
| sezonci A i B (početak) | 20 → **28** | isto; s 20 su A i B u lipnju plaćali 8 zapošljavanja koje C nema, pa je pravednost (§11.3) padala na 12,9 boda |
| A zadovoljstvo | 78 → **80** | G točno 60 (G = zadovoljstvo centrirano na C) |
| A moral | 42 → **40** | Z 29,75 (cilj 30); moral ≈ 35 iz §5 dao bi Z ≈ 23 |
| A opterećenje (početak) | 1,95 → **1,83** | O 49,25 (cilj 50), i dalje „iznad 1,8" kako traži §5 |
| A fiksni | 150 000 → **177 000 €** | F 35,09 (cilj 35); 27 000 € iznad C = rata kredita za prošlu obnovu |
| B zadovoljstvo | 70 → **65** | G 45 (cilj 45); jedino zadovoljstvo hrani G, pa ga se ne može postići drugim poljem |
| B fiksni | 90 000 → **112 500 €** | F 74,99 (cilj 75); 37 500 € ispod C (stari, otplaćeni hotel bez kredita) |

Nepromijenjeno: cijene, popunjenost VI/VII/IX (DZS), trošak zaposlenog, CPOR, ostali prihod, mehanike, A ocjena 9,1 i
blagajna 120 000 €, B ocjena 8,0, oprema 40, iskustvo 55, blagajna 600 000 €, plaća 97 %.

### Izlaz alata (`render`, 1 000 sjemena)

```text
== Početak (§11.1): G R Z O F ==
  A: 60.00  75.00  29.75  49.25  35.09
  B: 45.00  35.71  54.05  39.50  74.99
  C: 50.00  50.00  50.00  50.00  50.00
== Ekonomija referentnog hotela (§11.2), marža GOP-a ==
  jun: 22.48 %
  jul: 38.65 %
  aug: 49.12 %
  sep: 22.24 %
  sezona VI–IX: 35.90 %
  kolovoz: grupa istiskuje individualne goste: da
== Medijan napretka po profilu i strategiji (§11.3), bodova ==
  A stedisa    -10.62
  A brizni     8.16
  A lovac      -4.99
  A nasumicni  -3.88
  B stedisa    -18.27
  B brizni     8.22
  B lovac      -5.96
  B nasumicni  -6.69
  C stedisa    -12.02
  C brizni     13.93
  C lovac      -4.04
  C nasumicni  -2.76
  profil A (sve strategije): -4.99
  profil B (sve strategije): -5.96
  profil C (sve strategije): -4.04
== Probe krajnjih radnji (§11.4), napredak A / B / C ==
  cijena 60 %              A=-24.95  B=-17.08  C=-16.81
  cijena 150 %             A=-10.81  B=-19.35  C=-12.53
  marketing 40 000 EUR     A=-0.56  B=-6.92  C=0.28
  sezonci +korak           A=-3.07  B=-8.73  C=-3.26
  sezonci −korak svaki mjesec A=-26.50  B=-31.45  C=-24.07
  plaća 115                A=4.90  B=2.99  C=6.32
  plaća 90                 A=-9.42  B=-12.85  C=-8.87
  obuka Full               A=2.02  B=-3.80  C=3.30
  održavanje Intensive     A=2.09  B=-3.68  C=3.17
  održavanje Deferred      A=-4.69  B=-12.22  C=-3.94
  odgovaranje da           A=0.27  B=-5.68  C=1.90
  obnova u lipnju          A=7.63  B=1.49  C=10.12
== Ponovljivost (§11.5): da ==
Sve provjere prolaze.
```

### Za Leonovu provjeru

1. **Kolovoz 95,5 % umjesto dogovorenih najviše 95 %.** Istiskivanje traži potražnju > 3 720 − 175 = 3 545
   soba-noći. Referentni C do kolovoza blago gubi ocjenu (8,40 → 8,37, jer mu je cilj zadovoljstva ≈ 67–68, a kreće s
   70), pa je f_ugled 0,9984. Uz 95 % potražnja je 3 528 i grupa ne istiskuje nikoga; najmanja vrijednost koja radi je
   ≈ 95,5 % (potražnja 3 547). Istiskivanje je pritom **tanko**: samo ≈ 2 soba-noći kod pasivnog C (kod igrača s boljom
   ocjenom ili marketingom više). Za vidljiviju lekciju: 96 % daje ≈ 20 soba-noći (marža VIII 49,3 %).
2. **Lipanj i rujan ≈ 22 %, kolovoz ≈ 49 %.** Sezona je 35,9 %, blizu donje granice 35 %. Raspon po mjesecima ostaje
   (nalaz 1 u §3.2): drži ga sezonski hod cijene i popunjenosti uz stalne troškove. Ako želiš sezonu bliže sredini
   cilja, najjeftinije je podići srpanj (sada 78 %, a kolovoz 95,5 % — taj skok je velik).
3. **Pravednost prolazi, dominacija je blizu:** medijani profila −4,99 / −5,96 / −4,04 (razlika 1,92, prag 5). Nijedna
   proba nije dominantna, ali „obnova u lipnju" je najbolja za A i C; od dominacije je dijeli samo B (kod B je bolja
   „plaća 115").
