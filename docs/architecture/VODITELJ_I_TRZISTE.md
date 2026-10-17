# Voditelj odjela i uvid u tržište — nacrt za raspravu

> **Status:** nacrt za raspravu (13.10.); **pet pitanja §3 Leon je odgovorio 17.10.2026.** (✅, [ADR-056](../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md)). Cigle ostaju u fazi C i LJUDI (L2a-5). Povod: Leonove dvije ideje od 13.10. —
> *„menadžer odjela bi trebao upravljati odjelom i biti odgovoran za odjel"* i *„graf i uvid u trenutno stanje: koliko
> turista dolazi, kolika je zainteresiranost"*. Odluke idu u ADR, cigle u [ROADMAP/igra.md](../plan/ROADMAP/igra.md).

## 1. Voditelj odjela

### Što već imamo
- **Odluka postoji:** [ADR-028](../records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md) t. 12 — „Voditelj odjela — svakako".
- **Opisan je u modelu sloja 2:** [MODEL_V2.md](./MODEL_V2.md) §3 — voditelj je jedan od ljudi odjela (5–6 od 58 u profilu C),
  vještina „vođenje" pomiče motivaciju cijelog tima (±5 bodova, [PROC]) i ubrzava rast vještina; odjel bez voditelja trpi kaznu.
- **Raspoređen je:** cigla **L2a-5** u fazi LJUDI (prosinac–siječanj, prekretnica P3), uz nalog „postavi voditelja" (L2a-7) i
  ekran „Ljudi" (L2a-10). Otvoreno za Leona: iznos kazne bez voditelja i ±5.
- **U kodu danas nema nijedne osobe.** Odjel ima samo brojeve (`DeptState`: stalni, sezonci, stanje prostora, zadovoljstvo;
  `crates/sim-hotel-v2/src/department.rs`), a moral je jedan za cijeli hotel. Riječ „voditelj" postoji samo kao oznaka kome se
  prepušta situacija (`Game::situation_leader`, grana situacija) i kao uloga u tekstu predložaka („voditelj restorana").
- **Leonova odluka P4** (situacije): prve 3 situacije tjedna su obvezne, ostale smije „prepustiti voditelju" (ishod kvalitete 2).
  Gumb dolazi u W-1 (L1-5).

### Što Leonova ideja dodaje
MODEL_V2 voditelja vidi kao **polugu motivacije**. Leon traži više: voditelj **upravlja** odjelom i **odgovara** za njega.
To su dvije nove stvari:
- **Upravlja:** student smije odjel prepustiti voditelju (ne samo situaciju). Voditelj tada sam određuje sezonce, održavanje i
  razinu cijena odjela po pravilu, kao robot, a dobrota tih odluka ovisi o njegovoj vještini. Student vodi hotel, a ne svaku
  stavku. To je prava lekcija managementa: delegiranje.
- **Odgovara:** ocjena odjela (već postoji: stanje prostora, zadovoljstvo gostiju, usklađenost ljudi) postaje **njegova ocjena**.
  Student je vidi uz ime, i prema njoj ga nagrađuje (povišica), zadržava ili mijenja. U razredu (ADR-037) dobar voditelj je
  meta preotimanja.

### Prijedlog: dva koraka
**Korak 1 — sada, bez nove mehanike (~4–6 h [PROC]).** Svaki odjel dobiva **imenovanog voditelja** iz sadržaja: ime, odjel i
jedna rečenica (npr. „Ivana Kovač, voditeljica recepcije, 8 godina u hotelu"). Isti ljudi po profilu (ADR-028), pa u sloju 2
postaju pravi `Zaposlenik`. Na ekranu Odjeli piše „Odgovorna: Ivana Kovač" uz ocjenu odjela, situacije se prepuštaju njoj po
imenu (W-1), a lik u situaciji odjela je ona. Brojke igre se ne mijenjaju, balans je isti. Temelj: imena se pišu jednom, u
katalogu ljudi koji sloj 2 ionako treba (L2a-1).

**Korak 2 — u L2a-5 (sloj 2a, faza LJUDI).** Voditelj postaje osoba s vještinama: utječe na motivaciju i rast tima (MODEL_V2),
**plus** Leonova ideja: nalog „prepusti odjel voditelju" (pravilo odluka kao kod robota, kvaliteta po vještini „organizacija"),
zadovoljstvo voditelja veže se uz ocjenu odjela, a zamjena voditelja (otkaz, novi s burze, unaprjeđenje iz tima) ima cijenu.
Dira balans, pa ide kroz probe L2a-8 („samo prepusti sve" ne smije biti dominantna strategija).

**Sudari na koje treba paziti:** riječ „voditelj" u dokumentima već znači i studenta (vrijeme voditelja, MODEL_V2 §6) i glavnu
sesiju gradnje (ADR-038) — za igru predlažem **„voditelj odjela"** uvijek punim imenom, a student je **direktor**.
„Prepusti voditelju" (P4) danas daje fiksnu kvalitetu 2; u koraku 2 kvaliteta bi ovisila o voditelju — to mijenja P4.

## 2. Uvid u tržište: koliko gostiju dolazi i kolika je zainteresiranost

### Što jezgra već zna, a student ne vidi
Svaki krug simulacija računa (`crates/sim-hotel-v2/src/market.rs`, sve u noćenjima soba):

| veličina | značenje | student danas vidi |
|---|---|---|
| `base_demand` | koliko gostiju sezona sama donosi (lipanj 72 %, srpanj 78 %, kolovoz 106 %, rujan 73 % soba) | ne |
| `demand` | zainteresirani nakon cijene, ugleda i marketinga | samo u rečenici „Zašto" |
| faktori cijene, ugleda, marketinga | koliko je svaki podigao ili spustio zanimanje | samo u rečenici |
| grupe | noćenja grupa | da (situacije) |
| `sold` | smješteni gosti | da (Pokazatelji) |
| `available`, `staff_capacity` | sobe u funkciji i koliko osoblje stigne | posredno |
| odbijeni = zainteresirani − smješteni | gosti koje hotel nije primio (puno, premalo osoblja) | samo po odjelu u rečenici |

API to danas ne šalje (ostaje u dnevniku igre). Web ima jedan graf (krivulja napretka na Kraju sezone, ručni SVG).
[MODEL_V2.md](./MODEL_V2.md) („Namjerno ne radimo") isključuje potražnju po segmentu i simulaciju po danu — prijedlog to poštuje:
najmanja jedinica je krug (tjedan ili mjesec), cijeli hotel.

### Prijedlog: ekran ili kartica „Tržište"
1. **Graf kroz sezonu:** po krugu tri vrijednosti — zainteresirani, smješteni, kapacitet — i istaknuti odbijeni. Tablica uz graf
   (UI_PRAVILA: boja nikad sama).
2. **Rastav zanimanja zadnjeg kruga** (primjer tjedna u lipnju, 120 soba × 7 dana × 72 %): „sezona donosi 605 noćenja; tvoja
   cijena −8 %, ugled +3 %, marketing +5 % → zainteresirano 602 noćenja".
   Student vidi **zašto** je došlo toliko gostiju, ne samo koliko.
3. **Što slijedi:** očekivana sezonska razina idućeg kruga iz kataloga (npr. „kolovoz: najjači mjesec, potražnja veća od broja
   soba"). Pošteno označeno kao sezonska očekivanja, ne točna prognoza.
**Cijena [PROC]:** novi DTO s polja `market` + ruta (~3 h, `schema.rs` je zajednička datoteka — uz dogovor kolosijeka), graf
poopćenjem postojećeg `ProgressChart` (~4 h, nakon spajanja grane obnove gdje živi), ekran i UI čuvar (~4 h). Balans se ne mijenja.

## 3. Pitanja za Leona — odgovorena 17.10. (✅ [ADR-056](../records/DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md))

Pitanja su u [Leonovu pregledu](../plan/2026-10-17-leonov-pregled/04-voditelj-i-platforma.md) bila D1–D5.

| # | pitanje | Leon 17.10. | što vrijedi |
|---|---|---|---|
| 1 (D1) | voditelj — korak 1 sada ili u L2a-5? | *„Može po tvojoj preporuci, ne zaboravi: svaki radnik i voditelj mora imati svoju crtu koliko je jak, a koliko slab… može doći loš voditelj i dobar voditelj."* | ✅ imenovani voditelji **kad igra počne uzimati ljude iz banke** (K5), ne prije; do tada popravak temelja. **Svaki radnik i voditelj ima vidljivu crtu jak / slab**: izvodi je jezgra iz polja `razina` i sjemena (vještine, osobine; [BANKA_LJUDI](./BANKA_LJUDI/01-lik.md) §1.1, §1.2), pokazuje ekran Ljudi (L2a-10). Voditelji su podvrsta radnika (`strucnjak`), trebat će ih „puno puno različitih" (A9) |
| 2 (D2) | „odgovoran" znači (a), (b) ili (c)? | *„C"* | ✅ **(c) oboje**, u L2a-5: vodi odluke odjela kad mu ih student prepusti **i** ocjena odjela je njegova ocjena. Uz A9: voditelj radi i obuke i može koordinirati cijeli odjel |
| 3 (D3) | tržište — gdje? | *„Može tržište s grafovima."* | ✅ **kartica „Tržište" na ekranu s pokazateljima**, s grafom kroz sezonu i rastavom (§2), bez nove stavke izbornika |
| 4 (D4) | jedinica: gosti ili noćenja? | *„Može preporuka."* | ✅ **noćenja**, broj gostiju u zagradi |
| 5 (D5) | prognoza prije odluke? | *„Odlično, jako je bitno da imamo primjere ponude i potražnje u igri."* | ✅ **samo sezonsko očekivanje** (§2 t. 3); kako cijena mijenja broj gostiju student uči iz rastava nakon kruga |

**Gdje to ide:** voditelj (korak 2, t. 2) je cigla **L2a-5** u fazi LJUDI; kartica „Tržište" (t. 3–5) je cigla faze C ili
iza nje ([ROADMAP/igra](../plan/ROADMAP/igra.md)). Ništa od ovoga ne ulazi u popravak temelja.
