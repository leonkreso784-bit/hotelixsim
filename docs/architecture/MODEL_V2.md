# HotelixSim — model v2 („put B"): hotel od stvari, ne od brojeva

> **Što je ovo:** arhitektura za put B: kako hotel u jezgri od skupa brojeva postaje skup **stvari s imenom i
> stanjem** (odjeli, ljudi, sobe, recenzije), sloj po sloj, a da demo i stare igre ostanu netaknuti.
> **Što** student dobiva po dodacima piše u [PROSIRENJA.md](../product/PROSIRENJA.md) (P1–P9). **Zašto** ovim putem piše u
> [ADR-026](../records/DECISIONS.md#adr-026). Ovdje je samo **kako**.
> **Status:** nacrt arhitekta. Leon je odgovorio na prvih 12 pitanja (§9), a nova pitanja i pretpostavke 🟨 čekaju
> njegov pregled. Sloj 0 je na `main`; sloj 1 je izgrađen na grani `model-v2/sloj1` (što je odstupilo od ovog nacrta:
> §8 „Sloj 1 — izgrađeno" i [ADR-029](../records/DECISIONS/ADR-029-sloj-1-pretpostavke-plana.md)); slojevi 2–4 nisu.
> Oznake brojki kao u [istraživanju](../research/HOTELSKI_POKAZATELJI.md): **[P]** provjereno · **[PROC]** procjena.

**Rječnik za ovaj dokument** (pojmovi koji se ponavljaju):

| pojam | značenje |
|---|---|
| **entitet** | „stvar" u igri s vlastitim imenom i stanjem: soba 214, konobar Marko, restoran |
| **agregat** | jedan broj za cijeli hotel (npr. moral = 55); danas je hotel samo od agregata |
| **izvedeno** | ne sprema se, nego se svaki put izračuna iz entiteta (moral hotela = prosjek motivacije ljudi) |
| **jezgra** | Rust kod koji računa krug igre (`crates/sim-hotel`); ekran samo prikazuje |
| **razdoblje** | jedan krug igre: **tjedan** ili **mjesec**, kako profesor odabere pri stvaranju igre (§4) |
| **dnevnik** | popis događaja igre u bazi; iz njega se igra uvijek može ponovno složiti (ARHITEKTURA §9.1) |
| **snimak pravila** | sve brojke iz `content/hotel/*.json` kopirane u igru pri stvaranju; igra ih nosi zauvijek |
| **sjeme / tok kocke** | broj iz kojeg „slučajnost" nastaje uvijek jednako; svaka svrha ima svoj tok |
| **USALI** | jedinstveni kontni plan za hotele (Uniform System of Accounts for the Lodging Industry): odjeli, njihovi troškovi, GOP |
| **rubrika** | popis kriterija po kojima AI ocjenjuje odgovor (danas za slobodan odgovor, ADR-019) |

---

## 0 · Sažetak na jednoj stranici

1. **v1 se zamrzava, v2 je novi paket.** Današnja jezgra (`sim-hotel`) ostaje točno kakva jest. Ona i dalje vrti demo
   i sve igre stvorene na njoj (Leon: stare igre ostaju igrive). Model v2 živi u novom paketu `sim-hotel-v2`, a igra
   po snimku pravila zna kojim se modelom računa (§6). Zato demo na `main` ne može puknuti zbog puta B.
2. **Krug igre je postavka: tjedan ili mjesec.** Profesor bira pri stvaranju igre. v2 računa **razdoblje** od N dana,
   a svaka brzina (trošenje, plaće, odlasci) zadana je po danu ili po vrsti razdoblja (§4). To ulazi u **temelj
   sloja 1**, jer je tamo jeftinije nego naknadno.
3. **v2 i dalje „govori v1".** Svako v2 razdoblje daje i današnji sažetak (sobe, stalni, sezonci, moral, ocjena…) kao
   **izvedeni** prikaz entiteta. Pet indeksa, mentor, „Zašto" i ljestvica rade dalje, a entiteti dodaju dubinu ispod
   njih (§3.9).
4. **Ljudi kao u Simsima:** osobine s dobrim i lošim stranama, traka za svaku vještinu, jezici s razinom, očekivana
   plaća koja raste s vještinom. Uz to **burza rada**, **agencija za strane radnike** i **smještaj radnika** (§5).
5. **Nacionalnost određuje samo činjenice:** jezike, naknadu agencije, potrebu za smještajem, trošak i rok dozvole.
   **Nikad** ne određuje vještinu, pouzdanost, osobnost ni kvalitetu (§2, t. 9).
6. **Razgovori s AI-jem gotovo uvijek:** pregovori o plaći, upozorenje, motivacijski razgovor, razgovor za posao, gost
   s pritužbom. AI glumi i ocjenjuje po rubrici, a **brojke računaju pravila** (§5.5). Svaka igra ima gornju granicu
   troška.
7. **Konkurencija „kao Booking":** student čita ocjene i recenzije drugih hotela, ali ne vidi njihove ljude (§7.1).
8. **Detalj samo uz uzrok**, a svaki sloj ima **ekvivalencijski test**: v2 podešen „kao v1" mora dati iste brojke.
9. **Slojevi:** 0 pokazatelji (S) → 1 temelj v2 + razdoblja + odjeli po USALI-ju (L+) → 2 ljudi (XL, u tri dijela) →
   3 sobe (M) → 4 recenzije i konkurencija (L). Ukupno ≈ 175–250 sati rada agenata (§8). Najmanji „wow" za profesora
   su slojevi 0 i 1 (§10).

---

## 1 · Gdje smo danas (model v1)

| što | kako je danas | gdje |
|---|---|---|
| Stanje hotela | 18 polja, sve agregati | `crates/sim-hotel/src/state.rs:24-43` |
| Sobe | jedan broj `rooms: u16` (120 u sva tri profila) | `state.rs:26`, `content/hotel/profiles.json` |
| Osoblje | dva broja: `permanent` i `seasonal`; odluka je „ciljani broj sezonaca" | `state.rs:27-28`, `plan.rs:24-33` |
| Ljudi i oprema | moral, iskustvo, oprema i zadovoljstvo kao bodovi 0–10 000 | `state.rs:31-34` |
| Ocjena portala | jedan broj; svaki mjesec pomak 25 % prema cilju iz zadovoljstva | `state.rs:35`, `resolve.rs:413-421` |
| Ostali prihod | ravnih 30 % prihoda od soba (hrana, piće, usluge) | `resolve.rs:266-269`, `catalog.json` `other_revenue_bp: 3000` |
| Troškovi | plaće + CPOR 22 € po sobi + održavanje + marketing + jedan `fixed_cents` (150 000 € za C) | `resolve.rs:270-296` |
| Sobe izvan funkcije | izvedeno iz opreme: `(40 − oprema) / 4` | `market.rs:61-63` |
| Kapacitet osoblja | jedan za cijeli hotel: zaposleni × dani × 2,4 | `market.rs:87` |
| Odlasci osoblja | očekivana vrijednost bez kocke; obračun nema nikakve slučajnosti | `resolve.rs:197-199`, potpis `resolve.rs:444-450` |
| Vrijeme | krug je uvijek kalendarski mjesec: `Month` (4 mjeseca), tablice `MonthTable<[T; 4]>`; `Month` se koristi u 42 Rust datoteke | `month.rs:12-19`, `month.rs:53` |
| Plan | bez plana za mjesec vrijedi **zadani** plan, a ne prošlomjesečni | `crates/game/src/game.rs:411-419` |
| Pokazatelji | računa se samo GOPPAR, po svim sobama × dani; **ADR, RevPAR i TRevPAR ne postoje** | `resolve.rs:309` |

---

## 2 · Načela

1. **Dubina samo uz uzrok.** Polje koje ne ulazi ni u jednu formulu ne ide u stanje. Ukras (hobi, rečenica „o meni")
   smije biti tekst izveden iz sjemena, ali ne i broj u stanju.
2. **Jedan izvor istine.** Svako „zašto" (ekran „Zašto", mentor, profesor) dolazi iz retka objašnjenja jezgre
   (`Line`, `crates/sim-hotel/src/explain.rs:59-64`) i iz `game::debrief_facts` (`crates/game/src/mentor.rs:755`),
   kao danas ([ADR-023](../records/DECISIONS.md#adr-023)). Ekran ništa ne računa (ARHITEKTURA §13).
3. **Determinizam ostaje** (ARHITEKTURA §7.2): cijeli brojevi (bodovi 0–10 000, novac u centima), samo uređene mape
   (`BTreeMap`), redoslijed po ID-u. Svaka nova slučajnost dobiva **svoj tok**:
   `Rng::derive(sjeme, [oznaka svrhe, razdoblje, slot hotela, …])` (`crates/engine/src/rng.rs:41`). Postojeći tokovi
   (`game:robot-plan`, `game:robot-answer`, `crates/game/src/command.rs:93,110`) se ne diraju.
4. **Šum s nultim zbrojem.** Kocka raspoređuje, a ne mijenja zbroj. Broj odlazaka i prosječnu ocjenu recenzija računa
   formula; kocka bira **tko** odlazi i **koja** soba ili gost. Balans tako ostaje stabilan, a priče su različite.
5. **Stare igre i iskreno obećanje (ARHITEKTURA §17).** Igra se zauvijek računa modelom i jezgrom s kojom je stvorena
   (§6.2).
6. **Ekvivalencija s v1.** v2 u „degeneriranom" katalogu daje iste brojke kao v1 za A, B i C kroz četiri mjeseca.
   Degenerirani katalog znači: mjesečna razdoblja, jedan odjel, jednaki ljudi s jednom vještinom, bez jezika i
   osobina, jednake sobe i bez šuma. Gdje v2 **namjerno** mijenja pravilo, test tu razliku navodi poimence.
7. **Demo je siguran.** Sve se radi na granama `model-v2/…`. `main` dobiva sloj 0 (samo prikaz) nakon demoa, a v2 tek
   uz Leonov OK, i to sa zadanim modelom v1 dok se izričito ne prebaci.
8. **Pet indeksa ostaje** (G R Z O F, centrirani na referentni hotel, ADR-013). Mijenja se samo odakle indeks čita
   ulaze.
9. **Nacionalnost određuje samo činjenice, nikad osobu.** Podrijetlo radnika smije odrediti samo **jezike koje govori,
   naknadu agencije, potrebu za smještajem, trošak radne dozvole i rok dolaska**. Ne smije odrediti vještinu,
   pouzdanost, osobnost, osobine, kvalitetu rada ni rizik odlaska. Te se vrijednosti za **svakoga** izvlače iz istih
   raspodjela, a tekstovi i osobine nemaju stereotipa. Ni plaća ne ovisi o podrijetlu: ista vještina u istoj ulozi
   znači istu tržišnu plaću. **Zašto:** igra je nastavni materijal fakulteta. Student mora naučiti procjenjivati
   **osobu** po onome što radi i zna, a ne po tome odakle dolazi. Pravilo se provjerava i testom: zamjena podrijetla
   dvaju kandidata ne smije promijeniti ništa osim navedenih činjenica.
10. **AI priča, pravila računaju.** AI glumi lik i ocjenjuje po rubrici (kvaliteta 1–5, kriterij ispunjen ili nije,
    ADR-019). Brojke iz te kvalitete računa jezgra, unutar granica iz kataloga. AI nikad ne zapisuje broj u stanje.
    Bez AI-ja svaki razgovor ima rezervu s ponuđenim odgovorima.

---

## 3 · Model entiteta

### 3.1 Pregled

```text
Hotel (v2)
├── odjeli          BTreeMap<KodOdjela, Odjel>            8–11 po hotelu (ovisi o profilu)
├── ljudi           BTreeMap<IdZaposlenika, Zaposlenik>   ~55–75 (više u vrhu sezone), uz voditelja svakog odjela
├── smještaj        Smjestaj                              kreveti za radnike: najam u gradu i/ili sobe hotela
├── narudžbe        Vec<NarudzbaAgencije>                 strani radnici na putu (dolaze za N dana)
├── sobe            BTreeMap<IdSobe, Soba>                120
├── recenzije       bazen (zbroj, broj) + recenzije zadnjeg razdoblja u izvješću
└── ostaje iz v1    kategorija, blagajna, odgođeni učinci, grupe, zadovoljstvo (ima pamćenje)
```

Burza rada (kandidati razdoblja) nije u stanju hotela. **Izvodi se** iz sjemena igre, razdoblja i uloge (§5.1).

### 3.2 Odjel (sloj 1)

Odjeli prate USALI. Katalog kaže koje **vrste** odjela postoje, a profil kaže koje hotel **ima**.

| USALI razred | odjeli u igri | prihod | izravni troškovi |
|---|---|---|---|
| Rooms (operativni) | Smještaj (recepcija + domaćinstvo) | prodane noći × cijena | plaće, potrošni materijal i rublje po prodanoj sobi |
| F&B (operativni) | Restoran · Lobby bar / kafić · Rooftop bar · Bar uz bazen | gosti × udio koji ga koristi × prosječni račun | plaće, trošak namirnica i pića (% prihoda) |
| Other Operated | Wellness i spa | isto, po tretmanu | plaće, materijal |
| Rentals / Misc | Parking, najam lokala | fiksno + po gostu | gotovo nikakav |
| Undistributed | Uprava (A&G) · Prodaja i marketing · Održavanje (POM) · Energija | — | plaće + odluke (marketing, održavanje) + stalni dio |
| ispod GOP-a | Fiksne naknade (osiguranje, najam, porez na imovinu) | — | stalni iznos → EBITDA (Leon: da, po USALI-ju) |

**Što student vidi u odjelu** (Leon: *„stisne wellness → stanje prostora, koliko je zaradio, koliko su gosti bili
zadovoljni"*):

- **stanje prostora** (0–100, troši se i održava);
- **prihod, trošak i rezultat** po razdoblju;
- **zadovoljstvo gostiju odjelom**, izvedeno iz kvalitete usluge, stanja prostora i cijene; isti broj ulazi u G;
- ljudi u odjelu, voditelj, opterećenje i kapacitet.

**Polja odjela.** Spremljeno: `kod`, `otvoren` (npr. rooftop zatvoren u lipnju), `stanje`, `razina_cijene`
(90–120 % kataloške). Izvedeno svako razdoblje: kapacitet, opterećenje, kvaliteta usluge, udio gostiju koji ga koriste,
zadovoljstvo gostiju odjelom, prihod, troškovi i rezultat.

**Kvaliteta usluge odjela** je današnja formula kvalitete (`resolve.rs:382-389`: 40 % iskustvo, 30 % moral, 30 %
manjak opterećenja). Razlika je u tome što koristi ljude i opterećenje **tog** odjela, a „iskustvo" postaje vještine
ljudi u njihovim ulogama (§3.4). Kvaliteta hotela je prosjek odjela, ponderiran brojem gostiju koje odjel dotakne.

**Odjeli po profilu** (Leon: da, otprilike ovako) [PROC]:

| profil | odjeli osim Smještaja i neraspoređenih | zašto |
|---|---|---|
| **A** „Ugledni, ali umorni" | Restoran · Lobby bar · Wellness i spa · Rooftop bar · Parking | puno sadržaja nosi ugled, ali troši ljude kojih je premalo |
| **B** „Profitabilni, ali zapušteni" | Restoran · Bar uz bazen (stara oprema) · Parking · Najam lokala | malo sadržaja, zdrava blagajna, stara oprema |
| **C** „Prosječni" | Restoran · Lobby bar · Mali wellness · Parking | referentni hotel |

**Sloj 1 je izgradio** (pravila: [ADR-032](../records/DECISIONS/ADR-032-sloj-1-odjeli-u-obracunu-i.md)): održavanje
svih odjela knjiži se u neraspoređeni odjel Održavanje (P9); naknada za upravljanje i I&T se ne modeliraju (P10).
Smještaj, Uprava, Prodaja i marketing i Održavanje imaju **normu osoblja** po kategoriji hotela (ljudi na 100 soba,
`staffing_tenths_per_100_rooms`), koju provjera sadržaja traži od početnog stanja profila. **Odlasci** se računaju za
hotel i dijele po odjelima razmjerno broju ljudi kroz sezonu (`departures`, `season_departures`). Ekran pokazuje
**ocjenu odjela** (prosjek stanja prostora, zadovoljstva gostiju odjela i usklađenosti ljudi s poslom; najam samo
stanje prostora) — [ADR-035](../records/DECISIONS/ADR-035-sucelje-odjela-odluke-u-detalju-odjela.md).

### 3.3 Zaposlenik (sloj 2)

Leon: *„kao Sims svijet"*. Svaka osoba ima osobnost, dobre i loše strane i vidljivu traku za svaku vještinu.

| polje | spremljeno / izvedeno | napomena |
|---|---|---|
| `id` | spremljeno | `Z-0001`…; novi dobiva sljedeći broj (`sljedeci_id` u stanju) |
| `ime`, `prezime` | spremljeno kao indeks u banku imena | hrvatska i regionalna imena; strana za radnike iz agencije |
| `izgled` | izveden iz (sjeme profila, `id`) | postojeći `AvatarLook` (`apps/web/src/avatar/`) |
| `uloga`, `odjel` | spremljeno | recepcioner, sobarica, kuhar, konobar, šanker, terapeut, tehničar, **voditelj odjela** |
| `ugovor` | spremljeno | stalni · sezonski do datuma · preko agencije |
| `vjestine` | spremljeno, 0–10 000 po vještini | 3–4 vještine uloge (§3.4); na ekranu traka |
| `osobine` | spremljeno, 2–3 koda | 1–2 dobre i 0–1 loša; neke skrivene dok se ne otkriju (§5.1) |
| `jezici` | spremljeno: (jezik, razina 0–4) | 0 nikako · 1 osnovno · 2 komunikativno · 3 tečno · 4 materinji |
| `motivacija` | spremljeno, 0–10 000 | današnji moral, ali po osobi |
| `placa` | spremljeno, u centima po mjesecu | ugovorena; mijenja se povišicom ili pregovorom |
| `ocekivana_placa` | izvedeno | tržišna plaća uloge × faktor vještine (§5.2) |
| `podrijetlo` | spremljeno | lokalno · iz druge regije · iz inozemstva (agencija); samo za činjenice (§2, t. 9) |
| `treba_smjestaj` | spremljeno | prebivalište je daleko (Osijek ili inozemstvo); činjenica, ne osobina |
| `staz_dana`, `obuke` | spremljeno | povijest za profil i rast vještina |
| `rizik_odlaska` | izvedeno | današnja `turnover_bp` (`state.rs:76-80`) iz **njegove** motivacije, osobina i razlike do očekivane plaće |
| `opterecenje`, `trosak_razdoblja` | izvedeno | iz odjela, plaće i smještaja |

**Isti ljudi za isti profil** (Leon: da). Početne ljude generira jezgra iz specifikacije profila i **fiksnog sjemena
profila** u snimku. Prosjeci se pritom ispravljaju tako da točno pogode brojke profila (npr. prosječna motivacija
C = 55,00). Svi studenti s hotelom A imaju iste ljude.

**Profil C u lipnju** [PROC]: Recepcija 7 · Domaćinstvo 18 · Kuhinja 10 · Posluga restorana 9 · Bar 5 · Wellness 3 ·
Održavanje 3 · Uprava i prodaja 3 = **58** (danas 30 stalnih + 28 sezonaca). Od njih su 5–6 voditelji odjela.

**Voditelj odjela** (Leon: svakako) ulazi u **prvi prolaz** sloja 2. To je jedna uloga i dvije formule, a bez nje
lekcija o vođenju (Management) nema nositelja. Vještina „vođenje" voditelja pomiče cilj motivacije cijelog tima
(±5 bodova [PROC]) i ubrzava rast vještina tima. Odjel bez voditelja trpi kaznu.

### 3.4 Vještine, osobine, jezici (katalog)

**Vještine po ulozi** (3–4, sve 0–100 na ekranu) [PROC]:

| uloga | vještine | gdje djeluju |
|---|---|---|
| recepcioner | rad s gostima · sustav (PMS) · prodaja (upsell) · jezici | kvaliteta Smještaja, prihod od nadoplata |
| sobarica | temeljitost · brzina | kvaliteta i kapacitet domaćinstva, teme „čistoća" |
| kuhar | kvaliteta jela · brzina · higijena | kvaliteta restorana, trošak namirnica (otpad) |
| konobar / šanker | brzina usluge · rad s gostima · poznavanje ponude · jezici | kvaliteta F&B, prosječni račun |
| terapeut | tretmani · prodaja | kvaliteta i prihod wellnessa |
| tehničar | popravci · prevencija | trošenje soba i odjela, kvarovi |
| voditelj odjela | vođenje · organizacija + vještine struke | motivacija i rast tima, kapacitet odjela |

**Rast vještine** = iskustvo u ulozi (po danu rada) + obuke, puta brzina učenja. Brzina učenja je osobina pojedinca.
Početnik kreće nisko i jeftino, a uz obuku i rad postaje sve bolji (Leon). „Iskustvo" iz v1 je prosjek vještina uloge.

**Osobine** (kao BitLife: dobre i loše strane, svaka s konkretnim učinkom iz kataloga) [PROC]:

| dobre | učinak | loše | učinak |
|---|---|---|---|
| Pouzdan | rizik odlaska −30 % | Nemiran | rizik odlaska +40 % |
| Brzo uči | rast vještina +50 % | Kasni | kapacitet −5 %, povremeni incident |
| Ljubazan | rad s gostima +10 bodova u ocjeni gosta | Svadljiv | motivacija kolega u odjelu −3 dok je tu |
| Marljiv | kapacitet +5 % | Nestrpljiv s gostima | teme pritužbi „neljubazno osoblje" |
| Rođeni vođa | vođenje +15 (bitno za voditelja) | Skup | traži povišicu ranije |
| Timski igrač | motivacija kolega +2 | Neorganiziran | organizacija −10 |
| Omiljen | ako ode ili bude otpušten, tim to osjeti | | |

Osobine se izvlače **iz iste raspodjele za svakoga** (§2, t. 9). Tekst „osobnosti" na profilu slaže se iz banke
rečenica po osobinama.

**Jezici.** Katalog ima **jezični sastav gostiju po mjesecu** (npr. njemački, engleski, slovenski, talijanski,
hrvatski, poljski; izvor DZS i turističke zajednice, pitanje N8). **Pokrivenost** odjela je udio gostiju koje odjel
može uslužiti na njihovom jeziku barem na razini „komunikativno". Ona pomiče kvalitetu recepcije i F&B-a i daje teme
recenzija („nitko nije govorio njemački"). Jezik se uči obukom (npr. tečaj hrvatskog ili njemačkog: +1 razina kroz
nekoliko tjedana).

### 3.5 Soba (sloj 3)

| polje | spremljeno / izvedeno | napomena |
|---|---|---|
| `id` | spremljeno, iz tlocrta | kat × 100 + redni broj (`214`) |
| `tip` | spremljeno | standard · superior s pogledom na more · obiteljska · apartman [PROC 70/30/12/8] |
| `stanje` | spremljeno, 0–10 000 | troši se po prodanoj noći te sobe; održavanje i obnova ga vraćaju |
| `status` | spremljeno | u funkciji · izvan funkcije (uzrok) · u obnovi (do datuma) · **smještaj radnika** (§5.4) |
| `obnovljena` | spremljeno | datum obnove; donosi doplatu i bolje recenzije |
| `zadnji_kvar` | spremljeno | tema (klima, kupaonica, namještaj…) |
| `noci`, `prihod` | u izvješću razdoblja | iz raspodjele prodanih noći |

Cijena: jedna osnovna cijena i fiksne doplate po tipu iz kataloga (Leon: da).

### 3.6 Recenzija i segment gosta (sloj 4)

**Recenzija** se sprema **kao kodovi, ne kao tekst**: `id`, razdoblje, segment, jezik gosta, soba, odjeli koje je gost
koristio, ocjena (1,0–10,0), 1–3 teme s predznakom (`dorucak.spora_usluga −`) i odgovor studenta (izbor pristupa).
Tekst se slaže pri čitanju iz **banke rečenica**, izborom iz (sjeme, `id`), pa je uvijek isti, a u dnevniku nema
teksta. AI tekst smije imati samo istaknuta recenzija (§8, sloj 4).

**Segment gosta** (par, obitelj, poslovni, grupa, wellness gost) je podatak iz kataloga: udio po mjesecu i što mu je
važno. Segmenti boje teme recenzija; **ne** uvode zasebnu potražnju (§7.3).

### 3.7 ID-jevi — uvijek iz pravila, nikad slučajni

| entitet | ID | zašto je stabilan |
|---|---|---|
| odjel | kod iz kataloga (`restoran`) | katalog je u snimku |
| soba | broj iz tlocrta (`214`) | tlocrt je u snimku |
| zaposlenik | `Z-` + redni broj u hotelu | brojač je u stanju; naloge jezgra primjenjuje redom |
| kandidat na burzi | (razdoblje, uloga, redni broj) | izveden iz sjemena igre, isti za sve hotele (§5.1) |
| narudžba agencije | (razdoblje, redni broj naloga) | redoslijed naloga u planu |
| recenzija | (razdoblje, redni broj u hotelu) | redoslijed iz raspodjele gostiju |

### 3.8 Kako agregati postaju izvedeni

| polje v1 (`state.rs`) | u v2 | spremljeno? |
|---|---|---|
| `rooms` | broj soba za prodaju (bez soba za radnike) | izvedeno |
| `permanent`, `seasonal` | brojanje ljudi po ugovoru (agencijski se broje kao sezonci) | izvedeno |
| `wage_pct`, `extra_wage_cents` | plaća svake osobe; „svima +x %" ostaje kao skupna radnja | izvedeno (prikaz) |
| `morale_pts` | prosjek motivacije po glavi | izvedeno |
| `experience_pts` | prosjek vještina uloge po glavi | izvedeno |
| `equipment_pts` | 70 % stanje soba + 30 % stanje odjela [PROC] | izvedeno |
| `satisfaction_pts` | anketa s pamćenjem (`resolve.rs:405`); cilj iz zadovoljstva odjelima i soba | spremljeno |
| `rating` | bazen recenzija | spremljeno kao bazen |
| `load_hundredths`, `turnover_bp` | po odjelu / stvarni odlasci (preračunato na 30 dana, §4.4) | u izvješću |
| `fixed_cents` | neraspoređeni troškovi + fiksne naknade (ispod GOP-a) | katalog |
| `cash`, `pending`, `groups`, `renovated`, `category` | isto (grupe po datumu, ne po mjesecu) | spremljeno |

U sloju 1 (sezonci još bez pojedinaca): korak sezonaca vrijedi za **zbroj pomaka po odjelima** (Σ |cilj − trenutno| ≤
`seasonal_step`), a najam i puštanje broje se po odjelu; kapacitet tržišta računa samo osoblje Smještaja; opterećenje
hotela je prosjek odjela ponderiran zaposlenima, a kvaliteta i cilj zadovoljstva prosjek ponderiran gostima
([ADR-032](../records/DECISIONS/ADR-032-sloj-1-odjeli-u-obracunu-i.md) t. 2 i 4).

### 3.9 Pet indeksa na novom modelu

Formule iz DEMO §4 i `crates/sim-hotel/src/indices.rs:82-128` ostaju iste.

| indeks | formula (ne mijenja se) | ulaz u v2 | što student vidi ispod |
|---|---|---|---|
| **G** gosti | zadovoljstvo | iz zadovoljstva odjelima, stanja soba i vrijednosti za novac | koji odjel ili tip sobe vuče dolje |
| **R** recenzije | (ocjena − 7,0) / 2,8 | bazen recenzija | recenzije i teme |
| **Z** zaposlenici | 60 % moral + 40 % zadržavanje | motivacija ljudi; odlasci preračunati na 30 dana | tko je otišao i zašto, tko je u riziku |
| **O** operativa | 45 % oprema + 30 % iskustvo + 25 % opterećenje | stanje soba i odjela, vještine, opterećenje po odjelu | sobe izvan funkcije, preopterećen odjel |
| **F** financije | 70 % GOPPAR prema referentnom + 30 % likvidnost | GOP po USALI-ju; likvidnost preračunata na 30 dana | odjelni rezultati |

`index_drivers` (`indices.rs:71-80`) dobiva pokretače odjela, ljudi i soba, pa „što je pokrenulo indeks" ide do
pojedinca: „Z je pao 4 boda: otišli su Marko (restoran) i Ivana (domaćinstvo), oboje s motivacijom ispod 35".

---

## 4 · Razdoblje: tjedan ili mjesec

Leon bi radije igru po **tjednima**, a **profesor bira** pri stvaranju igre. Time pada ranije pravilo „krug je uvijek
mjesec". Novo pravilo: **v2 računa razdoblje od N dana**. Mjesec je samo jedna od dvije vrste razdoblja.

### 4.1 Kalendar

- Snimak pravila dobiva postavku `razdoblje: tjedan | mjesec`. Iz nje se pri stvaranju igre gradi popis razdoblja
  (početni dan, broj dana) i sprema se u `GameCreated`.
- **Mjesec:** 4 razdoblja (30/31/31/30 dana) za punu sezonu; demo i dalje ima dva.
- **Tjedan:** sezona 1. 6. – 30. 9. ima 122 dana = 16 tjedana od 7 dana + **zadnje razdoblje od 10 dana**
  (21.–30. 9.) [prijedlog, pitanje N3]. Ukupno 17 krugova.
- **Sezonske tablice ostaju po mjesecu** u sadržaju (`MonthTable`, `month.rs:53`). Vrijednost za razdoblje je prosjek
  po danima mjeseci koje dotiče: tjedan 28. 6. – 4. 7. = 3 dana lipnja + 4 dana srpnja. Sadržaj se zato ne mijenja,
  a mjesečno razdoblje daje točno današnju vrijednost.

### 4.2 Sve brzine po danu ili po vrsti razdoblja

| veličina | v1 (po mjesecu) | v2 |
|---|---|---|
| prodaja, kapacitet, trošenje sobe, CPOR | već po danu / noći | isto, puta dani razdoblja |
| plaće, fiksni troškovi, smještaj radnika | mjesečni iznos | mjesečni iznos × dani / dani mjeseca (zaokruženo na cent, ostatak ide u zadnji dan mjeseca) |
| marketing, održavanje (odluke) | iznos za mjesec | iznos za razdoblje; granice iz kataloga po vrsti razdoblja |
| pomak motivacije (30 %), zadovoljstva (50 %), ocjene (25 %) | jedan korak | **koeficijent po vrsti razdoblja iz kataloga** (sloj 1, tjedan: moral 11 %, zadovoljstvo 14 %, ocjena 10 % [PROC], [ADR-033](../records/DECISIONS/ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md)); mjesec = današnji |
| odlasci | rizik po mjesecu | sloj 1: razmjerni dio mjesečnih odlazaka, `prorate(div_round(staff·turnover, 10 000), p)` (nacrt je imao „× dani / 30") |
| obuka | učinak idući mjesec | tečaj traje N dana (osoba je toliko odsutna); učinak nakon završetka |
| obnova | jedan mjesec | 2–4 tjedna po katu; sobe izvan prodaje točno toliko dana |
| recenzije | ~80 po mjesecu | ~20 po tjednu (iz istih boravaka) |

Koeficijente pomaka nije moguće točno preračunati cijelim brojevima (to bi tražilo potenciranje). Zato ih katalog
zadaje izričito i balans ih kalibrira tako da **tjedna i mjesečna igra s istim odlukama daju sličan kraj sezone**.
To postaje nova provjera balansa.

Sloj 1 je dodao **mrtvu zonu koraka:** u razdoblju koje nije cijeli mjesec, kad je razmak do cilja ≠ 0 a zaokruženi
korak 0, korak je ±1 jedinica prema cilju (`resolve::step_toward`); mjesečni krug je nepromijenjen. Tjedni korak
sezonaca je ±3. Pravila: [ADR-031](../records/DECISIONS/ADR-031-sloj-1-tjedni-krug-u-jezgri.md), tablica „Pravila
razdoblja" u [planu Sloja 1](../plan/2026-10-10-v2-sloj1.md).

### 4.3 Što to znači za igru

- **Situacije** danas imaju mjesec (`crates/sim-hotel/src/mechanic.rs:51`). U v2 dobivaju i neobavezan tjedan u
  mjesecu (1–4); u mjesečnoj igri tjedan se zanemaruje. Prijedlog je 1–2 situacije tjedno. *Sloj 1 to još nema:*
  situacija mjeseca dolazi u krugu koji sadrži 1. dan mjeseca i obračunava se s tim mjesecom (P13, ADR-031 t. 3).
- **Broj situacija po krugu (B2a, 15.10.):** postavka igre (`GameCreated.situation_counts`), tjedno 3/4/5, **mjesečno 4/5/6**
  (ispod 75 % / 75–89 % / od 90 % očekivane popunjenosti), dopušteno 3–8; mjesečni krug se izvlači kao jedan blok, isti predložak
  najviše jednom u krugu ([ADR-042](../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 12–14, pretpostavke
  [ADR-051](../records/DECISIONS/ADR-051-pretpostavke-faze-b-situacije-i-mjerenje.md)).
- **Učinak situacije na prostor (B1a):** hitni popravak diže stanje odjela svoje situacije (`economy.repair_urgent_condition_pts`,
  privremeno 100 = +1 bod; „oko +10" odlučuje Leon u L1-13, [ADR-050](../records/DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md)).
  Ekran odjela popravak još prikazuje kao „održavanje" (C5).
- **Grupe** (`state.rs:9-13`) se vežu za raspon datuma, ne za mjesec.
- **Plan se prenosi.** Danas bez plana vrijedi zadani plan (`game.rs:411-419`). Uz 17 krugova to bi bilo nepodnošljivo,
  pa u v2 **vrijedi prošli plan dok ga student ne promijeni**, za obje vrste razdoblja. Jednokratni nalozi
  (zaposli, obnovi) se ne ponavljaju. Sloj 1: krug bez plana uzima plan najbližeg ranijeg kruga koji ga ima, stisnut u
  granice novog kruga (ADR-031 t. 4).
- **Tjedni krug studenta** ima manje odluka po krugu: pogledaj što se dogodilo (3 najveća uzroka), odgovori na 1–2
  situacije ili razgovora, po želji prilagodi cijenu, ljude i recenzije, pa „Zatvori tjedan". Mjesečna lekcija
  (USALI izvještaj, mentor u četiri dijela) dolazi **na kraju svakog kalendarskog mjeseca**, i u tjednoj igri.
- **Indeksi i ljestvica** se računaju na kraju svakog razdoblja. Referentni hotel se vrti po istom kalendaru
  (`RefTable::compute`, `indices.rs:153-190`, danas po mjesecu). Profesor vidi i mjesečni sažetak.
- **Pregovori o plaći** ne mogu se ponavljati svaki tjedan: ista osoba traži najviše jednom u 4 tjedna.

### 4.4 Zamke koje treba riješiti u temelju

- **Likvidnost** dijeli blagajnu s troškovima razdoblja (`indices.rs` `raw_indices`, likvidnost). U tjednu bi
  likvidnost skočila oko 4 puta. Nazivnik se zato preračunava na mjesec — u sloju 1 na dane kalendarskog mjeseca u
  kojem krug počinje (`D₀`), ne na 30 dana, jer bi „/30" u srpnju i kolovozu prekršio ekvivalenciju s v1 (P16; isto
  kamata).
- **Zadržavanje (Z)** koristi mjesečnu stopu odlazaka (DEMO §4: „8 % mjesečno → 0"). U tjednoj igri se koristi stopa
  zadnjih 30 dana, ne tjedna.
- **Krug je danas `Month` posvuda:** u `game` (`closed`, `resolved`: `game.rs:78-79`), u događajima
  (`GameCreated.months`, `crates/game/src/event.rs:22`) i u API putanjama (`/rounds/{month}`,
  `crates/api/src/games.rs:37`; `/plans/{month}`, `crates/api/src/hotels.rs:85`). v2 uvodi **ključ kruga** kao kod
  (`jun` za mjesec, `t07` za tjedan). Za v1 ostaje `Month`, a putanje primaju oba koda.

### 4.5 Cijena i preporuka

Tjedni krug **ide u temelj sloja 1**. Ta podjela ionako dira sva mjesta na kojima je `Month` ključ kruga (§6.4), pa
razdoblje tamo košta **+12–18 h**. Kad bi se dodalo nakon slojeva 2–3, svaka brzina ljudi, soba i recenzija morala bi
se ponovno pisati i kalibrirati, procjena 30–40 h. **Rizici:** dvostruka kalibracija (balans vrti obje vrste
razdoblja), mali brojevi po tjednu (više zaokruživanja, zato koeficijenti iz kataloga i ostatak u zadnji dan) i
17 krugova po igri (AI trošak i vrijeme profesora, §5.5). v1 ostaje samo mjesečni, a v2 mjesečni je put
ekvivalencije s v1.

---

## 5 · Ljudi: burza rada, agencija, smještaj, razgovori

### 5.1 Burza rada

- Svako razdoblje burza nudi kandidate po ulozi [PROC 2–4 po ulozi; više u lipnju, malo lokalnih u srpnju i kolovozu].
  Kandidat ima isti oblik kao zaposlenik: vještine, osobine, jezike, traženu plaću, datum od kojeg može početi i
  potrebu za smještajem.
- **Vrste kandidata:** *početnik* (niske vještine, niska tražena plaća, često „Brzo uči"), *iskusni profesionalac*
  (visoke vještine, traži više) i sve između. Sve su to vrijednosti pojedinca, ne podrijetla.
- **Ista burza za sve hotele u igri** (izvedena iz sjemena igre, ne iz slota). Zaposlenje u jednom hotelu ne uklanja
  kandidata iz drugih, jer se hoteli još ne natječu za ljude (VIZIJA §3; pitanje N2).
- **Skrivena osobina:** prije zaposlenja vide se vještine, jezici i jedna osobina. Druga se otkrije **razgovorom za
  posao** (§5.5) ili nakon prva 4 tjedna rada. Tako razgovor ima vrijednost.

### 5.2 Plaća koja raste s vještinom

```text
očekivana_plaća = tržišna_plaća_uloge × (85 % + 30 % × prosjek_vještina / 100)      [PROC]
ako je očekivana plaća barem 8 % veća od stvarne → osoba traži povišicu (najviše jednom u 4 tjedna)
   prihvati  → nova plaća, motivacija +
   odbij     → motivacija −, rizik odlaska raste dok se razlika ne zatvori
   pregovaraj (AI razgovor) → kvaliteta 1–5 pomiče prag prihvaćanja i učinak na motivaciju, unutar granica kataloga
```

To je Leonova petlja: obučiš početnika, on postane izvrstan i tada traži više. Ulaganje u ljude ima cijenu.

### 5.3 Otpuštanje

- **Stalni:** otkazni rok (plaća se, a osoba radi s nižom motivacijom) i otpremnina iz kataloga. **Sezonac:** kraj
  ugovora bez troška na kraju razdoblja.
- **Kolege ne gube motivaciju svaki put** (Leon). Gube je samo u iznimkama: otpušten je netko s osobinom „Omiljen" ili
  s dugim stažem, ili je u istom razdoblju otpušteno više od 20 % odjela [PROC]. Inače nema učinka na tim.

### 5.4 Agencija za strane radnike i smještaj

- **Nalog:** „5 radnika za domaćinstvo". Agencija ih dovodi nakon **roka** (npr. 3 tjedna; iz kataloga, po zemlji).
  Naplaćuje se **naknada agencije** po osobi i **trošak radne dozvole i administracije**.
- Agencijski radnik je zaposlenik kao svaki drugi: vještine, osobine i motivacija izvlače se iz **istih raspodjela**.
  Podrijetlo određuje samo **jezike** (npr. materinji jezik zemlje + engleski na razini izvučenoj za tu osobu;
  hrvatski u početku 0–1), **rok**, **naknadu**, **dozvolu** i **potrebu za smještajem** (§2, t. 9).
- **Plaća je ista za istu vještinu.** To je prijedlog arhitekta (pitanje N1). Leon je rekao „jeftinija radna snaga",
  ali razlika u plaći po podrijetlu bila bi upravo pravilo koje ovaj dokument zabranjuje (a u Hrvatskoj i protuzakonita).
  Agencija zato nije jeftinija po glavi. Ona daje **ljude kad ih na lokalnoj burzi nema** (u srpnju i kolovozu), uz
  dodatne troškove. Student vidi ukupni trošak po osobi: plaća + naknada + dozvola + smještaj.
  *(Leonova želja 16.10. o nižoj plaći i težoj obuci radnika iz Azije sukobljava se s ovim: vidi stavku o stranim radnicima iz Azije u [BACKLOG](../records/BACKLOG.md); odluka ovdje ostaje dok Leon ne odluči.)*
- **Smještaj radnika** treba svakome s `treba_smjestaj`, bilo da je iz inozemstva ili iz Slavonije. Dvije mogućnosti:
  1. **najam kreveta u gradu**: trošak po krevetu po danu; više kreveta je skuplje u vrhu sezone [PROC];
  2. **sobe hotela za radnike**: 2 radnika po sobi. Nema najma, ali soba nije za prodaju. To je **izgubljeni prihod**
     koji ekran pokazuje („soba 108 bi u kolovozu donijela ~4 900 €").
- **Ekran agencije** po osobi i po skupini pokazuje koliko donose (kapacitet, kvaliteta odjela, prihod koji bez njih ne
  bi bio prodan) i koliko koštaju (plaća, naknada, smještaj). Lekcija je trošak radne snage i ograničenja kapaciteta.

### 5.5 Razgovori s AI-jem

Leon: odgovor na recenziju je **izbor bez AI-ja**, ali razgovori s ljudima i s gostima u situacijama trebaju biti
**s AI-jem gotovo uvijek** (slobodan tekst, zabavno). Postojeći put se koristi i dalje: lik na Haikuu, presuda po
rubrici na Sonnetu, rezerva bez AI-ja ([ADR-019](../records/DECISIONS.md#adr-019),
[ADR-024](../records/DECISIONS.md#adr-024)), najviše 6 poruka studenta (`crates/game/src/talk.rs:14`).

| razgovor | kada | što AI dobiva | rubrika ocjenjuje | što pravila iz kvalitete računaju |
|---|---|---|---|---|
| **Pregovori o plaći** | osoba traži povišicu (§5.2) ili je student pozove | vještine, osobine, plaća, očekivana plaća, staž | uvažavanje, argumenti, jasna ponuda, poštenje | student upisuje **broj** (nova plaća); prihvaćanje = ponuda ≥ očekivana − tolerancija(q); motivacija Δ ograničena |
| **Upozorenje** | incident (osobina „Kasni", pritužba gosta na osobu) | incident, povijest | činjenice, bez ponižavanja, jasno očekivanje | ponavljanje incidenta −/+, motivacija Δ; loš razgovor kod „Omiljen" dira tim |
| **Motivacijski razgovor** | student pozove (ograničeno vrijeme voditelja) | motivacija i njezini uzroci iz jezgre | slušanje, konkretna mjera, praćenje | cilj motivacije +0…+5 kroz 4 tjedna; mjera koju student obeća (npr. obuka) postaje nalog |
| **Razgovor za posao** | prije zaposlenja kandidata | kandidatove vještine i skrivena osobina | pitanja o radu, jasna očekivanja | otkriva skrivenu osobinu (uvijek); kvaliteta pomiče traženu plaću ±3 % |
| **Izlazni razgovor** | osoba odlazi | pravi uzrok odlaska iz jezgre | slušanje | ništa u brojkama; student sazna uzrok (mentor ga koristi) |
| **Gost s pritužbom** (situacija) | iz situacija, kao danas | soba, odjel, tema pritužbe iz jezgre | kao danas (ADR-019) | kao danas: `ServiceRecovery` (DEMO §6.2) |

**AI nikad ne zapisuje broj.** Lik zna samo činjenice koje mu jezgra da. Presuda daje kvalitetu 1–5 i kriterije, a
učinak je formula kataloga s granicama (DEMO §6.2). Ako AI ne radi, isti razgovor nudi 3–4 ponuđena odgovora.

**Vrijeme voditelja** ograničava broj razgovora koje student pokreće: 3 po tjednu ili 8 po mjesecu [PROC, pitanje N6].
Razgovori koje pokrene osoba (traži povišicu) ili situacija ne troše to vrijeme.

**Trošak** (cijene iz ADR-024; replika lika ≈ 0,0001 $, presuda ≈ 0,008 $; mentor na Sonnetu ≈ 0,01–0,02 $ [PROC]):

| | mjesečna igra (4 kruga) | tjedna igra (17 krugova) |
|---|---|---|
| razgovori (situacije + ljudi) | ~20 × (6 × 0,0001 + 0,008) ≈ 0,18 $ | ~50 × 0,0086 ≈ 0,45 $ |
| mentor | 4 × ~0,015 ≈ 0,06 $ | 17 × ~0,015 ≈ 0,26 $ (ili samo na kraju mjeseca: 0,06 $) |
| istaknute recenzije (Haiku) | zanemarivo | zanemarivo |
| **po studentu po igri** | **≈ 0,25 $** | **≈ 0,45–0,75 $** |

**Gornja granica:** 2 $ po studentu po igri (`ai_usage`, ARHITEKTURA §9.3) [prijedlog]. Iznad toga razgovori prelaze
na ponuđene odgovore, a profesor to vidi. Za grupu od 30 studenata to je najviše 60 $ po igri.

---

## 6 · Kako se v2 uklapa u igru

### 6.1 Paketi

```text
engine ◄── sim-hotel (v1, ZAMRZNUT) ◄── sim-hotel-v2 (koristi Index, IndexSet, score, calc, MonthTable iz v1)
               ▲                              ▲
               └──────── game (bira model po snimku) ◄── content (čita i nove datoteke)
```

`sim-hotel-v2` počinje kao kopija v1 na početku sloja 1, uz razdoblje umjesto mjeseca. Sloj 0 ne mijenja brojke, pa
ide u v1. Pravila ovisnosti (ARHITEKTURA §6.2) i zabrane (`crates/sim-hotel/src/lib.rs:2-7`) vrijede i za v2.

**Kako je izgrađeno (sloj 1).** Obračun v2 je podijeljen po fazama u `crates/sim-hotel-v2/src/resolve/` (`mod.rs`
redoslijed faza i zajednički tipovi; `situations.rs`, `people.rs`, `departments.rs`, `money.rs`, `guests.rs`);
`PeriodReport` nosi `accounts: PeriodAccounts`, a `money.rs` iz njih slaže v1 sažetak novca. Sve grananje v1/v2 je u
`crates/game/src/model.rs` (`Model`, `CoreState`, `CorePlan`, `CoreRefs`, `RoundInfo`); `app` i `api` zovu samo te
tipove (test `dispatch_lives_in_one_place`). U `game` su još `departments.rs` (odjeli, račun dobiti, budžet),
`costs.rs` (cijene odluka), `season.rs` (kraj sezone) i `season_mentor.rs`.

### 6.2 Stare igre (Leon: ostaju igrive na v1)

Danas je `GameCreated.core_version` = `sim_hotel::CORE_VERSION` = `0.1.0` (`lib.rs:50`, `Cargo.toml:3`), a obračun se
odbija ako se verzija ne slaže (`crates/game/src/resolve.rs:111-117`; web dobiva grešku 500 preko
`crates/app/src/error.rs:170`). Uz to `fold` pri svakom čitanju igre **ponovno računa** referentnu tablicu i početne
indekse trenutnim kodom (`game.rs:131-141`). Svaka promjena formula u `sim-hotel` zato bi tiho promijenila napredak
**starih** igara. Odluka:

1. Snimak dobiva polja `model` (`"v1"` kad ga nema) i `razdoblje`, s `serde(default, skip_serializing_if)`. Zato stari
   otisak (`legacy_fingerprint`, `crates/content/src/lib.rs:607`) ostaje isti. *Sloj 1:* vrsta kruga i kalendar
   krugova su u postavkama igre (`GameCreated.rounds`, nikad se ne računaju ponovno), a snimak nosi koeficijente obiju
   vrsta (P15).
2. **v1 igre** se zauvijek čitaju, nastavljaju i obračunavaju v1 kodom, s istim brojkama. `sim-hotel` ostaje `0.1.0`.
   „Zlatni test" (ista sezona sa sjemenom 7 → isti otisak izvješća) ruši CI ako se ijedan v1 broj pomakne.
3. **v2 igre** provjeravaju svoju verziju (od `2.0.0`, raste po sloju). Probna v2 igra iz starijeg sloja dobiva poruku
   „igra je stvorena starijom verzijom", a ne grešku 500.
4. v1 se ne briše dok postoji v1 igra.

### 6.3 Dnevnik i događaji (samo dodavanje)

| što | promjena | zašto ovako |
|---|---|---|
| `GameCreated` | snimak s novim datotekama, `model`, `razdoblje`; popis razdoblja | puni snimak pravila ostaje u igri (ARHITEKTURA §7.2) |
| `PlanSet.plan` | `nalozi: Vec<Nalog>` (zaposli, otpusti, plaća, obuka, premjesti, narudžba agencije, smještaj, razina cijene odjela, otvori/zatvori odjel, obnova, soba izvan funkcije, odgovor na recenziju) | „zadnji plan vrijedi" ostaje jedino pravilo; roboti pišu isto |
| razgovori s ljudima | isti događaji kao razgovor u situaciji (poruka, odgovor, presuda), s ključem razgovora umjesto koda situacije | jedan put za AI, isti red poslova i rok presude |
| `RoundResolved.hotels[]` | `detalj` (v2): entiteti nakon razdoblja, odjelni račun, noći po sobi, odlasci i dolasci s razlozima, recenzije | `report` (v1 oblik) i dalje nosi sažetak |
| `SCHEMA_VER` (`event.rs:9`) | ostaje 1 dok su polja dodatna s `serde(default)` | obrazac kao `free_text` (`event.rs:100-102`) |

v2 sprema samo stanje entiteta **nakon** razdoblja; danas se spremaju `before` i `after` (`crates/game/src/dto.rs:538-547`).

Sloj 1: `HotelResolution.detail` nosi stanje v2 nakon kruga i račun odjela kruga (`accounts`, `PeriodAccountsDto`);
budžet i zbrojevi sezone računaju se pri čitanju. Novi događaj `SeasonDebriefRecorded` (sezonski mentor, jedan po
ljudskom hotelu, tek nakon zadnjeg kruga, nije u feedu; [ADR-036](../records/DECISIONS/ADR-036-kraj-sezone-poredak-financije-robota-i.md)).
`SCHEMA_VER` je i dalje 1.

### 6.4 Gdje je najveći tehnički trošak

`game` je danas čvrsto vezan na v1 tipove (`HotelState`, `MonthPlan`, `MonthReportDto`) i na `Month` kao ključ kruga
(`fold`, `state_at`, `effective_plan`, `robot_moves`, `resolve_round`, API putanje). v2 traži da `game` drži „stanje v1
ili v2" i „ključ kruga" te da grana na tim mjestima. To je **prvi posao sloja 1**: 14–20 h s razdobljem. Ništa drugo u
sloju 1 ne teče dok ovo nije gotovo.

### 6.5 API (samo dodavanje)

Postojeće putanje ostaju i primaju i kod razdoblja. Nove putanje vrijede za vlastiti hotel (i za profesora):

| putanja | sloj | što vraća |
|---|---|---|
| `GET /v1/hotels/{hotel_id}/kpi` | 0 | popunjenost, ADR, RevPAR, TRevPAR, GOPPAR, marža, udio rada, točka pokrića |
| `GET /v1/hotels/{hotel_id}/pl?round=&scope=round\|season` | 1 | račun dobiti po USALI-ju s budžetom; upit je `round` (kod kruga kao u putanjama: `jun`…`sep`, `t01`…`t17`), ne `period` (P14) |
| `GET /v1/hotels/{hotel_id}/departments` · `/{code}` | 1 | odjeli: prostor, prihod, zadovoljstvo gostiju, ljudi, odlasci, potražnja |
| `GET /v1/games/{game_id}/season` | 1 | kraj sezone: poredak, krivulja, financije i odluke robota (tuđi ljudski hotel samo indekse), sezonski mentor; 409 `season_not_over` dok traje |
| `GET /v1/hotels/{hotel_id}/staff` · `/{employee_id}` | 2 | ljudi; profil s vještinama, osobinama, jezicima i poviješću |
| `GET /v1/games/{game_id}/labour-market?period=` | 2 | burza rada (ista za sve hotele) |
| `GET /v1/hotels/{hotel_id}/agency` · `/housing` | 2 | narudžbe, troškovi i doprinos agencijskih radnika; kreveti |
| `POST /v1/hotels/{hotel_id}/talks` (+ poruke, odgovor) | 2 | razgovor s osobom (isti tok kao razgovor u situaciji) |
| `GET /v1/hotels/{hotel_id}/rooms` · `/{room_id}` | 3 | tlocrt i kartica sobe |
| `GET /v1/hotels/{hotel_id}/reviews?period=&topic=` | 4 | recenzije s tekstom, teme, odgovori |
| `GET /v1/games/{game_id}/listing` | 4 | „Booking" svih hotela u igri (§7.1) |
| `PUT …/plans/{round}` | 1–4 | postojeći; sloj 1 dodaje `departments` (bez njih se zadržavaju odjeli važećeg plana) i vraća plan kakav je spremljen; granice plana nose cijene odluka (`costs`, `maintenance`, `seasonal_worker_cents`); kasnije `nalozi` |
| `GET /v1/games/{game_id}/compare` | 0–4 | profesor: hoteli jedan uz drugoga |

---

## 7 · Konkurencija, profesor, granice

### 7.1 Konkurencija „kao Booking" (Leon: da)

Ekran **„Hoteli u okolici"** izgleda kao stranica za rezervacije. Za svaki hotel u igri (student i roboti) prikazuje
ime, zvjezdice, ocjenu, broj recenzija, cijenu „od" (osnovna cijena razdoblja), sadržaje (odjeli: spa, rooftop…) i
recenzije koje se mogu čitati, s filtrom po temi. Lekcija je Marketing: pozicioniranje i što gosti hvale kod drugih.
Uz to stoje MPI/ARI/RGI (vlastito ÷ prosjek ostalih × 100). **Ljudi i račun dobiti drugih hotela su skriveni**
(pretpostavka 🟨 13). Odgovori na recenzije su izbor pristupa, a ne tekst studenta, pa nema tuđeg osobnog teksta.
Recenzije robota nastaju istim obračunom, pa ekran traži sloj 4.

### 7.2 Profesorov pogled (P9)

| pogled | sadržaj | kolegij |
|---|---|---|
| **USALI račun dobiti po hotelu**, hoteli jedan uz drugoga | odjelni rezultati, neraspoređeni troškovi, GOP, GOPPAR, fiksne naknade, EBITDA | Accounting |
| **Budžet i ostvareno** | referentni hotel (`indices.rs:153-190`) prikazan kao „budžet", s odstupanjem po stavci | Accounting (budžetiranje) |
| **Pokazatelji prema konkurenciji** | popunjenost, ADR, RevPAR, MPI / ARI / RGI | Economics, E-Business |
| **Ljudi** | fluktuacija, udio troška rada, vještine i motivacija po odjelu, agencija i smještaj, razgovori i njihove presude | Management (HRM, vođenje) |
| **Ugled** | ocjena, teme, odgovori | Marketing |
| **AI trošak** | po studentu, koliko je do granice | — |

### 7.3 Rizici i što namjerno ne radimo

| rizik | kako ga držimo |
|---|---|
| Balans se raspadne | ekvivalencijski test + `--check` svaki sloj, za mjesec i tjedan; kalibracija je dio sloja |
| Profili s različitim odjelima nisu fer | §11.3 pravednost mjeri; ako ne prolazi, odjeli se izjednače po vrijednosti |
| Student se utopi u detaljima | „Zašto" pokazuje tri najveća uzroka; dubina je klik niže |
| Stereotip se uvuče kroz sadržaj | načelo 9 + test zamjene podrijetla + Leonov pregled banke rečenica i imena |
| `game` s dva modela i dvije vrste razdoblja postane zapetljan | jedno mjesto grananja, isti scenariji testirani za oba modela |
| v1 se slučajno promijeni | zlatni test v1 |
| AI trošak u tjednoj igri | vrijeme voditelja, gornja granica po studentu, rezerva bez AI-ja |
| Sadržaja je mnogo (imena, osobine, rečenice, rubrike) | počinje odmah paralelno; Leon pregledava po komadima |
| Rok prezentacije | linije reza (§10) |

**Namjerno ne radimo:** simulaciju po danu i rezervacije po gostu (najmanji krug je tjedan); svijet koji izmišlja AI
(AI ne stvara ljude, sobe ni brojke); zajedničko tržište gostiju ili radnika među hotelima; potražnju po segmentu i
OTA kanale; smjene po danu, jelovnik i nabavu; dob zaposlenika (ne mijenja ništa, a otvara diskriminaciju po dobi);
plaću ili kvalitetu po podrijetlu.

---

## 8 · Slojevi

Svaki sloj: gradi se na svojoj grani, dokaže (testovi, `cargo run -p balance --release -- --seeds 1000 --check`, pregled
u pregledniku na mobitelu i laptopu, [ADR-027](../records/DECISIONS.md#adr-027)), pregleda, i tek onda ide sljedeći.
Sati su rad agenata [PROC]: S ≤ 10, M 10–25, L 25–45, XL > 45.

### Sloj 0 — pokazatelji (P2) · **S, 6–10 h** (u izradi na zasebnoj grani)

- **Student vidi:** popunjenost, **ADR**, **RevPAR**, TRevPAR, GOPPAR, GOP maržu, udio rada, CPOR i **točku pokrića**,
  svaki s jednom rečenicom formule.
- **Mehanika:** nijedna nova. Čiste funkcije nad postojećim izvješćem, računaju se pri čitanju. Smije u v1 i u `main`
  nakon demoa.
- **Ponuda soba:** RevPAR i popunjenost računaju se nad **svim** sobama × dani (Leon: da), da vrijedi
  RevPAR = ADR × popunjenost. Sobe izvan funkcije su „izgubljene noći". Web danas računa nad raspoloživima
  (`HotelScreen.tsx:132`).

### Sloj 1 — temelj v2, razdoblja, odjeli po USALI-ju (P3, P4) · **L+, 50–70 h**

- **Temelj (jedan agent, prvi):** kopija jezgre u `sim-hotel-v2`; `game` s dva modela i ključem kruga; razdoblje
  tjedan/mjesec (§4); plan koji se prenosi; zlatni test v1; ekvivalencija. **25–35 h**.
- **Student vidi:** odjeljak **„Odjeli"** (wellness, restoran, kafić… samo što hotel ima). Klik na odjel pokazuje stanje
  prostora, prihod, trošak, rezultat, zadovoljstvo gostiju, broj ljudi i „Zašto". Ekran **„Račun dobiti"** po USALI-ju:
  odjeli → odjelni rezultati → neraspoređeni troškovi → **GOP** → fiksne naknade → EBITDA. Na laptopu popis i detalj,
  na mobitelu popis pa puni ekran.
- **Student radi:** sezonce po odjelu (broj, ljudi tek u sloju 2), razinu cijene odjela, otvara ili zatvara sezonski
  odjel, raspoređuje održavanje po odjelima.
- **Lanac:**

  ```text
  premalo kuhara u kolovozu → opterećenje kuhinje iznad norme → kvaliteta restorana pada
     → manje gostiju večera u hotelu (38 % → 31 %) → prihod F&B pada
     → zadovoljstvo gostiju restoranom i G padaju → ocjena sporo pada → potražnja sljedećih tjedana
  ```

- **Sadržaj:** `departments.json`, profili s odjelima, raspad `fixed_cents` na neraspoređene troškove i fiksne naknade
  ispod GOP-a (Leon: svakako), koeficijenti po vrsti razdoblja. Kalibracija: za C ostali odjeli ≈ 30 % prihoda soba;
  sezonska GOP marža 35–45 % ostaje cilj nakon raspada (pitanje N9).
- **Balans:** marže odjela u pojasu (Smještaj 65–80 %, F&B 15–35 %, wellness 20–40 % [PROC]); udio troška rada
  30–40 % (LRH ≈ 35 % [P]); tjedna i mjesečna igra istih odluka završavaju blizu (nova provjera); §11.1–11.5 prolaze.
- **Paralelno nakon temelja:** formule odjela · sadržaj · ekrani nad lažnim podacima · profesorov račun dobiti ·
  balans.

**Sloj 1 — izgrađeno** (grana `model-v2/sloj1`; isporuka u [CHANGELOG](../records/CHANGELOG.md), pretpostavke
[ADR-029](../records/DECISIONS/ADR-029-sloj-1-pretpostavke-plana.md) i ADR-031…036):
- **Odgođeno** ([BACKLOG](../records/BACKLOG.md) `[sloj1]`): profesorov račun dobiti (P17), „Zašto" po odjelu
  (P18; detalj pokazuje razlaganje brojkama), tjedan u mjesecu za situacije (P13), zbirni mjesečni mentor u tjednoj
  igri (P3).
- **Student radi** u detalju odjela (sezonski radnici, razina cijena, otvoren, održavanje); plan kruga drži odluke
  hotela i sažetak odjela; svaka odluka pokazuje cijenu ([ADR-035](../records/DECISIONS/ADR-035-sucelje-odjela-odluke-u-detalju-odjela.md)).
- **Balans:** uz gornje provjere i „C: ostali ≈ 30 % prihoda soba" (pojas 25–35 %); Brižni drži barem 80 %
  sezonskog plana. Brižni u tjednoj igri planira sezonce unaprijed jer smije ±3 po krugu (ADR-029, dopuna).
- **Kraj sezone:** poredak po napretku, financije i odluke robota, sezonski mentor
  ([ADR-036](../records/DECISIONS/ADR-036-kraj-sezone-poredak-financije-robota-i.md)).
- **Zadana igra** je v2 tjedna ([ADR-034](../records/DECISIONS/ADR-034-zadani-model-igre-je-v2-tjedna.md)).

### Sloj 2 — ljudi (P5, P6) · **XL, 80–110 h, u tri dijela**

#### 2a · Pojedinci, vještine, osobine, voditelji, burza, otpuštanje — 40–50 h

- **Student vidi:** odjeljak **„Ljudi"** po odjelima: lik, ime, uloga, trake vještina, osobine (dobre zeleno, loše
  crveno, skrivene „?"), jezici s razinom, motivacija, plaća prema očekivanoj i oznaka rizika. Profil osobe ima
  povijest, obuke i „zašto je motivacija pala". **Burza rada** je popis kandidata s filtrima po ulozi, vještini i
  jeziku.
- **Student radi:** zaposli s burze, otpusti, povisi plaću, pošalji na obuku (vještina ili jezik), premjesti,
  postavi voditelja.
- **Mehanika:** formule iz v1 po osobi. Cilj motivacije = 55 + 2 × (plaća prema očekivanoj − 100) − pola kazne
  opterećenja odjela + voditelj + obuka + osobine + situacije. Broj odlazaka je zaokruženi zbroj rizika, a tko odlazi
  bira kocka. Vještine rastu radom i obukom (§3.4), a očekivana plaća s vještinama (§5.2).

  ```text
  Ana, sobarica, početnica (temeljitost 22, „Brzo uči"), 1 650 € → tečaj + 8 tjedana rada → temeljitost 61
     → kvaliteta domaćinstva +, tema „čistoća +" u recenzijama → očekivana plaća 1 980 €
     → traži povišicu → odbijena: rizik odlaska 4 % → 9 % · prihvaćena: trošak, motivacija +
  ```

- **Roboti:** svaka strategija dobiva jednostavno pravilo za ljude (Štedljivi zapošljava početnike, Brižni obučava i
  prihvaća povišice, Lovac na popunjenost zapošljava do kapaciteta, Slučajni nasumce unutar granica).
- **Balans:** fluktuacija sezonaca 30–50 % po sezoni, a stalnih 15–25 % godišnje [PROC]. Nijedna radnja nije
  dominantna: „samo početnici", „samo profesionalci", „otpusti najslabije", „svima najveća plaća".

#### 2b · Jezici, agencija, smještaj — 15–25 h

- **Student vidi:** pokrivenost jezika po odjelu prema jezicima gostiju tog mjeseca, ekran agencije (narudžbe na putu,
  trošak i doprinos po osobi) i smještaj (kreveti u gradu, sobe za radnike, izgubljeni prihod).
- **Testovi:** **test zamjene podrijetla** (§2, t. 9): ista osoba s drugim podrijetlom razlikuje se samo u jezicima,
  roku, naknadi, dozvoli i smještaju. Zbroj troškova agencije u računu dobiti jednak je zbroju po osobama, do centa.

#### 2c · Razgovori s AI-jem — 25–35 h (od toga ~8 h rubrike i likovi)

- Pregovori o plaći, upozorenje, motivacijski razgovor, razgovor za posao, izlazni razgovor (§5.5), na postojećem
  putu lika i presude. Rubrike i rezervni odgovori za svaki tip. Vrijeme voditelja i gornja granica troška.
- **Testovi:** kvaliteta 1–5 daje učinak samo unutar granica kataloga; bez AI-ja isti razgovor radi s ponuđenim
  odgovorima; presuda koja zakasni ide neutralnim pravilom (ARHITEKTURA §8.3).

### Sloj 3 — sobe (P7) · **M, 15–25 h**

- **Student vidi:** tlocrt po katovima. Na laptopu je to mreža 120 pločica u boji stanja, a na mobitelu popis po katu s
  filtrima. Kartica sobe pokazuje tip, stanje, noći i prihod, kvarove, obnovu, recenzije te sobe i je li soba za
  radnike.
- **Student radi:** obnavlja kat ili tip sobe (2–4 tjedna izvan prodaje), bira prioritet održavanja, stavlja sobu
  izvan funkcije ili je daje radnicima (§5.4).
- **Mehanika:** prodane noći se raspoređuju po pravilu (tip, stanje, broj sobe), bez kocke. Trošenje = 150 + 8 ×
  noći sobe na mjesec [PROC]. Kvar ima šansu ispod praga stanja (tok `v2:kvar`). Lanac učenja o investiciji: obnova
  u lipnju košta malo izgubljenih noći, a ista obnova u kolovozu košta rasprodane noći.
- **Paralelno sa slojem 2:** da, ako je oblik `detalj` dogovoren unaprijed.

### Sloj 4 — recenzije i konkurencija (P8) · **L, 30–40 h** (od toga ~10 h banka rečenica)

- **Student vidi:** odjeljak **„Recenzije"** (ocjena, segment, jezik, tekst, soba, teme, kretanje ocjene) i
  **„Hoteli u okolici"** (§7.1).
- **Student radi:** odgovara na recenziju izborom pristupa (ispriča i mjera · zahvala · obrana · šablona), **bez AI-ja**
  (Leon). Broj odgovora je ograničen vremenom voditelja.
- **Mehanika:** broj boravaka = prodane noći / 4,2; recenziju piše ~12 % boravaka [PROC], pa ~20 tjedno i ~80 mjesečno
  za C. Ocjena gosta = cilj iz sobe, odjela, jezika i vrijednosti, uz šum s nultim zbrojem. Teme su najveći
  pokretači. Ocjena portala je bazen s istim pomakom kao v1 na mjesec.
- **AI:** samo „istaknuta recenzija" po razdoblju (Haiku), uz rezervu iz banke.
- **Ovisi o:** slojevima 1–3.

### Tablica veličina

| sloj | veličina | sati [PROC] | smije paralelno s |
|---|---|---|---|
| 0 | S | 6–10 | sve (ne dira brojke) |
| 1 | L+ | 50–70 | ništa dok temelj ne stoji |
| 2a | L | 40–50 | 3 |
| 2b | M | 15–25 | 2c, 3 |
| 2c | L | 25–35 | 2b, 3 |
| 3 | M | 15–25 | 2 |
| 4 | L | 30–40 | — (treba 1–3) |
| P9 profesor | M | 12–18 | uz svaki sloj |
| **ukupno** | | **≈ 195–275** | |

Iskreno: s paralelnim agentima to su **4–6 tjedana sesija**, a ne dana. Svih pet slojeva nije realno prije sredine
prosinca [PROC]. Sloj 1 je 20–25 h veći nego u prvom nacrtu zbog tjednih razdoblja. Sloj 2 je oko dva i pol puta veći
zbog vještina, burze, agencije i razgovora.

---

## 9 · Odluke (Leon 10.10.) i nova pitanja

### 9.1 Odlučeno

| # | pitanje | Leonova odluka | gdje u dokumentu |
|---|---|---|---|
| 1 | isti ljudi po profilu | **da**, a svi ljudi su razrađeni kao u Simsima: osobnost, dobre i loše strane, traka po vještini | §3.3, §3.4 |
| 2 | odjeli po profilu | **otprilike da**: odjeljak „Odjeli", u odjelu stanje prostora, zarada, zadovoljstvo gostiju | §3.2 |
| 3 | stare demo igre | **ostaju igrive na v1** | §6.2 |
| 4 | otpuštanje | **dopušteno** uz otkazni rok i otpremninu; kolege gube motivaciju samo iznimno. **Novo:** burza rada, rast vještina i plaće, jezici, agencija za strane radnike sa smještajem | §5 |
| 5 | broj mjeseci | **4 mjeseca** za prezentaciju. **Novo:** krug je tjedan ili mjesec, bira profesor | §4 |
| 6 | ponuda soba | **sve sobe** | §8 sloj 0 |
| 7 | GOP po USALI-ju | **svakako da**: osiguranje, najam i porez ispod GOP-a | §3.2 |
| 8 | odgovor na recenziju | **izbor bez AI-ja**. **Novo:** razgovori s ljudima i gostima gotovo uvijek s AI-jem | §5.5 |
| 9 | tuđi hoteli | **recenzije drugih hotela vidljive „kao Booking"**; ljudi skriveni (pretpostavka) | §7.1 |
| 10 | cijena po tipu sobe | **da**: osnovna cijena + doplate | §3.5 |
| 11 | imena i osobine | hrvatska, regionalna i strana imena; **nacionalnost samo za činjenice** (pravilo voditelja) | §2 t. 9 |
| 12 | voditelj odjela | **svakako**: ulazi u prvi prolaz sloja 2 | §3.3 |

### 9.2 Nova pitanja (preporuka je prva)

1. **Agencijski radnici „jeftiniji"?** Preporuka: **ista plaća za istu vještinu**. Agencija donosi ljude kad ih nema,
   uz naknadu, dozvolu i smještaj, pa je ukupno često **skuplja** po glavi. Niža plaća po podrijetlu krši načelo 9 i
   hrvatski zakon. Jeftini su **početnici**, bez obzira na podrijetlo.
2. **Burza rada:** preporuka je **ista burza za sve hotele, bez natjecanja** za kandidata. Natjecanje za ljude dolazi
   tek s međusobnim utjecajem (VIZIJA §3).
3. **Tjedni kalendar:** preporuka je **16 tjedana + zadnje razdoblje od 10 dana** (21.–30. 9.). Alternativa su
   17 tjedana s početkom 3. 6., ali onda lipanj nije cijeli.
4. **Zadana vrsta kruga** za nove v2 igre: preporuka **tjedan** (Leonova želja), a mjesec ostaje izbor profesora i put
   ekvivalencije s v1.
5. **Mentor u tjednoj igri:** preporuka je kratko „tri uzroka" svaki tjedan **bez AI-ja** i puni AI mentor **na kraju
   kalendarskog mjeseca** (manje troška, mjesečna lekcija ostaje).
6. **Vrijeme voditelja:** preporuka je **3 razgovora tjedno ili 8 mjesečno** koje pokreće student. Razgovori koje
   pokrenu ljudi ili situacije ne troše vrijeme.
7. **Gornja granica AI troška:** preporuka je **2 $ po studentu po igri**, zatim ponuđeni odgovori. Profesor može
   povisiti granicu.
8. **Jezični sastav gostiju:** preporuka je da ga agent istraži iz DZS-a (dolasci po zemlji) i podataka Kvarnera, a ne
   da ga pogađamo.
9. **GOP marža nakon raspada fiksnih:** preporuka je **zadržati cilj 35–45 %** za GOP i EBITDA prikazati ispod. Pojas
   potvrđuje profesor.
10. **Sobe za radnike:** preporuka je **dopustiti** ih (lekcija: izgubljena noć prema najmu u gradu), najviše 10 % soba
    [PROC].
11. **Skrivena osobina kandidata:** preporuka je **jedna skrivena**, a otkriva je razgovor za posao ili 4 tjedna rada.

---

## 10 · Redoslijed i linije reza

```text
nakon demoa         sloj 0 → main (samo prikaz)                          ◄─ rez A
                    sloj 1: temelj v2 + tjedan/mjesec + odjeli + USALI   ◄─ rez B: najmanji „wow" za profesora
                    2a (ljudi, voditelji, burza)  ║  3 (sobe)              ◄─ rez C: „Sims/BitLife" osjećaj
                    2c (razgovori s AI-jem) ║ 2b (jezici, agencija, smještaj)
                    4 (recenzije + „Booking")                            ◄─ puna slika
                    P9 profesor raste uz svaki sloj
```

| ako vrijeme stane nakon… | što se pokazuje | zašto je dovoljno |
|---|---|---|
| **sloja 0** | ADR, RevPAR, GOPPAR, točka pokrića | jezik Economicsa, ali hotel je i dalje brojke |
| **sloja 1** | Odjeli + USALI račun dobiti + budžet i ostvareno, **igra po tjednima** | profesor vidi svoj kolegij u igri; najmanji pravi „wow" |
| **1 + 2a** | ljudi s vještinama i osobinama, voditelji, burza, povišice | Sims dubina za studenta; Management + Accounting |
| **1 + 2a + 2c** | uz to razgovori s ljudima | najjači dojam uživo: student pregovara s konobarom |
| **1 + 2 + 3** | uz to tlocrt i povrat obnove | E-Business i investicije |

Ako ostaje vremena samo za jedan dio nakon 2a, preporuka je **2c (razgovori) prije 2b i 3**. Pregovor s konobarom
uživo je najjači trenutak prezentacije.

---

## 11 · Pretpostavke (🟨 čekaju Leonov pregled)

**Pet pretpostavki voditelja** (zapisane u [ADR-026](../records/DECISIONS.md#adr-026); Leon ih je 10.10. potvrdio
odgovorima u §9.1):

1. **ADR = prosječna cijena prodane sobe**, prikazan uz RevPAR, popunjenost i GOPPAR.
2. **Odjeli ovise o hotelu**; katalog i profil određuju.
3. **Zaposlenicima student upravlja**, ne samo da ih gleda.
4. **Recenzije su pojedinačne, tekst iz banke rečenica**, AI samo povremeno; student odgovara izborom.
5. **Slojevi redom 0 → 4**, svaki zasebno dokazan; sloj 2 sada u tri dijela.

**Pretpostavke arhitekta:**

6. v1 se zamrzava, a v2 je novi paket. Model i vrsta razdoblja biraju se u snimku pravila (§6). *Sloj 1:* vrsta
   razdoblja i kalendar su u postavkama igre, ne u snimku (P15, §6.2).
7. v2 izvješće uvijek nosi i v1 sažetak (§3.9, §6.3).
8. Kocka raspoređuje, a ne mijenja zbrojeve (§2, t. 4).
9. Nalozi su dio plana razdoblja i primjenjuju se pri obračunu. Plan se prenosi u sljedeće razdoblje (§4.3).
10. Tekst recenzija se ne sprema (§3.6).
11. Sve brojke [PROC] su procjene za prvu kalibraciju.
12. Razdoblje ulazi u temelj sloja 1; koeficijenti pomaka zadani su po vrsti razdoblja u katalogu (§4.2).
13. Ljudi i račun dobiti drugih hotela su skriveni; vide se samo „Booking" podaci i pokazatelji prema konkurenciji (§7.1).
14. Nacionalnost određuje samo jezike, naknadu, dozvolu, rok i smještaj. Plaća je ista za istu vještinu (§2, t. 9;
    §9.2 pitanje 1).
15. Razgovori s ljudima koriste postojeći put lika i presude; AI nikad ne zapisuje broj (§5.5).
16. Dob zaposlenika se ne modelira (§7.3).
