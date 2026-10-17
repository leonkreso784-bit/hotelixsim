# PLATFORMA §4 · Generirani hoteli

Dio indeksa [PLATFORMA.md](../PLATFORMA.md). Faza: GENERATOR HOTELA u „poslije" ([ROADMAP/platforma](../../plan/ROADMAP/platforma.md),
F3-1…8). Ovisnost o razredu: u razrednoj igri ([ADR-037](../../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md) t. 1)
svaki student ima svoj hotel u istoj igri, pa generator puni slotove 1..N.

> **17.10.2026.: Leon odlučio BANKA HOTELA prije generatora** ([ADR-056](../../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)).
> *„Trebat ćemo napraviti i banku hotela gdje će profesor moći dodijeliti hotele studentima koje želi."* Prvi korak nije
> generator nego **popis gotovih, pregledanih hotela** raznih veličina (10–15 hotela 4*, pečat u serijama kao banka ljudi);
> profesor dodjeljuje hotel po studentu ili nasumce. Generator iz ovog dijela dolazi kasnije i samo **puni banku**. Mjesto i
> tri komada (sadržaj, jezgra, platforma): [ROADMAP](../../plan/ROADMAP.md) „Pilot → produkcija i banka hotela". Oblik iz
> §4.2 (opis hotela u dnevniku, ključ profila kao tekst) i **pravednost iz §4.3 vrijede i za banku hotela**.

## 4.1 Danas

| što | gdje |
|---|---|
| tri fiksna profila, sva tri sa 120 soba (v1) | `content/hotel/profiles.json:4,19,34`; `crates/sim-hotel/src/state.rs:26` |
| profil je enum A/B/C | `crates/sim-hotel/src/profile.rs:7-14` |
| v2 profili s odjelima, ključ je isti enum | `content/hotel/v2/profiles.json`; `crates/sim-hotel-v2/src/catalog.rs:52-62` |
| igrač dobiva profil iz sjemena, a roboti A, B, C u rasporedu iz sjemena; imena su fiksna | `crates/game/src/command.rs:16-20, 28-34, 69-90` ([ADR-015](../../records/DECISIONS/ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md) t. 13) |
| u događaju je profil tekst `"A"/"B"/"C"` | `crates/game/src/event.rs:47-53` |
| referentna tablica (početak skale indeksa) računa se **po kategoriji** (4*) iz referentnog profila | `crates/game/src/game.rs:133-141`; `crates/sim-hotel/src/indices.rs:154-190` |

## 4.2 Predloženi oblik

1. **Generator je čista Rust funkcija** `generate(sjeme, postavke_razreda, slot) → HotelSpec`. Opis hotela sadrži ime,
   kategoriju, broj soba, vrste soba i tlocrt, odjele s početnim ljudima i stanjem te fiksne troškove. Funkcija živi
   uz jezgru v2 (vrijede brane determinizma [ADR-009](../../records/DECISIONS/ADR-009-ojacavanje-brana-determinizma-je-task-1.md)/[ADR-017](../../records/DECISIONS/ADR-017-odluke-voditelja-gradnje-u-koraku-3.md)). Kocka ima **vlastiti tok**
   `Rng::derive(sjeme, [label("gen:hotel"), slot])`, pa ne pomiče postojeće tokove ([MODEL_V2](../MODEL_V2.md) §2 t. 3).
2. **U `GameCreated` se zapisuje opis, ne samo sjeme.** Ako se generator kasnije promijeni, stara igra se i dalje
   čita iz svog zapisa (isto načelo kao puni snimak pravila). Najmanja promjena koda: `HotelSetup.profile` ostaje tekst,
   ali može upućivati na **profil zapisan u snimak** (npr. `"gen:1"`). Ključ kataloga v2 prelazi s enuma `Profile` na
   tekst (`catalog.rs:62`), a A/B/C ostaju kao „ručno napisani profili". v1 se ne dira.
3. **Granice generatora su podaci u `content/`.** Na primjer: broj soba 80–160 dok postoje samo tablice za četiri zvjezdice
   [PROC] (tri i pet zvjezdica su BACKLOG „drugi val"), skupovi odjela po arhetipu („obiteljski s bazenom", „gradski s barom
   na krovu", „wellness") i rasponi stanja. Profesor bira samo „raznolikost": isti / malo različiti / jako različiti.
4. **Imena** se biraju iz banke u `content/` (npr. „Vila", „Lungomare", „Bura", „Lovor"), uz **popis zabranjenih
   stvarnih hotela s Kvarnera**. Generirani hotel ne smije se zvati kao postojeći hotel, iz istog razloga kao
   [ADR-002](../../records/DECISIONS/ADR-002-ime-hotelixsim-nikad-skraceno-na-hotelix.md). Profesor može preimenovati hotel.
5. **Raspodjela u razredu.** Sjeme razreda generira jedan hotel po slotu, 1..N. Svi studenti dijele jednu igru, ali ne
   moraju imati isti hotel. Koliko hotela je različito, određuje postavka „raznolikost" iz t. 3; kod male raznolikosti
   nekoliko studenata ima usporedive hotele pa je usporedba pravednija. Točan omjer je proizvodna odluka (anketa kad faza krene).

## 4.3 Pravednost uz ocjenu napretka

Hoteli različite veličine smiju biti različiti, ali **mogućnost napretka mora biti jednaka**. Vrijedi jednako za hotele iz
banke (17.10.) i iz generatora. To traži tri stvari:

- **Referentna tablica po hotelu, ne po kategoriji.** Referenca postaje „isti taj hotel uz zadani plan", pa napredak
  znači „koliko si bolji od vlastitog nečinjenja". Cijena: po jedan izračun po hotelu, koji treba držati u predmemoriji.
- **Provjera ljestvica indeksa:** svaki indeks mora biti nezavisan od veličine (po sobi, udio, omjer). Sumnjiva je
  likvidnost u eurima bez gornje granice ([ADR-014](../../records/DECISIONS/ADR-014-kolovoz-rasprodan-lekcija-s3-i-blagajna.md)): mora ići po sobi ili relativno prema referenci. Provjeriti u
  `indices.rs` prije generatora [PROC].
- **Balans preko generatora:** `balance --generate --seeds 1000 --check` uzorkuje hotele. Prolaz: razlika medijana
  napretka po veličini i po arhetipu manja od praga, i nijedan arhetip ne pobjeđuje pod svim strategijama. Uz to
  **zlatni test**: postavka „A/B/C fiksno" daje bajt-iste igre kao danas. Ako hotel iz generatora padne na provjeri
  pravednosti, generator vuče ponovno iz istog toka (deterministički).

## 4.4 Ovisnosti

Generator mora znati **sve** što opis hotela sadrži: odjele (sloj 1), obnovu, vrste soba (dizajn u radu) i ljude
(sloj 2). **Ako se gradi prije nego što se ti slojevi slegnu, prepisuje se sa svakim slojem.** Zato ide u „poslije":
nakon što su vrste soba i obnova spojene, a ljudi dolaze u generator kao proširenje. Do tada razred koristi A/B/C.
