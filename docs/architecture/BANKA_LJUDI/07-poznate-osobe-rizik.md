# BANKA_LJUDI §7 · Likovi nalik poznatim osobama — nacrt rizika (C3)

Dio indeksa [BANKA_LJUDI.md](../BANKA_LJUDI.md). **Status:** 🟨 NACRT, ČEKA LEONA (17.10.2026.). Arhitektura i rizik idu kroz
dokument i raspravu ([ADR-006](../../records/DECISIONS/ADR-006-arhitektura-kroz-dokument-i-raspravu-ne.md)); ovo nije pravni
savjet nego popis onoga što treba odvagnuti. Dok Leon ne odluči, vrijedi §3.5: **nijedan lik ne smije nositi ime ni lik
stvarne poznate osobe**, a paralelna sesija banke takve likove ne dodaje.

## 7.1 Što Leon traži

Leon 17.10. (C3, doslovno): *„Pa ovo nije toliko veliki problem iskreno, može se riješiti, ja bih dodao možda ljude koji su
poznati i javni i političare, ali ne s pravim imenom već izmijenjenim imenom, ali da se opet vidi da su oni, isto kao što
BitLife radi; također bih napravio da lik malo liči na njih, ali bih pazio na granicu."*

**Tumačenje voditelja:** tri sastojka zajedno: (a) **izmijenjeno ime** koje se ipak prepoznaje, (b) **sličan izgled** crtića,
(c) **poznate i javne osobe, uključujući političare**. Svaki sastojak sam je manji rizik; zajedno čine **prikaz stvarne osobe**,
a ne izmišljen lik.

## 7.2 Aluzija ili prikaz: gdje je razlika

| | aluzija na tip | prikaz osobe (parodija) |
|---|---|---|
| što student prepozna | vrstu osobe („slavni TV kuhar", „nogometna zvijezda na odmoru") | **točno tu osobu** („to je X") |
| ime | izmišljeno, ne izvedeno iz pravog | izvedeno iz pravog (premetnuto, slovo zamijenjeno) |
| izgled | crtić iz roda, dobi, podrijetla, kao svi | crtež namjerno nalik licu, frizuri, znaku osobe |
| tko govori | AI glumi izmišljen lik | AI **govori u ime prepoznatljive osobe** |

Zadnji redak je najvažniji za nas: naši likovi nisu slika, nego **AI koji razgovara**. Lik nalik političaru u razgovoru sa
studentom izgovara rečenice koje stvarna osoba nikad nije rekla, a nitko ih unaprijed ne pročita (presuda i lik pišu se u
hodu). Kod klasične parodije autor svaku rečenicu napiše i za nju odgovara; kod nas je napiše model.

## 7.3 Pravni rizik u Hrvatskoj (opisno; [PROVJERITI] s pravnikom prije svake varijante osim 1)

- **Prava osobnosti.** Zakon o obveznim odnosima štiti prava osobnosti, među njima **ime, lik (prikaz osobe), čast, ugled i
  dostojanstvo** (opća odredba o pravima osobnosti, voditelj misli da je čl. 19), a povreda tih prava je **neimovinska šteta**
  za koju se traži naknada (voditelj misli da je čl. 1046). Za tužbu ne treba pravo ime: dovoljno je da je osoba **prepoznatljiva**.
  Izmijenjeno ime „da se opet vidi da su oni" upravo je prepoznatljivost.
- **Uvreda, sramoćenje, kleveta.** Kazneni zakon ih poznaje kao zasebna djela. Rizik raste s AI-jem: lik nalik političaru koji
  u razgovoru „prizna" mito ili kaže nešto ponižavajuće iznosi neistinitu činjenicu o prepoznatljivoj osobi.
- **Osobni podaci.** Prikaz prepoznatljive žive osobe u javnom sadržaju može biti obrada osobnih podataka (GDPR); igra već ima
  pravila za podatke studenata ([PLATFORMA §7](../PLATFORMA/07-gdpr-i-sigurnost.md)), a ovo bi bila nova vrsta podataka [PROVJERITI].
- **Političari.** Javne osobe moraju trpjeti više kritike, ali to štiti **kritiku i satiru u javnoj raspravi**, ne nastavni
  materijal koji ih glumi. Uz to: **javni fakultet** (FMTU, Sveučilište u Rijeci) od nastavnog materijala očekuje političku
  neutralnost; lik nalik političaru jedne stranke, a ne druge, sam je po sebi politička poruka [PROVJERITI etički kodeks].

## 7.4 Rizik kod profesora, fakulteta i inkubatora

- **Nastavni materijal fakulteta.** Profesor koji igru koristi na kolegiju odgovara za ono što studenti vide. Jedan lik nalik
  političaru dovoljan je da profesor ili dekanat kažu „ne" cijeloj igri, a pilot ovisi o jednom profesoru (Leonova ocjena cjeline
  17.10.: „ovisnost o jednom profesoru").
- **Inkubator** (rok 16.11.): ocjenjivači gledaju i pravni rizik proizvoda; poznate osobe bez dozvole su crvena zastavica.
- **Javni repo.** Repozitorij je javan (push = objava): svaki zapis banke odmah je objavljen, s imenom, opisom i povijesti
  izmjena. Brisanje iz povijesti gita teško je i ne briše kopije.
- **Ime HotelixSim.** Istu brigu već imamo za ime projekta ([ADR-002](../../records/DECISIONS/ADR-002-ime-hotelixsim-nikad-skraceno-na-hotelix.md)):
  ne smijemo nalikovati postojećem. Za ljude vrijedi isto, jače.

## 7.5 Rizik stereotipa

Banka ima pravilo da podrijetlo određuje samo činjenice, nikad osobinu (§3.3, ADR-028). Poznata osoba nosi **javnu sliku**:
političar iz susjedne zemlje, sportaš određenog podrijetla, pjevačica određenog žanra. Lik nalik njoj prenosi tu sliku na
**podrijetlo** (student vidi „takvi su ljudi odatle"). Stroj to ne može uhvatiti (raspodjele po podrijetlu ostaju iste), a
upravo je to stereotip koji §3.3 isključuje. Uz goste iz regije (BiH, Srbija, Slovenija) rizik je najveći.

## 7.6 Tehnički rizik

- **Izgled.** Crtić se danas izvodi iz roda, dobi, podrijetla i imena ([IZGLED_LIKA](../IZGLED_LIKA.md)); „malo liči na njih"
  traži novo polje s naputkom za izgled (frizura, brada, naočale, boja kose) i **novi format** banke (§4.3). Što je naputak
  precizniji, crtić je bliže prikazu osobe (§7.2).
- **AI lik.** Uputa lika (`lik.v3`) bi morala zabraniti politiku, stvarne događaje i izjave u ime osobe; model se toga ne drži
  uvijek, a provjera je moguća samo nakon razgovora (C3 sudac čita razgovor). Rezervni tekstovi lika isto.
- **Mjerenje.** Nijedna postojeća brana (`zabranjeno.json`, provjera imena bez dijakritika) ne zna je li „izmijenjeno ime"
  preblizu pravom; to bi bio ručni pregled svakog zapisa.

## 7.7 Što BitLife smije, a mi ne

Leon navodi BitLife (zabavna igra za mobitel) kao uzor. Razlika nije u tehnici nego u okviru:

| | zabavna igra (npr. BitLife) | HotelixSim |
|---|---|---|
| tko odgovara | tvrtka s pravnim timom, pod pravom zemlje sjedišta [PROVJERITI] | Leon osobno, bez obrta ili tvrtke (do Demo dana) |
| okvir | zabava i satira, igrač bira igru | **nastava**: studentu igru zadaje profesor, ulazi u ocjenu |
| tko piše tekst | autori unaprijed | AI u hodu |
| publika | igrači koji su pristali na šalu | studenti, profesori, fakultet, inkubator, javni repo |

Ono što je u zabavnoj igri šala, u zadanom nastavnom materijalu javnog fakulteta je stav fakulteta.

## 7.8 Tri varijante s cijenom

| | što | dobivamo | cijena [PROC] |
|---|---|---|---|
| **1 · ne** | samo izmišljeni likovi (danas, §3.5) | 0 rizika; sve kako je | 0 h; nema humora prepoznavanja |
| **2 · blaga aluzija** (preporuka voditelja) | bez političara (ni stranih) i bez živih osoba iz regije; **izmišljeno ime** koje nije izvedeno iz pravog; **arhetip zanimanja** („TV kuhar", „nogometna zvijezda", „influencerica", „glumac iz serije"); crtić kao svi, bez prepoznatljivog lica | lekcija **VIP gost** (privatnost, fotografi, posebni zahtjevi, zavist drugih gostiju): pravo hotelsko znanje, bez stvarne osobe | novo polje u zapisu `aluzija: true` (novi format §4.3, sesija banke, ~2–3 h); **profesor ga može isključiti** za razred (postavka razreda, platforma, ~2 h); Leon pregleda svaki takav zapis; uputa lika: nikad stvarna osoba, nikad politika |
| **3 · kako Leon želi** | poznate i javne osobe i političari, izmijenjeno ime po kojem se prepoznaju, crtić nalik licu | humor prepoznavanja | pravni savjet prije prvog zapisa (odvjetnik, €); pisana suglasnost profesora i fakulteta; naputak izgleda i novi format; stroža uputa lika i pregled razgovora; rizik tužbe ili zabrane igre na fakultetu; javni repo objavljuje sve |

**Preporuka voditelja: varijanta 2**, uz pravilo: svaki takav lik nosi u zapisu `aluzija: true`, profesor ga može isključiti
za svoj razred, a uputa lika zabranjuje da se lik predstavi kao stvarna osoba ili govori o politici. Razlog: lekcija koju
Leon vjerojatno želi (gost kojeg svi prepoznaju) dobiva se iz **tipa** osobe, a cijeli rizik iz §7.3–7.7 dolazi tek s
**prepoznatljivom stvarnom osobom**. Cijena ako je krivo: varijanta 2 → 3 kasnije je novi zapis; 3 → 1 nakon objave je
brisanje iz javne povijesti i moguće isprike.

## 7.9 Pitanja za Leona

1. **Koja varijanta:** 1, 2 (preporuka) ili 3?
2. **Ako 2:** koliko takvih likova (prijedlog: najviše 1 na 100 gostiju) i samo gosti ili i vanjski (npr. novinar s TV-a)?
3. **Političari:** prihvaćaš li da ih nema ni u jednoj varijanti osim 3, ni stranih?
4. **Profesor:** smiješ li ovo spomenuti na sastanku u tjednu od 20.10.2026. i pitati smije li takav lik u nastavni materijal?
5. **Ako 3:** jesi li spreman platiti pravni savjet prije prvog zapisa (iznos se tek treba saznati) i tražiti pisanu suglasnost
   fakulteta?
