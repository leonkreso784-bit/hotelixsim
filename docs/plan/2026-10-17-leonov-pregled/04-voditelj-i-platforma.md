# Leonov pregled · D — voditelj odjela i tržište · E — platforma i ostalo

Dio indeksa [2026-10-17-leonov-pregled.md](../2026-10-17-leonov-pregled.md). Ništa od ovoga ne koči popravak temelja;
D ide u fazu C ili LJUDI, E u kolosijek PLATFORMA.

## D — voditelj odjela i uvid u tržište

Tvoje dvije ideje od 13.10.; nacrt s cijenama: [VODITELJ_I_TRZISTE](../../architecture/VODITELJ_I_TRZISTE.md) §3.

### D1 · Imenovani voditelji odjela: sada ili kasnije
- **Što je:** svaki odjel dobije ime osobe koja ga vodi („Odgovorna: Ivana Kovač", situacije se prepuštaju njoj), bez nove
  mehanike (~4–6 h). Prava mehanika (vještine, „prepusti cijeli odjel") je cigla L2a-5.
- **Preporuka:** imena **kad igra počne uzimati ljude iz banke** (K5), ne prije: voditelji dolaze iz banke, pa se imena ne
  pišu dvaput; do tada popravak temelja.
- **Inače:** sada, iz posebnog popisa. **Cijena:** ~4–6 h i imena koja banka poslije zamijeni.

### D2 · Što znači „voditelj je odgovoran"
- **Preporuka:** (c) oboje, u L2a-5: vodi odluke odjela kad mu ih prepustiš **i** ocjena odjela je njegova ocjena (prema njoj
  ga nagrađuješ, zadržavaš ili mijenjaš).
- **Inače:** (a) samo vodi odluke ili (b) samo nosi ocjenu. **Cijena:** manje posla, ali pola lekcije o delegiranju.

### D3 · Gdje student vidi tržište (koliko gostiju dolazi i zašto)
- **Što je:** igra već računa koliko je gostiju zainteresirano, koliko je smješteno i koliko odbijeno, ali student to vidi
  samo u rečenici. Prijedlog je graf kroz sezonu i rastav „sezona donosi X, tvoja cijena −8 %, ugled +3 %…".
- **Preporuka:** kartica „Tržište" na ekranu s pokazateljima (popunjenost, ADR, RevPAR su već ondje), bez nove stavke izbornika.
- **Inače:** zaseban ekran (više mjesta za graf, ~2 h više, nova stavka izbornika) ili Moj hotel (već pun).

### D4 · Jedinica na grafu
- **Preporuka:** noćenja (hotelska mjera), uz broj gostiju u zagradi.
- **Inače:** samo gosti (razumljivije, ali nije mjera koju će koristiti u hotelu).

### D5 · Prognoza prije odluke
- **Preporuka:** samo sezonska očekivanja („kolovoz: potražnja veća od broja soba"); kako cijena mijenja broj gostiju
  student uči iz rastava nakon kruga.
- **Inače:** i „ako postaviš cijenu X, očekuj oko Y noćenja". **Cijena:** igra postaje lakša (student isproba cijenu
  prije zatvaranja kruga), iako jasnije uči tu vezu.

## E — platforma i ostalo

### E1 · Prijava: sesije umjesto JWT-a (L10)
- **Što je:** kako server pamti da je netko prijavljen. **Sesija:** server drži popis prijava, a preglednik samo ključ u
  kolačiću; odjava ili oduzimanje pristupa vrijedi odmah. **JWT:** potpisana potvrda koju nosi preglednik; teže ju je
  opozvati prije isteka.
- **Preporuka:** sesije. Vidim da upravo uređuješ `PLATFORMA/03-prijava.md`; ako tvoja izmjena već odgovara, napiši
  „E1 vidi 03".
- **Inače:** JWT. **Cijena:** sporija odjava i oduzimanje pristupa.
- **Odgovor ide u:** novi ADR i ispravak ARHITEKTURA §12 ([PLATFORMA dio 08](../../architecture/PLATFORMA/08-faze-i-pitanja.md), L10).

### E2 · Pitanja za profesore
- **Što je:** pismo profesorima (brojke, težine, AI) „spremno za slanje nakon tvog pregleda"; bilo je predviđeno 5.–11.10.
- **Preporuka:** ako još nije poslano: dopuni godinu studija uz potpis, odluči „Vi" (pojedinačno) ili „vi" (zajednički) i
  spominješ li zainteresiranog profesora, pa pošalji sada: pitanja 10–12 o brojkama hrane A10, A11 i L1-13.
- **Inače:** poslati uz verziju za profesore (~siječanj). **Cijena:** brojke L1-13 ostaju procjene.
- **Vidi:** [PITANJA_ZA_PROFESORE](../../product/PITANJA_ZA_PROFESORE.md) „Za nas".

### E3 · Plan platforme: šest pitanja ondje
- **Što je:** nacrt [plana platforme](../2026-11-platforma.md) (16.10.) ima vlastitih 6 pitanja s preporukom (§7): prijava za
  P3, kako profesor dobiva pristup za probu, Railway za P3 (~5–10 €/mj), ritam dvaju kolosijeka, redoslijed igre od
  studenog, vlastite situacije profesora. Pitanje o hostingu iz BACKLOG-a ondje je 3. pitanje.
- **Preporuka:** odgovori ondje, istim oblikom („P1 ok, P3 b"); ovdje se ne ponavljaju.

### E4 · Potvrda arhitekture (Rust + TypeScript)
- **Što je:** [ADR-003](../../records/DECISIONS/ADR-003-arhitektura-rust-jezgra-backend-typescript-sucelje.md) je još „nacrt":
  ARHITEKTURA kaže „prijedlog dok Leon ne potvrdi", a ROADMAP kaže da si je pregledao.
- **Preporuka:** potvrdi; sve je izgrađeno na njoj, promjena bi bila tjedni rada.
- **Inače:** rasprava ostaje otvorena. **Cijena:** dva dokumenta i dalje proturječe.

### E5 · Inkubator (rok 16.11.)
- **Preporuka:** prijaviti HotelixSim (ne Sokrat Study): ima jasnog kupca (fakultet), pilot na FMTU-u i više tvog vremena.
- **Inače:** Sokrat Study. **Vidi:** [ROADMAP](../ROADMAP.md).

### E6 · Vizija: što bira profesor
- **Što je:** tri otvorena pitanja iz tvoje vizije 10.10.: (1) je li „branša" odjel hotela ili vrsta objekta; (2) smišlja li
  AI situacije unaprijed uz profesorovo odobrenje ili u hodu; (3) bira li profesor duljinu igre ili ritam kruga.
- **Preporuka:** ne odlučivati sada; ide u nacrt profesorova ekrana (PLATFORMA) kao dokument s raspravom. Ako imaš prvi
  osjećaj, napiši ga. Voditeljev: (1) odjel hotela, (2) unaprijed uz odobrenje, (3) ritam (već tvoja odluka ADR-028 t. 5),
  a sezona ostaje cijela.

### E7 · Stari koncept ekrana: zatvoriti
- **Što je:** iz DIZAJN §3 još visi: ocjena na portalima do 10 ili do 5; „1/4" mjeseca; kratki nazivi kriterija; avatari.
- **Preporuka:** zatvoriti: ocjena do 10 (kao Booking; mentor već tako piše), avatari su riješeni ciglom IZGLED, a ostalo
  provjeriš kad igraš sezonu u L1-14.
- **Vidi:** [DIZAJN](../../product/DIZAJN.md) §3.
