# Leonov pregled · C — banka ljudi i izgled likova

Dio indeksa [2026-10-17-leonov-pregled.md](../2026-10-17-leonov-pregled.md). **Banka ljudi** je popis izmišljenih radnika,
gostiju i vanjskih osoba iz kojih igra uzima likove ([BANKA_LJUDI](../../architecture/BANKA_LJUDI.md)). **Serija** je jedna
skupina zapisa; **pečat** znači da se serija zaključa i više ne mijenja (stroj to čuva otiskom). Dok serija nije
zapečaćena, ispravak je besplatan. C1–C2 koče pečat; ostalo su sitnice i vanjski likovi.

### C1 · Pregled 120 ljudi i pečat serija
- **Što je:** serija-000 (10 radnika + 10 gostiju) i serija-001 (50 + 50) su OTVORENE i čekaju tvoj pregled.
- **Kako:** otvori `.superpowers/sdd/banka-ljudi-k1/pregled-svi.html` (lica, filtri po vrsti, ulozi, zemlji, rodu i dobi)
  ili `banka-ljudi.xlsx` u istoj mapi (listovi Radnici 60, Gosti 60, Vanjski) i u stupac **Odluka** upiši `OK`,
  `ispravi: …` ili `izbaci`. Za prvih 20 bilješke su u `pregled-20.md`. Gledaj ono što stroj ne vidi: zvuči li ime stvarno
  za zemlju i rod, padež i rod u opisu, skriveni stereotip, je li zanimanje gosta stvarno.
- **Koliko:** ~40 s po zapisu, 70–90 min ukupno.
- **Preporuka:** svih 120 sada (tvoja odluka 14.10.: prva serija cijela, poslije uzorak 10 po seriji); zatim glavna sesija
  pečati obje serije. Lice na stranici je privremeni crtić (C11): ocjenjuješ zapis, ne lice.
- **Inače:** uzorak 20 iz serije-001. **Cijena:** odstupa od tvoje odluke; neviđeni zapisi idu u igru.
- **Vidi:** [ADR-042](../../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 8.

### C2 · Rod po ulozi
- **Što je:** udio žena u svakoj ulozi. Ti si odlučio: stvarni omjer u hotelijerstvu, izvor Eurostat (anketa o radnoj snazi).
- **Pretpostavili smo:** brojke su **prijedlog, još nisu izmjerene iz Eurostata**: žene među sobarima 90 %, recepcija 70,
  uprava 75, terapeuti 75, prodaja 65, konobari 50, kuhari 45, šankeri 35, tehničari 5 %. Zato je danas 0 od 5 tehničara
  žena, 15 od 17 sobara žena, a ukupno 62 % žena (cilj banke 55 %).
- **Preporuka:** brojke prihvatiti privremeno i zapečatiti serije; agent izmjeri Eurostat (~1 h) prije sljedeće serije, a
  nove serije prate izmjereno. Tehničarku ne dodavati na silu: u većoj seriji doći će po omjeru.
- **Inače:** jedna tehničarka već sada. **Cijena:** jedan zapis, ništa drugo.
- **Vidi:** [ADR-052](../../records/DECISIONS/ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md) t. 3,
  [ADR-055](../../records/DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md) t. 3b,
  `.superpowers/sdd/banka-ljudi-k1/k2-review.md` pitanje 1.

### C3 · Imena koja možda pripadaju poznatim osobama
- **Što je:** lik ne smije nositi ime stvarne javne osobe. Dva su već zamijenjena (Markus Wieser → Leitgeb, Patrizia
  Moretti → Sartori). Nesigurni: Tamara Kinkela, Nensi Šuran, Balša Vujačić, Dragan Šipka, Sajeev Menon (radnici);
  Kees Bakker, Alain Girard, Helmut Steiner, Christina Wolf, Malin Lundgren (gosti).
- **Preporuka:** agent ih provjeri pretraživanjem interneta (~20 min, 0 €) i predloži zamjene; ti samo kažeš ok.
- **Inače:** provjeriš sam.

### C4 · Hoteli Kvarnera na popisu zabranjenih imena
- **Što je:** `zabranjeno.json` drži imena koja likovi ne smiju spominjati (stvarni hoteli, lanci, osobe). Danas 19 hotela,
  gotovo svi iz Opatije i Rijeke. Sumnjivi: „Excelsior" (poznat iz Dubrovnika i Zagreba), „Astoria" (nepotvrđen na
  Kvarneru), „Istra" (opće ime). Nema Krka, Lošinja, Raba ni Crikvenice.
- **Preporuka:** sumnjive ostaviti (višak zabranjenog ne šteti), a agent dopuni hotele otoka i Crikvenice iz javnog popisa
  (~20 min).
- **Inače:** izbaciti sumnjive. **Cijena:** nijedna, osim ako je neki ipak stvaran.

### C5 · Imena profesora FMTU-a
- **Što je:** lik ne smije nositi ime stvarnog profesora; danas je na popisu samo „Leon Kreso".
- **Preporuka:** pošalji popis ili poveznicu na stranicu nastavnika; agent ih prepiše u `zabranjeno.json`.

### C6 · Strani gradovi i imena na hrvatskom
- **Pretpostavili smo:** hrvatski oblici gradova (Trst, Pečuh, Krakov, Segedin, Đur, Lavov) i hrvatski zapis ćiriličnih
  imena (Andrij Savčuk).
- **Preporuka:** ok (dosljedno, tako piše i hrvatski tisak).
- **Inače:** izvorni oblik (Szeged, Győr, Lviv) u mjestu i u opisu.

### C7 · „Lokalno" za doseljene
- **Pretpostavili smo:** podrijetlo = gdje osoba živi. Tko se preselio na Kvarner (Zagreb → Rijeka, Dubrovnik → Ika,
  Vinkovci → Njivice) je „lokalno", a opis kaže odakle je.
- **Preporuka:** ok.
- **Inače:** po mjestu rođenja. **Cijena:** tri zapisa i jedno pravilo.

### C8 · Sitnice zemalja i jezika (jedno „ok")
- a. Uzbekistan kao izvor radnika (s Nepalom, Indijom, Filipinima i Bangladešom „Azija") dok ne stignu podaci MUP-a;
  b. gosti iz „ostalih" zemalja po jedan (Francuska, Irska, Švedska); c. talijanski 4 kod dvoje Istrana (talijanska
  zajednica); d. gost iz Dresdena (57) govori ruski 2 (škola u Istočnoj Njemačkoj); e. svih 50 novih gostiju ima zanimanje.
- **Preporuka:** ok sve; zemlje se zamijene kad agent izmjeri MUP-ove dozvole (tvoja odluka 16.10.).
- **Inače:** napiši slovo.

### C9 · Vanjski likovi: tko dolazi (pitanja 12–13 iz BANKA_LJUDI §6)
- **Što je:** treća vrsta u banci (tvoja odluka 16.10.): inspekcija, policija, vatrogasci i slični.
- **12. Isti vanjski u dva hotela iste razredne igre?** Preporuka: **da, ali rijetko** (jedan sanitarni inspektor pokriva
  cijelo mjesto). Inače nitko dvaput. **Cijena:** ~800 zapisa umjesto 600.
- **13. Koje uloge?** Preporuka: **svih 14** (uz postojeće i 5 novih: turistički inspektor, inspektor rada, porezni
  inspektor, djelatnik hitne, grad ili turistička zajednica). Inače samo uloge koje igra već ima, uz policiju i vatrogasce.
- **Vidi:** [BANKA_LJUDI §6](../../architecture/BANKA_LJUDI/06-pitanja.md), §1.5.

### C10 · Vanjski likovi: u igri i brojke (pitanja 14–16 i pretpostavke)
- **14.** Policija, vatrogasci i hitna kao likovi: preporuka **nova uloga u postojećim predlošcima** (P-DIJETE-IZGUBLJENO,
  P-DIZALO, P-GOST-POZLILO) nakon popravka temelja; inače posebni novi predlošci.
- **15.** Brojke 150 / 400 / 600 (prekretnice M1 / M2 / M3): preporuka da.
- **16.** Uniforma na liku policije, vatrogasaca i hitne, iz uloge: preporuka da.
- **Pretpostavke:** službe, dobavljači i susjedi samo iz Hrvatske (stranci samo u tržišnim ulogama, ~30 %); dob 20–80,
  službe do 65, inspektori i hitna od 25; ton (strog ili blag) daje opis lika u situaciji, ne zapis u banci.
- **Preporuka:** ok sve.
- **Vidi:** `.superpowers/sdd/banka-ljudi-k1/nacrt-vanjski-report.md` (13 pretpostavki; tehnički dio je u F1).

### C11 · Izgled lika: što student vidi
- **Što je:** tvoja odluka 16.10.: crtić iz roda, dobi i podrijetla (cigla IZGLED). Tri pretpostavke se vide:
  a. dok igra ne uzima ljude iz banke, podrijetlo se zna samo po popisu imena: osoblje „regija", gosti „strani"; nijedan
     ne dobiva tamnu put, tamniji tonovi dolaze tek iz banke sa zemljom (zato je Nepalka na stranici pregleda svijetle puti);
  b. dob iz riječi u opisu uloge (umirovljenica, baka…), inače odrasla; na stranici pregleda: mlad do 29, odrasla 30–54,
     stariji od 55;
  c. brada kod otprilike svakog trećeg muškarca, naušnice kod svake druge žene, frizure po rodu.
- **Preporuka:** ok; kad IZGLED bude spojen, pogledaj 10–20 lica jednog kruga (~10 min) i reci što ne valja.
- **Vidi:** [kartica IZGLED](../2026-10-14-popravak-temelja/task-13-izgled-lika.md) „Rizik i odluke" 2, 3, 6;
  [IZGLED_LIKA](../../architecture/IZGLED_LIKA.md).
