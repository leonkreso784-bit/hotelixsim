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

## Rezultat kalibracije (alat za balans)

> Mijenjani su samo podaci u `content/hotel/catalog.json` i `content/hotel/profiles.json`; formule, testni katalog i
> `scripts/oracle-jezgra.mjs` nisu dirani. Jedina izmjena koda u jezgri je granica provjere kataloga za baznu
> popunjenost (1–10 000 → 1–20 000 bp), jer ADR-014 traži potražnju veću od broja soba. Mjereno:
> `cargo run -q -p balance --release -- --seeds 1000 --check` → sve provjere DEMO §11 i nova provjera §7 S3 prolaze
> (izlaz 0).

### Kalibracija 1 (jezgra 11/12): što je promijenjeno i zašto

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

### Kalibracija 2: kolovoz rasprodan, likvidnost bez gornje granice (ADR-014, jezgra 13/14)

Polazno stanje poslije jezgre 13 (likvidnost bez granice, podaci iz kalibracije 1): padala su tri kriterija. B je
kretao s F 83,08 (cilj 75), razlika medijana profila bila je 16,6 boda (A −13,19, B +3,41, C −6,22), a B je imao
najveći medijan pod svim strategijama. Lekcija S3 bila je obrnuta (kolovoz +25 598 €, lipanj +19 184 €).

| podatak | prije → poslije | zašto |
|---|---|---|
| bazna popunjenost VIII | 95,5 % → **106 %** | ADR-014 t. 1: potražnja veća od broja soba. Potražnja referentnog C je 3 937 soba-noći prema 3 720 raspoloživih, pa grupa istiskuje punih 175 soba-noći. Lekcija se okreće već na 100 % (kolovoz −8 720 €); 106 % ostavlja rezervu za pad ocjene |
| plan sezonaca VIII / IX | 40 / 30 → **44 / 34** | rasprodan kolovoz dizao je maržu VIII na 51,0 %; četiri sezonca više (≈ 8 800 €) daju 49,3 %. Rujan je 34 jer plan smije pasti najviše 10 mjesečno |
| najviše sezonaca (`seasonal_max`) | 40 → **50** | bez toga plan od 44 nije dopušten; igrač u kolovozu može zaposliti više |
| B fiksni | 112 500 → **145 000 €** | glavni kompromis, vidi točku 3 ispod. Uz likvidnost bez granice svaki euro nižih fiksnih troškova svaki mjesec ulazi u blagajnu, pa F profila B raste sam od sebe |
| B blagajna | 600 000 → **678 000 €** | F 74,98 (cilj 75) uz više fiksne troškove |
| B stalni | 30 → **28** | s 30 stalnih B pobjeđuje i pod „lovcem na popunjenost"; s 28 ga lovac preoptereti (opterećenje iznad 1,8) |
| B plaća | 97 → **95 %** | i dalje „malo ispod tržišne"; moral B-a sad kroz sezonu polako klizi prema nižem cilju ako igrač ništa ne radi |
| A fiksni | 177 000 → **170 000 €** | viši od C (kredit za prošlu obnovu), ali manje, jer svaki euro razlike sad kroz sezonu ruši F |
| A blagajna | 120 000 → **37 000 €** | F 34,98 (cilj 35): „malo novca" je sad razina blagajne, a ne razlika u fiksnim troškovima |
| A stalni | 30 → **28** | prezaposleni ljudi: manje ljudi na istom poslu |
| A iskustvo / oprema | 60 / 60 → **70 / 75** | ugled A dolazi od dobrog proizvoda (oprema iz obnove za koju još plaća kredit, iskusni ljudi). Viši cilj zadovoljstva usporava pad G i R kroz sezonu |
| A opterećenje (početak) | 1,83 → **2,20** | uz bolju opremu i iskustvo drži O na 49,75 (cilj 50); i dalje „iznad 1,8" kako traži §5 |

Nepromijenjeno: cijene, popunjenost VI/VII/IX, plan sezonaca VI/VII, trošak zaposlenog, CPOR, ostali prihod, sve
mehanike i sve vrijednosti profila C; A zadovoljstvo, ocjena i moral; B zadovoljstvo, ocjena, moral, iskustvo, oprema.

**Lekcija S3 (provjera §7, isti odgovor „kako je ponuđeno": 25 soba × 7 noći, −25 %, referentni C, zadani plan):**

| | kolovoz | lipanj |
|---|---|---|
| prije (kolovoz 95,5 %) | +25 598 € | +19 184 € |
| poslije (kolovoz 106 %) | **−9 953 €** | +19 184 € |

U kolovozu grupa sad zauzima sobe koje bi platili individualni gosti po punoj cijeni, pa prihvaćanje gubi gotovo
10 000 € GOP-a; u lipnju puni prazne sobe. Provjera u alatu za balans pada ako kolovoz donese koliko i lipanj ili više.

**Obnova u lipnju (proba, zadani plan, srednji odgovor na situacije), napredak bez obnove → s obnovom:**

| | A | B | C |
|---|---|---|---|
| kalibracija 1 (likvidnost do 100) | −1,05 → +7,63 | −7,21 → +1,49 | −0,26 → +10,12 |
| jezgra 13 (bez granice, stari podaci) | −8,35 → −3,46 | +14,87 → +14,04 | +0,30 → +7,57 |
| **kalibracija 2** | **−0,39 → +5,02** | **−3,03 → +2,03** | **−0,90 → +6,21** |
| F na kraju sezone, bez → s obnovom | 29,53 → 23,39 | 77,72 → 70,83 | 48,59 → 42,09 |

Obnova više nije besplatna: od 150 000 € F pada za 6–7 bodova do kraja sezone (prije je rastao). Ukupni učinak je i
dalje pozitivan (+5,1 do +7,1), jer oprema (+25) i zadovoljstvo (+3) dižu O, G i R. Obnova je najbolja proba za A i
C, ali nije dominantna: za B je bolja „plaća 115" (+3,41 prema +2,03).

### Izlaz alata (`render`, 1 000 sjemena)

```text
== Početak (§11.1): G R Z O F ==
  A: 60.00  75.00  29.75  49.75  34.98
  B: 45.00  35.71  54.05  39.50  74.98
  C: 50.00  50.00  50.00  50.00  50.00
== Ekonomija referentnog hotela (§11.2), marža GOP-a ==
  jun: 22.48 %
  jul: 38.65 %
  aug: 49.29 %
  sep: 20.33 %
  sezona VI–IX: 35.81 %
  kolovoz: grupa istiskuje individualne goste: da
  S3 grupa, dobit GOP-a od prihvaćanja: kolovoz -9 953 EUR, lipanj 19 184 EUR (§7: kolovoz < lipanj)
== Medijan napretka po profilu i strategiji (§11.3), bodova ==
  A stedisa    -4.93
  A brizni     14.41
  A lovac      -7.86
  A nasumicni  -5.09
  B stedisa    -2.56
  B brizni     25.70
  B lovac      -5.90
  B nasumicni  -3.21
  C stedisa    -7.54
  C brizni     20.86
  C lovac      -4.32
  C nasumicni  -3.06
  profil A (sve strategije): -4.93
  profil B (sve strategije): -2.56
  profil C (sve strategije): -4.32
== Probe krajnjih radnji (§11.4), napredak A / B / C ==
  cijena 60 %              A=-18.29  B=-23.83  C=-17.78
  cijena 150 %             A=-14.05  B=-18.38  C=-14.85
  marketing 40 000 EUR     A=0.44  B=-2.73  C=-0.55
  sezonci +korak           A=-13.62  B=-12.06  C=-10.95
  sezonci −korak svaki mjesec A=-30.35  B=-19.50  C=-17.85
  plaća 115                A=0.29  B=3.41  C=2.53
  plaća 90                 A=-5.76  B=-3.84  C=-6.82
  obuka Full               A=-0.18  B=-2.64  C=1.24
  održavanje Intensive     A=-0.26  B=-2.17  C=1.14
  održavanje Deferred      A=-1.64  B=-4.46  C=-3.64
  odgovaranje da           A=1.10  B=-1.39  C=1.03
  obnova u lipnju          A=5.02  B=2.03  C=6.21
== Ponovljivost (§11.5): da ==
Sve provjere prolaze.
```

### Za Leonovu provjeru

1. **Kolovoz je rasprodan (106 %).** Referentni C ima potražnju 3 937 soba-noći za 3 720 soba i prodaje sve. Kod B
   (ocjena 8,0) potražnja je ≈ 3 867, pa grupa kod njega istiskuje samo dio od 175 soba-noći, kod A (ocjena 9,1) sve.
   To je namjerno: što je hotel traženiji, to je grupa s popustom skuplja.
2. **Marža: lipanj 22,5 %, srpanj 38,7 %, kolovoz 49,3 %, rujan 20,3 %, sezona 35,8 %.** Sezona je blizu donje granice
   35 %; rujan je pao (22,2 → 20,3 %) jer plan smije pustiti najviše 10 sezonaca, pa rujan nosi 34 umjesto 30.
3. **B ima fiksne troškove samo 3 % niže od C (145 000 prema 150 000 €).** §5 kaže „niži fiksni troškovi". S
   likvidnošću bez granice razlika u fiksnim troškovima svaki mjesec ulazi u blagajnu i F profila B raste sam od sebe;
   ocjena napretka to još pojača, jer B kreće s F 75 i gore ima samo 25 bodova prostora. Izmjereno (sve ostalo kao
   sada, blagajna B svaki put postavljena na F 75):

   | B fiksni | medijan A / B / C | razlika | provjere |
   |---|---|---|---|
   | 112 500 € (kalibracija 1) | −4,93 / +3,76 / −4,32 | 8,69 | B najbolji pod svim strategijama, obnova dominantna |
   | 130 000 € | −4,93 / +3,26 / −4,32 | 8,19 | B najbolji pod svim strategijama |
   | 140 000 € | −4,93 / −0,69 / −4,32 | 4,24 | prolazi, tijesno |
   | **145 000 €** | **−4,93 / −2,56 / −4,32** | **2,37** | **prolazi** |
   | 150 000 € (= C) | −4,93 / −4,34 / −4,32 | 0,61 | prolazi, ali B više nije „niži" |

   Ako priča traži jasno niže fiksne troškove za B, to se ne može samo podacima: trebala bi promjena formule F ili
   ocjene napretka, a to je nova odluka.
4. **A dobiva dio napretka od početnog opterećenja.** A kreće s opterećenjem 2,20 (stanje prije sezone), a s planom
   sezonaca u lipnju radi na ≈ 1,6, pa kazna opterećenja u O nestane sama od sebe. Sa starim podacima A (30 stalnih,
   oprema i iskustvo 60, opterećenje 1,83) A pada na medijan −12,96 i razlika profila je 10,4 boda: A-ov ugled (G 60,
   R 75) kroz sezonu klizi prema razini C, a likvidnost bez granice više ne vraća F kao prije. Prepoznaješ li hotel
   koji je ugledan zbog dobre opreme i iskusnih ljudi, ali ih je premalo?
5. **Obnova je najbolja proba za A i C** (vidi tablicu gore): sad košta 6–7 bodova F, ali se u istoj sezoni isplati.
   Je li to namjera (obnova od 150 000 € kao dobra odluka već u prvoj sezoni) ili treba skuplja ili slabija obnova?
