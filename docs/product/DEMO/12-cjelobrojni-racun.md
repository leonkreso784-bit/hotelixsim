# DEMO §12 — pravila cjelobrojnog računa i provjera za Leona

Dio [DEMO.md](../DEMO.md)

## 12 · Pravila cjelobrojnog računa — pretpostavke (04.10.)

Brojke u §2–§4 pisane su decimalno radi čitljivosti. Ovdje je pravilo kako ih jezgra stvarno računa (ARHITEKTURA §7.2:
bez `f32`/`f64`). **Svaka stavka P1–P32 je [PRETPOSTAVKA]** (Leon: „radi bez pitanja"); razlog je uz nju. Ne mijenja
nijednu formulu ni brojku iz §2–§11, samo kaže u čemu se računa i gdje se zaokružuje. P1–P32 su implementirana u `engine::calc` i `crates/sim-hotel`.

### 12.1 Opća pravila

| # | pravilo | razlog |
|---|---|---|
| P1 | **Jedna politika zaokruživanja: polovica od nule** (`half away from zero`). Pomoćna funkcija `div_round(a, b)` za `b > 0`: `(2a + sgn(a)·b) / (2b)` cjelobrojno. Jedina iznimka je kapacitet osoblja (P14), koji se zaokružuje dolje. | Student je može provjeriti na papiru (0,5 → 1; −0,5 → −1). Simetrična je pa izglađivanje morala ne klizi na jednu stranu. Bankarsko zaokruživanje je točnije u zbroju, ali ga je teže objasniti, a razlika u 5 mjeseci je zanemariva. |
| P2 | **Zaokružuje se jednom, na kraju formule.** Brojnik se računa u `i128`, dijeli se jednom (`mul_div(a, b, c)`), rezultat se ograniči (clamp) na raspon veličine i tek se onda sprema. Međurezultati se ne spremaju. | Više koraka zaokruživanja gomila pogrešku i daje različite brojke ovisno o redoslijedu množenja. |
| P3 | **Stanje na kraju mjeseca je zaokruženo i spremljeno; sve izvedeno računa se iz spremljenog.** Prikaz (npr. „8,4" ili „50,3") zaokružuje se samo za oči i nikad se ne vraća u račun. | Ponovljivost bajt za bajt (§11, točka 5): dnevnik ne ovisi o tome kako je UI nešto prikazao. |
| P4 | **Preljev:** novac je `i64` centi, sve ostalo `i32` ili manje, a umnošci idu kroz `i128`. `mul_div` vraća `Err(Overflow)` ako rezultat ne stane u ciljni tip; nema tihog omotavanja (`overflow-checks = true` i u release gradnji). | `i64` centi drži do ≈ 92 bilijuna €, a najveći umnožak potražnje (≈ 10²⁰) ne stane u `i64`, pa mora `i128`. Tiha greška u novcu gora je od pada. |
| P5 | **Dijeljenje s nulom:** svaki nazivnik je ili konstanta, ili parametar kataloga koji se provjerava pri učitavanju (`> 0`), ili ima izričit zaštitni slučaj naveden dolje. `mul_div` s `c = 0` vraća `Err`, nikad 0. | Greška u katalogu se vidi odmah pri učitavanju, ne kao čudna brojka u mjesecu 4. |
| P6 | **Granice se primjenjuju nakon izračuna** (kao u §6) i uvijek cjelobrojno, bez zaokruživanja granice: npr. cijena ≥ 60 % reference provjerava se kao `cijena·100 ≥ 60·ref`. | Granica `1,5 × 135 € = 202,50 €` ne smije zaokruživanjem postati 203 €. |

### 12.2 Jedinice pohrane

Skale 0–100 (zadovoljstvo, moral, iskustvo, oprema, kazna, pet indeksa) spremaju se kao **`i32` u stotinkama boda**
(5 500 = 55,00), raspon 0–10 000. Koeficijenti su u **baznim bodovima** (10 000 = 1,0; 0,3 = 3 000).
Tzv. „p. b." iz §6.2 čitamo kao **postotni bod** = 500 baznih bodova za „5 p. b." (P24).

| # | veličina | pohrana | zaokruživanje / korak (kad i gdje) | zaštita, razlog |
|---|---|---|---|---|
| P7 | novac (blagajna, trošak, prihod, GOP, marketing, kompenzacije) | `i64`, centi | cijeli centi; zaokružuje se samo u dijeljenjima koja su ispod navedena | `i64` jer blagajna i GOP mogu biti negativni |
| P8 | cijena (odluka) | `i64`, centi; **korak 1 €** (`cijena % 100 == 0`) | bez zaokruživanja; ulaz se odbija ako nije cijeli euro | Jednostavno za studenta; cijena 135 € = `13500`. `cijena_ref > 0` provjerava katalog (P5) |
| P9 | broj soba (ukupno, izvan funkcije), dani `D` | `u16`, `u8` | izvan funkcije = `div_round(4000 − oprema, 400)`, zatim `0..=sobe_ukupno` | Tvrdih „120" nema u kodu: `sobe_ukupno` je parametar kataloga (u demou 120, §3.1). `D` dolazi iz kalendara mjeseca (30/31) |
| P10 | sobe-noći (raspoloživo, potražnja, prodano, grupe) | `u32` | potražnja: **jedno** `div_round` na cijelom umnošku `120·D·bazna·f_c·f_u·f_m / 10¹⁶` (sve u bp), zatim `max(0)` | Najveći umnožak ≈ 10²⁰ → `i128`. Bez međuzaokruživanja faktora |
| P11 | bazna popunjenost | `u16`, bp (72 % = 7 200); smije biti > 10 000 (rasprodan kolovoz, ADR-014), katalog provjerava 1–20 000 | nema; podatak iz kataloga | |
| P12 | `f_cijena` | `i32`, bp | `10 000 + ε_bp·(cijena − ref)/ref` jednim `div_round`, zatim clamp **3 000–16 000** (0,3–1,6) | `ε_bp = −8 000` za 4*. `ref > 0` (P5). Clamp je kao u §3.1 |
| P13 | `f_ugled`, `f_marketing` | `i32`, bp | `f_ugled = 10 000 + div_round(54·(ocjena_st − 840), 10)`, najmanje 0; `f_marketing = 10 000 + div_round(1 500·M, M + 1 500 000)` | 0,0054 po bodu ljestvice 0–100 = 54 bp; 10·ocjena = `ocjena_st/10`, pa se **ne množi dvaput**. `M ≥ 0` pa je nazivnik ≥ 1 500 000 centi (0 se ne pojavljuje) |
| P14 | `kapacitet_osoblja` | `u32` | `zaposleni·D·24/10` **cjelobrojno dolje** (jedina iznimka od P1) | Tvrdo ograničenje ne smije se prekoračiti zaokruživanjem prema gore |
| P15 | prodano, grupno, individualno | `u32` | `prodano = min(grupe + potražnja, raspoloživo, kapacitet_osoblja)`; `grupno = min(grupe, prodano)`; `individualno = prodano − grupno` | Pravilo „grupe prve" iz §3.1 ostaje, samo je zapisano izvedeno. Nema dijeljenja |
| P16 | opterećenje | `u16`, stotinke (1,80 = 180) | `div_round(prodano·100, zaposleni·D)`, clamp 0–300 | **Zaštita:** `zaposleni == 0` → kapacitet 0, opterećenje 300 (nema dijeljenja). Stalnih je u demou ≥ 30 pa se ne događa, ali jezgra to ne pretpostavlja |
| P17 | `kazna_opterećenja` | `i32`, stotinke boda | **linearno** `clamp(100·(opt − 180), 0, 6 000)`: 0 do 1,80, 60 pri 2,40 | Oblik krivulje u §3.3 nije zadan („raste do 60"); linearna je najjednostavnija i cijela. Zamjena kvadratnom = tablica od 61 vrijednosti u katalogu, bez promjene koda |
| P18 | plaća u odnosu na tržište | `u16`, cijeli postotak (90–115, korak 1) | nema zaokruživanja: `trošak_rada = zaposleni·220 000·plaća%/100` je uvijek cijeli broj centi (`2 200·plaća%`) | Korak 1 % iz §6.1 to jamči. Bonusi i `novi·150 000` su cijeli centi |
| P19 | zadovoljstvo, moral, iskustvo, oprema | `i32`, stotinke boda, clamp 0–10 000 | svaka formula jedan `div_round` na kraju; konstante poput „+3", „−4", „20" ulaze kao 300, −400, 2 000 | Δ iz situacija su `i32` stotinke (npr. `2·(q−3)` → `200·(q−3)`); granica ±8 = ±800 |
| P20 | `moral'`, `zadovoljstvo'` (izglađivanje) | kao P19 | `moral + div_round(3 000·(cilj − moral), 10 000)`; za zadovoljstvo faktor 5 000 | Polovica od nule znači da se razlika od ±1 stotinke zaustavi, što je zanemarivo. Cilj morala: `5 500 + 200·(plaća% − 100) − div_round(kazna, 2)` |
| P21 | `fluktuacija` | `u16`, bp mjesečno (3,75 % = 375) | `300 + div_round(15·max(0, 6 000 − moral), 100)`; **odlasci = `div_round(zaposleni·fluk_bp, 10 000)`**, `0..=zaposleni` | 3,75 % od 50 = 1,875 → **2**. Određeno, bez nasumičnosti. Rizik: stepenica pri malom broju ljudi (vidi dolje) |
| P22 | `iskustvo'` („mješavina") | kao P19 | **težinski prosjek po glavi:** `div_round(iskustvo·(Z − n) + 2 000·n, Z)` gdje je `Z = zaposleni`, `n = odlasci + novozaposleni` (najviše `Z`), zatim `+ 300 + bonus_obuke` i clamp | Novi dolaze s 20 (= 2 000). **Zaštita:** `Z == 0` → iskustvo ostaje nepromijenjeno |
| P23 | ocjena portala | `u16`, stotinke (8,40 = 840), clamp 100–1 000 | cilj `div_round(14 000 + zadovoljstvo', 25)` (= 5,6 + 0,04·zad.); `ocjena' = ocjena + div_round(2 500·(cilj − ocjena), 10 000) + 5·[odgovaranje]` | „+0,05" = 5 stotinki. Ograničenje situacije ±0,15 = ±15. Ocjena i zadovoljstvo su različite jedinice (stotinke ocjene vs. stotinke boda): **ne miješati** u istoj formuli bez ove pretvorbe |
| P24 | popust (grupe) | `u16`, bp (25 % = 2 500) | grupna cijena = `div_round(cijena·(10 000 − popust), 10 000)` u centima, **jednom po sobi-noći**, zatim `× sobe-noći` | „smanjuje popust za 5 p. b." = **postotni bod** = −500 bp (25 % → 20 %), najmanje 0. Ne miješati s baznim bodom iz ARHITEKTURA §7.2 |
| P25 | `ostali_prihod` | `i64`, centi | `div_round(prihod_soba·3 000, 10 000)` **jednom na mjesečnom zbroju**, ne po noćenju | Zaokruživanje po noćenju dalo bi sustavnu razliku |
| P26 | CPOR, trošak rada, održavanje, fiksni | `i64`, centi | bez zaokruživanja (cjelobrojni umnošci centi: `2 200 · prodano` itd.) | |
| P27 | kamata na minus | `i64`, centi | nakon `blagajna += GOP − kapitalna`: ako `blagajna < 0` → `blagajna −= div_round(−blagajna, 100)` (1 % od iznosa u minusu, **jednom mjesečno**, bez kamate na kamatu u istom mjesecu) | Kamata se **ne ubraja u GOP** (GOP je operativni rezultat). 1 %/mj = 100 bp |
| P28 | `GOPPAR` | `i64`, centi po raspoloživoj sobi-noći | `div_round(GOP, sobe_ukupno·D)`; **nazivnik su sve sobe, uključujući one izvan funkcije** | STR definicija (raspoložive sobe); kvar tako kažnjava F, što je željeno. Naziv „raspoloživo" u §3.1 znači nešto drugo (soba bez izvanfunkcije), zato je ovdje izričito |
| P29 | `GOPPAR_ref(mj)` | `i64`, centi, izračunat pri izgradnji kataloga | isti izraz kao P28 za profil C bez odluka | **Zaštita:** nazivnik u indeksu je `max(GOPPAR_ref, 1 000)` (najmanje 10,00 € po sobi-noći), jer referenca može biti ≤ 0 u slabom mjesecu |
| P30 | indeksi G, R, Z, O, F | `i32`, stotinke boda; sirove vrijednosti nisu ograničene (sirovi F nema gornju granicu, R može biti negativan), clamp 0–10 000 vrijedi za centrirani indeks (§4) | jedan `div_round` po indeksu na kraju. R: `div_round((ocjena_st − 700)·10 000, 280)`. Z: zadržavanje `= 10 000 − div_round(25·fluk_bp, 2)`, zatim `div_round(6 000·moral + 4 000·zadržavanje, 10 000)`. F: `GOPPAR_indeks = 5 000 + div_round(5 000·(GOPPAR − ref), max(ref, 1 000))`; `likvidnost = div_round(5 000·blagajna, fiksni + trošak_rada)`, bez gornje granice (ADR-014); negativna blagajna → 0 | Težine su bp (O: 4 500/3 000/2 500; F: 7 000/3 000). Z: 12,5 po 1 % = 25/2 stotinke po bp. `fiksni > 0` (P5) pa je nazivnik likvidnosti > 0 |
| P31 | `g_i` i `napredak` | `g_i` `i32` bp (−10 000..10 000); `napredak` `i32` stotinke boda | `g_i = div_round(10 000·(kraj − start), 10 000 − start)` ako `kraj ≥ start`, inače `... / start`; `napredak = div_round(Σ g_i, 5)` (w = 2 000 bp svaki) | **Zaštita:** `start = 10 000` i `kraj = 10 000` → **g = 0** (prostora za rast nema). `start = 0` može ući samo u prvu granu (nazivnik 10 000); druga grana (`/ start`) nastupa samo ako je `kraj < start`, pa je `start ≥ 1`. Zadnji korak zaokružuje se jednom, ne po indeksu |
| P32 | recenzije (broj novih), situacije `q` | `u32`; `u8` 1–5 | broj recenzija (kad se definira): `div_round(prodano·stopa_bp, 10 000)` | §2 ne daje formulu za broj recenzija; ovo je samo pravilo za kasniju. `q` izvan 1–5 odbija se pri učitavanju presude |

### 12.3 Najrizičnije pretpostavke (za Leonovu provjeru)

1. **P21 odlasci bez nasumičnosti:** `div_round` daje stepenicu (pri 3,75 %: 40 ljudi → 1,5 → 2; 39 ljudi → 1,46 → 1). Kad bude vidljivo u alatu za balans, alternativa je prijenos ostatka (`ostatak_odlazaka` u stanju) ili sjeme-RNG.
2. **P28/P29 GOPPAR s nazivnikom svih soba i podom reference 10 €:** utječe na F i izravno na profile u §5 i kalibraciju (§2.1). Drukčiji nazivnik pomiče sve financije.
3. **P17 linearna kazna opterećenja** i **P1 polovica od nule:** prva mijenja oblik cijele mehanike opterećenja, druga ostavlja sitnu pristranost prema većim brojkama (zanemarivu, ali provjerljivu u alatu za balans).

---

## Za Leonovu provjeru — najnesigurnije pretpostavke

1. ~~Elastičnost cijene~~ → riješeno 04.10.: po kategoriji (§2.0), Leon „otprilike da".
2. ~~Referentne cijene po mjesecima~~ → riješeno 04.10. (§2.0). Otvoreno ostaje: ostali prihod 30 % prihoda soba.
3. Trošak zaposlenog 2 200 € mjesečno i omjer stalnih i sezonaca po mjesecu (`content/hotel/catalog.json`, `seasonal_plan`).
4. Lekcija u S3: 25 % popusta grupi u kolovozu. Je li to realna ponuda agencije?
5. Kriteriji rubrika u S1–S4. To je tvoje područje i ondje si ti mjerilo.
6. Priče profila A i B (§5). Prepoznaješ li takve hotele iz prakse?
