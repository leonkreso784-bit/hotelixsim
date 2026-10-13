# PLATFORMA §8 · Faze, temelj ili zakrpa, pitanja

Dio indeksa [PLATFORMA.md](../PLATFORMA.md). Faze i cigle žive u [ROADMAP/platforma](../../plan/ROADMAP/platforma.md)
(sati [PROC]); ovdje je samo što ulazi u koju fazu i zašto tim redom. Prijašnje oznake F0–F5 iz nacrta od 12.10. zamijenjene
su imenima faza iz [ROADMAP](../../plan/ROADMAP.md).

## 8.1 Faze

| faza (ROADMAP) | sadržaj iz ovog dokumenta | cigle | sati [PROC] | prekretnica |
|---|---|---|---|---|
| **SPREMNO ZA INTERNET** | 6.4 (postavke, slika, poslužitelj servira web, SIGTERM, `Secure`, ograničenja, kopije, Sentry, `ai_usage` + granice, predmemorija folda), 6.5, mjerenje kašnjenja; izlazak na Railway Hobby tek uz Leonov OK | F0-1…14, F0-S1, S2 | 48 | P2 (dio P3) |
| **RAZRED** | 2.3 (tablice), 2.4 (CRUD, vrata kruga, raspored, neaktivni), 3 (računi, kod igre), obavijesti, AI granica po igri; ovisi o fazi RAZRED U JEZGRI u [igra](../../plan/ROADMAP/igra.md) | F1-2…17, ID-1…3, RZ-1…4, AAI-0 | 116 | P2: profesor na internetu stvori igru, studenti uđu kodom |
| **PROFESOR** | 2.5 (projekcija, pregled uživo), pregled presuda, račun dobiti, profesorove vlastite situacije | F2-1…6, F2b-* | 117 | P3 |
| **PILOT** | 7 (obavijest, rokovi, anonimizacija, DPA-ovi), uloga baze, radnici i trake, domena, kopije izvan Railwaya, ispravak presude, proba 30 studenata | F0-7, 8, 12, 15, S3…S5, F2-7, 10, 11, G-1…9 | 95 | P4 |
| **poslije** | generator hotela (dio 04), AAI@EduHr i Google/Microsoft (3.2), SSE, sučelje na engleskom (ADR-037 t. 8) | F3, F5, SSE, ENGLESKI | ~105 | — |

Igrini slojevi (dio 05) idu u kolosijek IGRA. **Redoslijed:** SPREMNO ZA INTERNET može početi odmah, jer dira druge datoteke
od igre. Platformski kod ide u nove module (`app/src/platform/`, `api/src/platform/`, migracije od `0005`) da se smanje sukobi
s igrom. Administrativni dio AAI (zahtjev fakultetu) pokreće se **sada**, jer traje najdulje, a ne košta kod.

## 8.2 Temelj ili zakrpa (cijena svake)

| tema | zakrpa | cijena zakrpe | temelj | cijena temelja |
|---|---|---|---|---|
| razred | profesor sam vodi 4 robota ili jedna igra s gumbom „zatvori za sve" | brzi čekaju spore; ne radi kroz semestar | jedna igra s N hotela, obračun po hotelu, vrata kruga (ADR-037) | ~30 h u jezgri (RAZRED U JEZGRI) |
| prijava | jedna lozinka za razred | nema identiteta, nema GDPR brisanja po osobi | korisnici + identiteti + sesije (2.3, dio 03) | RAZRED, ID-1…3 |
| web i API | dvije zadane domene | kolačići pucaju u Safariju i privatnom prozoru | jedna adresa (6.1) | ~2–4 h |
| profesorov pregled | fold cijele igre pri svakom osvježavanju | radi za 3 studenta, puca za 30 | projekcija u istoj transakciji | ~1 dan |
| generirani hoteli | nasumični broj soba u postojećem profilu | indeksi i pravednost tiho krivi | opis u snimku + referenca po hotelu + balans | F3 |
| rezervacije | brojka „rezervirano" izmišljena za ekran | ekran laže, a „detalj samo uz uzrok" ([ADR-026](../../records/DECISIONS/ADR-026-put-b-hotel-od-brojeva-postaje.md) t. 2) pada | knjiga u jezgri (5.1 A) | ~25–40 h |
| AI trošak | ručno gledanje računa kod Anthropica | prekoračenje se vidi tek na računu | granice u `app` (2.6) | ~1 dan |
| kopije | „Railway ionako čuva" | snimke diska nisu provjereno vraćanje | vlastiti `pg_dump` + proba vraćanja | ~0,5 dana + 0–5 €/mj [PROC] |

## 8.3 Pitanja

### Leonove odluke (stanje 13.10.)

| oznaka | tema | stanje |
|---|---|---|
| L1 | razred | **riješeno:** jedna igra s N hotela, studenti jedni protiv drugih (ADR-037 t. 1) |
| L2 | tko zatvara krug | **riješeno:** profesor otvara, student zatvara svoj hotel (ADR-037 t. 2) |
| L3 | mijenja li se ADR-004 | **riješeno:** ne; lokalno do profesorova OK, izlazak uz izričit OK (dio 06) |
| L4 | hosting | **preporuka kontrolora 13.10., čeka Leonovu potvrdu i ADR:** Railway Hobby + Postgres na Railwayu, ~5–10 €/mj; Supabase samo kao IdP kasnije |
| L5 | domena | preporuka: `hotelixsim.com` na Cloudflareu, tek za stvarne studente; do tada `*.up.railway.app` (D6) |
| L6 | prijava studenta | **riješeno:** korisničko ime + lozinka, pa AAI (ADR-037 t. 7); D4 (profesor za probu) otvoreno |
| L7 | generirani hoteli u razredu | otvoreno: koliko različitih hotela (4.2 t. 5) |
| L8 | rezervacije | preporuka A, knjiga kao zbroj (5.1) |
| L9 | redoslijed | **riješeno:** dva kolosijeka ([ADR-038](../../records/DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md)) |
| L10 | sesije umjesto JWT-a | **preporuka, čeka Leonovu potvrdu i ADR** (3.1); tek tada se ispravlja ARHITEKTURA §12 |

Otvorene odluke (D2 lobby, D3 veličina igre, D4, D6, D7 uklonjeni student, D8 imena ili nadimci, D10–D12 profesorove situacije,
plaća tuđeg radnika) vode se u [ROADMAP/platforma](../../plan/ROADMAP/platforma.md) → „Odluke".

### Pitanja za fakultet i profesora

1. **F1 · AAI@EduHr:** hoće li FMTU ili Sveučilište u Rijeci registrirati HotelixSim kao resurs kod Srca? Tko potpisuje i tko
   je kontakt? Dobivamo li SAML ili OIDC i koje atribute? *(Pitati odmah: administracija traje najdulje.)*
2. **F2 · Voditelj obrade:** je li fakultet voditelj obrade, a Leon izvršitelj uz ugovor o obradi? Tko je službenik za zaštitu podataka?
3. **F3 · Pravna osnova:** ulazi li igra u ocjenu kolegija (javni interes) ili je pilot dobrovoljan (pristanak uz alternativu)?
4. **F4 · Rok čuvanja:** koliko dugo nakon kolegija čuvamo igre, uključujući AI razgovore, i što točno znači redakcija
   kad student ode iz zajedničke igre?
5. **F5 · Mjesto obrade:** smije li AI obrada studentskog teksta ići u SAD uz DPA, ili fakultet traži EU (Bedrock ili Vertex u
   EU)? Traži li fakultet smještaj u Hrvatskoj ili je EU dovoljan?
6. **F6 · Vidljivost:** vide li studenti imena kolega u listi i ljestvici ili samo nadimke? (Isto što i D8; pitanje 9 u
   [PITANJA_ZA_PROFESORE](../../product/PITANJA_ZA_PROFESORE.md).)
7. **F7 · Opseg pilota:** koliko grupa, koliko studenata i kakav raspored (na satu ili kroz semestar)? O tome ovise
   opterećenje i trošak.
8. **F8 · Trošak:** tko plaća AI i hosting u pilotu (Leon, fakultet, inkubator)?
9. **F9 · Asistent:** treba li profesoru uloga asistenta i što ona smije?
