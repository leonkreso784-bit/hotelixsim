# Postojeće hotelske simulacije — kako rade i što naučiti

> Istraživanje 2026-10-04 (agent, web). Stranice proizvođača uglavnom nisu bile dostupne (403/PDF), pa se dio
> oslanja na sažetke pretrage. **Interne formule potražnje nisu javne ni za jednu simulaciju.**

## Provjereno u izvorima

**HOTS (Hotel Operations Tactics & Strategy)** — slideshare.net/HOTS_Simulation/hots-brochure
- Odluke: cijena sobe, yield management, cijene hrane i pića, oglašavanje, osoblje, trening, energija, obnova,
  udobnost gostiju, kapitalna ulaganja. Hotel ima 125–250 soba; svi timovi kreću s istim položajem.
- Ocjena: tim na početku postavlja ciljeve i mjere uspjeha; neki kolegiji koriste HOTS Balanced Scorecard (30 %).
  Bond University ocjenjuje apsolutnim razredima, ne rangom.
- Trajanje: 2 simulirane godine po kvartalima u jednom semestru.
- Kritike: upute i mehanika doživljeni kao „neprijateljski"; razumijevanje mehanike korelira sa zabavom (r = 0,50)
  — repository.rit.edu/theses/7192/

**Cesim Hospitality** — cesim.com
- Hotel 140 soba + restoran; tri segmenta: odmor, posao, konferencije.
- Odluke: cijene po tipu stope (last-minute, redovna, early-bird), radni dan / vikend; šest kanala promocije; jelovnik;
  osoblje (stalno, privremeno, fleksibilno) i plaće; trening; housekeeping vlastiti ili vanjski; nabava.
- Potražnja ovisi o cijeni, promociji, kvaliteti i reputaciji iz zadovoljstva gostiju; niska cijena odvlači tuđe goste.
- Pobjeda: kumulativni profit (instruktor smije birati druge mjere). Izvještaji: GOPPAR, ADR, RevPAR, popunjenost.
- Runda = dva simulirana tjedna; 5–12 rundi + do 3 probne.

**Hotel Business Management Simulation (Marketplace / ILS)** — samo iz sažetka pretrage: odluke o ulaganjima,
troškovima i cijenama; cilj je razumjeti uzročno-posljedične veze među pokazateljima.

**Ostalo:** „Inn or Out" (SMU, 15 min, prihvati/odbij rezervaciju — mikro-model yield managementa); „The Hotel Game"
(INFORMS, prozirni i neprozirni kanali).

**Literatura o poslovnim simulacijama:**
- natjecanje vodi u „logiku igre" umjesto učenja, pa i u varanje;
- neprozirni modeli potiču pokušaje i pogreške i krive mentalne modele, a prozirniji daju bolji rezultat;
- previše varijabli prekida vezu odluka → rezultat;
- studenti navode nejasne ciljeve (13 %) i kasnu povratnu informaciju (10 %) kao glavne izazove.

## Pouke za HotelixSim

1. Ne ocjenjivati samo profit. Već smo odlučili ocjenjivati napredak; zadržati više kriterija da nijedna poluga ne pobjeđuje.
2. Dizajn protiv dominantnih strategija: opadajući povrat (marketing, popusti), kazna s odgodom za pretjerano rezanje
   troškova, i prije izlaska tisuće strategija kroz jezgru da se provjeri da nijedna ne pobjeđuje svugdje.
3. Isti šokovi za sve, a učinak ovisi o položaju hotela, pa se model ne može „riješiti" napamet.
4. Prozirnost: uz svaku posljedicu reći zašto se dogodila („zašto je popunjenost pala").
5. Pravila objasniti u grubim crtama; crna kutija stvara krive mentalne modele.
6. Moguće preuzeti iz HOTS-a: student prije prve runde zapiše cilj, ocjena mjeri pomak i obrazloženje.
7. Probna runda i više kraćih rundi za brzu povratnu informaciju.
8. Početi s malo odluka i dubinu dodavati kasnije.

Dominantne strategije u HOTS-u i Cesimu nisu dokumentirane; to je zaključak, ne izvor.
