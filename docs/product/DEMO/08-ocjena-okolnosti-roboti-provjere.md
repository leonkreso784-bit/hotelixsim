# DEMO §8–§11 — ocjena, splet okolnosti, roboti, provjere prije demoa

Dio [DEMO.md](../DEMO.md)

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
- Težine w su pitanje za profesore (pitanje 8). Do odgovora su jednake [PROC].
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
2. **Ekonomija:** profil C bez odluka ima sezonsku (VI–IX) GOP maržu 35–45 %, nijedan mjesec iznad 50 %, a grupa
   iz S3 u kolovozu istiskuje individualne goste (§2.1, ADR-013), a njezino prihvaćanje „kako je ponuđeno" donosi u
   kolovozu manje GOP-a nego u lipnju (§7, ADR-014).
3. **Pravednost profila:** 3 profila × 4 strategije × 1 000 sjemena. Razlika medijana napretka između profila manja
   je od praga (prijedlog: 5 bodova), a nijedan profil ne pobjeđuje pod svim strategijama.
4. **Nema dominantne radnje:** nijedna pojedinačna radnja na krajnjoj vrijednosti (npr. marketing 40 000 €) ne daje
   najbolji napredak pod svim profilima.
5. **Ponovljivost:** isto sjeme i isti dnevnik daju iste brojke, bajt za bajt.
6. **Prozirnost:** svaka promjena (`Delta`) ima uzrok koji se može prikazati studentu.
7. **AI presude:** za S1 i S2 po 20 odgovora koje je ocijenio Leon. AI se slaže unutar ±1 u barem 80 % slučajeva
   [PROC] prije nego što ide pred profesora. Mjerilo je alat `cargo run -p ai --bin mjerilo -- S1 [S2]` s ocjenama u
   `content/hotel/mjerilo/S1.json` i `S2.json` (Leon upisuje `leon_quality`; bez ocjena ili ključa ne mjeri i izlazi s kodom 2).

---
