# DEMO §2–§3 — stvarni sloj hotela, tržište i obračun mjeseca

Dio [DEMO.md](../DEMO.md)

## 2 · Stvarni sloj hotela

**Hotel:** 4*, Opatijska rivijera, **120 soba**, otvoren sezonski. Mjerilo je Liburnia Riviera Hoteli 2024.:
popunjenost 67 %, ADR 132 €, trošak osoblja oko 35 % prihoda [IZVOR].

### 2.0 Kategorija (Leon, 04.10.)

**Kategorija (3*, 4*, 5*) je svojstvo hotela, a ne snaga.** Mijenja cijene, očekivanja gostiju, osjetljivost na cijenu,
broj zaposlenih po sobi i troškove. Sve brojke koje ovise o standardu su **tablice po kategoriji u katalogu**
(podaci, ne kod). Svaki indeks se mjeri prema referentnom hotelu **iste kategorije** (§4), pa je hotel s 3* koji je
dobar za svoj standard jednako dobar kao takav hotel s 5*. To je načelo iz VIZIJA §2: „zvjezdice ne znače da je
netko jači, nego drugačiji standard".

| kategorija | lipanj | srpanj | kolovoz | rujan | elastičnost ε |
|---|---|---|---|---|---|
| 3* | 95 € | 120 € | 130 € | 95 € | −1,0 |
| **4*** | **135 €** | **165 €** | **175 €** | **135 €** | **−0,8** |
| 5* | 220 € | 280 € | 300 € | 220 € | −0,5 |

Leon je cijene i osjetljivost potvrdio kao „otprilike da" [PROC]. Poredak elastičnosti (jeftiniji hotel osjetljiviji)
slaže se s istraživanjem: economy −0,31, luksuz −0,15 [IZVOR]. Ostale brojke po kategoriji (zaposleni po sobi,
trošak zaposlenog, CPOR, fiksni troškovi, očekivana ocjena) dobivaju tablicu u planu jezgre; zasad su zadane samo za 4*.

**U demou su svi hoteli 4*** (Leon, 04.10.). Jezgra podržava sve tri kategorije od početka, a miješane kategorije
ulaze kad alat za balans pokaže da je mjerenje prema vlastitoj kategoriji fer.

Pouka iz postojećih simulacija: previše varijabli prekida vezu između odluke i rezultata. Demo zato ima **malo
varijabli, a svaka ima jasan uzrok**.

| skupina | varijabla | jedinica | napomena |
|---|---|---|---|
| **kapacitet** | sobe | broj | 120 |
| | sobe izvan funkcije | broj | zbog kvarova; smanjuje prodaju |
| **cijena i prodaja** | cijena (odluka) | € po noćenju | igrač postavlja za mjesec |
| | prodane sobe-noći | broj | rezultat tržišta (§3) |
| | popunjenost · ADR · RevPAR | % · € · € | izvedeno, formule iz [istrazivanje](../../research/HOTELSKI_POKAZATELJI.md) |
| | grupe | sobe-noći, cijena | iz prihvaćenih ponuda agencija |
| **novac** | prihod soba · ostali prihod | € | ostali prihod (hrana, piće, usluge) = 30 % prihoda soba [PROC] |
| | trošak rada · CPOR · održavanje · marketing · fiksni | € | CPOR = varijabilni trošak po prodanoj sobi |
| | GOP · GOPPAR · GOP marža | € · € · % | izvedeno |
| | blagajna | € | početno stanje + GOP; ispod nule = kredit uz kamatu |
| **osoblje** | stalni · sezonci | broj | sezonci se mogu zaposliti ili pustiti svaki mjesec |
| | plaća u odnosu na tržište | % | 100 % = tržišni prosjek |
| | moral | 0–100 | |
| | fluktuacija | % mjesečno | odlasci / prosječan broj zaposlenih |
| | iskustvo | 0–100 | pada kad dolaze novi ljudi, raste obukom i vremenom |
| | opterećenje | prodane sobe po zaposlenom po danu | iznad praga pada kvaliteta i moral |
| **oprema** | stanje opreme | 0–100 | troši se svaki mjesec; održavanje ga vraća |
| **gosti** | zadovoljstvo gostiju (anketa) | 0–100 | rezultat kvalitete usluge, opreme i vrijednosti za novac |
| | ocjena na portalima | 1,0–10,0 | sporo prati zadovoljstvo, kao prosjek recenzija |
| | broj novih recenzija | broj | razmjeran broju gostiju |

### 2.1 Početne vrijednosti — profil C (prosječan)

Profili A i B su u §5. Svi kreću 1. lipnja.

Brojke žive u `content/hotel/*.json` (kalibrirane alatom za balans); ovdje je samo smjer.

| varijabla | smjer za C | izvor |
|---|---|---|
| cijena (referentna za mjesec) | raste od lipnja do kolovoza, rujan kao lipanj | [PROC] iz LRH prosjeka 132 € i sezonskog hoda |
| bazna popunjenost uz referentnu cijenu | VI, VII, IX kao prosjek hrvatskih hotela; VIII rasprodan: potražnja veća od broja soba (4* u Opatiji u vrhuncu, ADR-014) | [IZVOR] DZS 2025. · [PROC] za VIII |
| stalni / sezonci | stalna jezgra + sezonci koji prate potražnju (plan sezonaca po mjesecu u katalogu) | [PROC] oko LRH-ovih ≈ 0,4 zaposlena po sobi [IZVOR], više u vrhuncu |
| prosječni trošak zaposlenog | jedan iznos za kategoriju | [PROC] iz LRH 22,1 mil. € / 835 zaposlenih [IZVOR] |
| plaća u odnosu na tržište | tržišna | |
| moral · iskustvo | srednje | [PROC] |
| CPOR | jedan iznos za kategoriju | [PROC] |
| fiksni troškovi | toliki da sezonska marža bude u cilju (§11) | [PROC] energija, osiguranje, uprava, najam opreme |
| stanje opreme | iznad praga gašenja soba | [PROC] |
| zadovoljstvo gostiju · ocjena portala | ocjena je neutralna za potražnju (§3.1) | [PROC] |
| blagajna | dovoljna za sezonu bez kredita | [PROC] |

**Cilj kalibracije (ADR-013):** profil C bez odluka igrača ima sezonsku (VI–IX) GOP maržu 35–45 % i nijedan mjesec
iznad 50 %; kolovoz je rasprodan (potražnja veća od broja soba), pa grupa iz S3 istiskuje individualne goste i u
kolovozu donosi manje GOP-a nego u lipnju (ADR-014). Profil C kreće s **50 u svakom
indeksu** (§4) jer su indeksi centrirani na njega. Izmjereni rezultat je u `docs/research/KALIBRACIJA_*.md`.

---

## 3 · Tržište i obračun mjeseca

Redoslijed slijedi faze obračuna (ARHITEKTURA §7.5). `D` = broj dana u mjesecu, `ref` = referentna vrijednost za
taj mjesec (§2.1).

### 3.1 Potražnja i prodaja

```text
raspoloživo     = (120 − sobe izvan funkcije) × D
potražnja       = 120 × D × bazna_popunjenost(mj) × f_cijena × f_ugled × f_marketing
f_cijena        = 1 + ε × (cijena / cijena_ref − 1)              ε i cijena_ref po kategoriji (§2.0), ograničeno na 0,3–1,6
f_ugled         = 1 + 0,0054 × (10 × ocjena_portala − 84)
f_marketing     = 1 + 0,15 × M / (M + 15 000 €)                   M = mjesečni marketing
kapacitet_osoblja = zaposleni × D × 2,4
prodano         = min(grupe + potražnja, raspoloživo, kapacitet_osoblja)
```

- **Elastičnost** (za 4* ε = −0,8, tablica u §2.0) je kalibracijska varijabla [PROC]. Izvor daje −0,14 do −0,31 za
  urbane hotele u SAD-u [IZVOR], ali turistički hotel uz jaku konkurenciju osjetljiviji je na cijenu. Linearni oblik je namjeran: lako se
  objasni studentu i računa se cijelim brojevima.
- **Ugled:** +1 bod ocjene na ljestvici 0–100 daje +0,54 % potražnje, prema Cornellovoj studiji [IZVOR]. Ocjena 8,4
  je neutralna.
- **Marketing** ima opadajući povrat: 15 000 € daje +7,5 %, a 45 000 € tek +11,25 %. Tako „sve u marketing" nije
  dominantna strategija.
- **Grupe se prodaju prve.** Kad je potražnja veća od kapaciteta, grupa **istiskuje** skuplje individualne goste.
  To je lekcija iz S3 (§7).

### 3.2 Novac

```text
prihod_soba     = individualno_prodano × cijena + grupno_prodano × grupna_cijena
ostali_prihod   = 0,30 × prihod_soba
trošak_rada     = zaposleni × 2 200 € × plaća% + novi_zaposleni × 1 500 € + obuka
troškovi        = trošak_rada + 22 € × prodano + održavanje + marketing + fiksni + kompenzacije iz situacija
GOP             = prihod_soba + ostali_prihod − troškovi
blagajna       += GOP − kapitalna ulaganja;  ispod nule kamata 1 % mjesečno
```

Trošak novog zaposlenog (1 500 €) je samo zapošljavanje i uvođenje [PROC]. Izvori daju 5 900 USD do 10 000 € za
cijelu zamjenu [IZVOR], ali ostatak te cijene ovdje se plaća kroz pad iskustva.

### 3.3 Ljudi, oprema i gosti

```text
opterećenje     = prodano / (zaposleni × D)                  zdravo ≤ 1,8 · kazna raste do 60 pri 2,4
moral'          = moral + 0,3 × (cilj_morala − moral)
cilj_morala     = 55 + 2 × (plaća% − 100) − 0,5 × kazna_opterećenja + bonus_obuke + Δ iz situacija
fluktuacija     = 3 % + 0,15 % × max(0, 60 − moral)           mjesečno; odlasci se zamjenjuju automatski
iskustvo'       = mješavina (stari ljudi zadrže iskustvo, novi dolaze s 20) + 3 + bonus_obuke
oprema'         = oprema − 4 − (1 ako je opterećenje > 1,8) + učinak održavanja
sobe izvan funkcije = max(0, (40 − oprema) / 4) + kvarovi iz situacija
kvaliteta_usluge = 0,4 × iskustvo + 0,3 × moral + 0,3 × (100 − kazna_opterećenja)
vrijednost      = 70 − 100 × (cijena / cijena_ref − 1),  ograničeno 0–100
cilj_zadovoljstva = 0,45 × kvaliteta_usluge + 0,30 × oprema + 0,25 × vrijednost
zadovoljstvo'   = 0,5 × zadovoljstvo + 0,5 × cilj_zadovoljstva + Δ iz situacija
ocjena_portala' = ocjena + 0,25 × ((5,6 + 0,04 × zadovoljstvo') − ocjena) + odgovaranje na recenzije
```

Lanac **zaposlenik → usluga → gost → recenzija → potražnja** je service-profit chain [IZVOR za smjer, PROC za
jačinu]. Ocjena portala sporo prati zadovoljstvo, kao prosjek recenzija. Jedan dobar mjesec ne popravi ugled.

---
