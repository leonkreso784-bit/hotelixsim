# ADR-060 — Leonove odluke 18.10. i pretpostavke sesije 9: uniforme, klik 75 % gdje se piše, pilot prije proizvoda
**Datum:** 2026-10-18 · **Status:** ✅ **Leonove odluke 18.10.**, 🟨 t. 17–20 ČEKA LEONOV PREGLED (ankete i tekst, sesija 9) ·
**Vezano:** [ADR-004](ADR-004-sve-se-gradi-i-pokazuje-lokalno.md), [ADR-047](ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md) t. 3,
[ADR-056](ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md), [ADR-057](ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md) t. 8,
[ADR-059](ADR-059-pretpostavke-sesije-8-pravilo-b-izgled-k7-pravilo-c.md) t. 4–5, [kartica IZGLED-c](../../plan/2026-10-14-popravak-temelja/task-13c-izgled-uniforme.md) §11,
[kartica L1-13](../../plan/2026-10-14-popravak-temelja/task-14-l1-13-kalibracija.md), [plan platforme](../../plan/2026-11-platforma.md)

**Izvori.** Leon je 17.–18.10.2026. (sesija 9) odgovorio na pet anketa (IZGLED-c, L1-13 prva, druga i treća, banka K5) i u tekstu odobrio
pilot. Brojke klika su iz dijagnoze `.superpowers/sdd/2026-10-14-popravak-temelja/l113-1b-dijagnoza.md` i mjerenja `l113-1b-provjere/tablica-klik.txt`
(1000 sjemena). Točke 1–16 su Leonove odluke; 17–20 su pretpostavke voditelja ([ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)), svaka s cijenom.

## A — uniforme vanjskih likova (IZGLED-c, spojeno 879da8a + 69762f1)
1. **Liječnik je nova uloga vanjskog `lijecnik`:** bijela kuta i stetoskop. To je liječnik izvan hitne (turistička ambulanta, dežurni liječnik,
   obiteljska medicina). Liječnik hitne ostaje `djelatnik_hitne` u odori hitne. Banka dodaje 10–20 liječnika; paralelna sesija ih je napisala 15
   (grana `sadrzaj/banka-ljudi` 106ede1, još nije spojena). Broj uloga vanjskih raste s 14 na 15.
2. **Hitna:** crvena odora sa znakom hitne, **plavom zvijezdom života**. **Crveni križ ne**, jer je zaštićen znak: po Ženevskim konvencijama
   i Zakonu o Hrvatskom Crvenom križu uporaba mu je ograničena (Crveni križ, sanitetska služba oružanih snaga), pa ga hitna i aplikacija ne smiju
   koristiti kao svoj znak. Leon je pitao zašto i prihvatio to objašnjenje.
3. **Vatrogasac nosi kacigu** (`hat: "fire"`). Kaciga, kao i policijska kapa, pokriva kosu iznad čela. Polja lica ostaju ista.

## B — L1-13 kalibracija (dijelovi 1 i 1b spojeni 7f22802 i 9813b8a)
4. **Klik nosi ~75 % učinka vlastitog odgovora.** Klik na ponuđeni odgovor u situaciji s likom gubi i dio mehanike (stanje odjela, novac, gosti),
   ne samo ocjenu (5 → 4). Ovo razrješava ADR-059 t. 4 (nalaz I1: klik je nosio 92–95 %) i ostvaruje očekivanje iz ADR-047 t. 3.
5. **Klik gubi dio dobiti, a trošak plaća cijeli** (druga anketa). Umanjuje se samo pozitivan dio učinka: zadovoljstvo, moral, ocjena, oprema i noćenja grupe
   (najmanje 1). Trošak, povišice i negativni dijelovi ostaju puni.
6. **Brana „B3 klik" mjeri samo situacije s likom (M1, treća anketa).** Omjer Δ klika i Δ vlastitog najboljeg, oba prema šutnji na tim
   situacijama, mora biti ≤ 75 %, tjedno i mjesečno. **Ukupno klik i dalje nosi ~89–94 % težine vlastitog**, i to Leon prihvaća. Razlog: situacije
   bez lika nose 39 % (tjedno B) i 70 % (mjesečno B) ukupne Δ, a u njima student nema što pisati, pa su klik i vlastiti odgovor isti. Dijagnoza
   je pokazala da ni dobit 0 ne spušta ukupni omjer ispod 75 % (tjedno A 75,4 %, mjesečno B 81,1 %). Trošak tu nije uzrok, nego klik smanjuje
   (−76 i −58 stotinki boda, tjedno i mjesečno B).
7. **`click_gain_bp` = 8500**, tj. klik zadrži 85 % dobiti. Leon je u anketi vidio raspon 85–88 %, a ne 90 %, pa 9000 nije odobren (pregled I1).
   Izmjereno na 1000 sjemena: tjedno A/B/C 73,9 / 67,5 / 73,9 %, mjesečno 70,7 / 65,5 / 72,6 % (brana ≤ 75,0 %). Test sadržaja traži korake od 500.
8. **Razine značaja situacije:** **ključna / važna / svakodnevna**, izvedene iz `weight` 4 / 3 / 1–2. Student vidi razinu, a bodovi idu po razini (dio 2).
9. **„Zadovoljan gost" znači ocjenu 4 ili 5.** Uz hitni popravak takav odgovor vraća +2 boda stanja (neto +1), a ocjena 1–3 ostavlja −1
   (`FAULT_FIXED_MIN_QUALITY = 4`). Nadjačava ADR-046 t. 1 („i uz ocjenu 1").
10. **Kvar prostora je 9 predložaka** (uključeni i P-KLIMA i P-DOTRAJALO): vrsta „kvar" ili popravak opreme bez vanjskog uzroka. Kvar skida 1 bod
    stanja prostora. U repou je to iza ključa i zasad isključeno, a uključuje se u dijelu 4.
11. **Situacija „s gostima"** je ona čiji je lik u ulozi gosta ili je na odjelu uz goste. Takvih je 27 od 55 predložaka (dio 3).

## C — pilot prije proizvoda (Leon 18.10. ~03:30, tekst)
12. **Deploy je ODOBREN kao pilot na kraju faze B**, nakon Leonova OK-a za spajanje `model-v2/sloj1` → `main`. Leon: *„kada faza B bude gotova
    deployaj, nije bitno, ovo je tek pilot"*. To je izričit OK unaprijed za **taj jedan** deploy ([ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)).
    Ne vrijedi za kasnije deploye. **Prije objave** voditelj Leonu daje prijedlog hostinga s cijenom €/mj i najjeftinijom varijantom koja čuva temelj
    (Leon sam plaća trošak). Uz to **zaštita AI troška**: pristupni kod ili ograničenje AI poziva, jer prijave još nema, a ključ je Leonov.
13. **Bez odobrenja i podrške profesora i fakulteta ne gradi se cijeli proizvod do studenog.** Leon: *„bez odobrenja i podrške profesora i fakulteta
    nema smisla graditi cijeli proizvod do studenog"*. Prioritet je igra koju profesor može vidjeti. Velike cigle platforme čekaju profesorov odgovor.
14. **Što se mijenja:** [ADR-004](ADR-004-sve-se-gradi-i-pokazuje-lokalno.md) („Railway tek za pilot"). Pilot dolazi ranije, odmah nakon faze B,
    a pružatelj se bira u prijedlogu. [ADR-057](ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md) t. 8 (platforma od studenog)
    sada vrijedi samo uz podršku fakulteta. [Plan platforme](../../plan/2026-11-platforma.md) (§5 hosting za P3, §7 pitanje 3) ostaje nacrt:
    izlazak na internet je sada pilot igre, a ne platforme.

## D — banka ljudi (anketa na pitanja pisaca K5)
15. **Tehničarka R-00239 postaje muškarac** (rod po statistici, ~3 % žena). Albanka R-00212 odgovara. Ulogu sanitarnog inspektora i susjeda
    dobivaju i muškarci i žene, a omjer se popravlja postupno.
16. **Strani radnici „ostalo":** u `mjesta.json` dolaze 2–3 prave treće zemlje s MUP-ova popisa dozvola (npr. Egipat), a ne više Ukrajine.
    Otvoreno ostaje: talijanski kod lokalnih vanjskih, liječnici u idućim serijama i rupa 011–050 u seriji-000.

## E — pretpostavke voditelja (🟨 ČEKA LEONOV PREGLED)
17. **Put „srednji" se u balansu računa kao klik** (RULING na čuvarovo razilaženje u 1b). U igri je svaki odabir ponuđenog odgovora u situaciji
    s likom klik, pa i srednji, a inače test pariteta igre i balansa pada čim je `click_gain_bp` < 10 000. Pomiču se stupac „srednji" i koraci
    klik − srednji i srednji − prepušteno. Svaki korak lanca mora ostati > 0. Ovo zatvara ADR-059 t. 5. Nije nova odluka o igri, nego
    ista igra u mjerenju. Cijena ako je krivo: jedan uvjet u `v2_answers.rs` i ponovno mjerenje.
18. **Dio 4 (kalibracija) ponovno mjeri klik.** A8 čini šutnju gorom, pa se omjer Δ pomiče i `click_gain_bp` se može promijeniti. Tablica
    prije/poslije (težina vlastitog i klika, A/B/C, tjedno i mjesečno) ide Leonu prije spajanja. Cijena: jedno mjerenje od 1000 sjemena, možda i anketa.
19. **Minor nalazi pregleda su odgođeni** (popis u [BACKLOG](../BACKLOG.md) [sloj1]): PRAVILO-c 2, IZGLED-c 1, L1-13-1 M1–M3 i L1-13-1b M1–M7.
    Najvažniji je 1b M5: mentor ne zna da klik na slabiji odgovor gubi 15 % dobiti. Cijena: sitni popravci u dijelu 4 ili nakon faze B.
20. **Boja hitne `#C8102E` i tamnoplava `#1F2F4D`** (policija, vatrogasci) uzete su iz prijedloga kartice. Leon ih potvrđuje pri pregledu reda
    „Službe" na `/likovi` (kartica §10 t. 3). Šutnja na kvaru daje −1,04, a ne točno −1, jer nosi i privremeni popravak s faktorom kruga
    (L1-13-1 M2; tablica je u `l113-1-provjere/tablica-kvar.txt`). Cijena: jedna boja u `CLOTHES`.

**Dopuna 18.10. (banka ljudi, izvor: pitanja pisaca K5 i bilješke sesije banke 9b).** Uz t. 1: 15 liječnika spojeno je u `model-v2/sloj1`
(19160c9), banka ima 530 ljudi ([BANKA_LJUDI](../../architecture/BANKA_LJUDI.md)). Uz t. 15: rod u ulogama vanjskih (sanitarni inspektor,
susjed) popravlja se miješano, a ne samo dodavanjem žena. Uz t. 16: „ostale" zemlje se ne pune Ukrajinom. Pravila i ciljevi uloge `lijecnik`
(služba iz Hrvatske, dob 25–65, cilj 4 %, rod ž 65 % procjena) pretpostavke su voditelja banke, s 🟨 u [§1.5 i §2.4](../../architecture/BANKA_LJUDI/02-koliko.md);
dva otvorena pitanja (talijanski kod lokalnih vanjskih, liječnici u idućim serijama) su u [BANKA_LJUDI §6](../../architecture/BANKA_LJUDI/06-pitanja.md) t. 17–18.

**Dopuna 18.10. (K6, izvor: Leonove ankete i bilješke sesije banke 10b; spojeno ead0daf, banka 730 ljudi).** Uz t. 15: R-00239 je sada
muški tehničar Dario Benazić (07a46b5). Uz t. 16 (otvoreno): talijanski kod vanjskih Leon smatra nebitnim (*„pusti da vanjski svi znaju pričat
eng i hrv"*), pa stroj traži **svaki vanjski `hr` ≥ 2 i `en` ≥ 2**; liječnici su u idućim serijama na **pauzi** dok udio ne padne prema 4 %.
Leon (tekstom): *„bitniji su gosti i radnici nego vanjski likovi"*, pa je K6 dao radnike i goste serija-005 i 006, a vanjski serija-003 čeka.
Treće zemlje **EG, TR, ID** i polje zemlje **`daleka`** pretpostavke su sesije banke, s 🟨 u [BANKA_LJUDI §2.2](../../architecture/BANKA_LJUDI/02-koliko.md)
i [§3.2](../../architecture/BANKA_LJUDI/03-pravila.md); ovaj ADR ne dobiva novu 🟨 točku. Rupa 011–050 u seriji-000 ostaje otvorena.
