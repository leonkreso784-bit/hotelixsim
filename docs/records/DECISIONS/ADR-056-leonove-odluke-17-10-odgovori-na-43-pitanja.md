# ADR-056 — Leonove odluke 17.10.: odgovori na 43 pitanja paketa pretpostavki i razgovor
**Datum:** 2026-10-17 · **Status:** ✅ **Leonova odluka 17.10.** (Word s 43 odgovora + ankete i proza u razgovoru, sesija 8) ·
**Vezano:** [paket pitanja](../../plan/2026-10-17-leonov-pregled.md), [ADR-050](ADR-050-popravak-podize-odjel-iznos-u-l1-13.md),
[ADR-051](ADR-051-pretpostavke-faze-b-situacije-i-mjerenje.md), [ADR-055](ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md),
[ADR-057](ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md), [ADR-058](ADR-058-pretpostavke-sesije-7-izgled-vanjski-platforma-k2.md)

**Izvori.** Leon je 17.10.2026. u Wordu odgovorio na svih 43 pitanja (doslovno prepisano u
`.superpowers/sdd/2026-10-14-popravak-temelja/leonovi-odgovori-2026-10-17.md`), a zatim u razgovoru s voditeljem odgovorio na
ankete i pitanja u prozi (`…/leonove-odluke-2026-10-17-razgovor.md`). Gdje se razlikuju, vrijedi razgovor (noviji). „Ok" znači
da prihvaća preporuku iz paketa. Ono što je voditelj protumačio stoji kao „tumačenje", a pretpostavke koje iz toga slijede su u ADR-058.

## A — pravila igre i brojke
- **A1 popravak i bodovi.** Leon: *„Hitni popravak ne rješava stanje tog odjela, već samo privremeno popravlja klimu. Pokvarena klima je
  rezultat neodržanog odjela"*; kvar je −1 bod, a popravak odmah uz zadovoljnog gosta +2 (neto +1). Anketa: bodovi su **napredak
  odjela 0–100** (stanje Smještaja). Mijenja ADR-050 (Izmjena 17.10.): mehanika „popravak diže odjel" ukida se u L1-13, privremeni
  +1 ostaje do tada, „oko +10" više nije cilj. **Sučelje soba** (svaka soba vidljiva i obnovljiva zasebno) je nova mehanika, faza C ili kasnije.
- **A2 čestoća kvarova.** *„Prepuštam tebi da pravedno odlučiš brojke"* → L1-13 bira čestoću (održan i zapušten hotel) koja drži težinu ~20 %.
- **A3 težina situacija 20 %.** Anketa: **isti broj situacija, slabiji učinak svake**; L1-13 donosi tablicu izmjerenih inačica.
- **A4 ok:** jednom slabijem odgovoru daje se trošak (npr. pretjerana naknada); radi L1-13 (sadržaj i balans).
- **A5 ok svih pet** (anketa, nakon objašnjenja): zadano za nove v2 igre, roboti neograničeni, prepisano od 70 % = klik, AI ne stigne → 4,
  vrijedi tjedno, mjesečno i u obveznim situacijama → ADR-055 t. 1 ✅.
- **A6 ok granica** (bigrami: barem dvije oznake, svaka s 70 % parova susjednih riječi, prije provjere „prepisao jedan"). Leon za poruku:
  *„kopirao si, podnio sve ponuđene odgovore…"*, i *„to treba istrenirat i pazit"* (tumačenje: pratiti lažno uhvaćene prave odgovore; danas 0 od 229).
- **A7 broj situacija.** Leon: *„ako je pun hotel, više gostiju će raditi situacije i probleme… Broj situacija koje nisu vezane za goste
  ostaje isti"* → **situacije s gostima rastu s popunjenošću, ostale ne** (ADR-051 Izmjena 17.10.). Anketa: „rješava voditelj, korisnik
  bira" = voditelj odjela iz faze LJUDI (L2a-5); u popravku temelja se zbog toga ništa ne mijenja.
- **A8 ok:** šutnja = ocjena 1 i „nitko nije reagirao"; S3 bez odgovora propada bez kazne; L1-13 popravlja 16 predložaka → ADR-015 t. 2 ✅.
- **A9 ok preporuka:** plaća po **ulozi i razini**, obuka sporija po znanju hrvatskog i engleskog, nikakvo pravilo po podrijetlu →
  pravilo voditelja iz ADR-028 ✅, sukob s MODEL_V2 §5.4 riješen. Dodatak: svaki radnik se trenira **pojedinačno ili u skupini** odjela,
  student bira **trenera među radnicima**; voditelji su najvažniji (*„trebat ćemo puno puno različitih voditelja"*: drže treninge, može im se
  prepustiti koordinacija odjela). Faza LJUDI (L2a-5, L2a-6, L2b-3).
- **A10 broj ljudi.** *„To ovisi o broju soba. Moramo pripaziti da studenti ne dobiju uvijek isti hotel sa 120 soba"* → broj soba varira
  među hotelima; istražiti prave hotele na internetu prije L1-13 i banke hotela. Norma 4\* (ADR-032 t. 9) ostaje 🟨 do tog istraživanja.
- **A11 ok privremeno** → ADR-033 t. 2–7 ✅ kao procjene. Razgovor: *„Neće svakom radniku biti potreban smještaj"*; student smještaj
  **kupuje, iznajmljuje ili koristi svoj hotel** → smještaj radnika je zasebna mehanika (faza LJUDI: ekran, trošak, kapacitet); 2.500 € ostaje
  dok mehanika ne dođe, tada se smještaj izdvaja iz te brojke.
- **A12 v1 zatvoren** (9 pitanja o brojkama prve igre). Leon: *„mora se paziti na plaće i analizirati prave plaće; rijetko tko u hotelu ima
  2.200 € osim direktora i nekih voditelja"* → analiza pravih plaća po ulozi prije L1-13; priče hotela A i B i obnovu Leon provjerava u L1-14.
- **A13 ok sve** → ADR-013, 019, 022, 023, 025, 026, 027, 029, 031, 035, 036 i 048 ✅. *„Morat ćemo napraviti mjere potrošnje obavezno"*,
  razgovor: **oboje** — AI trošak po igri (izmjeriti prije pilota) i potrošnja hotela (energija, voda) kao pokazatelj u igri (faza C ili kasnije).

## B — tekstovi i mjerilo AI-ja
- **B1 ok:** Leon slijepo pregledava sve tri datoteke (B4a, B4b, B4c) prije L1-15; voditelj zatim šalje jedan krug B4-ispravaka.
- **B2 značaj situacija.** *„Treba napraviti tablicu situacija (značajne, polu značajne, ne toliko značajne), bolje imenovati i rasporediti;
  značajnije situacije i pametnije odluke nose više bodova"* → L1-13: imenovanje tri razine i bodovi po razini (predlošci već imaju
  `major`/`minor`, student ih ne vidi). Napomene a–e iz paketa ostaju preporuka do slijepog pregleda (ADR-055 t. 6).
- **B3 ok:** voditelj sam popravlja P-RANI-DOLAZAK, rezervni tekst lika i objašnjenje „Zaposleni"; agent piše nacrt opisa S3 i rubrika S3 i S4, Leon potvrđuje.
- **B4 ok:** Leon upisuje 40 ocjena za mjerilo AI suca (~40 min) prije C3 i prije svake zamjene modela suca.
- **B5 ok:** Leon čita kriterije S1 i S2 (~10 min); nacrt S1b i S5–S8 iz prve igre se zatvara (v1 zamrznut).

## C — banka ljudi i izgled likova
- **C1 pečat.** *„Pogledao sam ljude, super su"*; anketa: **zapečatiti svih 120** (serija-000 i serija-001). Banka ima **tri vrste** (radnici,
  gosti, vanjski), a unutar radnika **podvrste voditelji, studenti i radnici**; student ih sve može trenirati do razine voditelja (najviša).
- **C2 ok:** rod po ulozi privremeno; agent mjeri Eurostat (~1 h) prije sljedeće serije; tehničarka se ne dodaje na silu.
- **C3 otvoreno.** Leon: *„dodao bih možda ljude koji su poznati i javni, i političare, ali ne s pravim imenom, već izmijenjenim, da se
  opet vidi da su oni, kao što BitLife radi; lik bi malo ličio na njih, ali bih pazio na granicu"*. Ništa se ne gradi dok Leon ne pročita
  [nacrt rizika](../../architecture/BANKA_LJUDI/07-poznate-osobe-rizik.md); deset nesigurnih imena čeka isti nacrt.
- **C4 ime hotela.** *„Student sam imenuje svoj hotel prije nego što uđe u igru"*; razgovor: *„nazove hotel kako god hoće"* — slobodno ime,
  bez popisa; `zabranjeno.json` ostaje samo za likove banke (najmanja brana vulgarnosti 🟨, ADR-058). Hoteli Kvarnera na popisu zabranjenih:
  preporuka (sumnjive ostaviti, dopuniti otoke i Crikvenicu), Leonu *„nije toliko bitno"*.
- **C5 popis profesora ne dolazi:** *„neće se pogoditi; ako se pogodi, riješit ćemo"*.
- **C6–C8 ok:** hrvatski oblici gradova i imena; „lokalno" = gdje osoba živi; sitnice zemalja i jezika a–e.
- **C9 ok:** isti vanjski smije u dva hotela iste razredne igre, rijetko; svih 14 uloga (uz 5 novih).
- **C10 ok:** policija, vatrogasci i hitna kao nova uloga u postojećim predlošcima nakon popravka temelja; brojke 150 / 400 / 600;
  uniforma iz uloge; službe, dobavljači i susjedi samo iz Hrvatske, dob 20–80, ton daje opis lika.
- **C11 ok:** tri vidljive pretpostavke IZGLED-a (podrijetlo do K5, dob iz opisa, brada, naušnice i frizure); Leon pogleda 10–20 lica kad IZGLED bude spojen.

## D — voditelj odjela i tržište
- **D1 preporuka:** imena voditelja kad igra počne uzimati ljude iz banke (K5). Dodatak: *„svaki radnik i voditelj mora imati svoju crtu
  koliko je jak, a koliko slab"* — vidljiva snaga i slabost, jer može doći loš i dobar voditelj.
- **D2 = c:** voditelj vodi odluke odjela kad mu ih se prepusti **i** ocjena odjela je njegova ocjena (L2a-5).
- **D3:** *„može tržište s grafovima"* — kartica „Tržište" na ekranu s pokazateljima. **D4:** noćenja, uz broj gostiju u zagradi.
- **D5 samo sezonsko očekivanje** prije odluke; Leon: *„jako je bitno da imamo primjere ponude i potražnje u igri"*.

## E — platforma i ostalo
- **E1 ok: sesije**, ne JWT (L10); ARHITEKTURA §12 treba uskladiti ([BACKLOG](../BACKLOG.md)).
- **E2:** pismo profesoru poslano, odgovora još nema; sastanak sljedeći tjedan (*„profesor ne zna ni što sam napravio"*).
- **E3:** *„Hosting je najmanje problem, mislim da ćemo koristiti Cloudflare, još nije sigurno. Profesor će dobiti svoje vlastite informacije za
  prijavu. Ja bih ful volio Supabase uključiti, ali me strah zbog latencije i to nije dobro"* → pružatelj nije odlučen; profesor ima vlastiti račun;
  preostalih 5 pitanja [plana platforme](../../plan/2026-11-platforma.md) §7 otvoreno.
- **E4:** *„ti provjeri duboko i reci mi je li ok"* → zadatak: dubinski pregled arhitekture (najjači model), nalaz Leonu; ADR-003 ✅ tek nakon nalaza.
- **E5 ok:** na inkubator (rok 16.11.) ide HotelixSim, ne Sokrat Study.
- **E6 profesor bira.** Anketa: „branša" = **odjel hotela** (recepcija, restoran, wellness, održavanje). Word: profesor bira AI situacije iz
  **strukturiranog izbornika** (unaprijed, ne u hodu), piše vlastite, vidi studentove razgovore s gostom i zaposlenikom, ocjenjuje po tome, pali i
  gasi AI mentora i **otvara tjedan ili mjesec** (student tek tada ide dalje; ritam bira profesor). *„Trebat će nam nekoliko sesija da inženjerski
  postavimo temelje profesorovog sučelja."*
- **E7 ok:** stari koncept ekrana se zatvara (ocjena do 10; avatari su IZGLED; ostalo u L1-14).

## F — tehničko
- **F1 ok:** sve iz tablice F1 ✅ — ADR-008, 013 (t. 2–4, 6–9), 015, 017, 019, 026 t. 5, 029, 030, 031, 032 t. 1–8, 033 t. 1 i 8, 039, 051 (t. 1,
  4–13), 053 t. 1–5, 055 t. 2, 4, 5, kartica IZGLED rizik 1, 4, 5, 7, 8, nacrt VANJSKI 1, 5–7, 9–13, provjere stroja banke, bočni izbornik weba.
- **F2 ok:** riješene stavke brišu se iz BACKLOG-a „Otvoreno za Leona".

## Banka hotela (Leonova odluka, mjesto 🟨)
Leon: *„trebat ćemo napraviti i banku hotela gdje će profesor moći dodijeliti hotele studentima koje želi."* Popis gotovih, pregledanih hotela
raznih veličina (rješava A10). Mjesto u rasporedu, serije i tri komada (sadržaj, jezgra, platforma) su voditeljev prijedlog: ADR-058 i
[ROADMAP](../../plan/ROADMAP.md).

## Pilot vs produkcija (Leonov pogled)
Leon: *„ovo što već imamo je pilot, a ovo što gradimo je već produkcijska platforma; spremamo se za profesorov DA, jer znam da će biti da."*
Standard „gotovo": 40 studenata igra tri mjeseca bez Leona uz računalo. Redoslijed nakon L1-13 (platforma do najmanje igrive razredne igre
prije soba, voditelja, smještaja i banke hotela) je voditeljev prijedlog, 🟨 ADR-058.

**Cijena ako krivo:** odluke A1, A3, A7 i B2 dolaze u igru tek u L1-13 i nose vlastito mjerenje; do tada je igra bajt-ista. Ostalo su
statusi ADR-ova i stavke rasporeda.
