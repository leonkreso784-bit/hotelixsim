# BANKA_LJUDI §1 · Što je lik

Dio indeksa [BANKA_LJUDI.md](../BANKA_LJUDI.md).

Tri vrste zapisa. **Radnik** puni početne ljude hotela, burzu rada i agenciju (faza LJUDI, L2a i L2b). **Gost** puni
uloge `"from": "guest"` u situacijama, kasnije razgovore i imena u recenzijama (R4-1). **Vanjski** (Leonova odluka 16.10.,
§1.5) puni uloge `"from": "partner"`: inspekcija, policija, vatrogasci, dobavljači, agencije, novinari, susjedi; do K5 oni
ostaju u `names.json`.

## 1.1 Temeljno načelo: banka čuva **tko je osoba**, jezgra računa **koliko vrijedi**

Banka sprema identitet i činjenice životopisa. **Brojke koje ulaze u formule ne spremaju se u banku:** vještine,
osobine, motivacija, plaća i rizik odlaska izvlači jezgra iz **istih raspodjela za svakoga**, iz sjemena i razine
iskustva. Tri razloga: (1) **pravilo o nacionalnosti** ([MODEL_V2](../MODEL_V2.md) §2 t. 9,
[ADR-028](../../records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md)) — da autor (čovjek ili agent) upisuje vještine i
osobine, stereotip bi ušao kroz tisuće ručnih odluka, a ovako ga test zamjene podrijetla (L2b-5) isključuje strojno;
(2) **balans ne ovisi o tekstu** — nova serija ne pomiče brojke igre (§4.3); (3) **priča ne smije izdati skrivenu osobinu**
— zato je opis neutralan prema osobinama (§3.3).

Iznimka su **jezici**: oni su činjenica podrijetla i životopisa („tri godine radila u Austriji" → njemački), pa se spremaju
u banku i ulaze u pokrivenost jezika (L2b-2). Zato ih provjera drži strože (§3).

## 1.2 RADNIK (`R-00001`…)

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

**Voditelj odjela** se ne piše u banku kao uloga: postaje ga `strucnjak` (ili `iskusan`) struke tog odjela, pri stvaranju
hotela ili nalogom „postavi voditelja" (L2a-5, L2a-7). Tako nema posebne zalihe voditelja koja bi mogla presušiti.

### Podvrste radnika (Leonova odluka 17.10., C1; [ADR-056](../../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md))

**Leon:** *„trebamo raditi tri vrste i unutar njih više vrsta; unutar radnika trebamo dodati voditelje, studente i radnike.
Njih student može sve istrenirati da budu na razini voditelja (što je najviša razina)."* Uz A9: *„trebat ćemo puno puno
različitih voditelja… oni rade treninge i može im se prepustiti da koordiniraju cijeli odjel."* Uz D1: svaki radnik i voditelj
ima **vidljivu crtu koliko je jak, a koliko slab** (*„može doći loš voditelj i dobar voditelj"*).

**Kako to sjeda na postojeće polje `razina` (opis značenja; JSON i Rust se ovdje ne mijenjaju):**

| podvrsta (Leon) | u banci danas | značenje u igri 🟨 (tumačenje voditelja) |
|---|---|---|
| **student / praktikant** | `razina: pocetnik`, `godine_iskustva` 0–1, dob većinom 19–25 | početnik koji je ujedno na studentskom ugovoru ili praksi: niža plaća, kraći boravak (sezona); **status ugovora nije razina** pa traži novo polje (npr. `ugovor: student`) **novim formatom** (§4.3), kad ga faza LJUDI zatreba |
| **radnik** | `razina: pocetnik` ili `iskusan` | obični zaposlenik struke |
| **voditelj** | `razina: strucnjak` | najviša razina; može voditi odjel, raditi obuke i preuzeti koordinaciju odjela (L2a-5). „Razina voditelja" = `strucnjak` |

- **Svi su trenirljivi do razine voditelja:** obuka i iskustvo dižu `pocetnik` → `iskusan` → `strucnjak`. To je mehanika jezgre
  (L2a-5, rast vještina i obuka), ne polje banke: banka čuva **početnu** razinu.
- **Crta „jak / slab"** je ono što jezgra izvodi iz razine i sjemena (vještine, osobine; §1.1), a ekran Ljudi (L2a-10) pokazuje
  uz svakog radnika i voditelja. Dvojica `strucnjak` mogu biti različito dobra; zato student vidi crtu, ne samo naziv razine.
- **„Puno različitih voditelja":** udio `strucnjak` u banci (§2.2: 20 %) daje ~600 mogućih voditelja u M3; ako faza LJUDI pokaže
  manjak po struci, sesija banke diže udio u sljedećim serijama (cilj u `raspodjele.json`), bez promjene formata.

## 1.3 GOST (`G-00001`…)

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

## 1.4 Primjer jednog zapisa svake vrste

```json
{ "id": "R-00001", "ime": "Mirela", "prezime": "Knežević", "rod": "z", "dob": 34, "uloga": "recepcioner",
  "razina": "iskusan", "godine_iskustva": 8, "podrijetlo": "lokalno", "zemlja": "HR", "mjesto": "Kastav",
  "jezici": [["hr", 4], ["en", 3], ["de", 3]],
  "opis": "Osam godina radi na recepcijama u Opatiji. Njemački je naučila u dvije sezone u Salzburgu." }
{ "id": "G-00001", "ime": "Anna", "prezime": "Müller", "rod": "z", "dob": 41, "zemlja": "AT", "mjesto": "Graz",
  "jezici": [["de", 4], ["en", 2]], "segment": "obitelj", "zanimanje": "učiteljica",
  "opis": "Dolazi s mužem i dvoje djece. Opatiju je prvi put vidjela kao dijete, s roditeljima." }
```

## 1.5 VANJSKI (`V-00001`…)

**Leonove odluke 16.10. (sesija 7, tekstom):** (1) banka dobiva **treću vrstu: vanjske ljude**, *„pod vanjski radnici
mislim na inspekciju, policiju i vatrogasce"*; (2) **tri agenta, svaki svoju vrstu** (gosti, radnici, vanjski), svaki piše
samo svoju mapu (§5.5); (3) **zasebna sesija** koja radi samo na banci i provjerava je (§5.6; od 17.10. **paralelna**, istodobno s glavnom). Time ADR-042 t. 8
(pitanje 10, „partneri nakon M1") vrijedi u novom obliku: 🟨 vanjski kreću odmah, korakom K2v, usporedo s K2 i K3.

**Tko je vanjski.** Osoba koja u situaciji dolazi **izvana**: nije zaposlenik hotela (radnik) ni gost koji spava u hotelu.
Hotel s njom pregovara, surađuje ili joj odgovara (inspektor, policajac, dobavljač, novinarka, susjed). Igra danas za njih ima
10 uloga `"from": "partner"` u 55 predložaka i 40 imena (`names.json` → `partner`, 20 ž + 20 m) bez ičega drugog.

### Uloge

| kod | tko | danas u igri (predložak, rod uloge) | primjer `ustanova` (generički) |
|---|---|---|---|
| `inspektor_sanitarni` | sanitarni inspektor | P-INSPEKCIJA (m, `title`) | sanitarna inspekcija |
| 🟨 `inspektor_turisticki` | turistički inspektor (kategorizacija, cjenik, boravišna pristojba) | — | turistička inspekcija |
| 🟨 `inspektor_rada` | inspektor rada (ugovori, radno vrijeme, strani radnici) | — | inspekcija rada |
| 🟨 `inspektor_porezni` | porezni inspektor (fiskalizacija, gotovina) | — | ispostava porezne uprave |
| `policajac` | policajac ili policajka | — (spominje se u P-NESTALA-STVAR, P-PIJANI-GOST, P-DIJETE-IZGUBLJENO) | policijska postaja |
| `vatrogasac` | vatrogasac ili vatrogaskinja | — (spominje se u P-DIZALO) | javna vatrogasna postrojba |
| 🟨 `djelatnik_hitne` | liječnik ili medicinski tehničar hitne pomoći | — (spominje se u P-GOST-POZLILO) | ispostava hitne medicinske pomoći |
| `novinar` | novinar, autor putopisnog bloga | P-NOVINAR (ž) | putopisni časopis |
| `dobavljac` | dobavljač robe ili usluge | P-DOBAVLJAC (m), P-PRAONICA (m) | dobavljač hrane za ugostitelje |
| `organizator_dogadanja` | organizator vjenčanja i događanja | P-VJENCANJE (ž) | agencija za vjenčanja |
| `zastupnik_agencije` | turistička agencija ili portal za rezervacije | P-LAST-MINUTE (ž), P-WELLNESS-PAKET (ž) | turistička agencija |
| `poslovni_klijent` | predstavnik tvrtke koja traži smještaj ili preporučuje hotel | P-PREPORUKA (ž), P-RADNICI-TVRTKE (m) | građevinska tvrtka |
| `susjed` | susjed, mjesni odbor, vlasnik apartmana | P-SUSJEDI-BAZEN (m) | mjesni odbor (neobvezno) |
| 🟨 `lokalna_uprava` | službenik grada ili turističke zajednice | — | turistička zajednica grada |

Policija, vatrogasci i inspekcija su Leonove; sanitarna inspekcija i sedam uloga „danas u igri" već postoje u predlošcima.
🟨 Inspekcija je razdijeljena po vrsti (četiri uloge), jer svaka dolazi zbog drugog odjela i druge pogreške; hitna i
grad/turistička zajednica su prijedlog. Policija, vatrogasci i hitna danas nemaju **ulogu** u predlošku, nego se spominju samo
u rubrikama i odgovorima; da postanu likovi, predložak dobiva novu ulogu (sadržaj situacija, ne banka; pitanje 14).
**Službe** su četiri inspekcije, policajac, vatrogasac, djelatnik hitne i lokalna uprava; **tržišne** uloge su novinar,
organizator, zastupnik agencije i poslovni klijent; dobavljač i susjed su lokalni, ali nisu službe.

### Polja zapisa

| polje | tip | dopušteno | obvezno | napomena |
|---|---|---|---|---|
| `id` | tekst | `V-` + 5 znamenki | da | kao kod radnika i gosta |
| `ime`, `prezime` | tekst | kao kod radnika | da | ne iz `names.json` (§3.7) |
| `rod` | kod | `z` · `m` | da | predložak bira rod uloge („Došao si"), banka ga mora imati |
| `dob` | broj | 🟨 20–80 | da | službe najviše 65 (mirovina), inspektori i djelatnik hitne najmanje 25 (studij i staž) 🟨 |
| `uloga` | kod | 14 kodova iz tablice | da | jedini filtar izbora uz rod (§4.4) |
| `ustanova` | tekst | 🟨 2–60 znakova, malim slovima osim prvog | ne | **generička** vrsta ustanove ili tvrtke, nikad vlastito ime (§3.7) |
| `zemlja` | kod | ISO 3166 iz `mjesta.json` | da | 🟨 dodano: službe, dobavljač, susjed → `HR`; strani samo u tržišnim ulogama |
| `mjesto` | tekst | službe i susjed: katalog `lokalno`; ostali iz HR: `lokalno` ili `regija`; strani: slobodno | da | gdje osoba radi ili živi |
| `jezici` | popis (jezik, razina) | kao kod radnika | da | materinji 4; iz HR: `["hr", 4]` |
| `opis` | tekst | 40–220 znakova, najviše 2 rečenice | ne | činjenice službe i posla („dvanaest godina u sanitarnoj inspekciji"), ne ponašanje |

**Što vanjski NEMA:** `razina` i `godine_iskustva` (ne rade u hotelu, jezgra iz njih ništa ne računa), `podrijetlo` (zemlja
i mjesto dovoljni su za jezik i priču), `segment` i `zanimanje` (to su polja gosta; ulogu nosi `uloga`, tvrtku `ustanova`).
**Ponašanje** (strogost, ton) 🟨 **nije polje**: ton daje `character.brief` predloška (P-INSPEKCIJA: *„Nisi neprijatelj
hotela…"*), pa bi polje u banci proturječilo predlošku, a nijedna situacija ga ne troši. Ako ga kasnije situacije budu
trošile, dolazi novim formatom (§4.3).

**Opis i ustanova ne smiju suziti ulogu uže od predloška** 🟨: predložak i dalje daje `types` („glavni dobavljač mesa"), a
banka osobu. Dobavljač iz banke zato ne piše „ribarnica" ako ga predložak može nazvati dobavljačem mesa; agent serije dobiva
popis `types` po ulozi i piše ono što pristaje svima. To provjerava čovjek (§3.6).

**Veza s `external_cause`.** `external_cause` je zastavica **predloška** (uzrok kvara izvan stanja hotela, ADR-042 t. 7), ne
svojstvo osobe, pa je banka ne sprema. Od šest takvih predložaka dva imaju vanjski lik (P-INSPEKCIJA, P-DOBAVLJAC), tri
nemaju uloge (P-TOPLINSKI-VAL, P-OLUJA, P-NESTANAK-STRUJE), a P-TERMINAL ima gosta. Ako P-OLUJA dobije vatrogasca, to je nova
uloga u predlošku; zastavica i uvjeti kvara se ne mijenjaju.

```json
{ "id": "V-00001", "ime": "Davor", "prezime": "Jelušić", "rod": "m", "dob": 47, "uloga": "inspektor_sanitarni",
  "ustanova": "sanitarna inspekcija", "zemlja": "HR", "mjesto": "Matulji", "jezici": [["hr", 4], ["en", 2]],
  "opis": "Dvanaest godina radi u sanitarnoj inspekciji na Kvarneru. Prije toga bio je tehnolog u pogonu za preradu ribe." }
```
