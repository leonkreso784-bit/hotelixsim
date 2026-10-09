# HotelixSim — dizajn sučelja (Leonov koncept)

> Što ekran mora pokazati i kako izgleda. Izvor: Leonov logo i koncept ekrana u [dizajn/](./dizajn/).
> Odluka o Tailwindu i konceptu: [ADR-020](../records/DECISIONS.md#adr-020). Što igra radi: [DEMO.md](./DEMO.md).

![Logo](./dizajn/logo.png)

## 1 · Marka

- **Ime:** uvijek „HotelixSim"; u logu „Hotelix" tamno, „Sim" plavo ([ADR-002](../records/DECISIONS.md#adr-002)).
- **Znak:** plavi zaobljeni kvadrat s bijelim „H" koje je ujedno ulaz u hotel (svijetloplava vrata i stube).
- **Boje:** primarna plava `#246BFD` · tamna `#151B2B` (tekst, naslovi) · bijela `#FFFFFF`. Uz to: zelena za
  napredak (+), crvena za pad (−), svijetloplava pozadina kartica.
- **Slogan:** „Kratki potezi. Jasni uzroci. Vidljiv napredak." · uvodni naslov „Tvoj hotel. Svaka odluka se računa."
  · „Mjeri se napredak. U odnosu na početno stanje tvog hotela."
- **Stil:** svijetlo, čisto, zaobljene kartice, debeli bezserifni naslovi, ikone uz svaki podatak, mobitel prvi.

## 2 · Ekrani iz koncepta

![Koncept ekrana](./dizajn/koncept-ekrana.png)

| ekran | što pokazuje | odakle podaci (API) |
|---|---|---|
| **Moj hotel** | oznaka mjeseca („LIPANJ · 1/4"), pet indeksa s trakama, kartice popunjenost · prihod · blagajna · zaposleni, ocjena na portalima, „Situacije mjeseca" s brojem, „Plan mjeseca", gumb „Zatvori mjesec", donja traka Hotel · Situacije · Tijek | `GET /v1/me/hotel`, `/v1/hotels/{id}` |
| **Situacija** | naslov i opis, razgovor s likom kao chat s avatarima, „Razgovor · 2/6", polje „Tvoj odgovor", „Predaj odgovor", „AI obrazloženje" po kriterijima, napomena „AI ocjenjuje odgovor. Pravila računaju brojke." | `…/situations`, `…/messages`, `…/conversation`, `…/response` |
| **Mjesec završen** | veliki napredak od početka („+8"), promjene indeksa s kratkim uzrokom, mentor s avatarom, ljestvica napretka (Tvoj hotel i tri robota), „Nastavi u srpanj" | `/v1/hotels/{id}` (`last_month`, `mentor`), ljestvica igre, feed |

Donja traka „Tijek" = feed hotela (vremenska crta). Profesorov pogled nije u konceptu (DEMO korak 6).

## 3 · Gdje se koncept i igra ne slažu (riješiti u planu ekrana)

1. **AI obrazloženje:** koncept ima trake 1–5 po kriteriju („Empatija 5/5"). Igra za svaki kriterij kaže samo
   **ispunjen / nije** uz rečenicu razloga, a ukupna ocjena 1–5 je broj ispunjenih ([ADR-019](../records/DECISIONS.md#adr-019) t. 2).
   Prikaz: pet redaka ✓/✗ s razlogom i ukupno „4/5". Kratki nazivi kriterija („Empatija", „Rješenje"…) u sadržaju
   još ne postoje — rubrike imaju duge rečenice; kratki naziv treba dodati u `situations.json`.
2. **Ocjena na portalima:** koncept „3,8" (ljestvica do 5); igra računa na ljestvici do 10 (npr. 8,40). Prikazati
   „8,4 / 10" ili preračunati — odluka.
3. **Broj mjeseci:** koncept „1/4" i vremenska crta lipanj–rujan; demo igra lipanj i srpanj (DEMO §1.5, kod igre
   ima dva mjeseca). Prikaz „1/2" ili proširenje igre na četiri mjeseca — odluka.
4. **Uzroci promjena:** kartice „+6 Brza reakcija na kvar klime" traže ljudski tekst uzroka; API daje kodove uzroka
   (`Cause`) — ekran treba hrvatske tekstove za kodove.
5. **Avatari likova i mentora:** koncept ima ilustrirane avatare; slike još ne postoje (gošća, šef kuhinje, mentor).
6. **„Nastavi u srpanj":** u igri srpanj otvara obračun lipnja; gumb samo vodi na „Moj hotel" za srpanj.
