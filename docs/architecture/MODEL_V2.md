# HotelixSim — model v2 („put B"): hotel od stvari, ne od brojeva

> **Što je ovo:** arhitektura za put B — kako hotel u jezgri od skupa brojeva postaje skup **stvari s imenom i
> stanjem** (odjeli, zaposlenici, sobe, recenzije), sloj po sloj, a da demo i stare igre ostanu netaknuti.
> **Što** student dobiva po dodacima piše u [PROSIRENJA.md](../product/PROSIRENJA.md) (P1–P9); **zašto** ovim putem u
> [ADR-026](../records/DECISIONS.md#adr-026). Ovdje je samo **kako**.
> **Status:** 🟨 nacrt arhitekta, čeka Leonov pregled. Ništa od ovoga još nije izgrađeno.
> Oznake brojki kao u [istraživanju](../research/HOTELSKI_POKAZATELJI.md): **[P]** provjereno · **[PROC]** procjena.

**Rječnik za ovaj dokument** (pojmovi koji se ponavljaju):

| pojam | značenje |
|---|---|
| **entitet** | „stvar" u igri s vlastitim imenom i stanjem: soba 214, konobar Marko, restoran |
| **agregat** | jedan broj za cijeli hotel (npr. moral = 55); danas je hotel samo od agregata |
| **izvedeno** | ne sprema se, nego se svaki put izračuna iz entiteta (moral hotela = prosjek motivacije ljudi) |
| **jezgra** | Rust kod koji računa mjesec (`crates/sim-hotel`); ekran samo prikazuje |
| **dnevnik** | popis događaja igre u bazi; iz njega se igra uvijek može ponovno složiti (ARHITEKTURA §9.1) |
| **snimak pravila** | sve brojke iz `content/hotel/*.json` kopirane u igru pri stvaranju; igra ih nosi zauvijek |
| **sjeme / tok kocke** | broj iz kojeg „slučajnost" nastaje uvijek jednako; svaka svrha ima svoj tok |
| **USALI** | jedinstveni kontni plan za hotele (Uniform System of Accounts for the Lodging Industry): odjeli, njihovi troškovi, GOP |

---

## 0 · Sažetak na jednoj stranici

1. **v1 se zamrzava, v2 je novi paket.** Današnja jezgra (`sim-hotel`) ostaje točno kakva jest i dalje vrti demo i sve
   igre stvorene na njoj. Model v2 živi u novom paketu `sim-hotel-v2`; igra po snimku pravila zna kojim se modelom
   računa (§4). Tako demo na `main` ne može puknuti zbog puta B.
2. **v2 i dalje „govori v1".** Svaki v2 mjesec daje i današnji sažetak (sobe, stalni, sezonci, moral, ocjena…) kao
   **izvedeni** prikaz entiteta. Pet indeksa, mentor, „Zašto", ljestvica i postojeći ekrani rade bez promjene;
   entiteti se dodaju kao **dubina ispod** njih (§3.8).
3. **Detalj samo uz uzrok:** svaki novi podatak ulazi u formulu i u objašnjenje jezgre (`Line`), ne samo na ekran.
4. **Ekvivalencijski test:** v2 podešen kao „jedan odjel, svi ljudi isti, sve sobe iste, bez šuma" mora dati **iste
   brojke kao v1**. To dokazuje da je v2 proširenje, a ne nova igra (§2, t. 6).
5. **Vrijeme ostaje mjesec.** Soba ima stanje i noći u mjesecu, ne rezervacije po danu (§6).
6. **Slojevi:** 0 pokazatelji (S) → 1 odjeli po USALI-ju (L) → 2 zaposlenici (L) → 3 sobe (M) → 4 recenzije (M–L).
   Najmanji „wow" za profesora = sloj 0 + 1 (§10).

---

## 1 · Gdje smo danas (model v1)

| što | kako je danas | gdje |
|---|---|---|
| Stanje hotela | 18 polja, sve agregati | `crates/sim-hotel/src/state.rs:24-43` |
| Sobe | jedan broj `rooms: u16` (120 u sva tri profila) | `state.rs:26`, `content/hotel/profiles.json` |
| Osoblje | dva broja: `permanent`, `seasonal`; odluka je „ciljani broj sezonaca" | `state.rs:27-28`, `plan.rs:24-33` |
| Ljudi i oprema | moral, iskustvo, oprema, zadovoljstvo kao bodovi 0–10 000 | `state.rs:31-34` |
| Ocjena portala | jedan broj, svaki mjesec 25 % prema cilju iz zadovoljstva | `state.rs:35`, `resolve.rs:413-421` |
| Ostali prihod | ravnih 30 % prihoda od soba (hrana, piće, usluge) | `resolve.rs:266-269`, `catalog.json` `other_revenue_bp: 3000` |
| Troškovi | plaće + CPOR 22 € po sobi + održavanje + marketing + jedan `fixed_cents` (150 000 € za C) | `resolve.rs:270-296` |
| Sobe izvan funkcije | izvedeno iz opreme: `(40 − oprema) / 4` | `market.rs:61-63` |
| Kapacitet osoblja | jedan za hotel: zaposleni × dani × 2,4 | `market.rs:87` |
| Odlasci osoblja | očekivana vrijednost, bez kocke; obračun mjeseca nema slučajnosti uopće | `resolve.rs:197-199`, potpis `resolve.rs:444-450` |
| Pokazatelji | računa se samo GOPPAR (po svim sobama × dani); **ADR, RevPAR, TRevPAR ne postoje** ni u jezgri ni na webu | `resolve.rs:309` |

Danas hotel „ima" restoran samo kao 30 % u jednoj formuli, i 58 ljudi samo kao dva broja. Zato student ne može
djelovati na pojedinca, a profesor ne vidi račun dobiti kakav uči na Accountingu.

---

## 2 · Načela

1. **Dubina samo uz uzrok.** Polje koje ne ulazi ni u jednu formulu ne ulazi ni u model. Ukras (hobi konobara) smije
   biti u tekstu, ne u stanju.
2. **Jedan izvor istine.** Svako „zašto" (ekran „Zašto", mentor, profesor) dolazi iz retka objašnjenja jezgre
   (`Line`, `crates/sim-hotel/src/explain.rs:59-64`) i iz `game::debrief_facts` (`crates/game/src/mentor.rs:755`),
   kao danas ([ADR-023](../records/DECISIONS.md#adr-023)). Ekran ništa ne računa (ARHITEKTURA §13).
3. **Determinizam ostaje kakav jest** (ARHITEKTURA §7.2): cijeli brojevi (bodovi 0–10 000, novac u centima), samo
   uređene mape (`BTreeMap`), redoslijed po ID-u, nikad po vremenu. Nove slučajnosti dobivaju **svaka svoj tok**:
   `Rng::derive(sjeme, [oznaka svrhe, mjesec, slot hotela, …])` (`crates/engine/src/rng.rs:41`). Postojeći tokovi
   (`game:robot-plan`, `game:robot-answer`, `crates/game/src/command.rs:93,110`) se ne diraju, pa nova kocka ne pomiče
   stare.
4. **Šum s nultim zbrojem.** Gdje kocka daje raznolikost (koja soba, koji gost, tko odlazi), ona raspoređuje, ali ne
   mijenja zbroj: broj odlazaka i prosjek ocjena recenzija računa formula, kocka bira **tko** i **koja**. Tako balans
   ostaje stabilan, a pojedinačne priče su različite.
5. **Stare igre i iskreno obećanje (ARHITEKTURA §17).** Igra se zauvijek računa modelom i jezgrom s kojom je
   stvorena (§4.2). Ne obećajemo iste brojke stare igre na novoj jezgri — i ne pokušavamo: v1 kod se ne mijenja.
6. **Ekvivalencija s v1.** Svaki sloj ima test: v2 u „degeneriranom" katalogu (jedan odjel, jednaki ljudi, jednake
   sobe, šum isključen) daje iste `MonthReport` brojke kao v1 za A, B, C kroz četiri mjeseca. Gdje v2 **namjerno**
   mijenja pravilo (npr. soba se gasi po vlastitom stanju, ne po prosjeku), test tu razliku navodi poimence, a balans
   je mjeri.
7. **Demo je siguran.** Sve na granama `model-v2/…`; `main` dobiva samo sloj 0 (prikaz) nakon demoa, a v2 tek kad
   Leon kaže, i to uz zadani model v1 dok se izričito ne prebaci.
8. **Profesorovih pet indeksa ostaje** (G R Z O F, centrirani na referentni hotel, ADR-013). Mijenja se samo odakle
   indeks čita: iz entiteta umjesto iz spremljenog agregata.

---

## 3 · Model entiteta

### 3.1 Pregled

```text
Hotel (v2)
├── odjeli        BTreeMap<KodOdjela, Odjel>        8–11 po hotelu (ovisi o profilu)
├── zaposlenici   BTreeMap<IdZaposlenika, Zaposlenik>  ~55–75 (raste u kolovozu)
├── sobe          BTreeMap<IdSobe, Soba>            120
├── recenzije     bazen (zbroj, broj) + recenzije zadnjeg mjeseca u izvješću
└── ostaje iz v1  kategorija, blagajna, odgođeni učinci, grupe, „obnova već bila", zadovoljstvo (s pamćenjem)
```

### 3.2 Odjel (sloj 1)

Odjeli prate USALI. Katalog kaže koje **vrste** postoje; profil kaže koje hotel **ima**.

| USALI razred | odjeli u igri | prihod | izravni troškovi |
|---|---|---|---|
| Rooms (operativni) | Smještaj (recepcija + domaćinstvo) | prodane noći × cijena | plaće, potrošni materijal i rublje po prodanoj sobi (današnji CPOR) |
| F&B (operativni) | Restoran · Lobby bar / kafić · Rooftop bar · Bar uz bazen | gosti × udio koji koristi × prosječni račun | plaće, trošak namirnica i pića (% prihoda) |
| Other Operated | Wellness i spa | isto, po tretmanu | plaće, materijal |
| Rentals / Misc | Parking, najam lokala | fiksno + po gostu | gotovo nikakav |
| Undistributed | Uprava (A&G) · Prodaja i marketing · Održavanje (POM) · Energija | — | plaće + odluke (marketing, održavanje) + stalni dio |
| ispod GOP-a | Fiksne naknade (osiguranje, najam, porez na imovinu) | — | stalni iznos → EBITDA |

**Polja odjela.** Spremljeno: `kod`, `otvoren` (da/ne — npr. rooftop zatvoren u lipnju), `stanje` (0–10 000, oprema
odjela), `razina_cijene` (90–120 % kataloške), `broj_ljudi` (samo u sloju 1; od sloja 2 izvedeno iz zaposlenika).
Izvedeno svaki mjesec: kapacitet, opterećenje, kvaliteta usluge, udio gostiju koji ga koriste, prihod, troškovi,
odjelni rezultat.

**Kvaliteta usluge odjela** je današnja formula kvalitete (`resolve.rs:382-389`: 40 % iskustvo, 30 % moral, 30 %
manjak opterećenja), ali s ljudima i opterećenjem **tog** odjela. Kvaliteta hotela = prosjek po tome koliko gostiju
koji odjel dotakne. Kad je hotel jedan odjel, to je točno današnja brojka (načelo 6).

**Prijedlog odjela po profilu** [PROC], da se profili razlikuju po priči (DEMO §5):

| profil | odjeli osim Smještaja i neraspoređenih | zašto |
|---|---|---|
| **A** „Ugledni, ali umorni" | Restoran · Lobby bar · Wellness i spa · Rooftop bar · Parking | puno sadržaja nosi ugled, ali i troši premalo ljudi |
| **B** „Profitabilni, ali zapušteni" | Restoran · Bar uz bazen (stara oprema) · Parking · Najam lokala | malo sadržaja, zdrava blagajna, stara oprema |
| **C** „Prosječni" | Restoran · Lobby bar · Mali wellness · Parking | referentni hotel |

### 3.3 Zaposlenik (sloj 2)

| polje | spremljeno / izvedeno | napomena |
|---|---|---|
| `id` | spremljeno | redni broj u hotelu (`Z-0001`); novi dobiva sljedeći (`sljedeci_id` u stanju) |
| `ime`, `prezime` | spremljeno kao **indeks u banku imena** iz snimka | tekst se slaže pri čitanju |
| `izgled` | izvedeno iz (sjeme profila, `id`) | postojeći `AvatarLook` (`apps/web/src/avatar/`) |
| `uloga` | spremljeno | recepcioner, sobarica, kuhar, konobar, šanker, terapeut, tehničar, voditelj… (katalog) |
| `odjel` | spremljeno | premještaj ga mijenja |
| `ugovor` | spremljeno | stalni · sezonski do mjeseca X |
| `placa_pct` | spremljeno | prema tržišnoj plaći uloge (danas jedan `wage_pct` za sve, `state.rs:29`) |
| `iskustvo`, `motivacija` | spremljeno, 0–10 000 | motivacija = današnji moral, ali po osobi |
| `staz_mjeseci`, `osobina` | spremljeno | jedna osobina iz kataloga (npr. „pouzdan": niži rizik odlaska) |
| `rizik_odlaska` | izvedeno | današnja `turnover_bp` (`state.rs:76-80`) iz **njegove** motivacije ± osobina |
| `opterecenje`, `trosak_mjeseca` | izvedeno | iz odjela i plaće |

**Početni ljudi su isti u svakoj igri za isti profil**: generira ih jezgra iz specifikacije profila i **fiksnog sjemena
profila** (u snimku), uz ispravak da prosjek točno pogodi brojke profila (npr. prosječna motivacija C = 55,00).
Pravednost: svi studenti s hotelom A imaju iste ljude. (Pitanje 1 u §9.)

**Profil C u lipnju** [PROC]: Recepcija 7 · Domaćinstvo 18 · Kuhinja 10 · Posluga restorana 9 · Bar 5 · Wellness 3 ·
Održavanje 3 · Uprava i prodaja 3 = **58** (danas 30 stalnih + 28 sezonaca, `profiles.json`).

### 3.4 Soba (sloj 3)

| polje | spremljeno / izvedeno | napomena |
|---|---|---|
| `id` | spremljeno, iz tlocrta u katalogu | broj sobe: kat × 100 + redni (`214`) |
| `tip` | spremljeno | standard · superior s pogledom na more · obiteljska · apartman [PROC 70/30/12/8 sobe] |
| `stanje` | spremljeno, 0–10 000 | troši se s prodanim noćima te sobe; održavanje i obnova ga vraćaju |
| `status` | spremljeno | u funkciji · izvan funkcije (uzrok) · u obnovi (do mjeseca) |
| `obnovljena` | spremljeno | mjesec obnove; donosi doplatu i bolje recenzije |
| `zadnji_kvar` | spremljeno | tema (klima, kupaonica, namještaj…) za recenzije i karticu sobe |
| `noci_mjeseca`, `prihod_mjeseca` | u izvješću mjeseca, ne u stanju | iz raspodjele prodanih noći |

### 3.5 Recenzija i segment gosta (sloj 4)

**Recenzija** se sprema **kao kodovi, ne kao tekst**: `id` (`R-jul-037`), mjesec, segment, soba, odjeli koje je gost
koristio, ocjena (desetinke 1,0–10,0), 1–3 teme s predznakom (`dorucak.spora_usluga −`, `osoblje.ljubazno +`),
odgovor (ako ga je student dao). Tekst se slaže pri čitanju iz **banke rečenica** u snimku, izborom iz
(sjeme, `id`) — isti tekst svaki put, nula bajtova teksta u dnevniku. Samo istaknuta recenzija smije imati AI tekst
(§5, sloj 4).

**Segment gosta** je samo podatak iz kataloga (par, obitelj, poslovni, grupa, wellness gost: udio po mjesecu i što mu
je važno). Segmenti boje recenzije i težine tema; **ne** uvode zasebnu potražnju po segmentu (§8).

### 3.6 ID-jevi — uvijek iz pravila, nikad slučajni

| entitet | ID | zašto je stabilan |
|---|---|---|
| odjel | kod iz kataloga (`restoran`) | katalog je u snimku |
| soba | broj iz tlocrta (`214`) | tlocrt je u snimku |
| zaposlenik | `Z-` + redni broj u hotelu | brojač je u stanju; zapošljavanje ide redom naloga |
| kandidat za posao | (mjesec, uloga, 1–3) | izveden iz sjemena igre i slota hotela |
| recenzija | (mjesec, redni broj u hotelu) | redoslijed iz raspodjele gostiju |

Nikad uuid ni vrijeme: isti dnevnik daje iste ID-jeve na svakom računalu.

### 3.7 Kako agregati postaju izvedeni

| polje v1 (`state.rs`) | u v2 | spremljeno? |
|---|---|---|
| `rooms` | broj soba | izvedeno |
| `permanent`, `seasonal` | brojanje zaposlenika po ugovoru | izvedeno |
| `wage_pct`, `extra_wage_cents` | plaća svake osobe; „svima +x %" ostaje kao skupna radnja | izvedeno (prosjek za prikaz) |
| `morale_pts` | prosjek motivacije po glavi | izvedeno |
| `experience_pts` | prosjek iskustva po glavi | izvedeno |
| `equipment_pts` | 70 % prosjek stanja soba + 30 % prosjek stanja odjela [PROC] | izvedeno |
| `satisfaction_pts` | anketa gostiju; ima **pamćenje** (pola puta prema cilju, `resolve.rs:405`) | spremljeno, cilj se računa iz odjela i soba |
| `rating` | bazen recenzija: zbroj i broj, stari dio slabi 25 % mjesečno | spremljeno kao bazen |
| `load_hundredths`, `turnover_bp` | rezultat mjeseca po odjelu / stvarni odlasci | u izvješću, ne u stanju |
| `fixed_cents` | neraspoređeni troškovi po odjelu + fiksne naknade | katalog |
| `cash`, `pending`, `groups`, `renovated`, `category` | isto | spremljeno |

### 3.8 Pet indeksa na novom modelu

Formule iz DEMO §4 i `crates/sim-hotel/src/indices.rs:82-128` ostaju **iste**; mijenja se samo odakle dolaze ulazi.

| indeks | formula (ne mijenja se) | ulaz u v2 | što student sada vidi ispod |
|---|---|---|---|
| **G** gosti | zadovoljstvo | cilj iz kvalitete odjela, stanja soba i vrijednosti za novac | koji je odjel ili tip sobe vukao dolje |
| **R** recenzije | (ocjena − 7,0) / 2,8 | ocjena iz bazena recenzija | recenzije i teme koje su je pomaknule |
| **Z** zaposlenici | 60 % moral + 40 % zadržavanje | prosjek motivacije; stvarni odlasci / prosječni broj | tko je otišao i zašto, tko je u riziku |
| **O** operativa | 45 % oprema + 30 % iskustvo + 25 % opterećenje | stanje soba i odjela; iskustvo ljudi; opterećenje po odjelu | koje su sobe izvan funkcije, koji je odjel preopterećen |
| **F** financije | 70 % GOPPAR prema referentnom + 30 % likvidnost | GOP po USALI-ju (zbroj odjela − neraspoređeni) | odjelni rezultati |

`index_drivers` (`indices.rs:71-80`) dobiva pokretače odjela, ljudi i soba, pa „što je pokrenulo indeks" ide do
pojedinca: „Z je pao 4 boda: otišli su Marko (restoran) i Ivana (domaćinstvo), oboje motivacija ispod 35".

---

## 4 · Kako se v2 uklapa u igru

### 4.1 Paketi

```text
engine ◄── sim-hotel (v1, ZAMRZNUT) ◄── sim-hotel-v2 (koristi Month, Index, IndexSet, score, calc iz v1)
               ▲                              ▲
               └──────── game (bira model po snimku) ◄── content (čita i nove datoteke)
```

`sim-hotel-v2` počinje kao kopija v1 na početku sloja 1 (sloj 0 ne mijenja brojke, pa ide u v1, §5). Pravila
ovisnosti (ARHITEKTURA §6.2) i zabrane (`lib.rs:2-7`: bez decimalnih brojeva, bez prelijevanja) vrijede i za v2.

### 4.2 Stare igre — točno što se događa

Danas: `GameCreated.core_version` = `sim_hotel::CORE_VERSION` = verzija paketa `0.1.0` (`lib.rs:50`,
`crates/sim-hotel/Cargo.toml:3`), a obračun se **odbija** ako se ne slaže (`crates/game/src/resolve.rs:111-117`;
web dobiva grešku 500 preko `crates/app/src/error.rs:170`). Uz to, `fold` pri svakom čitanju igre **ponovno
računa** referentnu tablicu i početne indekse trenutnim kodom (`crates/game/src/game.rs:131-141`). Posljedica: svaka
promjena formula u `sim-hotel` tiho bi promijenila početne indekse i napredak **starih** igara. Zato:

1. **Snimak pravila dobiva polje `model`** (`"v1"` kad ga nema — sve današnje igre). Novo polje ima
   `serde(default, skip_serializing_if)`, pa stari otisak (`legacy_fingerprint`, `crates/content/src/lib.rs:607`)
   ostaje isti.
2. **v1 igre** se zauvijek čitaju, nastavljaju i ponovno obračunavaju v1 kodom, s istim brojkama. `sim-hotel` ostaje
   `0.1.0`; „zlatni test" (ista sezona sa sjemenom 7 → isti otisak cijelog izvješća) ruši CI ako se v1 broj pomakne.
3. **v2 igre** provjeravaju svoju verziju (`sim_hotel_v2::CORE_VERSION`, od `2.0.0`). Unutar v2 svaki sloj diže
   verziju; v2 igra stvorena u sloju 1 nastavlja se samo jezgrom sloja 1 (poruka „igra je stvorena starijom verzijom",
   ne 500). Za pilot je to dovoljno: v2 igre prije pilota su probne.
4. v1 se nikad ne briše dok postoji v1 igra koju netko želi otvoriti.

### 4.3 Dnevnik i događaji (samo dodavanje)

| što | promjena | zašto ovako |
|---|---|---|
| `GameCreated.snapshot` | nove datoteke sadržaja (§5) + `model` | puni snimak pravila ostaje u igri (ARHITEKTURA §7.2) |
| `PlanSet.plan` (`PlanDto`) | novo polje `nalozi: Vec<Nalog>` (zaposli, otpusti, plaća, obuka, premjesti, razina cijene odjela, otvori/zatvori odjel, obnova sobe/kata, soba izvan funkcije, odgovor na recenziju) | nalog je dio plana mjeseca: „zadnji plan vrijedi" ostaje jedino pravilo; roboti ga pišu istim putem |
| `RoundResolved.hotels[]` | novo polje `detalj` (v2): entiteti **nakon** mjeseca, odjelni račun, noći po sobi, odlasci i dolasci s razlozima, recenzije mjeseca | `report` (v1 oblik) i dalje nosi sažetak, pa postojeći ekrani i mentor rade |
| `SCHEMA_VER` (`crates/game/src/event.rs:9`) | ostaje 1 dok su sva polja dodatna s `serde(default)` | isti obrazac kao `free_text` (`event.rs:100-102`) |

Stanje na početku mjeseca (`Game::state_at`, `game.rs:368-386`) za v2 čita `detalj` prethodnog mjeseca umjesto
`report.after`. Danas se `before` i `after` spremaju oba (`crates/game/src/dto.rs:538-546`); v2 sprema samo `after`
entiteta (`before` je prethodni `after`), da dnevnik ne raste dvostruko.

### 4.4 Gdje je najveći tehnički trošak

`game` je danas čvrsto vezan na v1 tipove (`HotelState`, `MonthPlan`, `MonthReportDto` u `fold`, `state_at`,
`effective_plan`, `robot_moves`, `resolve_round`). Model v2 traži da `game` drži `enum` „stanje v1 ili v2" i grana na
pet mjesta. To je **prvi posao sloja 1** i najveći pojedinačni komad (§5). Isplati se jer se radi jednom.

### 4.5 API (samo dodavanje)

Postojeće putanje ostaju. Nove, samo za vlastiti hotel (i profesora):

| putanja | sloj | što vraća |
|---|---|---|
| `GET /v1/hotels/{hotel_id}/kpi` | 0 | popunjenost, ADR, RevPAR, TRevPAR, GOPPAR, marža, udio rada, točka pokrića, po mjesecima |
| `GET /v1/hotels/{hotel_id}/pl?month=` | 1 | račun dobiti po USALI-ju |
| `GET /v1/hotels/{hotel_id}/departments` · `/{code}` | 1 | odjeli; u odjelu ljudi, prihod, troškovi, stanje, rezultat po mjesecima |
| `GET /v1/hotels/{hotel_id}/staff` · `/{employee_id}` · `/candidates` | 2 | ljudi, profil s poviješću, kandidati mjeseca |
| `GET /v1/hotels/{hotel_id}/rooms` · `/{room_id}` | 3 | sve sobe (tlocrt), kartica sobe s poviješću |
| `GET /v1/hotels/{hotel_id}/reviews?month=&topic=` | 4 | recenzije s tekstom, teme, odgovori |
| `PUT …/plan` | 1–4 | postojeći, s `nalozi` |
| `GET /v1/games/{game_id}/compare` | 0–4 | profesor: hoteli jedan uz drugoga (§7) |

Tipovi za web se i dalje generiraju (OpenAPI → klijent); ručnih tipova nema.

---

## 5 · Slojevi

Svaki sloj: gradi se na svojoj grani, dokaže (testovi, `cargo run -p balance --release -- --seeds 1000 --check`,
pregled u pregledniku na mobitelu i laptopu, [ADR-027](../records/DECISIONS.md#adr-027) raspored), pregleda i tek
onda ide sljedeći. Veličine su sati rada agenata [PROC]; S ≤ 10 h, M 10–25 h, L 25–45 h.

### Sloj 0 — pokazatelji (P2) · **S, 6–10 h**

- **Student vidi:** karticu „Pokazatelji" na Mom hotelu i ekran s tablicom po mjesecima: popunjenost, **ADR**,
  **RevPAR**, TRevPAR, GOPPAR, GOP marža, udio troška rada, CPOR, **točka pokrića** (pri kojoj popunjenosti GOP = 0).
  Uz svaki broj jedna rečenica formule („ADR = prihod od soba / prodane noći"). Laptop: tablica; mobitel: kartice
  koje se listaju po mjesecu.
- **Mehanika:** nijedna nova. Čiste funkcije u `sim-hotel/src/kpi.rs` nad postojećim `MonthReport`; **ne spremaju
  se**, računaju se pri čitanju. Brojke igre se ne mijenjaju, zato sloj 0 smije u v1 i u `main` nakon demoa.
- **Odluka o ponudi soba:** RevPAR i GOPPAR dijele se sa **svim** sobama × dani (kao danas GOPPAR, `resolve.rs:309`),
  a popunjenost isto, da vrijedi RevPAR = ADR × popunjenost; sobe izvan funkcije prikazuju se zasebno kao
  „izgubljene noći". Danas web računa popunjenost nad raspoloživim sobama (`HotelScreen.tsx:132`) — mijenja se
  (pitanje 6 u §9).
- **Testovi:** RevPAR = ADR × popunjenost (u granici zaokruživanja); dijeljenje nulom (mjesec bez prodaje); ista
  brojka kroz API i ekran.
- **Paralelno:** jezgra + API (jedan agent) · ekran (drugi, nad dogovorenim oblikom) · profesorova tablica (treći).

### Sloj 1 — odjeli po USALI-ju (P3, P4) · **L, 30–45 h**

- **Student vidi:** odjeljak **„Dijelovi hotela"**: kartica po odjelu (samo oni koje hotel ima) s prihodom, troškom,
  rezultatom, brojem ljudi, opterećenjem, stanjem i trendom. Klik → odjel: račun odjela po mjesecima, ljudi (u sloju 1
  broj po ulozi), stanje opreme, „Zašto se promijenio". Ekran **„Račun dobiti"** u obliku USALI: odjeli → odjelni
  rezultati → neraspoređeni troškovi → **GOP** → fiksne naknade → EBITDA. Laptop: dva stupca (popis | detalj);
  mobitel: popis pa puni ekran.
- **Student radi** (nalozi u planu): broj sezonaca **po odjelu** (zamjenjuje današnji jedan `seasonal_target`);
  razina cijene odjela 90–120 %; otvori/zatvori sezonski odjel (rooftop u lipnju?); raspodjela održavanja po
  odjelima.
- **Mehanika:** gosti u kući = prodane noći × gosti po sobi. Prihod odjela = gosti × udio koji ga koristi × prosječni
  račun. Udio pada s višom cijenom odjela i nižom kvalitetom. Kapacitet odjela (ljudi × norma) ograničava broj
  posluženih; višak je izgubljen prihod i kazna opterećenja. Odjelni troškovi = plaće ljudi odjela + trošak robe
  (% prihoda) + ostalo. Prvi uzročni lanac koji student vidi:

  ```text
  premalo kuhara u kolovozu → opterećenje kuhinje iznad norme → kvaliteta restorana pada
     → manje gostiju večera u hotelu (udio 38 % → 31 %) → prihod F&B pada
     → zadovoljstvo gostiju (G) pada → ocjena portala sporo pada → potražnja iduće mjesece
  ```

- **Sadržaj:** `content/hotel/departments.json` (vrste, USALI razred, norme, udio korištenja po mjesecu, prosječni
  račun po kategoriji, trošak robe %, stanje), `profiles.json` (popis odjela profila, ljudi po odjelu),
  `catalog.json` (raspad `fixed_cents` na neraspoređene troškove i fiksne naknade; `other_revenue_bp` ostaje samo u v1
  snimku). Kalibracija: za C zbroj ostalih odjela ≈ 30 % prihoda soba (današnja brojka), F&B najveći dio [PROC].
- **Događaji/API:** `model: "v2"`, `nalozi`, `detalj` s odjelima (§4.3); `/pl`, `/departments`.
- **Objašnjenje i mentor:** `Line` dobiva `opseg` (hotel · odjel) i nove uzroke (`Odjel(kod)`, `KapacitetOdjela`,
  `CijenaOdjela`). `debrief_facts` dobiva „odjeli": najveći rast i pad odjelnog rezultata, riječima i u eurima.
- **Balans (nove provjere):** odjelne marže u pojasu (Smještaj 65–80 %, F&B 15–35 %, wellness 20–40 % [PROC]); udio
  troška rada 30–40 % prihoda (LRH ≈ 35 % [P]); TRevPAR/RevPAR za C 1,25–1,40; zatvaranje odjela nije dominantna
  radnja; postojeće §11.1–11.5 i dalje prolaze, s profilima koji se razlikuju po odjelima.
- **Testovi:** ekvivalencija (jedan odjel = v1); zbroj odjela = ukupno, do centa; model u snimku bira jezgru; v1 zlatni
  test; stari v1 dnevnik se čita i nastavlja.
- **Redoslijed i paralelno:** prvo **jedan** agent: kopija jezgre + `game` s dva modela (§4.4, 10–14 h, ništa drugo
  ne smije teći dok ovo nije gotovo). Zatim paralelno: formule odjela (jezgra) · sadržaj JSON · ekrani nad lažnim
  podacima · profesorov račun dobiti · provjere balansa.

### Sloj 2 — zaposlenici kao pojedinci (P5, P6) · **L, 30–40 h**

- **Student vidi:** odjeljak **„Ljudi"**: popis po odjelima (lik, ime, uloga, motivacija, iskustvo, plaća, opterećenje,
  oznaka rizika odlaska), filtri „u riziku", „preopterećeni", „sezonci". Profil osobe: povijest po mjesecima
  (motivacija, iskustvo, obuke, premještaji), „zašto je motivacija pala" (iz jezgre). Unutar odjela iz sloja 1 broj
  ljudi postaje stvarni popis.
- **Student radi** (nalozi): **zaposli** (kandidati mjeseca: po 3 po ulozi, svaki s iskustvom, traženom plaćom i
  osobinom), **otpusti** (stalni: otkazni rok i otpremnina iz kataloga, kolege u odjelu −motivacija; sezonac: kraj
  ugovora na kraju mjeseca bez troška), **povisi plaću** jednoj osobi, **pošalji na obuku** (osoba je taj mjesec dio
  vremena odsutna — manji kapacitet odjela), **premjesti** u drugi odjel (iskustvo u novoj ulozi se prepolovi).
  Skupne radnje iz v1 ostaju kao prečaci: „svima plaća x %", „obuka cijelog odjela", „sezonci po planu".
- **Mehanika:** današnje formule po osobi. Cilj motivacije = 55 + 2 × (njegova plaća % − 100) − pola kazne
  opterećenja **njegovog odjela** + obuka + situacije + osobina; pomak 30 % mjesečno (`resolve.rs:185-196`). Broj
  odlazaka = zaokruženi zbroj rizika svih ljudi (isti kao v1 kad su svi jednaki); **tko** odlazi bira kocka
  (`v2:odlasci`, mjesec, slot), ponderirano rizikom. Odlazak se zamjenjuje kandidatom s iskustvom 20 (kao v1,
  `resolve.rs:199-211`) samo ako je tako zadano; inače odjel ostaje kraći. Lanac:

  ```text
  Marko, konobar: plaća 92 %, opterećenje restorana 2,3 → motivacija 31 → rizik odlaska 7,4 %
     → u srpnju odlazi → zamjena s iskustvom 20 → kvaliteta restorana −3 → zadovoljstvo −1
     → Z pada (odlazak + motivacija), O pada (iskustvo), F trošak zapošljavanja 1 500 €
  ```

- **Sadržaj:** `roles.json` (uloge, odjel, tržišna plaća, norma), `names.json` (banka hrvatskih i regionalnih imena),
  osobine u `catalog.json`, generator početnih ljudi po profilu.
- **Roboti:** svaka strategija dobiva jednostavno pravilo za ljude (Štedljivi: otpušta najskuplje sezonce; Brižni:
  obuka i povišica najmanje motiviranima; Lovac na popunjenost: zapošljava do kapaciteta; Slučajni: nasumični nalozi
  unutar granica), da balans mjeri i pojedinačne radnje.
- **Objašnjenje i mentor:** uzroci `Odlazak(id)`, `Zaposlenje(id)`, `Premjestaj(id)`; činjenice „ljudi": tko je
  otišao i zašto (najviše tri), tko je najveći rizik idući mjesec.
- **Balans:** fluktuacija sezonaca 30–50 % sezonski, stalnih 15–25 % godišnje [PROC iz istraživanja]; probe za
  dominantne radnje: „otpusti najmanje motivirane svaki mjesec", „svima najveća plaća", „sve na obuku" — nijedna ne
  smije pobijediti za sva tri profila (DEMO §11.4).
- **Testovi:** ekvivalencija (jednaki ljudi = v1); ista sezona, isto sjeme → isti ljudi odlaze; početni prosjeci =
  brojke profila; nalog na nepostojeću osobu se odbija pri upisu plana (`MonthPlan::validate`, `plan.rs:83`).
- **Paralelno:** jezgra (formule po osobi) · sadržaj (imena, uloge) · ekrani popisa i profila · roboti i balans.

### Sloj 3 — sobe sa stanjem (P7) · **M, 15–25 h**

- **Student vidi:** odjeljak **„Sobe"**: tlocrt po katovima (laptop: mreža 120 pločica obojenih po stanju; mobitel:
  popis po katu s filtrima „izvan funkcije", „najlošije", „tip"). Klik → kartica sobe: tip, stanje, noći i prihod po
  mjesecu, kvarovi, obnova, recenzije gostiju te sobe (od sloja 4). ADR po tipu sobe na ekranu pokazatelja.
- **Student radi:** **obnova** kata ili tipa sobe (umjesto današnje jedne obnove cijelog hotela, `catalog.json`
  `renovation_cents`): sobe su taj mjesec **izvan prodaje**, zatim stanje +40 i doplata „obnovljena"; **prioritet
  održavanja** (raspodijeli na sve ili na najgore); **ručno izvan funkcije** (soba se popravlja umjesto da gosti pišu).
- **Mehanika:** prodane noći se raspoređuju po sobama pravilom (tip prema potražnji, pa bolje stanje, pa niži broj) —
  bez kocke. Trošenje sobe = 150 + 8 × njezine noći [PROC; 30 noći ≈ današnjih −4]. Kvar: soba ispod praga stanja
  ima šansu kvara (`v2:kvar`, mjesec, slot, soba) → izvan funkcije dok se ne popravi. Stanje sobe ulazi u ocjenu
  gosta te sobe (sloj 4) i u O kroz prosjek. Lanac učenja o investiciji:

  ```text
  obnova 3. kata u lipnju: 20 soba izvan prodaje u mjesecu niske potražnje (izgubljeno ≈ 430 noći)
     → stanje 45 → 85, doplata +15 % → više ADR i bolje recenzije srpanj–rujan
     → ekran pokazuje povrat ulaganja; ista obnova u kolovozu košta rasprodane noći
  ```

- **Sadržaj:** `rooms.json` (tlocrt, tipovi, doplate, trošenje, prag kvara, cijena obnove po sobi); profili dobivaju
  raspodjelu početnog stanja (B: stari 4. kat).
- **Paralelno sa slojem 2:** da — drugi entiteti, druge datoteke; dijele samo `detalj` u događaju (dogovor oblika
  unaprijed).
- **Balans:** udio soba izvan funkcije realan (0–5 % za C [PROC]); obnova nije dominantna; kolovozna obnova lošija od
  lipanjske (provjera kao S3 u ADR-014).

### Sloj 4 — recenzije s tekstom (P8) · **M–L, 20–30 h** (od toga ~8 h pisanje banke rečenica)

- **Student vidi:** odjeljak **„Recenzije"**: popis po mjesecu (ocjena, segment, tekst, soba, teme), filtri po temi i
  ocjeni, oblak „o čemu gosti pišu" (koliko puta doručak, čistoća, osoblje…), kretanje ocjene. Laptop: popis | čitanje;
  mobitel: kartice.
- **Student radi:** **odgovara na recenziju**: bira pristup (ispriča i konkretna mjera · zahvala · obrana ·
  šablona) [prvi rez bez AI-ja]; voditelj ima ograničen broj odgovora mjesečno, pa bira na koje. Današnji „odgovaraj na
  recenzije" (1 500 €, +0,05 ocjene, `catalog.json`) ostaje kao skupna radnja (agencija odgovara na sve, prosječno).
- **Mehanika:** broj boravaka = prodane noći / prosječni boravak (4,2 [PROC]); recenzija od ~12 % boravaka [PROC]
  → za C u srpnju ≈ 80 recenzija. Ocjena gosta = očekivana ocjena iz njegove sobe, odjela koje je koristio i
  vrijednosti za novac (današnji cilj 5,6 + 0,04 × zadovoljstvo, `resolve.rs:413`) + šum s nultim zbrojem; teme su
  dva najveća pokretača te ocjene. Ocjena portala = bazen: stari dio 75 %, novi mjesec 25 % (isto kao v1). Dobar
  odgovor na lošu recenziju: mali plus ocjeni i potražnji (gosti čitaju odgovore) [PROC].
- **AI:** samo „istaknuta recenzija" (jedna mjesečno po hotelu, najlošija ili najbolja) smije dobiti AI tekst na
  Haikuu, kao događaj s AI metom i rezervom iz banke (isti obrazac kao lik i mentor, ARHITEKTURA §10). Sve ostalo
  je banka rečenica.
- **Sadržaj:** `reviews.hr.json`: tema → rečenice po predznaku i segmentu (≈ 30 tema × 2 predznaka × 4 inačice
  [PROC]); Leon pregledava jezik.
- **Ovisi o:** slojevima 1–3 (teme pokazuju na odjele i sobe). Bez sloja 3 teme o sobi koriste prosjek opreme.
- **Balans:** varijacija ocjene mjesec na mjesec ≤ 0,3 za C bez odluka; broj recenzija u pojasu; odgovaranje nije
  dominantno.

### Tablica veličina

| sloj | veličina | sati [PROC] | najveći komad | smije paralelno s |
|---|---|---|---|---|
| 0 | S | 6–10 | ekran i profesorova tablica | sve (ne dira brojke) |
| 1 | L | 30–45 | `game` s dva modela + kalibracija | — (temelj ostalih) |
| 2 | L | 30–40 | formule po osobi, nalozi, roboti | slojem 3 |
| 3 | M | 15–25 | raspodjela noći, obnova, tlocrt | slojem 2 |
| 4 | M–L | 20–30 | banka rečenica, odgovori | — (treba 1–3) |
| P9 profesor v2 | M | 10–15 | usporedni pregled | uz svaki sloj |

Ukupno ≈ 110–165 sati rada agenata. S paralelnim agentima to su **tjedni, ne dani**; iskreno: svih pet slojeva nije
realno prije sredine studenog [PROC].

---

## 6 · Mjerilo vremena i veličina

**Krug ostaje mjesec.** Igra nema dane, pa ni „rezervaciju za 14. srpnja". Sve „dnevno" je prosjek mjeseca:

| entitet | što znači „stanje" u mjesečnoj igri | što **ne** glumimo |
|---|---|---|
| soba | stanje na kraju mjeseca, noći prodane u mjesecu, kvar koji ju je izbacio, mjesec obnove | koji gost je spavao koje noći |
| zaposlenik | motivacija i iskustvo na kraju mjeseca, opterećenje kao prosjek mjeseca, odsutnost na obuci u danima | smjene, raspored po danu, bolovanja po danu |
| odjel | gosti posluženi u mjesecu, kapacitet u mjesecu | jelovnik, nabava, zalihe |
| recenzija | napisana u mjesecu, o boravku u mjesecu | datum i sat |

**Brojke po hotelu i mjesecu:** 120 soba, 55–75 ljudi, 8–11 odjela, 60–100 recenzija.

| | račun | zaključak |
|---|---|---|
| obračun jedne runde (4 hotela) | ~300 soba-ažuriranja + ~300 ljudi + ~400 recenzija | zanemarivo (mikrosekunde) |
| balans 1 000 sjemena × 3 profila × 4 strategije × 4 mjeseca | ~6 mil. soba, ~3 mil. ljudi, ~4 mil. recenzija (bez teksta) | sekunde; cilj: `--check` ispod 60 s |
| `RoundResolved` u dnevniku | po hotelu ≈ 5 KB sobe + 8 KB ljudi + 2 KB odjeli + 5 KB recenzije (kodovi) ≈ 20 KB; 4 hotela ≈ 80 KB | za Postgres JSONB sitno; zato tekst recenzija nije u dnevniku |
| web | 120 pločica, 75 redaka | bez virtualnog popisa |

---

## 7 · Profesorov pogled (P9)

Danas profesorovog pogleda nema (DIZAJN §2: „DEMO korak 6"). v2 mu daje ono što prepoznaje s kolegija:

| pogled | sadržaj | kolegij |
|---|---|---|
| **USALI račun dobiti po hotelu**, hoteli jedan uz drugoga | odjelni prihodi i rezultati, neraspoređeni troškovi, GOP, GOPPAR, EBITDA | Accounting |
| **Budžet i ostvareno** | referentni hotel (C, zadani plan) jezgra već računa svaki mjesec (`indices.rs:153-190`); prikazan kao „budžet", s odstupanjem po stavci | Accounting (budžetiranje) |
| **Tablica pokazatelja prema konkurenciji** | popunjenost, ADR, RevPAR; **MPI / ARI / RGI** = vlastito ÷ prosjek ostalih hotela iste igre × 100 | Economics, E-Business |
| **Ljudi** | fluktuacija, udio troška rada, motivacija po odjelu, tko je otišao | Management (HRM) |
| **Ugled** | ocjena, broj recenzija, glavne teme, odgovori studenta | Marketing |
| **Odluke studenta** | nalozi po mjesecu uz učinak iz objašnjenja | sve |

MPI/ARI/RGI su pošteni i bez zajedničkog tržišta: hoteli ne dijele goste, ali uspoređuju se rezultati istog
mjeseca pod istim pravilima. Student vidi samo pokazatelje prema konkurenciji i ljestvicu, **ne** ljude ni recenzije
drugih hotela (pitanje 9).

---

## 8 · Rizici i što namjerno ne radimo

**Rizici**

| rizik | kako ga držimo |
|---|---|
| Balans se raspadne s novim slojem | ekvivalencijski test + `--check` svaki sloj; kalibracija je dio sloja, ne „poslije" |
| Profili s različitim odjelima nisu fer | §11.3 pravednost mjeri; ako ne prolazi, odjeli se izjednače po vrijednosti (ne po imenu) |
| Student se utopi u detaljima | „Zašto" pokazuje tri najveća uzroka; dubina je klik niže, nikad na prvom ekranu |
| `game` s dva modela postane zapetljan | jedno mjesto grananja (enum), testovi za oba modela na istim scenarijima |
| v1 se slučajno promijeni | zlatni test v1; `sim-hotel` dirati samo dodavanjem (sloj 0) |
| Pisanje sadržaja (imena, rečenice) traje | počinje odmah paralelno; Leon pregledava jezik po komadima |
| Profesor ospori brojke [PROC] | svaka brojka u katalogu s oznakom izvora; pitanja u PITANJA_ZA_PROFESORE |
| Rok prezentacije | linije reza (§10); svaki sloj sam za sebe je pokaziv |

**Namjerno ne radimo**

- **Simulaciju po danu** ni rezervacije po gostu — mjesec ostaje krug igre.
- **Slobodno izmišljen svijet AI-jem**: AI ne stvara ljude, sobe ni brojke; piše samo tekst istaknute recenzije, lika i
  mentora, nad brojkama jezgre.
- **Zajedničko tržište** među hotelima (VIZIJA §3: tek kad je osnovna petlja fer).
- **Potražnju po segmentu** gostiju, kanale prodaje s provizijama (OTA), raspored smjena, jelovnik i nabavu — dobri
  kandidati za kasnije, ali svaki je novi sloj.
- **Uređivanje entiteta mimo pravila** (npr. „dodaj sobu"): hotel se mijenja samo nalozima koje jezgra provjerava.

---

## 9 · Odluke za Leona

Pet pretpostavki iz ADR-026 ovdje se ne ponavlja (§11). Ovo su **nova** proizvodna pitanja; preporuka je prva.

1. **Isti ljudi u svakoj igri za isti profil?** Preporuka: **da** (fiksno sjeme profila) — pravedno i profesor ih može
   upoznati. Alternativa: druga imena svaku igru, ista statistika.
2. **Odjeli po profilu kao u §3.2?** Preporuka: A s wellnessom i rooftopom, B s barom uz bazen i najmom, C s malim
   wellnessom; konačno potvrđuje balans.
3. **Stare demo igre:** preporuka **nastavljive zauvijek na v1** (§4.2, cijena ~4–6 h u sloju 1). Alternativa: samo
   za čitanje — jeftinije za 2 h, ali profesorova demo igra ne bi mogla dalje.
4. **Otpuštanje stalnih radnika usred sezone:** preporuka **dopušteno uz otkazni rok i otpremninu** (iz kataloga, po
   uzoru na hrvatsko radno pravo) i pad motivacije kolega; sezonci bez troška na kraju mjeseca.
5. **Broj mjeseci za prezentaciju v2:** preporuka **sva četiri (VI–IX)**, jer se tek tada vidi povrat obnove i
   fluktuacija; demo ostaje na dva.
6. **Popunjenost i RevPAR nad svim sobama** (sobe izvan funkcije su „izgubljene noći")? Preporuka: **da**, jer tada
   RevPAR = ADR × popunjenost točno; pitati i profesora.
7. **GOP po USALI-ju:** osiguranje, najam i porez idu **ispod** GOP-a (danas su u `fixed_cents`, unutar GOP-a).
   Preporuka: **da**, uz isti cilj sezonske GOP marže 35–45 % (ADR-013 t. 5) nakon nove kalibracije, a EBITDA ispod;
   profesor potvrđuje pojas.
8. **Odgovor na recenziju:** preporuka **izbor pristupa bez AI-ja** u sloju 4; slobodan tekst s AI presudom kasnije
   (isti put kao slobodan odgovor na situaciju, ali košta).
9. **Vidi li student ljude i recenzije drugih hotela?** Preporuka: **ne** — samo MPI/ARI/RGI i ljestvicu; profesor
   vidi sve.
10. **Cijena po tipu sobe:** preporuka **jedna osnovna cijena + fiksne doplate iz kataloga** u sloju 3; da student
    sam postavlja cijenu po tipu (lekcija revenue managementa) ide kasnije.
11. **Imena i osobine:** preporuka hrvatska i regionalna imena, osobine samo o radu (pouzdan, ambiciozan, iskusan
    početnik…), bez nacionalnosti i stereotipa.
12. **Voditelj odjela** kao posebna uloga čija motivacija vuče tim (Management: vođenje)? Preporuka: **da, ali u
    drugom krugu sloja 2**, kad osnovni pojedinci rade.

---

## 10 · Redoslijed i linije reza

```text
sad (prije demoa)   ništa na main; sloj 0 na grani
nakon demoa         sloj 0 → main (samo prikaz)       ◄─ rez A
                    sloj 1 (temelj v2 + odjeli + USALI) ◄─ rez B: najmanji „wow" za profesora
                    sloj 2 ║ sloj 3 (paralelno)        ◄─ rez C: „BitLife" osjećaj za studenta
                    sloj 4                             ◄─ puna slika
                    P9 profesor raste uz svaki sloj
```

| ako vrijeme stane nakon… | što se pokazuje profesoru | zašto je dovoljno |
|---|---|---|
| **sloja 0** | ADR, RevPAR, GOPPAR, točka pokrića, tablica prema konkurenciji | govori jezikom Economicsa, ali „wow" je slab — hotel je i dalje brojke |
| **sloja 1** | „Dijelovi hotela" + USALI račun dobiti + budžet i ostvareno, hoteli jedan uz drugoga | profesor vidi **svoj** kolegij u igri; to je najmanji pravi „wow" |
| **sloja 1 + 2** | uz to profili ljudi, tko je otišao i zašto, zapošljavanje | student dobiva BitLife dubinu; Management + Accounting |
| **sloja 1 + 2 + 3** | uz to tlocrt soba i povrat obnove | E-Business (PMS) i investicije |

Ako je datum prezentacije blizu, a ostaje vremena za samo jedan veliki sloj nakon prvog: **sloj 2 prije sloja 3**
(ljudi su ono što je Leon prvo spomenuo i najviše mijenjaju dojam).

---

## 11 · Pretpostavke (🟨 čekaju Leonov pregled)

**Pet pretpostavki voditelja** (zapisane i u [ADR-026](../records/DECISIONS.md#adr-026)):

1. **ADR = prosječna cijena prodane sobe** (prihod soba / prodane noći), prikazan uz RevPAR, popunjenost i GOPPAR.
2. **Odjeli ovise o hotelu** — profili A/B/C imaju različite odjele; katalog određuje.
3. **Zaposlenicima student upravlja** (zaposli, otpusti, plaća, obuka, premještaj), ne samo gleda.
4. **Recenzije su pojedinačne, tekst iz banke rečenica po uzroku**, AI samo povremeno; student može odgovoriti.
5. **Slojevi redom 0 → 4**, svaki se gradi, dokaže i pregleda zasebno.

**Pretpostavke arhitekta u ovom dokumentu:**

6. v1 se zamrzava, v2 je novi paket; model se bira poljem `model` u snimku pravila (§4).
7. v2 izvješće uvijek nosi i v1 sažetak, pa postojeći ekrani, indeksi i mentor rade bez promjene (§3.8, §4.3).
8. Kocka raspoređuje, ne mijenja zbrojeve (šum s nultim zbrojem, §2 t. 4).
9. Nalozi su dio plana mjeseca; primjenjuju se pri obračunu, ne odmah (§4.3).
10. Tekst recenzija se ne sprema; slaže se pri čitanju iz banke u snimku (§3.5).
11. Sve brojke označene [PROC] su procjene za prvu kalibraciju, ne istina.
