# Banka ljudi — radnici i gosti kao sadržaj igre

> **Status:** 🟨 NACRT (14.10.); K0 ✅ (odgovori §6 u ADR-042 t. 8); **K1a spojen (adf5a4d) i K1b spojen** (16.10.; serija-000 = 10 radnika + 10 gostiju, serije OTVORENE do Leonova pregleda 20 zapisa; sljedeće K2) (usporedo s popravkom: [ADR-052](../records/DECISIONS/ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md), kartica u ledgeru `.superpowers/sdd/banka-ljudi-k1/`; rod = Eurostat LFS, zemlje = MUP, uloga smije ovisiti o podrijetlu). Povod: Leonova odluka
> [ADR-042](../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 5 — *„jako puno likova, radnike i goste,
> ~2000 ili više, strukturno, korak po korak, oprezno"*, uz dva agenta posvećena samo tome. Ovo je prvi korak: **format i
> pravila**. Slijede provjera sadržaja u Rustu (`crates/content`) i punjenje u serijama (§5). Arhitektura ide kroz dokument i
> raspravu ([ADR-006](../records/DECISIONS/ADR-006-arhitektura-kroz-dokument-i-raspravu-ne.md)); pitanja su u §6.

**Riječ „baza".** Banka ljudi **nije PostgreSQL**. To su JSON datoteke sadržaja u repou (`content/hotel/v2/ljudi/`), kao
predlošci situacija. Ulaze u git, pregledavaju se kao tekst, a provjerava ih Rust pri učitavanju sadržaja i u testovima. PostgreSQL čuva samo
igre (dnevnik događaja), a igra iz banke pamti **koje** je likove uzela (ID), ne njihov tekst.

**Danas.** Likovi situacija dobivaju ime iz `content/hotel/v2/names.json` (grana situacija): 3 izvora (gost, zaposlenik,
partner) × 2 roda × 20 imena = 120 imena, samo „Ime Prezime", bez ičega drugog. Provjera traži samo neprazno i
neponovljeno ime (`crates/content/src/templates_check.rs`, `check_names`). Izgled lika izvodi web iz imena
(`apps/web/src/avatar/look.ts`, `lookFromSeed`) i ne zna rod ni dob, pa gošća može dobiti bradu. Opis „tko je lik" daje
predložak (`types`, npr. „umirovljenica na godišnjem odmoru"), a ne ime.

## 1. Što je lik

Dvije vrste zapisa. **Radnik** puni početne ljude hotela, burzu rada i agenciju (faza LJUDI, L2a i L2b). **Gost** puni
uloge `"from": "guest"` u situacijama, kasnije razgovore i imena u recenzijama (R4-1). Partneri (dobavljač, inspektor,
agencija) zasad ostaju u `names.json`; treća vrsta je pitanje 10.

### 1.1 Temeljno načelo: banka čuva **tko je osoba**, jezgra računa **koliko vrijedi**

Banka sprema identitet i činjenice životopisa. **Brojke koje ulaze u formule ne spremaju se u banku:** vještine,
osobine, motivacija, plaća i rizik odlaska izvlači jezgra iz **istih raspodjela za svakoga**, iz sjemena i razine
iskustva. Tri razloga: (1) **pravilo o nacionalnosti** ([MODEL_V2](./MODEL_V2.md) §2 t. 9,
[ADR-028](../records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md)) — da autor (čovjek ili agent) upisuje vještine i
osobine, stereotip bi ušao kroz tisuće ručnih odluka, a ovako ga test zamjene podrijetla (L2b-5) isključuje strojno;
(2) **balans ne ovisi o tekstu** — nova serija ne pomiče brojke igre (§4.3); (3) **priča ne smije izdati skrivenu osobinu**
— zato je opis neutralan prema osobinama (§3.3).

Iznimka su **jezici**: oni su činjenica podrijetla i životopisa („tri godine radila u Austriji" → njemački), pa se spremaju
u banku i ulaze u pokrivenost jezika (L2b-2). Zato ih provjera drži strože (§3).

### 1.2 RADNIK (`R-00001`…)

| polje | tip | dopušteno | obvezno | napomena |
|---|---|---|---|---|
| `id` | tekst | `R-` + 5 znamenki | da | nikad se ne mijenja ni ne koristi ponovno |
| `ime`, `prezime` | tekst | 2–30 znakova, latinica s dijakriticima, razmak, `-`, `'` | da | izvorni pravopis (`Dvořák`, `Nagy`); samo nominativ (§3.4) |
| `rod` | kod | `z` · `m` | da | ide u gramatiku teksta i izgled; isti kod kao `gender` predloška (`f`/`m`) |
| `dob` | broj | 18–65 | da | godine na početku sezone igre |
| `uloga` | kod | `recepcioner` · `sobar` · `kuhar` · `konobar` · `sanker` · `terapeut` · `tehnicar` · 🟨 `referent_uprave` · 🟨 `referent_prodaje` | da | struka, MODEL_V2 §3.3; naziv u rodu daje katalog (sobar/sobarica). Voditelj **nije** uloga u banci (vidi niže) |
| `razina` | kod | `pocetnik` · `iskusan` · `strucnjak` | da | jedini ulaz banke u brojke: jezgra iz nje izvlači vještine (MODEL_V2 §5.1) |
| `godine_iskustva` | broj | 0 … `dob` − 16 | da | mora pristajati razini (§3.2) |
| `podrijetlo` | kod | `lokalno` · `regija` · `inozemstvo` | da | MODEL_V2 §3.3; samo za činjenice |
| `zemlja` | kod | ISO 3166 (`HR`, `BA`, `NP`…) | da | `lokalno`/`regija` → `HR` |
| `mjesto` | tekst | iz kataloga mjesta 🟨 | da | prebivalište; za priču i za smještaj |
| `jezici` | popis (jezik, razina) | ISO 639-1 + razina 0–4 (MODEL_V2 §3.3) | da | materinji na 4 obvezan |
| `opis` | tekst | 40–220 znakova, najviše 2 rečenice | ne (🟨 pitanje 4) | samo činjenice puta: gdje je radio, zašto je tu (§3.3) |

**Izvodi igra, ne sprema se u banku:** `izgled` (iz `id`, `rod`, `dob`; nova funkcija uz `lookFromSeed`, koja ostaje za
stara imena), naziv uloge u rodu, `treba_smjestaj` (🟨 iz podrijetla: `regija` i `inozemstvo` → da), vještine, osobine
(i koja je skrivena), motivacija, plaća i očekivana plaća (MODEL_V2 §5.2), rizik odlaska, rok/naknada/dozvola agencije
(iz kataloga po zemlji, L2b-1). U stanju hotela `Zaposlenik` dobiva polje `lik: R-00412` umjesto „indeksa u banku imena"
(🟨 malo proširenje MODEL_V2 §3.3; `id` `Z-0001` ostaje brojač hotela).

**Voditelj odjela** se ne piše u banku: postaje ga `strucnjak` (ili `iskusan`) struke tog odjela, pri stvaranju hotela ili
nalogom „postavi voditelja" (L2a-5, L2a-7). Tako nema posebne zalihe voditelja koja bi mogla presušiti.

### 1.3 GOST (`G-00001`…)

| polje | tip | dopušteno | obvezno | napomena |
|---|---|---|---|---|
| `id` | tekst | `G-` + 5 znamenki | da | |
| `ime`, `prezime` | tekst | kao kod radnika | da | 🟨 pitanje 1 (preporuka: oboje) |
| `rod` | kod | `z` · `m` | da | |
| `dob` | broj | 18–85 | da | djeca nisu zapisi; spominju se samo u opisu roditelja |
| `zemlja` | kod | ISO 3166 | da | određuje materinji jezik (činjenica) |
| `mjesto` | tekst | slobodno, stvarni grad te zemlje | da | „iz Graza" za AI lik i recenziju |
| `jezici` | popis (jezik, razina) | kao kod radnika | da | AI lik govori hrvatski; jezik ulazi u pokrivenost i teme recenzija |
| `segment` | kod | `par` · `obitelj` · `poslovni` · `grupa` · `wellness` | da | MODEL_V2 §3.6 |
| `zanimanje` | tekst | kratko, bez imena tvrtke | ne | pomaže AI liku („učiteljica") |
| `opis` | tekst | 40–220 znakova, najviše 2 rečenice | ne | pozadina, ne raspoloženje: ljutnju i zahtjev daje predložak situacije |

**Izvodi igra:** izgled, „povratni gost" (opis uloge iz predloška, npr. „dolazi već petu godinu"), soba, raspoloženje i
sve brojke situacije. **Veza s predlošcima** (🟨, promjena predložaka, ne banke): uloga gosta dobiva neobvezne filtre
`segment`, `dob` (npr. „umirovljenica" → 65+), a `types` se vežu uz segment, da priča ne proturječi zapisu.

### 1.4 Primjer jednog zapisa svake vrste

```json
{ "id": "R-00001", "ime": "Mirela", "prezime": "Knežević", "rod": "z", "dob": 34, "uloga": "recepcioner",
  "razina": "iskusan", "godine_iskustva": 8, "podrijetlo": "lokalno", "zemlja": "HR", "mjesto": "Kastav",
  "jezici": [["hr", 4], ["en", 3], ["de", 3]],
  "opis": "Osam godina radi na recepcijama u Opatiji. Njemački je naučila u dvije sezone u Salzburgu." }
{ "id": "G-00001", "ime": "Anna", "prezime": "Müller", "rod": "z", "dob": 41, "zemlja": "AT", "mjesto": "Graz",
  "jezici": [["de", 4], ["en", 2]], "segment": "obitelj", "zanimanje": "učiteljica",
  "opis": "Dolazi s mužem i dvoje djece. Opatiju je prvi put vidjela kao dijete, s roditeljima." }
```

## 2. Koliko i kakvih

### 2.1 Koliko — obrazloženje brojkom

**Radnici u jednoj razrednoj igri** ([ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md):
jedna igra, do ~30 studenata, svaki svoj hotel; tuđi radnici su vidljivi i mogu se preoteti, pa **ista osoba ne smije biti
u dva hotela iste igre**):

| izvor | račun | radnika |
|---|---|---|
| početni ljudi | 30 hotela × ~65 (profili v2 danas: A 67, B 62, C 68, `content/hotel/v2/profiles.json`) | ~1 950 |
| burza rada | ~27 novih kandidata po razdoblju (2–4 po ulozi × 9 uloga, MODEL_V2 §5.1) × 17 tjedana [PROC] | ~450 |
| agencija | 30 hotela × ~6 stranih radnika po sezoni [PROC] | ~180 |
| **ukupno u igri** | | **~2 600** |
| igra za vježbu | 4 hotela × 65 + burza 450 + agencija 25 | ~735 |

**Gosti:** ~65 situacija po sezoni po hotelu [PROC] × ~0,87 uloga po predlošku × 54 % uloga su gosti (danas 26 od 48
uloga u 55 predložaka) ≈ **31 gost po hotelu po sezoni**; razred od 30 hotela ≈ **~930**. Recenzije (~80 mjesečno po hotelu)
ne trebaju zasebne likove: prikazuju ime i zemlju, a uzimaju ih iz iste banke gostiju, s ponavljanjem među hotelima.

**Cilj u tri prekretnice** (🟨 pitanje 6):

| prekretnica | radnika | gostiju | zašto baš tada |
|---|---|---|---|
| **M1** — prije L2a-1 (faza LJUDI, P3) | 800 | 300 | igra za vježbu (~735) i prvi mali razred; gosti za situacije jednog hotela kroz više igara |
| **M2** — prije razrednog pilota (P4) | 2 000 | 1 000 | razred od ~20 hotela bez ponavljanja; ~930 gostiju razreda |
| **M3** — puni razred | 3 000 | 1 500 | 30 hotela (~2 600) uz ~15 % zalihe; gosti uz filtre segmenta i dobi |

Zaliha nije ukras: filtri dijele banku na male bazene („umirovljenica" = 1 500 × 50 % × 18 % ≈ 135 zapisa).

### 2.2 Raspodjele radnika [PROC; izvor u stupcu]

| dimenzija | cilj (udio banke) | izvor ili pretpostavka |
|---|---|---|
| **uloga** | sobar 28 % · kuhar 14 % · konobar 14 % · recepcioner 11 % · uprava 9 % · tehničar 8 % · šanker 7 % · terapeut 5 % · prodaja 4 % | profil C (68 ljudi, `profiles.json`), pomaknuto prema sezonskim ulogama jer ih burza i agencija traže više |
| **razina** | početnik 35 % · iskusan 45 % · stručnjak 20 % | [PROC]; početnik je jeftin i slab (ADR-028 t. 4), stručnjaka treba ~1 po odjelu za voditelja |
| **dob** | 18–24: 22 % · 25–34: 30 % · 35–49: 32 % · 50–65: 16 % | [PROC]; najmanje 10 % početnika starijih od 35 (promjena zanimanja), da „početnik" ne znači „mlad" |
| **rod** | ukupno ž 55 % · m 45 %; **u svakoj ulozi najmanje 25 % svakog roda** | [PROC]; stvarni omjer u domaćinstvu je jače ženski — pitanje 5 |
| **podrijetlo** | lokalno 50 % · regija 25 % · inozemstvo 25 % | [PROC]; lokalno = Kvarner i Istra, regija = ostatak Hrvatske; inozemstvo puni agenciju (pitanje 2) |
| **zemlje (inozemstvo)** | BiH, Srbija, Sj. Makedonija, Nepal, Indija, Filipini, ostalo [PROC] | najčešće zemlje izdanih dozvola za rad u turizmu — **izvor treba potvrditi** (MUP, uz L2b-1) |
| **jezici (drugi)** | engleski: 0–4 = 10/20/35/30/5 % · njemački: 45/20/20/12/3 % · talijanski: 70/15/10/4/1 % | [PROC]; recepcioner ima engleski ≥ 2; isti udjeli za svako podrijetlo osim materinjeg jezika |

### 2.3 Raspodjele gostiju

| dimenzija | cilj | izvor |
|---|---|---|
| **zemlja** | AT 20 · DE 18 · HR 14 · HU 7 · SI 7 · IT 6 · US 4 · NL 3 · RS 3 · PL 3 · UA 2 · SK 2 · BA 2 · CH 2 · CZ 1,5 · UK 1,5 · ostale 4 (%) | [JEZICI_GOSTIJU.md](../research/JEZICI_GOSTIJU.md), noćenja TZG Opatija 2023.; „ostale" srezane s ~10 na 4 % jer izvorni udjeli s rasponima prelaze 100 % |
| **segment** | par 35 · obitelj 25 · wellness 15 · poslovni 15 · grupa 10 (%) | [PROC]; MODEL_V2 §3.6 nema udjele |
| **dob** | 18–29: 15 · 30–49: 40 · 50–64: 27 · 65+: 18 (%) | [PROC] |
| **rod** | 50 : 50 | predlošci danas 23 ž / 25 m uloga |
| **jezici** | materinji 4; engleski po istoj tablici kao radnici; HR gosti hrvatski 4 | činjenica zemlje; ostalo [PROC] |

Raspodjela je **cilj cijele banke**, ne svake serije. Serija smije odstupati da popuni manjak (§5.3).

## 3. Pravila kvalitete

### 3.1 Jedinstvenost
- `id` jedinstven u cijeloj banci; svaka serija ima **svoj raspon** brojeva, pa dva agenta ne mogu izdati isti ID.
- **Puno ime jedinstveno u cijeloj banci** (radnici, gosti i partneri zajedno), uspoređeno bez dijakritika i velikih
  slova (`Kovačević` = `Kovacevic`). Gost i radnik istog imena zbunili bi studenta.
- Ograničenja učestalosti [PROC]: jedno prezime najviše 0,6 % vrste, jedno ime najviše 1,5 %, da banka ne bude „sve
  Horvat i Ana".

### 3.2 Dosljednost zapisa
`godine_iskustva` ≤ `dob` − 16; razina prema iskustvu: početnik 0–2, iskusan 3–9, stručnjak ≥ 8 godina [PROC];
`lokalno`/`regija` ↔ `HR`; `mjesto` iz kataloga mjesta za lokalno i regiju; materinji jezik zemlje na razini 4; jezik
najviše jednom; ime i prezime pristaju rodu i zemlji (slovačko žensko prezime na `-ová` — ovo provjerava čovjek).

### 3.3 Bez stereotipa (ADR-028, MODEL_V2 §2 t. 9)
- Podrijetlo određuje **samo** jezike, rok i naknadu agencije, dozvolu i smještaj. Razina, dob i rod moraju imati **istu
  raspodjelu u svakoj skupini podrijetla** (stroj: odstupanje skupine od cijele banke najviše 5 postotnih bodova [PROC]).
- Opis je **neutralan prema osobinama**: bez riječi iz kataloga osobina i njihovih sinonima (pouzdan, marljiv, kasni,
  svadljiv, lijen, ljubazan…) i bez ocjene osobe. Opis kaže **što se dogodilo** (gdje je radio, odakle dolazi), ne **kakav
  je** čovjek. Skrivenu osobinu otkriva igra (MODEL_V2 §5.1), ne tekst.
- Bez rečenica o „narodu", „mentalitetu", „tipičnom" ponašanju zemlje; bez šala na račun podrijetla, dobi, roda,
  vjere, zdravlja ili izgleda.

### 3.4 Rod i padeži
`rod` je obvezan jer je tekst za AI pisan u rodu lika („Umorna si i ljuta", `Role.gender` u `templates.rs`); izvlačenje
traži lik roda uloge. Banka sprema **samo nominativ**, a predlošci stavljaju ime samo u nominativ („Ti si {gost.ime}");
treba li genitiv ili vokativ, dolazi polje `padezi` **novim formatom** (§4.3). Opis je u rodu lika („radila je", „došao
je"), bez obraćanja studentu u rodu ([UI_PRAVILA](../product/UI_PRAVILA.md) §3 t. 3).

### 3.5 Zabranjeni sadržaj
Stvarne poznate osobe (popis zabranjenih: političari, sportaši, javne osobe, profesori FMTU-a, Leon); stvarni hoteli s
Kvarnera (isti popis kao generirani hoteli, [PLATFORMA §4](./PLATFORMA/04-generirani-hoteli.md) t. 4) i robne marke;
maloljetni radnici; zdravlje, vjera, politika, spolnost kao obilježje lika; vitice `{}` (mjesta za zamjenu); ćirilica.

### 3.6 Tko što provjerava

| provjera | stroj (Rust, `crates/content`, test pri svakom `cargo test`) | čovjek (uzorak Leonu) |
|---|---|---|
| oblik | nepoznato polje pada (`deny_unknown_fields`), obvezna polja, kodovi iz popisa, rasponi | — |
| jedinstvenost | ID, puno ime (bez dijakritika), učestalost imena i prezimena | — |
| dosljednost | §3.2 osim imena prema zemlji | zvuči li ime stvarno za tu zemlju i rod |
| stereotip | raspodjele po podrijetlu (§3.3), zabranjene riječi u opisu | ton opisa, skriveni stereotip u frazi |
| tekst | duljina, broj rečenica, točka na kraju, bez `{}`, bez ćirilice | rod i padež u opisu, prirodan hrvatski |
| zabranjeno | popis imena i hotela | poznate osobe izvan popisa |
| raspodjela banke | izvještaj prema ciljevima (§2); na pečaćenju prekretnice odstupanje ≤ 3 p. b. [PROC] je uvjet | — |
| zapečaćeno | SHA-256 svake zapečaćene serije (§4.3) | — |

## 4. Gdje živi i kako se koristi

### 4.1 Datoteke

```text
content/hotel/v2/ljudi/
├── banka.json              popis verzija: verzija N = koje su serije zapečaćene + otisak; piše samo glavna sesija
├── raspodjele.json         ciljevi iz §2 kao podatak (stroj uspoređuje banku s njima)
├── mjesta.json             mjesta za lokalno i regiju; zemlje za inozemstvo
├── zabranjeno.json         zabranjena imena, hoteli i riječi u opisu
├── povuceni.json           ID-jevi povučeni od verzije N (zapis se nikad ne briše)
├── radnici/serija-001.json … (piše samo agent RADNICI)
└── gosti/serija-001.json   … (piše samo agent GOSTI)
```

Serija je **jedna datoteka od najviše 50 zapisa** (~1 000 redaka formatiranog JSON-a, ~25 KB): stane u jedan pregled i
jedan prolaz agenta. Zaglavlje datoteke: `{"format": 1, "vrsta": "radnici", "serija": 1, "od": "R-00001", "do": "R-00050",
"zapisi": [...]}`. Za M3 to je 60 datoteka radnika i 30 gostiju. Kod pravila (`crates/content/src/ljudi.rs`,
`ljudi_check.rs`) poštuje granicu od 1000 redaka (`check:kod`); JSON se ne broji, ali granica od 50 zapisa vrijedi.

### 4.2 Kako igra bira — deterministički po sjemenu

- **Ključ izbora** je isti kakav igra već ima: igra za vježbu i profili A/B/C → **sjeme profila** (isti hotel = isti
  ljudi, MODEL_V2 §3.3); razred i generirani hoteli → (sjeme igre, slot) ([PLATFORMA §4](./PLATFORMA/04-generirani-hoteli.md)).
- **Postupak:** iz verzije banke zapisane u igri uzmi zapise koji odgovaraju filtru (vrsta, uloga, rod, razina; za goste
  segment i dob), poredaj po ID-u, ispremiješaj vlastitim tokom kocke (`Rng::derive(sjeme, [label("ljudi:pocetni"), slot,
  uloga])`, novi tokovi, postojeći se ne diraju — MODEL_V2 §2 t. 3) i uzimaj redom **preskačući one koje igra već ima**.
  Hoteli se pune redom slota, pa nitko nije u dva hotela iste igre.
- **Burza** se izvodi pri otvaranju razdoblja iz stanja igre u tom trenutku (tok `ljudi:burza`), jer utjecaj među
  hotelima djeluje samo na prijelazu kruga (ADR-037). Nezaposleni kandidat ostaje na burzi 2–4 razdoblja [PROC] — manje
  potrošnje banke.
- **Gosti situacija** (🟨, zamjenjuje današnji izbor iz `names.json` u `crates/sim-hotel-v2/src/situations/fill.rs`):
  isti obrazac — imena slobodna u krugu, pa izbjegavanje ranijih, kao danas (`first_nonempty`).
- **Kad bazen presuši** (rijedak filtar): popušta se redom (razina → rod → dob/segment), a tek zatim se vraća lik koji je
  **napustio** tu igru. Ime se nikad ne slaže iz dijelova. Svaki korak popuštanja ima test.

### 4.3 Verzije i pečati — dodavanje ne mijenja stare igre ni balans

- Nova igra u snimak (`GameCreated`) zapisuje **`banka_ljudi: N`** i otisak te verzije. Stara igra se uvijek čita s
  verzijom s kojom je stvorena (isto načelo kao katalog u snimku, MODEL_V2 §6.2); igre bez tog polja koriste `names.json`
  kao danas.
- **Zapečaćena serija se ne mijenja.** Kad Leon odobri seriju, njen SHA-256 ide u `crates/content/tests/ljudi_sealed.rs`
  (obrazac `fixtures_sealed.rs`: konstanta, CRLF → LF, promjena je svjestan potez vidljiv u pregledu).
- **Ispravak** se radi povlačenjem: zapis ide u `povuceni.json` od verzije N, a ispravljeni lik dobiva novi ID u novoj
  seriji. Verzije prije N ga i dalje vide. Tako izbor u starim igrama ostaje bajt-isti.
- **Format** (polja) ima svoj broj. Novi format (npr. `padezi`) čita i stare serije; stara polja se ne preimenuju.
- **Balans** (`cargo run -p balance`) radi s verzijom banke iz `banka.json`, zadano posljednjom zapečaćenom za balans.
  Jer banka daje samo identitet, razinu i jezike, a raspodjele razine i jezika čuva stroj (§3.6), nova serija balans ne
  pomiče; podizanje zadane verzije ipak traži jedan `--check` kao dokaz.

## 5. Postupak punjenja korak po korak

### 5.1 Koraci

| korak | što | tko | gotovo kada |
|---|---|---|---|
| **K0** | ovaj dokument → Leonovi odgovori (§6) → ADR | glavna sesija, Leon | ADR s odlukama |
| **K1** ✅ 16.10. (K1a adf5a4d, K1b) | format u Rustu (`ljudi.rs`, `ljudi_check.rs`), `raspodjele.json`, `mjesta.json`, `zabranjeno.json`, `ljudi_sealed.rs`; **20 ručnih primjera** (10 + 10) u `serija-000` i po **jedan namjerno loš primjer za svako pravilo** (test pada s jasnom porukom) | jedan agent (Rust) | `cargo test -p content` zelen; svako pravilo §3.6 ima test |
| **K2** | prva serija: **50 radnika + 50 gostiju** | agent RADNICI, agent GOSTI | Leon pregleda **svih 100**; nalazi postaju pravila (stroj) ili upute agentima |
| **K3** | serije po 50, svaka uz provjeru i izvještaj raspodjela | oba agenta, paralelno | „serija gotova" (§5.3) |
| **K4** | pečat prekretnice (M1, M2, M3): `banka.json` nova verzija, raspodjela banke ≤ 3 p. b. od cilja | glavna sesija | izvještaj zelen; balans `--check` |
| **K5** | korištenje u igri: L2a-1/L2a-2 (početni ljudi), L2a-6 (burza), L2b-3 (agencija), R4-1 (recenzije), gosti situacija | faze IGRE ([ROADMAP/igra](../plan/ROADMAP/igra.md)) | po ciglama faza, ne ovdje |

### 5.2 Dva agenta i datoteke
- **Agent RADNICI** piše samo `radnici/serija-NNN.json`; **agent GOSTI** samo `gosti/serija-NNN.json`. Zajedničke datoteke
  (`banka.json`, `raspodjele.json`, `povuceni.json`, `ljudi_sealed.rs`, Rust kod) mijenja **samo glavna sesija**.
- **Raspon ID-jeva i broj serije** dodjeljuje glavna sesija u zadatku; **sudar imena** dvaju agenata hvata provjera pri
  spajanju, a ispravlja ga agent kasnije serije.
- **Novi agent za svaku seriju** (pouka „agenti i tokeni": jedan agent ne smije narasti preko ~400k tokena). Ulaz: ovaj
  dokument, ciljevi, izvještaj o **manjku** u banci (koje uloge, zemlje, dobi fale) i zadnje Leonove primjedbe. Izlaz:
  datoteka serije i kratki izvještaj u datoteci. Najviše dva agenta koja prevode Rust u isto vrijeme.
- Zasebna grana (prijedlog `sadrzaj/banka-ljudi`): nove datoteke + jedan redak u `crates/content/src/lib.rs`, bez sudara sa slojem 1.

### 5.3 Kriterij „serija gotova"
1. `cargo test -p content` zelen (oblik, jedinstvenost, dosljednost, zabranjene riječi, raspodjele po podrijetlu);
2. izvještaj raspodjela pokazuje da serija **smanjuje** manjak prema cilju, ne povećava ga;
3. uzorak Leonu: od K3 **10 zapisa po seriji** izabranih po pravilu (npr. svaki peti), Leon kaže OK ili daje primjedbe;
4. primjedbe su ispravljene (prije pečata se zapis smije mijenjati), pa se serija pečati.

### 5.4 Kada se staje
**Prekretnica dosegnuta** (M1, M2, M3) — staje se dok faza koja koristi banku ne kaže treba li više. **Sustavna greška**
(2 ili više od 10 zapisa u uzorku odbijeno) — stop, pravilo ide u stroj ili upute, tek onda dalje. **Promjena formata**
koju traži faza IGRE — stop do novog formata (§4.3). **Leon kaže stop** — banka nikad ne ide na račun faze koja je na redu.

## 6. Pitanja za Leona (preporuka je prva)

1. **Gosti s imenom i prezimenom ili samo s imenom?** Preporuka: **ime + prezime** — recepcija zna prezime gosta, AI lik
   se predstavlja punim imenom, a recenzija svejedno pokazuje samo ime i zemlju.
2. **Koliko stranih radnika?** Preporuka: **25 % banke** (~750 u M3), zemlje po službenoj statistici dozvola za rad (MUP).
3. **Raspon dobi?** Preporuka: radnici **18–65**, gosti **18–85**; djeca samo u opisu roditelja.
4. **Smije li lik imati „priču" od 2 rečenice?** Preporuka: **da**, do 220 znakova, samo činjenice puta (gdje je radio,
   zašto je tu), nikad osobine — da priča ne izda skrivenu osobinu.
5. **Rod po ulozi:** stvarni omjer (domaćinstvo uglavnom žene, održavanje uglavnom muškarci) ili **najmanje 25 % svakog
   roda u svakoj ulozi**? Preporuka: **25 % najmanje** — igra je nastavni materijal i ne smije učiti „sobarica = žena".
6. **Ciljni brojevi:** **3 000 radnika + 1 500 gostiju** u tri prekretnice (§2.1), ili stati na 2 000 ukupno? Preporuka:
   tri prekretnice; M2 (3 000 ukupno) je dovoljna za pilot.
7. **Razred: isti brojevi, različita lica?** Hoteli istog profila u razredu dobivaju **iste vještine i razine** (pravednost,
   ADR-028 t. 1), ali **različite ljude iz banke** (ADR-037: tuđi radnici su vidljivi i preotimaju se). Preporuka: da.
8. **Brojke izvan banke:** vještine, osobine i motivaciju izvlači **jezgra** iz razine (preporuka, §1.1) ili ih agenti
   upisuju ručno uz priču?
9. **Pregled:** Leon pregleda **svih 100** prve serije, zatim **uzorak od 10** po seriji (preporuka), ili svaku seriju cijelu?
10. **Partneri** (dobavljač, inspektor, agencija) kao treća vrsta zapisa? Preporuka: **da, nakon M1**, ~200 zapisa;
    do tada ostaju u `names.json`.
11. **Hrvatski kod radnika iz susjednih zemalja** (BiH, Srbija, Crna Gora): upisuje se materinji jezik kakav jest, a
    razumijevanje s hrvatskim igra broji kao **razinu 3** (preporuka) ili kao 4?
