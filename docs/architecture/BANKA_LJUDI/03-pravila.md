# BANKA_LJUDI §3 · Pravila kvalitete

Dio indeksa [BANKA_LJUDI.md](../BANKA_LJUDI.md).

## 3.1 Jedinstvenost
- `id` jedinstven u cijeloj banci; svaka serija ima **svoj raspon** brojeva, pa dva agenta ne mogu izdati isti ID.
- **Puno ime jedinstveno u cijeloj banci** (radnici, gosti, vanjski i partneri iz `names.json` zajedno), uspoređeno bez dijakritika i velikih
  slova (`Kovačević` = `Kovacevic`). Gost i radnik istog imena zbunili bi studenta.
- Ograničenja učestalosti [PROC]: jedno prezime najviše 0,6 % vrste, jedno ime najviše 1,5 %, da banka ne bude „sve
  Horvat i Ana".

## 3.2 Dosljednost zapisa
`godine_iskustva` ≤ `dob` − 16; razina prema iskustvu: početnik 0–2, iskusan 3–9, stručnjak ≥ 8 godina [PROC];
`lokalno`/`regija` ↔ `HR`; `mjesto` iz kataloga mjesta za lokalno i regiju; materinji jezik zemlje na razini 4; jezik
najviše jednom; ime i prezime pristaju rodu i zemlji (slovačko žensko prezime na `-ová` — ovo provjerava čovjek).

## 3.3 Bez stereotipa (ADR-028, MODEL_V2 §2 t. 9)
- Podrijetlo određuje **samo** jezike, rok i naknadu agencije, dozvolu i smještaj. Razina, dob i rod moraju imati **istu
  raspodjelu u svakoj skupini podrijetla** (stroj: odstupanje skupine od cijele banke najviše 5 postotnih bodova [PROC]).
- Opis je **neutralan prema osobinama**: bez riječi iz kataloga osobina i njihovih sinonima (pouzdan, marljiv, kasni,
  svadljiv, lijen, ljubazan…) i bez ocjene osobe. Opis kaže **što se dogodilo** (gdje je radio, odakle dolazi), ne **kakav
  je** čovjek. Skrivenu osobinu otkriva igra (MODEL_V2 §5.1), ne tekst.
- Bez rečenica o „narodu", „mentalitetu", „tipičnom" ponašanju zemlje; bez šala na račun podrijetla, dobi, roda,
  vjere, zdravlja ili izgleda.

## 3.4 Rod i padeži
`rod` je obvezan jer je tekst za AI pisan u rodu lika („Umorna si i ljuta", `Role.gender` u `templates.rs`); izvlačenje
traži lik roda uloge. Banka sprema **samo nominativ**, a predlošci stavljaju ime samo u nominativ („Ti si {gost.ime}");
treba li genitiv ili vokativ, dolazi polje `padezi` **novim formatom** (§4.3). Opis je u rodu lika („radila je", „došao
je"), bez obraćanja studentu u rodu ([UI_PRAVILA](../../product/UI_PRAVILA.md) §3 t. 3).

## 3.5 Zabranjeni sadržaj
Stvarne poznate osobe (popis zabranjenih: političari, sportaši, javne osobe, profesori FMTU-a, Leon); stvarni hoteli s
Kvarnera (isti popis kao generirani hoteli, [PLATFORMA §4](../PLATFORMA/04-generirani-hoteli.md) t. 4) i robne marke;
maloljetni radnici; zdravlje, vjera, politika, spolnost kao obilježje lika; vitice `{}` (mjesta za zamjenu); ćirilica.

## 3.6 Tko što provjerava

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

## 3.7 Vanjski: što vrijedi isto, što je posebno

**Isto kao radnici i gosti:** jedinstvenost ID-a i punog imena u cijeloj banci (§3.1; učestalost imena i prezimena unutar
vrste, poruka „od N vanjskih"); materinji jezik na 4, jezik najviše jednom; opis 40–220 znakova, najviše 2 rečenice, točka na
kraju, bez vitica i ćirilice, bez riječi iz `rijeci_opisa` (§3.3); zabranjene osobe, hoteli i marke (§3.5); ime pristaje rodu i
zemlji (čovjek).

**Posebno za vanjske** (🟨, stroj u K2v):
- **Ustanova je generička.** Vrsta ustanove ili tvrtke („sanitarna inspekcija", „turistička agencija za vikend-putovanja"),
  nikad vlastito ime ni mjesto u nazivu: stroj traži **malo slovo u svakoj riječi osim prve** i nijedan naziv iz
  `zabranjeno.json` (hoteli, marke). Mjesto ide u polje `mjesto`, ne u ustanovu.
- **Bez stvarnih inspektora, policajaca i vatrogasaca.** Lik ne smije nositi ime stvarnog službenika s Kvarnera; popis
  `osobe` u `zabranjeno.json` dopunjuje se kad Leon ili pregled nađe takvo ime. Opis ne spominje stvarne slučajeve, akcije ni
  postrojbe po imenu.
- **Imena ne iz `names.json`.** Puno ime ne smije biti među 40 partnerskih imena (ni među ostalih 80): dok igra ne čita banku
  (K5), ista osoba bi se pojavila dvaput. Ovo stroj već provjerava za radnike i goste (J2) i proširuje se na vanjske.
- **Službe su iz Hrvatske i s Kvarnera.** Službe, dobavljač i susjed imaju `zemlja: HR` i `mjesto` iz kataloga `lokalno`;
  dob službi najviše 65, inspektora i djelatnika hitne najmanje 25.
- **Bez stereotipa i u službama.** Opis ne kaže da je inspektor „strog" ili policajac „grub" (ton daje predložak, §1.5); rod
  po ulozi slijedi stvarni omjer, ali tekst nikad ne veže ulogu uz rod („kao žena u vatrogascima…" nije dopušteno).
  Strani vanjski su samo u tržišnim ulogama, što je činjenica tržišta, ne osobina zemlje.
