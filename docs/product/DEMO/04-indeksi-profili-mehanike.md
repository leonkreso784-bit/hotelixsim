# DEMO §4–§6 — indeksi, profili, katalog mehanika

Dio [DEMO.md](../DEMO.md)

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
likvidnost    = blagajna / (2 × (fiksni + trošak_rada)) × 100,  najmanje 0 (bez gornje granice — ADR-014)
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

Kako se razlikuju u stvarnom sloju. Brojke žive u `content/hotel/*.json` (kalibrirane alatom za balans); ovdje je
samo smjer.

- **A:** viša ocjena portala i zadovoljstvo od C, niži moral, opterećenje iznad 1,8, malo novca u blagajni, fiksni
  troškovi viši (kredit za prošlu obnovu).
- **B:** niža ocjena i zadovoljstvo od C, stara oprema, nešto manje iskustva, puno novca u blagajni, niži fiksni
  troškovi, plaća malo ispod tržišne, ali stabilan tim (moral iznad C).
- **C:** smjer iz §2.1.

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
| `HireSeasonal` / `ReleaseSeasonal` | sezonci | 0–50 ukupno, najviše ±10 mjesečno | kapacitet, opterećenje; zapošljavanje 1 500 € i pad iskustva; puštanje −3 moral |
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
