# ADR-062 — Leonove odluke 19.10. i pretpostavke sesije 11: dio 4 spojen, L1-13b prije pilota, održavanje po hotelu, pilot na Railwayu
**Datum:** 2026-10-19 · **Status:** ✅ **Leonove odluke 19.10.** (t. 1–9, četiri ankete, sesija 11), 🟨 t. 10–18 ČEKA LEONOV PREGLED ·
**Vezano:** [ADR-042](ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 2, 4, 7, 10, [ADR-045](ADR-045-trosak-odmah-ugled-kasnije-tjedni-krug.md),
[ADR-060](ADR-060-leonove-odluke-18-10-i-pretpostavke-sesije-9.md) t. 12–14, [ADR-061](ADR-061-leonove-odluke-18-19-10-i-pretpostavke-sesije-10.md),
[kartica L1-13b](../../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-l1-13b-izmiješani-odgovori-pojačano-održavanje-5-jedan-gumb-spremi-plan-opis-smještaja)

**Izvori.** Ledger `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md` („SESIJA 11"), analize `l113b-analiza-*.md`, pregledi
`l113b-{1,2,3,4}-review.md`, tablica `l113b-4-tablica.md` (1000 sezona) i prijedlog `pilot-hosting-prijedlog.md` (isti direktorij).
Točke 1–9 su Leonove odluke; 10–18 su pretpostavke voditelja i izvođača ([ADR-005](ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md)), svaka s cijenom.

## A — Leonove odluke 19.10.
**Prva anketa (dio 4 L1-13, sve preporučene):**
1. **Tri svakodnevna kvara nose 0,6** kao ostali bodovi situacije (P-KLIMA, P-DOTRAJALO, P-LOBBY-KAVA: mjesečno −0,6 / +1,2). Time je
   🟨 [ADR-061](ADR-061-leonove-odluke-18-19-10-i-pretpostavke-sesije-10.md) t. 6 potvrđena.
2. **Održavanje hotela B ostaje kakvo je:** B počinje sa sobama 40 i ima ~15 kvarova po sezoni bez obzira na održavanje.
3. **P-GOST-POZLILO ostaje negostinjska** (odjel `hotel`, ne puni dodatna mjesta kad je hotel pun).
4. **Spoji dio 4 i pushaj** (e4ec659, `CORE_VERSION` 2.6.0). Leon je izričito vidio da je tjedno u 1. krugu šutnja bolja od najboljeg odgovora
   ([ADR-045](ADR-045-trosak-odmah-ugled-kasnije-tjedni-krug.md) Dopuna 19.10.) i da tjedno u 1. krugu klik ocjene 4 daje isto što i 5.

**Druga anketa:**
5. **L1-13b cijela PRIJE pilota** (miješanje, održavanje +5, jedan gumb „Spremi plan", opis Smještaja), i to **planom u valovima**
   (neovisni dijelovi istodobno, svaki u svom stablu; Leon prvo vidi plan). BRZA GRADNJA ostaje prijedlog ([BACKLOG](../BACKLOG.md)).

**Treća anketa (L1-13b):**
6. **Opis Smještaja = tekst na stranici Smještaja** da stanje prostora određuje kvarove (ADR-042 t. 7).
7. **Pojačano održavanje:** cijena **ista**; +5 se računa **prije** trošenja zbog gužve (u gužvi +4); odjeli — Leon: *„to se mora izbalansirati
   isto s financijama, jer ponekad se moraju financije uzeti u obzir, ponekad održavanje"*. Izvedeno za **šest operativnih odjela**
   (Smještaj, Restoran, Lobby bar, Rooftop, Bar uz bazen, Wellness); tablica je pokazala da hotel **A štedi** (odgođeno najbolje), a **B i C
   ulažu** (pojačano najbolje; u B se to okrenulo). Parking i lokal dodaju samo 0,02–0,05 boda, pa ostaju isti.

**Četvrta anketa:**
8. **Push L1-13b** (e4ec659..7556d80).
9. **Pilot:** **Railway Hobby** (Amsterdam), **jedna lozinka za cijelu stranicu** (Basic Auth), stropovi AI troška **5 $/dan i 30 $/mj u
   aplikaciji**, **40 $/mj u Anthropic konzoli**, **najviše 3 igre po gostu**. Deploy i dalje tek uz izričit OK (ADR-060 t. 12).

## B — pretpostavke (🟨 ČEKA LEONOV PREGLED)
10. **Miješanje živi u `game` + `app`, ne u webu** — odstupa od doslovnog ADR-042 t. 10 („web šalje izvorni broj"). API vraća odgovore već
    izmiješanim redom; `chosen`, POST i dnevnik (`answer`) nose **prikazni** položaj; izvorni broj ostaje u događajima, obračunu, balansu, kod
    robota i u ulazu AI suca, a studentu se nigdje ne vidi (pregled opus provjerio API, dnevnik, mentora i web). Red daje
    `answer_order(sjeme igre, slot hotela, šifra primjerka, n)` (oznaka `game:answer-order`, Fisher–Yates); **pravilo je smrznuto** zlatnim
    testom: promjena traži dizanje verzije, inače klik usred kruga zapiše drugi odgovor. Cijena ako je krivo: web i ugovor API-ja na izvorni broj (~3–5 h), jezgra ista.
11. **v1 (zamrznuti demo) ostaje nepromiješan** (`Model::display_order` = identitet). Cijena: jedna grana u `model.rs` i testovi v1 (~1 h).
12. **„Spremi plan" je neaktivan dok nema izmjena.** `PlanSet` bez izmjena igri ne treba: krug bez njega dobiva isti plan (prenesen ili zadani).
    Jedina razlika: sezonski mentor kaže „plan se nijednom nije mijenjao" i dnevnik nema stavku `PlanChanged`. Cijena: omogućiti gumb na Planu (~1 h).
13. **Traka s gumbom „Spremi plan" vidi se i na Planu i na stranici odjela**; na odjelu piše i „Sprema se i: …" kad gumb sprema tuđe izmjene.
    Cijena: traka samo na Planu (~1–2 h weba i testova).
14. **Opis Smještaja ne kaže „stanje zgrade"**, nego „stanje prostora Smještaja (sobe i instalacije u zgradi)": kvarove određuje stanje
    prostora, a „Stanje zgrade" (`building_pts`) je zaseban pokazatelj koji na njih ne utječe. Cijena: tekst i test (~30 min).
15. **Tumačenje Leonove rečenice o financijama: nijedna razina održavanja nije najbolja uvijek** (ovisi o hotelu). Tablica to ispunjava
    (A odgođeno, B i C pojačano). Pregled je dodao: **zadana (redovna) razina nije najbolja ni u jednom hotelu**, a A odgođeno vodi pred
    zadanim mjesečno samo 0,17 boda ([BACKLOG](../BACKLOG.md), pitanje za Leona). Cijena: nova runda brojki održavanja i mjerenje 1000 sezona.
16. **Održavanje +5 bez prekidača sadržaja i bez nove verzije jezgre** (mijenja se šest brojki `intensive.equipment_pts`, `CORE_VERSION`
    ostaje 2.6.0; svaka igra nosi svoj snimak pravila). Brane zadanog plana su bajt-iste. Cijena: ključ i testovi (~2 h) ako treba prekidač.
17. **Test kostura `v2_b4_labels` (T2) vraća šest novih vrijednosti na staru prije otiska**, pa `SKELETON_SHA` ostaje isti (graditelj proširio
    po uzoru na L1-13 dio 4; pregled presudio: opravdano, ne slabi test). Cijena: novi otisak umjesto vraćanja (~15 min).
18. **Kolačić gosta 120 dana i besplatna domena `*.up.railway.app`** za pilot (voditelj prešutno uz četvrtu anketu; Leonu javljeno, nije
    potvrdio). Danas kolačić traje 7 dana bez obnove, pa bi student nakon tjedan dana izgubio hotel. Cijena: drugo trajanje je jedna
    konstanta; vlastita domena ~10 €/god i DNS (~1 h).

## Što se mijenja
- ADR-042 t. 2, 4, 7 i 10 su izvedeni (L1-13b, 13db069, 4be1db4, bdbdebe, 7556d80). ADR-042 t. 10 dobiva Dopunu 19.10. (upućuje na t. 10–11 ovdje).
- ADR-061 t. 6 je potvrđena (t. 1 ovdje); ADR-061 t. 9 (konačne brojke dijela 4) vrijedi jer je dio 4 spojen.
- Pilot: hosting i zaštita AI troška odlučeni (t. 9); plan pripreme pilota piše se u sesiji 12, prije gradnje ga vidi Leon.
