# HotelixSim — demo: opseg i dizajn igre

> **Status:** NACRT (2026-10-04). Arhitektura: [ARHITEKTURA.md](./ARHITEKTURA.md) v2.
> **Sve brojke i pravila ovdje su zapisane pretpostavke** dok profesori ne odgovore
> ([PITANJA_ZA_PROFESORE.md](./PITANJA_ZA_PROFESORE.md)) i dok ih alat za balans ne izmjeri.
> Oznake: **[IZVOR]** brojka iz istraživanja ([istrazivanje/](./istrazivanje/)) · **[PROC]** procjena, kalibrira se.
> Leon (student hotelijerstva) je stručnjak za domenu: svaka stavka ovdje je za njegovu provjeru.
>
> U dokumentu su brojke pisane decimalno radi čitljivosti. Jezgra ih računa cijelim brojevima (novac u centima,
> udjeli u baznim bodovima, ARHITEKTURA §7.2).

## Sadržaj

1. Opseg demoa
2. Stvarni sloj hotela
3. Tržište i obračun mjeseca
4. Pet indeksa
5. Profili A, B, C
6. Katalog mehanika
7. Situacije s rubrikama
8. Relativna ocjena
9. Splet okolnosti
10. Roboti
11. Što se provjerava prije demoa

---

## 1 · Opseg demoa

### 1.1 Što demo jest

Jedna igra: **igrač + tri robota**, svaki sa svojim hotelom, kroz **četiri runde**. Demo je prvi rez prave
arhitekture, a ne prototip koji se baca: ista jezgra, isti dnevnik, isti API.

| | demo | pilot (kasnije) |
|---|---|---|
| vrijeme | **demo način**: rundu zatvara **gumb** | kalendar kolegija |
| igrači | 1 čovjek + 3 robota | grupa studenata |
| prijava | **poveznica stvori gosta** | pravi pružatelj prijave (ARHITEKTURA §12) |
| sezona | 4 runde: lipanj, srpanj, kolovoz, rujan | po dogovoru s profesorom |
| situacije | predlošci + AI za presudu, likove i mentora | i AI-jem smišljene situacije |
| hosting | **lokalno** (Leonovo računalo) | Railway |

### 1.2 Vrijeme — demo način

- **Runda = jedan mjesec sezone.** Mjesec je prirodna jedinica za hotelske izvještaje, a DZS-ovi podaci o
  popunjenosti su mjesečni [IZVOR].
- Runda je otvorena dok igrač ne pritisne **„Zatvori mjesec"**. Tada se obračunaju svi hoteli zajedno, isto kao u
  pilotu. Razlika je samo u tome **tko** zatvara rundu, a to je jedna postavka igre (`time_mode: manual | calendar`).
- Trajanje situacije („1–7 dana") u demo načinu znači: situacija vrijedi do zatvaranja runde. Kalendarsko značenje
  se odlučuje za pilot (otvoreno u VIZIJA §4).
- Situacija bez odgovora do zatvaranja runde dobiva **ocjenu 1** po rubrici, jer je i ignoriranje odluka. To je
  drukčije od AI ispada, gdje presuda ide na pregled (ARHITEKTURA §8.3).

### 1.3 Tri robota

Roboti igraju **kroz jezgru, istim naredbama kao čovjek**: postavljaju radnje i biraju ponuđene odgovore. Ne pišu
slobodan tekst, pa ne troše AI. Strategije su u §10. Iste strategije ulaze i u alat za balans, pa su roboti ujedno
dokaz da obračun radi za više hotela.

### 1.4 Gost umjesto prijave

- `/demo` stvori **gosta**: novi korisnik bez lozinke, kolačić `httpOnly`, i novu igru s tri robota.
- Gost i njegova igra brišu se nakon 7 dana.
- Zaštita troška: najviše 3 nove demo igre po IP adresi na dan i tvrdi AI limit po igri (ARHITEKTURA §10.2).
- Ista poveznica s `?uloga=profesor` otvara i profesorov pogled na tu igru (§1.5). U pilotu to radi prava prijava.

### 1.5 Što profesor vidi i radi u 15 minuta

| min | korak | što pokazuje |
|---|---|---|
| 0–1 | Otvori poveznicu → „Tvoj hotel" | hotel se **dobije**; pet indeksa i stvarne brojke ispod njih |
| 1–6 | **Lipanj:** pritužba gosta (razgovor s likom, odgovor svojim riječima) i ponuda agencije (brojke) | sloboda izraza, AI glumi lik, pravila računaju |
| 6–7 | Izbornik radnji: cijena, sezonci, održavanje | potez u bilo kojem trenutku, svaki ima cijenu |
| 7–9 | **„Zatvori mjesec"** → obračun, mentor, ljestvica s tri robota | posljedice, objašnjenje, napredak umjesto zarade |
| 9–12 | **Srpanj:** zaposlenik traži povišicu; možda i splet okolnosti iz lipnja | odluke ostavljaju tragove |
| 12–15 | Profesorov pogled: **zašto je hotel tu gdje jest** (svaka promjena s uzrokom), presude AI-ja s kriterijima rubrike | prozirnost i pravednost |

Kolovoz i rujan su tu za onoga tko želi igrati dalje. U njima se splet okolnosti najjasnije vidi.

### 1.6 Što demo ne radi

Prava prijava · kalendar · profesor stvara igru ili mijenja postavke · AI smišlja situacije · zajedničko tržište ·
timovi · obavijesti na mobitel · engleski (tekstovi su ipak u prijevodima, ARHITEKTURA K9).

---

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
| | popunjenost · ADR · RevPAR | % · € · € | izvedeno, formule iz [istrazivanje](./istrazivanje/HOTELSKI_POKAZATELJI.md) |
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

| varijabla | C | izvor |
|---|---|---|
| cijena (referentna za mjesec) | VI 135 · VII 165 · VIII 175 · IX 135 € | [PROC] iz LRH prosjeka 132 € i sezonskog hoda |
| bazna popunjenost uz referentnu cijenu | VI 72 · VII 78 · VIII 83 · IX 73 % | [IZVOR] DZS 2025., hoteli u Hrvatskoj |
| stalni / sezonci | 30 / 20 | [PROC] ≈ 0,4 zaposlena po sobi kao LRH [IZVOR] |
| prosječni trošak zaposlenog | 2 200 € mjesečno | [PROC] iz LRH 22,1 mil. € / 835 zaposlenih [IZVOR] |
| plaća u odnosu na tržište | 100 % | |
| moral · iskustvo | 55 · 60 | [PROC] |
| CPOR | 22 € | [PROC] |
| fiksni troškovi | 120 000 € mjesečno | [PROC] energija, osiguranje, uprava, najam opreme |
| stanje opreme | 60 | [PROC] |
| zadovoljstvo gostiju · ocjena portala | 70 · 8,4 | [PROC] |
| blagajna | 300 000 € | [PROC] |

**Provjera računom (profil C, bez odluka igrača):**
lipanj ≈ 2 590 prodanih soba-noći, prihod ≈ 455 000 €, GOP marža ≈ 37 %;
kolovoz ≈ 3 090 soba-noći, prihod ≈ 702 000 €, GOP marža ≈ 57 %.
Lipanj je blizu europskog mjerila (HotStats ≈ 45 % u vrhuncu) [IZVOR]. **Kolovoz je previsok:** stvarni hotel u
vrhuncu ima više sezonaca i veće fiksne troškove. Kalibracija: sezonski hod broja sezonaca i fiksnih troškova, tako
da godišnja GOP marža bude 30–40 % [PROC]. To je prvi zadatak alata za balans.

Vrijednosti iz §2.1 su prvi pokušaj. Alat za balans ih podesi tako da profil C kreće s **50 u svakom indeksu** (§4).

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

## 4 · Pet indeksa (0–100)

Svaki indeks je čista funkcija stvarnog sloja na kraju mjeseca (ARHITEKTURA §7.3).

| indeks | formula | zašto ovako |
|---|---|---|
| **G** zadovoljstvo gostiju | `zadovoljstvo` (anketa) | izravna mjera |
| **R** recenzije | `(ocjena_portala − 7,0) / 2,8 × 100` | 8,4 → 50; 4* hotel ispod 7,0 je u krizi, a 9,8 je vrh |
| **Z** zaposlenici | `0,6 × moral + 0,4 × (100 − 12,5 × fluktuacija%)` | 8 % mjesečne fluktuacije → zadržavanje 0 |
| **O** operativa | `0,45 × oprema + 0,30 × iskustvo + 0,25 × (100 − kazna_opterećenja)` | stanje hotela i ljudi koji ga vode |
| **F** financije | `0,7 × GOPPAR_indeks + 0,3 × likvidnost` | vidi ispod |

**Financije se mjere u odnosu na referentni hotel, ne apsolutno.** Inače bi kolovoz izgledao kao vještina, a lipanj
kao neuspjeh. To je isto načelo kao STR-ov RGI [IZVOR].

```text
GOPPAR_indeks = 50 + 50 × (GOPPAR / GOPPAR_ref(mj) − 1),  ograničeno 0–100
                GOPPAR_ref = GOPPAR profila C bez ijedne odluke u tom mjesecu (računa jezgra)
likvidnost    = blagajna / (2 × (fiksni + trošak_rada)) × 100,  ograničeno 0–100
```

---

## 5 · Profili A, B, C

Svi imaju **zbroj 250**, a ocjena mjeri napredak u odnosu na **prostor za rast** (§8). Pravednost se mjeri alatom
za balans (§11), ne pretpostavlja se.

| profil | priča | G | R | Z | O | F |
|---|---|---|---|---|---|---|
| **A** „Ugledni, ali umorni" | odlične recenzije i vjerni gosti, ali prezaposleni ljudi, visoki troškovi, malo novca | 60 | 75 | 30 | 50 | 35 |
| **B** „Profitabilni, ali zapušteni" | zdrave financije i mirni ljudi, ali stara oprema i slabije recenzije | 45 | 35 | 55 | 40 | 75 |
| **C** „Prosječni" | ništa ne gori, ništa ne blista | 50 | 50 | 50 | 50 | 50 |

Kako se razlikuju u stvarnom sloju (smjer, točne brojke podesi alat):

- **A:** ocjena portala ≈ 9,1, zadovoljstvo ≈ 78, moral ≈ 35, opterećenje iznad 1,8, blagajna ≈ 120 000 €, fiksni
  troškovi viši (kredit za prošlu obnovu).
- **B:** ocjena ≈ 8,0, oprema ≈ 40, iskustvo ≈ 55, blagajna ≈ 600 000 €, plaća 97 % tržišne, ali stabilan tim.
- **C:** vrijednosti iz §2.1.

Svaki profil ima **očitu prvu ideju koja je samo djelomično dobra**. A bi htio dići plaće, ali nema novca. B bi htio
obnovu, ali ona troši likvidnost i djeluje tek idući mjesec. To je namjerno: igra nagrađuje redoslijed odluka.

---

## 6 · Katalog mehanika

Mehanike su kod, a brojke su podaci u katalogu (ARHITEKTURA §7.4). Sve granice se primjenjuju **nakon** izračuna.

### 6.1 Radnje iz izbornika (u bilo kojem trenutku runde)

| kod | radnja | parametar i granice | učinak u obračunu |
|---|---|---|---|
| `SetPrice` | cijena za mjesec | 60–150 % referentne | `f_cijena`, `vrijednost` |
| `SetMarketing` | marketing | 0–40 000 € | `f_marketing` |
| `HireSeasonal` / `ReleaseSeasonal` | sezonci | 0–40 ukupno, najviše ±10 mjesečno | kapacitet, opterećenje; zapošljavanje 1 500 € i pad iskustva; puštanje −3 moral |
| `SetWage` | plaća u odnosu na tržište | 90–115 %, korak 1 % | `cilj_morala`, trošak rada |
| `Training` | obuka | 0 / 5 000 / 10 000 € | iskustvo +4 / +7 i moral +2 **idući** mjesec |
| `Maintenance` | održavanje | odgođeno 3 000 · redovno 15 000 · pojačano 30 000 € | oprema +0 / +4 / +8; odgođeno ostavlja trag `DeferredMaintenance` |
| `Renovation` | obnova | 150 000 €, jednom u sezoni | oprema +25 i zadovoljstvo +3 **idući** mjesec |
| `ReplyToReviews` | odgovaranje na recenzije | da / ne, 1 500 € mjesečno | ocjena portala +0,05 mjesečno |

Zadano stanje (kad igrač ništa ne dira): referentna cijena, marketing 0, bez promjene osoblja, plaća 100 %, bez
obuke, **redovno** održavanje.

### 6.2 Mehanike situacija

Presuda (AI ili ponuđeni odgovor) bira mehaniku i daje **kvalitetu q = 1–5** po rubrici. Kvaliteta skalira učinak.

| kod | parametri | učinak (q = kvaliteta) | trag |
|---|---|---|---|
| `ServiceRecovery` | kompenzacija €, brzina | zadovoljstvo Δ = 2 × (q − 3); kompenzacija je trošak | q ≥ 4 → `PositiveRecovery`; q ≤ 2 → `AngryGuest` |
| `EquipmentRepair` | hitno / privremeno | hitno: trošak, soba odmah radi; privremeno: jeftinije | privremeno → `DeferredMaintenance` |
| `StaffRetention` | povišica %, bonus €, razgovor | moral Δ = 3 × (q − 3); povišica mijenja `plaća%` svima ili jednome | q ≤ 2 → `KeyStaffUnhappy` |
| `GroupBooking` | prihvati, sobe, popust %, termin | grupa ulazi u prodaju prije individualnih (§3.1); q ≥ 4 smanjuje popust za 5 p. b. | q ≥ 4 → `AgencyRelation` |
| `CrisisCommunication` | mjere, komunikacija, rezervni trošak | zadovoljstvo Δ = 3 × (q − 3); trošak rezervnog plana | q ≤ 2 → `AngryGuest` |

**Granice po situaciji** [PROC]: nijedna situacija ne pomiče zadovoljstvo ni moral više od ±8, a ocjenu portala više
od ±0,15.

---

## 7 · Situacije s rubrikama

Četiri osnovne situacije. Rubrika ima **pet kriterija**, a ocjena je broj ispunjenih (0 ili 1 ispunjen → 1).
Ponuđeni odgovori ocijenjeni su po istoj rubrici (ARHITEKTURA §10.3). Imena likova su izmišljena.

### S1 · „Klima ne radi, a vani je 34 °C"

**Lipanj** · individualna · recenzije + održavanje · problem · težina 2 · lik: gošća u sobi 214 (AI glumi)
· mehanike: `ServiceRecovery` + `EquipmentRepair`

Gošća s dvoje djece zove recepciju treći put. Klima u sobi ne radi od jutra i prijeti recenzijom.

| # | kriterij |
|---|---|
| 1 | Brza isprika i preuzimanje odgovornosti, bez prebacivanja krivnje (vrućina, tehničar) |
| 2 | Trenutno rješenje: druga soba ili prijenosni uređaj, odmah |
| 3 | Razmjerna kompenzacija (večera, popust na noćenje), ne povrat cijelog boravka |
| 4 | Uzrok se popravlja i prijavljuje održavanju |
| 5 | Provjera kasnije (follow-up) |

| ponuđeni odgovor | kriteriji | q |
|---|---|---|
| Ispričati se, odmah premjestiti obitelj, ponuditi večeru, poslati tehničara, sutra nazvati gošću | 1 2 3 4 5 | 5 |
| Ispričati se, premjestiti obitelj i poslati tehničara | 1 2 4 | 3 |
| Vratiti novac za cijeli boravak | 1 | 1 |
| Objasniti da su zbog toplinskog vala svi uređaji opterećeni i poslati tehničara kad stigne | 4 | 1 |

### S2 · „Glavni kuhar traži povišicu ili odlazi"

**Srpanj** · individualna · radna snaga · mješovito · težina 3 · lik: glavni kuhar (AI glumi) · mehanika:
`StaffRetention`

Usred sezone glavni kuhar traži 20 % veću plaću. Konkurent mu je dao ponudu.

| # | kriterij |
|---|---|
| 1 | Saslušati ga i otkriti pravi razlog (plaća, opterećenje, priznanje) |
| 2 | Odluka u okviru budžeta, s obrazloženjem |
| 3 | Pravednost prema ostatku tima: bez presedana koji ruši moral drugih |
| 4 | Nenovčani elementi: raspored, odgovornost, priznanje, razvoj |
| 5 | Plan B: zamjenik ili nasljednik iz tima |

| ponuđeni odgovor | kriteriji | q |
|---|---|---|
| Razgovor; 8 % povišice i bonus na kraju sezone; on bira i obučava zamjenika | 1 2 3 4 5 | 5 |
| Bonus na kraju sezone ako ostane, bez razgovora | 2 5 | 2 |
| Odmah dati 20 % | 1 | 1 |
| Odbiti: „nitko nije nezamjenjiv" | — | 1 |

### S3 · „Agencija nudi grupu za kolovoz"

**Lipanj** · individualna · financije · prilika · težina 3 · odgovor **brojkama** (prihvati / odbij / protuponuda) ·
mehanika: `GroupBooking`

Agencija nudi 25 soba na 7 noći **u kolovozu** (175 soba-noći) uz 25 % popusta na referentnu cijenu.

| # | kriterij |
|---|---|
| 1 | Prepoznaje da je kolovoz vrh sezone (bazna popunjenost 83 %), pa grupa istiskuje skuplje goste |
| 2 | Uspoređuje vrijednost grupe s vrijednošću istisnutih individualnih gostiju |
| 3 | Protuponuda: manji popust, manje soba ili pomak na lipanj ili rujan |
| 4 | Uvjeti: depozit, rok otkaza, rok za puštanje neprodanih soba |
| 5 | Ukupna vrijednost: potrošnja grupe u hotelu i budući posao s agencijom |

Brojke računaju **pravila** (§3.1), ne rubrika. Rubrika ocjenjuje obrazloženje i daje mali bonus na uvjete. Lekcija
koju pravila sama pokažu: ista ponuda je dobra u lipnju, a loša u kolovozu.

### S4 · „Toplinski val i ograničenja vode" — kolektivna

**Kolovoz** · **kolektivna** (svi hoteli dobiju istu) · operativa + sigurnost · problem · težina 4 · mehanika:
`CrisisCommunication`

Vodovod najavljuje ograničenja potrošnje vode na tjedan dana usred toplinskog vala.

| # | kriterij |
|---|---|
| 1 | Zdravlje gostiju i osoblja prvo (voda za piće, ranjive skupine, rashlađivanje) |
| 2 | Jasna i iskrena obavijest gostima, prije nego što sami primijete |
| 3 | Štednja koja ne uništava boravak (praonica, ručnici, bazen), a ne gašenje tuševa |
| 4 | Rezervni plan s cijenom (cisterna, dobavljač) |
| 5 | Briga o osoblju (smjene, pauze, vrućina) |

Ista situacija pogađa hotele **različito**: hotel s lošom opremom ili preopterećenim ljudima trpi više. To je pouka
iz postojećih simulacija: zajednički šok, različit učinak [IZVOR].

### Situacije iz spleta okolnosti

Ove situacije nastaju samo kad ih okine trag (§9). Rubrike se pišu poslije prve četiri, istim oblikom.

| | situacija | okidač | mehanike |
|---|---|---|---|
| S5 | „Pukao je bojler — nema tople vode u krilu B" (12 soba izvan funkcije) | T1 | `EquipmentRepair` + `ServiceRecovery` |
| S6 | „Voditeljica recepcije daje otkaz usred sezone" | T2 | `StaffRetention` |
| S7 | „Loša recenzija se širi" | T3 | `ServiceRecovery` (javni odgovor) |
| S8 | „Gost se vraća s preporukom" — **prilika** (poslovna grupa u rujnu) | T4 | `GroupBooking` |
| S1b | „Novi ljudi, nove greške" — pritužba zbog neiskusnog osoblja | T5 | `ServiceRecovery` |

### Raspored

| runda | osnovni utori | utori iz spleta |
|---|---|---|
| lipanj | S1, S3 | — |
| srpanj | S2 | najviše jedan |
| kolovoz | S4 | najviše jedan |
| rujan | — | najviše dva; ako ništa ne okine, dolazi S8 kao prilika |

Najviše dvije situacije po rundi po hotelu. Kad se okine više tragova nego što ima mjesta, prednost ima veća
težina, a kod jednake težine kocka iz sjemena.

---

## 8 · Relativna ocjena

Mjeri se **normalizirani dobitak** po indeksu. To je ustaljena mjera napretka u istraživanju obrazovanja (Hake, 1998)
i mjeri koliko je od mogućeg prostora za rast stvarno ostvareno:

```text
g_i = (kraj_i − start_i) / (100 − start_i)    ako je kraj_i ≥ start_i
g_i = (kraj_i − start_i) / start_i            ako je kraj_i < start_i
napredak = 100 × Σ w_i × g_i                  w_i = 0,2 za svih pet indeksa → raspon −100 do +100
```

- Hotel A s recenzijama 75 i hotel B s recenzijama 35 tako su usporedivi: +5 bodova A-u vrijedi 0,2 prostora, a B-u
  0,08.
- Težine w su pitanje za profesore (pitanje 13). Do odgovora su jednake [PROC].
- **Zatvaranje sezone:** nakon rujna se **naplate otvoreni tragovi**. Na primjer, odgođeno održavanje spušta opremu
  kao da je kvar nastupio. Bez toga bi se isplatilo „rezati sve u zadnjem mjesecu".
- Ljestvica prikazuje napredak nakon svake runde (ARHITEKTURA §12: ostali vide ukupnu ocjenu i pet indeksa).
- **Poznati rizik:** mehanike dodaju bodove, a ne ovise o razini indeksa. Bod pri vrhu zato vrijedi više `g` nego
  bod u sredini, pa A i B (svaki ima indeks na 75) mogu lakše skupljati napredak od C-a (sve na 50). Dva moguća
  lijeka: opadajući učinak mehanika pri vrhu ljestvice ili mjera bez te asimetrije. Bira se po rezultatu alata za
  balans (§11, točka 3), ne unaprijed.

---

## 9 · Splet okolnosti

Svaki okidač čita tragove i stanje na kraju runde i baca kocku iz vlastitog toka `(igra, runda, hotel, okidač)`
(ARHITEKTURA §7.2). Pogodak stvara utor u **idućoj** rundi.

| | okidač | uvjet | vjerojatnost [PROC] | situacija |
|---|---|---|---|---|
| **T1** | Odgođeno održavanje | trag `DeferredMaintenance` | 15 % + 15 % za svaku dodatnu uzastopnu rundu + max(0, 50 − oprema) %, najviše 80 % | S5 |
| **T2** | Nezadovoljni ključni ljudi | moral < 40 ili trag `KeyStaffUnhappy` | 25 % + max(0, 40 − moral) %, najviše 70 % | S6 |
| **T3** | Ljuti gost | trag `AngryGuest` ili ocjena portala pala ≥ 0,2 u rundi | 40 % | S7 |
| **T4** | Zadovoljan gost ili partner | trag `PositiveRecovery` ili `AgencyRelation` | 50 % | S8 |
| **T5** | Novi ljudi, nove greške | fluktuacija > 6 % mjesečno | 30 % | S1b |

Tragovi se brišu kad se uzrok ukloni: redovno održavanje dva mjeseca zaredom briše `DeferredMaintenance`, a moral
iznad 50 briše `KeyStaffUnhappy`. Dobar potez ostavlja dobar trag (T4), pa splet okolnosti ne kažnjava samo.

---

## 10 · Roboti

| | **Štediša** | **Brižni** | **Lovac na popunjenost** |
|---|---|---|---|
| cijena | referentna | +5 % | −15 % |
| marketing | 0 | 5 000 € | 25 000 € |
| sezonci | otpušta 5 u lipnju | drži opterećenje ≤ 1,8 | ne zapošljava |
| plaća | 95 % | 103 % | 100 % |
| obuka | ne | 5 000 € mjesečno | ne |
| održavanje | odgođeno | redovno | redovno |
| situacije | najjeftiniji ponuđeni odgovor | odgovor s najvišim q | prihvaća sve grupe; inače srednji odgovor |

**Očekivano ponašanje — hipoteza, ne obećanje:** Štediša vodi u lipnju zbog financija, a onda ga sustignu T1 i T2.
Lovac puni hotel, ali preopterećenje ruši ljude i operativu. Brižni sporo kreće i najbolje završava. Ako alat za
balans pokaže drukčije, mijenjaju se brojke u katalogu, a ne priča.

Za alat za balans postoji i četvrta strategija, **Nasumični**, s odlukama iz kocke unutar granica.

---

## 11 · Što se provjerava prije demoa

Demo nije spreman dok svaka stavka nema izmjereni rezultat.

1. **Početak:** profil C kreće s 50 (±1) u svakom indeksu; A i B imaju ciljane vrijednosti iz §5 (±2).
2. **Ekonomija:** profil C bez odluka ima GOP maržu 30–40 % kroz sezonu (§2.1, provjera računom).
3. **Pravednost profila:** 3 profila × 4 strategije × 1 000 sjemena. Razlika medijana napretka između profila manja
   je od praga (prijedlog: 5 bodova), a nijedan profil ne pobjeđuje pod svim strategijama.
4. **Nema dominantne radnje:** nijedna pojedinačna radnja na krajnjoj vrijednosti (npr. marketing 40 000 €) ne daje
   najbolji napredak pod svim profilima.
5. **Ponovljivost:** isto sjeme i isti dnevnik daju iste brojke, bajt za bajt.
6. **Prozirnost:** svaka promjena (`Delta`) ima uzrok koji se može prikazati studentu.
7. **AI presude:** za S1 i S2 po 20 odgovora koje je ocijenio Leon. AI se slaže unutar ±1 u barem 80 % slučajeva
   [PROC] prije nego što ide pred profesora.

---

## Za Leonovu provjeru — najnesigurnije pretpostavke

1. ~~Elastičnost cijene~~ → riješeno 04.10.: po kategoriji (§2.0), Leon „otprilike da".
2. ~~Referentne cijene po mjesecima~~ → riješeno 04.10. (§2.0). Otvoreno ostaje: ostali prihod 30 % prihoda soba.
3. Trošak zaposlenog 2 200 € mjesečno i omjer 30 stalnih prema 20 sezonaca.
4. Lekcija u S3: 25 % popusta grupi u kolovozu. Je li to realna ponuda agencije?
5. Kriteriji rubrika u S1–S4. To je tvoje područje i ondje si ti mjerilo.
6. Priče profila A i B (§5). Prepoznaješ li takve hotele iz prakse?
