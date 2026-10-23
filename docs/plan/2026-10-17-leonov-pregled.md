# Leonov pregled — sve što čeka tvoju riječ, kao pitanja s preporukom

**Status:** 🟩 AKTIVAN · **Područje:** kvaliteta · **Otvoren:** 2026-10-17 · **Za:** Leon — dva dana odluka nakon PRAVILO-a i
IZGLED-a, prije faze C

> Područje je „kvaliteta" samo zato što brana `check:docs` zna četiri područja (igra, platforma, kvaliteta, dokumentacija),
> a u kvaliteti nema drugog aktivnog plana. Ovo nije plan gradnje nego popis tvojih odluka; kad su odgovorene, ide u arhivu.

## Zašto

Od 04.10. Claude radi bez pitanja i ono što bi trebao odlučiti ti zapisuje kao pretpostavku 🟨 (ADR-005). Nakupilo ih se
preko stotinu, u dvadesetak ADR-ova, karticama, nacrtima i BACKLOG-u. Ti si 16.10. prihvatio da prije faze C uzmemo dva
dana samo za tvoje odluke, da gradimo na njima, a ne na pretpostavkama. Ovdje je sve skupljeno u **43 numerirana pitanja**,
svako s preporukom.

## Kako odgovoriti

- Napiši broj i: `ok` (prihvaćaš preporuku), `ne` (biraš „Inače" ili kažeš svoje) ili `napomena: …`.
  Primjer: „A1 ok, A2 ne – 3 kvara održan, 8 zapušten, C2 vidi napomenu".
- Skupna pitanja (A5, A13, C8, F1) prihvaćaš jednim „ok"; ako ti smeta samo jedna točka: „A13 ok osim d".
- Može i kroz Claude: reci „pitaj me blok 1" i voditelj pitanja s pravim izborom postavi kao anketu (preporuka prva);
  brojke i činjenice (A2, A10, A11) pita tekstom.
- „Ne znam" je valjan odgovor: pretpostavka ostaje 🟨, a voditelj zapiše što bi pomoglo (npr. pitanje profesoru).
- Voditelj na `/kraj` odgovore pretvara u ADR (✅ Leonova odluka), briše riješeno iz BACKLOG-a i skraćuje popis 🟨 u CLAUDE.md.

## Redoslijed (prvo ono što koči gradnju)

| blok | pitanja | zašto tada | min |
|---|---|---|---|
| 1 | A1–A4 | L1-13 (jedna zajednička kalibracija brojki) ne može krenuti bez cilja | 15 |
| 2 | A5–A6 | PRAVILO-a se upravo gradi; prag je jedna brojka prije spajanja | 5 |
| 3 | C1–C2 | pečat serija banke ljudi; dok nisu zapečaćene, ne idu u igru | 70–90 |
| 4 | B1 | slijepi pregled tekstova koči L1-15 (završnu provjeru faze B) | 60–90 |
| 5 | B2 | napomene uz tekstove, tek nakon B1 | 10 |
| 6 | A7–A13 | pravila koja student osjeti i brojke iz prakse | 25 |
| 7 | C3–C11 | sitnice banke, vanjski likovi, izgled | 15 |
| 8 | B3–B5 | sadržaj situacija i mjerilo AI-ja (40 ocjena su zaseban sat) | 15 (+40) |
| 9 | D1–D5, E1–E7 | voditelj odjela, tržište, platforma | 20 |
| 10 | F1–F2 | tehničko i već riješeno, jedno „ok" | 5 |

Ukupno ~4–5 h (s mjerilom ~5–6 h): npr. prvi dan blokovi 1–4, drugi dan 5–10.

## Dijelovi

| grupa | datoteka | pitanja |
|---|---|---|
| A — pravila igre i brojke | [01-pravila-i-brojke.md](#leonov-pregled--a--pravila-igre-i-brojke) | A1–A13 (13) |
| B — tekstovi odgovora i mjerilo AI-ja | [02-tekstovi.md](#leonov-pregled--b--tekstovi-odgovora-b4-i-mjerilo-ai-ja) | B1–B5 (5) |
| C — banka ljudi i izgled likova | [03-banka-ljudi.md](#leonov-pregled--c--banka-ljudi-i-izgled-likova) | C1–C11 (11) |
| D — voditelj odjela i tržište · E — platforma i ostalo | [04-voditelj-i-platforma.md](#leonov-pregled--d--voditelj-odjela-i-tržište--e--platforma-i-ostalo) | D1–D5, E1–E7 (12) |
| F — tehničko i već riješeno | [05-tehnicko.md](#leonov-pregled--f--tehničko-jedno-ok-i-već-riješeno) | F1–F2 (2 skupna, ~25 stavki) |

Svako pitanje ima isti oblik: **Što je** (za nekoga tko pojam ne zna) · **Pretpostavili smo** · **Preporuka** (prva) ·
**Inače** i cijena · **Vidi** (ADR, dokument ili datoteka).

## Pojmovi (jednom, na početku)

- **🟨 pretpostavka:** odluka koju je Claude donio umjesto tebe da gradnja ne stane; vrijedi dok je ne potvrdiš ili promijeniš.
- **L1-13:** jedna zajednička kalibracija svih brojki igre nakon faze B (popravak, kvarovi, težina situacija, šutnja).
- **Težina situacija:** koliki dio razlike između dobre i loše sezone dolazi od odgovora na situacije, a ne od plana.
- **Oznaka:** tekst ponuđenog odgovora koji student vidi. **Rubrika:** kriteriji po kojima AI sudac ocjenjuje odgovor.
- **Balans / `--check`:** računalo odigra 1000 sezona i provjeri da igra ostaje poštena; pada li, brojka se ne smije spojiti.
- **Banka ljudi, serija, pečat:** vidi uvod dijela C.

## Kamo ide odgovor

| pitanja | ide u |
|---|---|
| A1–A4 | novi ADR „Leonove odluke za L1-13" + plan popravka red L1-13 |
| A5–A6 | ADR-055 t. 1 i kartica PRAVILO (ADR sesije 7 za B13) |
| A7, A8 | ADR-051 t. 2–3; ADR-015 t. 2 |
| A9 | novi ADR „strani radnici: plaća i obuka"; MODEL_V2 §5.4, ROADMAP/igra L2a-6 i L2b-3, ADR-028 (pravilo voditelja) |
| A10, A11 | ADR-032 t. 9, ADR-033; `content/hotel/v2/departments.json` |
| A12 | BACKLOG (zatvaranje), DEMO §12 „Za Leonovu provjeru" |
| A13 | ADR-013, 019, 022, 023, 025, 026, 027, 029, 031, 035, 036, 048 → ✅ |
| B1–B2 | tri dokumenta `b4?-pregled-tekstova.md` → B4-ispravci; ADR-055 t. 6 |
| B3–B5 | BACKLOG „Sadržaj situacija"; `content/hotel/` (situations, catalog, mjerilo); SITUACIJE_S1b_S5-S8 |
| C1–C2 | ledger banke ljudi (pečat); ADR-055 t. 3; `raspodjele.json` |
| C3–C8 | `zabranjeno.json`, serije banke; BANKA_LJUDI §1–§3 |
| C9–C10 | BANKA_LJUDI §6 (pitanja 12–16) i novi ADR „vanjski u banci ljudi" |
| C11 | ADR IZGLED (piše `/kraj`) |
| D1–D5 | VODITELJ_I_TRZISTE §3 → ADR + ROADMAP/igra (L2a-5, faza C) |
| E1 | novi ADR „prijava: sesije" + ARHITEKTURA §12, PLATFORMA dio 08 (L10) |
| E2, E3 | PITANJA_ZA_PROFESORE; plan platforme §7 |
| E4, E5 | ADR-003; ROADMAP |
| E6, E7 | BACKLOG „vizija" → nacrt PLATFORME; DIZAJN §3 |
| F1, F2 | svi ADR-ovi iz tablice F1 → ✅; BACKLOG „Otvoreno za Leona" |

## Izvori i što nije uključeno

Pročitano 17.10.: svih 23 ADR-a sa statusom 🟨 ([DECISIONS.md](../records/DECISIONS.md)) i 🟨 dijelovi ADR-028,
kartice PRAVILO i IZGLED (`task-12`, `task-13`), [BACKLOG](../records/BACKLOG.md) „Otvoreno za Leona",
[SLJEDECA_SESIJA](../SLJEDECA_SESIJA.md), [VODITELJ_I_TRZISTE](../architecture/VODITELJ_I_TRZISTE.md) §3,
[BANKA_LJUDI §6](../architecture/BANKA_LJUDI/06-pitanja.md), ledgeri i izvještaji u `.superpowers/sdd/2026-10-14-popravak-temelja/`
i `.superpowers/sdd/banka-ljudi-k1/` (`k2-*-report.md`, `k2-review.md`, `pregled-20.md`, `nacrt-vanjski-report.md`).

Nije uključeno: odluke koje si već donio (✅ ADR-ovi i odluke sesije 7: redoslijed, B13, stil lika), pitanja iz nacrta
[plana platforme](./2026-11-platforma.md) §7 (samo pokazivač E3) i stvari koje tek dolaze (L1-13 donosi vlastitu anketu s
izmjerenim brojkama nakon A1–A4).

## Leonov pregled · A — pravila igre i brojke

Dio indeksa [2026-10-17-leonov-pregled.md](./2026-10-17-leonov-pregled.md). Odgovor: broj + `ok` / `ne` / `napomena`.
A1–A4 koče L1-13 (jednu zajedničku kalibraciju brojki), A5–A6 koče spajanje PRAVILO-a; ostalo je redom vrijednosti.

#### A1 · Popravak u situaciji: „oko +10" ili manje
- **Što je:** kad u situaciji izabereš hitni popravak (klima, bojler…), stanje tog odjela (0–100) poraste. Ti si 14.10.
  rekao „oko +10" i 15.10. „mehanizam sad, iznos u L1-13".
- **Pretpostavili smo:** privremeno **+1**, jer već +2 ruši provjeru težine situacija (tjedno hotel B 57,6 % uz granicu 50 %).
- **Preporuka:** cilj „oko +10" ostaje; L1-13 ga uklapa uz rjeđe kvarove (A2) i slabiji učinak po situaciji (A3) i donosi
  ti tablicu s 3–4 izmjerene inačice prije spajanja. Uvjet: ekran odjela popravak pokazuje kao popravak, ne kao „održavanje".
- **Inače:** iznos koji stane bez drugih promjena (+1 do +2). **Cijena:** student popravak ne osjeti.
- **Vidi:** [ADR-050](../records/DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md), plan popravka red L1-13.

#### A2 · Koliko često dolaze kvarovi
- **Što je:** kvarovi (klima, bojler, praonica, brave, curenje vode, dizalo) su situacije; javljaju se samo dok je stanje
  Smještaja ispod 65 odnosno 80. Što češće dolaze, to veći dio napretka nose situacije.
- **Pretpostavili smo:** čestoću u predlošcima (koliko puta po sezoni, koliko vjerojatno) pisali su agenti; nitko je nije
  usporedio s praksom.
- **Preporuka:** napiši grubu brojku iz prakse: koliko ozbiljnih kvarova ima hotel 4\* od 120 soba od lipnja do rujna,
  (a) dobro održan, (b) zapušten. L1-13 je uzima kao cilj.
- **Inače:** „ne znam" → L1-13 bira čestoću koja drži težinu u 15–25 %. **Cijena:** brojka nije iz prakse.
- **Vidi:** [ADR-052](../records/DECISIONS/ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md) t. 1,
  [ADR-054](../records/DECISIONS/ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md) t. 1.

#### A3 · Kako spustiti težinu situacija na 15–25 %
- **Što je:** težina = koliki dio razlike između dobre i loše sezone dolazi od odgovora na situacije, a ne od plana
  (cijena, ljudi, održavanje). Ti si odlučio ~20 %, pojas 15–25 % (ADR-042 t. 15).
- **Danas:** tjedna igra 41,6 / 48,9 / 41,2 % (hoteli A / B / C), mjesečna 25,7 / 33,8 / 20,3 %. Tjedna igra ima 3–5
  situacija po tjednu, oko 65 po sezoni.
- **Preporuka:** **broj situacija ostaje, slabi učinak svake** (manje novca i bodova po odgovoru): student i dalje svaki
  tjedan vježba razgovor s gostom i zaposlenikom.
- **Inače:** manje situacija po tjednu (npr. uvijek 3). **Cijena:** manje vježbe; ili oboje pomalo.
- **Vidi:** [ADR-049](../records/DECISIONS/ADR-049-situacije-dvadeset-posto-mjeri-se-od-ravnala.md),
  `.superpowers/sdd/2026-10-14-popravak-temelja/b4c-anketa-brojke.md`.

#### A4 · Samo najbolji odgovor troši novac
- **Što je:** u 11 od 12 takvih predložaka iz B4b i 4 od 6 iz B4c samo najbolji odgovor gostu nešto daje (naknadu, piće,
  popust). Pravilo „biraj jedini koji gostu nešto daje" pogodilo je najbolji 4 od 5 puta u pokusu.
- **Pretpostavili smo:** ništa; tekst to ne može sakriti, to je mehanika.
- **Preporuka:** **jednom slabijem odgovoru dati trošak** (npr. pretjeranu naknadu, kao drugi odgovor u P-KLIMA): uči da
  novac nije isto što i dobar postupak. Radi se u L1-13 (sadržaj i balans).
- **Inače:** ostaviti. **Cijena:** pametan student pogađa najbolji bez čitanja, i nakon miješanja odgovora.
- **Vidi:** [BACKLOG](../records/BACKLOG.md) „Otvoreno za Leona", `b4b-pregled-tekstova.md` (Napomene voditelja).

#### A5 · Pravilo vlastitog odgovora: pet pretpostavki (jedno „ok")
- **Što je:** tvoja odluka 15.10.: klik na ponuđeni odgovor najviše 4, svojim riječima do 5. Kartica PRAVILO dodala je:
  a. vrijedi zadano za **nove** v2 igre (profesor bira kad dobije ekran igre);
  b. **roboti nisu ograničeni**: u demu robot klikom dobije 5, student 4 (u razredu robota nema);
  c. **prepisan ponuđeni odgovor** računa se kao klik ako s njim dijeli barem 70 % riječi, u oba smjera; redoslijed riječi
     ne spašava, sinonimi prolaze (njih hvata tek novi sudac u C3);
  d. **AI ne stigne ocijeniti** napisani odgovor → 4, bez troška;
  e. vrijedi tjedno i mjesečno, i u obveznim situacijama; stare probne v2 igre opet ne mogu dalje (nova verzija jezgre 2.5.0).
- **Preporuka:** ok svih pet.
- **Inače:** (b) ograničiti i robote. **Cijena:** ~2 h, mijenjaju se mjere robota i test „igra i balans daju isto".
  (c) 60 % (strože) ili 80 % (blaže): jedna brojka.
- **Vidi:** [ADR-055](../records/DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md) t. 1,
  [kartica PRAVILO](../archive/2026-10-14-popravak-temelja-kartice/dio-4.md#task-pravilo-vlastiti-odgovor-vrijedi-više-od-klika-klik-najviše-4-vlastite-riječi-do-5) „Rizik i odluke".

#### A6 · Zalijepljeni odgovori: kada se smatra da je zalijepio
- **Što je:** ti si 16.10. (sesija 7) odlučio: tko u polje zalijepi više ponuđenih odgovora dobije neutralnu 3, bez učinka
  (kao šutnja) i poruku „to nije tvoja odluka". Ostaje odrediti granicu.
- **Pretpostavili smo:** tekst pokriva **barem dvije** ponuđene oznake, svaku s barem 70 % njezinih **parova susjednih
  riječi** (pregled PRAVILO-a 17.10.: pojedinačne riječi lažno su hvatale dobar vlastiti odgovor u obveznoj situaciji; parovi
  na cijelom sadržaju 0 lažnih od 229, a sve zalijepljene uhvate); ta provjera ide prije provjere „prepisao jedan".
- **Preporuka:** ok.
- **Inače:** već jedna cijela oznaka i pola druge. **Cijena:** student koji citira dio odgovora pa ga objasni može biti
  lažno uhvaćen.
- **Vidi:** kartica PRAVILO (Cilj, test 13), [BUGS](../records/BUGS.md) B13.

#### A7 · Broj situacija ne pada kad je hotel prazan
- **Što je:** u krugu dolazi 3–8 situacija (prve 3 su obvezne); danas tjedno 3–5, mjesečno 4–6.
- **Pretpostavili smo:** broj **ne ovisi o popunjenosti** (izvedeno, nisi pitan); barem jedna prilika (ponuda, pohvala) u
  svakom krugu, u mjesečnoj igri od srpnja.
- **Preporuka:** ok: svaki krug jednako vježbe; puniji hotel ionako ima više kvarova kroz uvjete stanja.
- **Inače:** broj raste s popunjenošću. **Cijena:** srpanj i kolovoz teži, lipanj i rujan prazniji; ponovni balans.
- **Vidi:** [ADR-051](../records/DECISIONS/ADR-051-pretpostavke-faze-b-situacije-i-mjerenje.md) t. 2–3.

#### A8 · Šutnja (situacija bez odgovora)
- **Što je:** situacija bez odgovora daje ocjenu 1 i posljedice „nitko nije reagirao".
- **Pretpostavili smo:** tako od 05.10.; ponuda agencije (S3) bez odgovora propada bez kazne. Pravilo „šutnja nikad nije
  bolja od odgovora" danas krši 16 predložaka; to popravlja L1-13.
- **Preporuka:** ok, uz L1-13 koji tih 16 popravlja.
- **Inače:** i S3 bez odgovora nosi kaznu. **Cijena:** jedna izmjena sadržaja.
- **Vidi:** [ADR-015](../records/DECISIONS/ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md) t. 2.

#### A9 · Radnici iz Azije: plaća i obuka
- **Što je:** tvoja želja 16.10.: radnici iz Azije (Nepal, Indija, Filipini) s plaćom 800–900 €, bez pada morala ostalima,
  i puno teža obuka. Sudara se s [MODEL_V2](../architecture/MODEL_V2.md) §5.4 („ista plaća za istu vještinu"; razlika
  po podrijetlu u Hrvatskoj je protuzakonita) i s 🟨 pravilom iz ADR-028: podrijetlo određuje samo činjenice (jezike,
  trošak agencije, smještaj, dozvolu), nikad vještinu ni plaću.
- **Pretpostavili smo:** ništa nije građeno (dolazi s burzom rada L2a-6 i agencijom L2b-3).
- **Preporuka (voditelj):** isti ishod bez pravila po podrijetlu: plaća po **ulozi i razini** (agencijski početnici na
  najnižoj plaći uloge, ≈ 800–900 €), obuka sporija po **znanju hrvatskog i engleskog** (Azija 0–1, BiH, Srbija, Crna Gora 4),
  moral ostalih se ne mijenja jer se njihova plaća ne mijenja; plaća ispod najniže za ulogu = odluka s rizikom inspekcije.
  Student vidi istu stvarnost, a igra ne uči da je plaća po zastavi pravilo.
- **Inače:** pravilo po podrijetlu kako si rekao. **Cijena:** nastavni materijal fakulteta pokazuje nejednaku plaću po
  nacionalnosti kao pravilo igre; to bi trebao odobriti i profesor.
- **Vidi:** [ADR-028](../records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md) (pravilo voditelja), BACKLOG.

#### A10 · Koliko ljudi treba hotelu 4\*
- **Što je:** raspon ljudi na 100 soba po odjelu; igra provjerava da su početni hoteli u tom rasponu.
- **Pretpostavili smo [procjena]:** Smještaj 17–25, Uprava 5–7, Prodaja i marketing 1,5–3, Održavanje 3–6. Hoteli 3\* i 5\*
  još nemaju brojke.
- **Preporuka:** potvrdi ili ispravi iz prakse; to znaš bolje od bilo kojeg izvora koji imamo.
- **Inače:** pitati profesore (E2). **Cijena:** čeka odgovor.
- **Vidi:** [ADR-032](../records/DECISIONS/ADR-032-sloj-1-odjeli-u-obracunu-i.md) t. 9, `content/hotel/v2/departments.json`.

#### A11 · Brojke odjela koje student vidi u računu dobiti
- **Pretpostavili smo [procjena, hotel 4\* na moru]:** restoran: prosječni račun 35 €, nabava hrane i pića 30 % prihoda;
  wellness: plaća terapeuta 2.500 €, materijal 5 € po tretmanu, nabava 15 %; radnik Smještaja košta hotel 2.500 € mjesečno
  (bruto ≈ 1.750 € + doprinosi + smještaj i hrana sezonaca); Uprava 3.500 €, Prodaja 3.300 €, Održavanje 2.900 € po osobi;
  ostali prihodi (hrana, piće, usluge) 25–35 % prihoda od soba; u tjednoj igri najviše ±3 sezonska radnika po tjednu.
- **Preporuka:** potvrdi ili ispravi one koje znaš (npr. „A11 restoran 40 €"); ostale ostaju procjena.
- **Vidi:** [ADR-033](../records/DECISIONS/ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md) t. 2–7.

#### A12 · Stare brojke v1 (demo od 04.10.)
- **Što je:** BACKLOG još drži 9 pitanja o brojkama prve igre (radnik 2.200 €, 30 stalnih + 20 sezonaca, najviše 50 sezonaca,
  25 % popusta agenciji u S3, sezonska marža 35,8 %, fiksni troškovi hotela B, priče hotela A i B, obnova 150.000 €,
  ostali prihod 30 %).
- **Pretpostavili smo:** v1 je zamrznut (stare igre se igraju do kraja, nova igra je v2 i ne mijenja se odgovorom na v1).
- **Preporuka:** **zatvoriti** ih za v1; iste brojke za v2 su A10 i A11, a priče hotela A i B i obnovu provjeriš kad igraš
  sezonu u L1-14 („prepoznajem li ovaj hotel?").
- **Inače:** odgovoriti i za v1. **Cijena:** ništa se u igri ne bi promijenilo.
- **Vidi:** BACKLOG „Otvoreno za Leona", [DEMO §12](../product/DEMO/12-cjelobrojni-racun.md) „Za Leonovu provjeru".

#### A13 · Ono što već igraš (jedno „ok")
- **Što je:** pretpostavke koje student osjeti, a ti ih vidiš u igri od 10.–13.10.:
  a. napredak 50 = prosječan hotel bez odluka (ADR-013 t. 1); b. kad AI zakaže, igra piše rezervni tekst, obračun čeka
  presude najviše 120 s (ADR-019 t. 4–5); c. slobodan odgovor prvi, promjene indeksa „od početka" (ADR-022);
  d. mentor u četiri dijela, do ~170 riječi, brojka uvijek s imenom i jedinicom (ADR-023); e. presuda ne spominje redne
  brojeve (ADR-025); f. odjeli ovise o hotelu, zaposlenicima upravlja student, recenzije iz banke rečenica (ADR-026 t. 4);
  g. raspored za računalo (ADR-027); h. tjedno se stanje miče barem za 1 prema cilju, a plan se prenosi iz tvog zadnjeg
  (ADR-031 t. 2, 4); i. sezona uvijek cijela, situacija mjeseca u prvom tjednu mjeseca, odjel bez vlastitog „Zašto"
  (ADR-029 P6, P13, P18); j. ekran Odjeli: usklađenost ljudi, rečenice, najam, cijena uz odluku, nespremljene izmjene
  (ADR-035 t. 4–10); k. kraj sezone: dijeljeno mjesto, sezonski mentor (ADR-036 t. 3–9); l. lik zaključuje razgovor u
  5. i 6. poruci (ADR-048).
- **Preporuka:** ok sve. Ako ti je nešto smetalo dok si igrao, napiši slovo („A13 ok osim d").

## Leonov pregled · B — tekstovi odgovora (B4) i mjerilo AI-ja

Dio indeksa [2026-10-17-leonov-pregled.md](./2026-10-17-leonov-pregled.md). B1 koči L1-15 (završnu provjeru faze B);
B2 čitaš tek **nakon** slijepog izbora.

#### B1 · Slijepi pregled tekstova B4a, B4b, B4c
- **Što je:** agenti su prepisali ponuđene odgovore u 59 situacija (B4a 19, B4b 22, B4c 18) tako da najbolji ne odskače
  duljinom ni oblikom. Ti si mjera: ako najbolji pogodiš po obliku, a ne po sadržaju, tekst nije dobar.
- **Kako:** u mapi `.superpowers/sdd/2026-10-14-popravak-temelja/` otvori `b4a-pregled-tekstova.md`, pa `b4b-…` i `b4c-…`.
  Odgovori su izmiješani i označeni slovima. Za svaku situaciju upiši `Moj izbor najboljeg:` slovo, pa `Odluka:` ✔
  (prihvaćam) ili ✎ s tvojim tekstom. **Odjeljke „Ključ" i „Napomene voditelja" na kraju ne gledaj prije izbora.**
  Na kraju usporedi s Ključem i uz promašaje napiši je li te prevario oblik ili sadržaj.
- **Koliko:** ~1–1,5 min po situaciji: B4a ~25 min, B4b ~30 min, B4c ~25 min; slobodno u tri sjedenja.
- **Preporuka:** sve tri cijele, prije L1-15.
- **Inače:** samo situacije „s napomenom" (5 + 14 + 14 = 33). **Cijena:** izbor više nije slijep, a ostalih 26 nitko ne
  provjeri.
- **Odgovor:** upisuješ izravno u te tri datoteke; voditelj ih čita i šalje jedan krug ispravaka (B4-ispravci, ~1 h,
  brojke igre ostaju iste).

#### B2 · Napomene koje već znamo (tek nakon B1)
- **Što je:** pregledi su našli sitnice koje ne ruše igru, ali ih ti vidiš kao student:
  a. **B4a (N1):** tri slabija odgovora nose radnju koju rubrika nagrađuje, a nemaju kriterij: P-PRISTUPACNOST („rampa u
     planu"), P-KRIZA-OSOBLJA („za dva tjedna"), P-BOJLER („provjeriti ostale"). Bodovi su isti, ali student koji napiše tu
     radnju može biti uparen sa slabijim odgovorom. Ostale napomene B4a stoje u Ključu (S3, S4, P-POHVALA,
     P-BUCNA-SOBA, P-PRISTUPACNOST).
  b. **B4a (N4):** P-BOJLER, slabiji odgovor „20 % popusta tjedan dana" ne kaže na što je popust.
  c. **B4b:** P-DIJETE-IZGUBLJENO: najbolji ima sve korake drugog i još neke („biraj onaj s najviše koraka" pogađa);
     P-SLOBODNI-DANI: slabiji doslovno sadrži kriterij najboljeg („pisani zahtjevi"); P-PREPORUKA: „njezinoj tvrtki",
     „15 % popusta".
  d. **B4c:** nakon popravka nema čistog slučaja „najbolji = slabiji + još nešto", ostala su 3 granična (P-POSEBNA-PREHRANA
     kruh, P-LEZALJKE i P-SPA-TERMIN razlikuju se samo isprikom); test dopušta najviše 2 od 18 čistih.
  e. **Svih 59 (N3):** „biraj odgovor čije riječi ima i neki drugi" pogađa najbolji u 21 od 57 (slučajno ~27 %).
- **Preporuka:** a, b, c popraviti u B4-ispravcima nakon tvog pregleda; d prihvatiti; e rješava miješanje odgovora u L1-13b.
- **Inače:** što ti u slijepom pregledu nije smetalo, ostaviti.
- **Vidi:** „Napomene voditelja" na kraju `b4b-` i `b4c-pregled-tekstova.md`,
  [ADR-055](../records/DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md) t. 6.

#### B3 · Sitnice sadržaja situacija
- **Što je:**
  a. P-RANI-DOLAZAK: opis kaže „jedna gošća", odgovori „Stigli su";
  b. rezervni tekst lika „Gošća je previše ljuta da bi nastavila razgovor" zvuči kao ocjena studenta;
  c. S3 (ponuda agencije) nema opis (samo naslov), a S3 i S4 nemaju rubriku (kriterije za ocjenu);
  d. objašnjenje indeksa „Zaposleni" predugo je i tehničko.
- **Preporuka:** a, b, d voditelj popravi sam (samo tekst, npr. b → „Gošća je prekinula razgovor."); za c agent napiše
  nacrt opisa S3 i rubrika S3 i S4, a ti potvrdiš (~15 min).
- **Inače:** ti napišeš rubrike S3 i S4 sam.
- **Vidi:** BACKLOG „Sadržaj situacija", `content/hotel/situations.json`, `content/hotel/catalog.json`.

#### B4 · Mjerilo AI suca: tvojih 40 ocjena
- **Što je:** 40 nacrta studentskih odgovora za S1 i S2; ti upišeš ocjenu 1–5 (`leon_quality`), a stroj izmjeri slaže li se
  AI sudac s tobom (cilj: barem 80 % ocjena unutar ±1 od tvoje).
- **Pretpostavili smo:** da je sudac dobar, bez te mjere.
- **Preporuka:** da, ~40 min, prije C3 (novi sudac koji ocjenjuje i razgovor) i prije svake zamjene modela suca.
- **Inače:** kasnije. **Cijena:** ne znamo koliko sudac griješi, pa ne možemo ni probati jeftiniji model.
- **Vidi:** `content/hotel/mjerilo/S1.json`, `S2.json`;
  [ADR-047](../records/DECISIONS/ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md) t. 5.

#### B5 · Rubrike sidara S1 i S2 i stari nacrt S1b, S5–S8
- **Što je:** rubrika = kriteriji po kojima sudac ocjenjuje odgovor; ti si mjerilo. S1b i S5–S8 su nacrt iz prve igre
  (drugi val), napisan prije nego što je v2 dobila 55 predložaka.
- **Preporuka:** pročitaj kriterije S1 i S2 (~10 min; S3 i S4 su u B3c); nacrt S1b, S5–S8 **zatvoriti** (v1 je zamrznut,
  v2 ima svoje predloške).
- **Inače:** prepisati S1b, S5–S8 u v2 predloške. **Cijena:** ~2–3 h i ponovni balans.
- **Vidi:** [SITUACIJE_S1b_S5-S8](../product/SITUACIJE_S1b_S5-S8.md), DEMO §12 „Za Leonovu provjeru" t. 5.

## Leonov pregled · C — banka ljudi i izgled likova

Dio indeksa [2026-10-17-leonov-pregled.md](./2026-10-17-leonov-pregled.md). **Banka ljudi** je popis izmišljenih radnika,
gostiju i vanjskih osoba iz kojih igra uzima likove ([BANKA_LJUDI](../architecture/BANKA_LJUDI.md)). **Serija** je jedna
skupina zapisa; **pečat** znači da se serija zaključa i više ne mijenja (stroj to čuva otiskom). Dok serija nije
zapečaćena, ispravak je besplatan. C1–C2 koče pečat; ostalo su sitnice i vanjski likovi.

#### C1 · Pregled 120 ljudi i pečat serija
- **Što je:** serija-000 (10 radnika + 10 gostiju) i serija-001 (50 + 50) su OTVORENE i čekaju tvoj pregled.
- **Kako:** otvori `.superpowers/sdd/banka-ljudi-k1/pregled-svi.html` (lica, filtri po vrsti, ulozi, zemlji, rodu i dobi)
  ili `banka-ljudi.xlsx` u istoj mapi (listovi Radnici 60, Gosti 60, Vanjski) i u stupac **Odluka** upiši `OK`,
  `ispravi: …` ili `izbaci`. Za prvih 20 bilješke su u `pregled-20.md`. Gledaj ono što stroj ne vidi: zvuči li ime stvarno
  za zemlju i rod, padež i rod u opisu, skriveni stereotip, je li zanimanje gosta stvarno.
- **Koliko:** ~40 s po zapisu, 70–90 min ukupno.
- **Preporuka:** svih 120 sada (tvoja odluka 14.10.: prva serija cijela, poslije uzorak 10 po seriji); zatim glavna sesija
  pečati obje serije. Lice na stranici je privremeni crtić (C11): ocjenjuješ zapis, ne lice.
- **Inače:** uzorak 20 iz serije-001. **Cijena:** odstupa od tvoje odluke; neviđeni zapisi idu u igru.
- **Vidi:** [ADR-042](../records/DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 8.

#### C2 · Rod po ulozi
- **Što je:** udio žena u svakoj ulozi. Ti si odlučio: stvarni omjer u hotelijerstvu, izvor Eurostat (anketa o radnoj snazi).
- **Pretpostavili smo:** brojke su **prijedlog, još nisu izmjerene iz Eurostata**: žene među sobarima 90 %, recepcija 70,
  uprava 75, terapeuti 75, prodaja 65, konobari 50, kuhari 45, šankeri 35, tehničari 5 %. Zato je danas 0 od 5 tehničara
  žena, 15 od 17 sobara žena, a ukupno 62 % žena (cilj banke 55 %).
- **Preporuka:** brojke prihvatiti privremeno i zapečatiti serije; agent izmjeri Eurostat (~1 h) prije sljedeće serije, a
  nove serije prate izmjereno. Tehničarku ne dodavati na silu: u većoj seriji doći će po omjeru.
- **Inače:** jedna tehničarka već sada. **Cijena:** jedan zapis, ništa drugo.
- **Vidi:** [ADR-052](../records/DECISIONS/ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md) t. 3,
  [ADR-055](../records/DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md) t. 3b,
  `.superpowers/sdd/banka-ljudi-k1/k2-review.md` pitanje 1.

#### C3 · Imena koja možda pripadaju poznatim osobama
- **Što je:** lik ne smije nositi ime stvarne javne osobe. Dva su već zamijenjena (Markus Wieser → Leitgeb, Patrizia
  Moretti → Sartori). Nesigurni: Tamara Kinkela, Nensi Šuran, Balša Vujačić, Dragan Šipka, Sajeev Menon (radnici);
  Kees Bakker, Alain Girard, Helmut Steiner, Christina Wolf, Malin Lundgren (gosti).
- **Preporuka:** agent ih provjeri pretraživanjem interneta (~20 min, 0 €) i predloži zamjene; ti samo kažeš ok.
- **Inače:** provjeriš sam.

#### C4 · Hoteli Kvarnera na popisu zabranjenih imena
- **Što je:** `zabranjeno.json` drži imena koja likovi ne smiju spominjati (stvarni hoteli, lanci, osobe). Danas 19 hotela,
  gotovo svi iz Opatije i Rijeke. Sumnjivi: „Excelsior" (poznat iz Dubrovnika i Zagreba), „Astoria" (nepotvrđen na
  Kvarneru), „Istra" (opće ime). Nema Krka, Lošinja, Raba ni Crikvenice.
- **Preporuka:** sumnjive ostaviti (višak zabranjenog ne šteti), a agent dopuni hotele otoka i Crikvenice iz javnog popisa
  (~20 min).
- **Inače:** izbaciti sumnjive. **Cijena:** nijedna, osim ako je neki ipak stvaran.

#### C5 · Imena profesora FMTU-a
- **Što je:** lik ne smije nositi ime stvarnog profesora; danas je na popisu samo „Leon Kreso".
- **Preporuka:** pošalji popis ili poveznicu na stranicu nastavnika; agent ih prepiše u `zabranjeno.json`.

#### C6 · Strani gradovi i imena na hrvatskom
- **Pretpostavili smo:** hrvatski oblici gradova (Trst, Pečuh, Krakov, Segedin, Đur, Lavov) i hrvatski zapis ćiriličnih
  imena (Andrij Savčuk).
- **Preporuka:** ok (dosljedno, tako piše i hrvatski tisak).
- **Inače:** izvorni oblik (Szeged, Győr, Lviv) u mjestu i u opisu.

#### C7 · „Lokalno" za doseljene
- **Pretpostavili smo:** podrijetlo = gdje osoba živi. Tko se preselio na Kvarner (Zagreb → Rijeka, Dubrovnik → Ika,
  Vinkovci → Njivice) je „lokalno", a opis kaže odakle je.
- **Preporuka:** ok.
- **Inače:** po mjestu rođenja. **Cijena:** tri zapisa i jedno pravilo.

#### C8 · Sitnice zemalja i jezika (jedno „ok")
- a. Uzbekistan kao izvor radnika (s Nepalom, Indijom, Filipinima i Bangladešom „Azija") dok ne stignu podaci MUP-a;
  b. gosti iz „ostalih" zemalja po jedan (Francuska, Irska, Švedska); c. talijanski 4 kod dvoje Istrana (talijanska
  zajednica); d. gost iz Dresdena (57) govori ruski 2 (škola u Istočnoj Njemačkoj); e. svih 50 novih gostiju ima zanimanje.
- **Preporuka:** ok sve; zemlje se zamijene kad agent izmjeri MUP-ove dozvole (tvoja odluka 16.10.).
- **Inače:** napiši slovo.

#### C9 · Vanjski likovi: tko dolazi (pitanja 12–13 iz BANKA_LJUDI §6)
- **Što je:** treća vrsta u banci (tvoja odluka 16.10.): inspekcija, policija, vatrogasci i slični.
- **12. Isti vanjski u dva hotela iste razredne igre?** Preporuka: **da, ali rijetko** (jedan sanitarni inspektor pokriva
  cijelo mjesto). Inače nitko dvaput. **Cijena:** ~800 zapisa umjesto 600.
- **13. Koje uloge?** Preporuka: **svih 14** (uz postojeće i 5 novih: turistički inspektor, inspektor rada, porezni
  inspektor, djelatnik hitne, grad ili turistička zajednica). Inače samo uloge koje igra već ima, uz policiju i vatrogasce.
- **Vidi:** [BANKA_LJUDI §6](../architecture/BANKA_LJUDI/06-pitanja.md), §1.5.

#### C10 · Vanjski likovi: u igri i brojke (pitanja 14–16 i pretpostavke)
- **14.** Policija, vatrogasci i hitna kao likovi: preporuka **nova uloga u postojećim predlošcima** (P-DIJETE-IZGUBLJENO,
  P-DIZALO, P-GOST-POZLILO) nakon popravka temelja; inače posebni novi predlošci.
- **15.** Brojke 150 / 400 / 600 (prekretnice M1 / M2 / M3): preporuka da.
- **16.** Uniforma na liku policije, vatrogasaca i hitne, iz uloge: preporuka da.
- **Pretpostavke:** službe, dobavljači i susjedi samo iz Hrvatske (stranci samo u tržišnim ulogama, ~30 %); dob 20–80,
  službe do 65, inspektori i hitna od 25; ton (strog ili blag) daje opis lika u situaciji, ne zapis u banci.
- **Preporuka:** ok sve.
- **Vidi:** `.superpowers/sdd/banka-ljudi-k1/nacrt-vanjski-report.md` (13 pretpostavki; tehnički dio je u F1).

#### C11 · Izgled lika: što student vidi
- **Što je:** tvoja odluka 16.10.: crtić iz roda, dobi i podrijetla (cigla IZGLED). Tri pretpostavke se vide:
  a. dok igra ne uzima ljude iz banke, podrijetlo se zna samo po popisu imena: osoblje „regija", gosti „strani"; nijedan
     ne dobiva tamnu put, tamniji tonovi dolaze tek iz banke sa zemljom (zato je Nepalka na stranici pregleda svijetle puti);
  b. dob iz riječi u opisu uloge (umirovljenica, baka…), inače odrasla; na stranici pregleda: mlad do 29, odrasla 30–54,
     stariji od 55;
  c. brada kod otprilike svakog trećeg muškarca, naušnice kod svake druge žene, frizure po rodu.
- **Preporuka:** ok; kad IZGLED bude spojen, pogledaj 10–20 lica jednog kruga (~10 min) i reci što ne valja.
- **Vidi:** [kartica IZGLED](../archive/2026-10-14-popravak-temelja-kartice/dio-4.md#task-izgled-izgled-lika-iz-roda-dobi-i-podrijetla) „Rizik i odluke" 2, 3, 6;
  [IZGLED_LIKA](../architecture/IZGLED_LIKA.md).

## Leonov pregled · D — voditelj odjela i tržište · E — platforma i ostalo

Dio indeksa [2026-10-17-leonov-pregled.md](./2026-10-17-leonov-pregled.md). Ništa od ovoga ne koči popravak temelja;
D ide u fazu C ili LJUDI, E u kolosijek PLATFORMA.

### D — voditelj odjela i uvid u tržište

Tvoje dvije ideje od 13.10.; nacrt s cijenama: [VODITELJ_I_TRZISTE](../architecture/VODITELJ_I_TRZISTE.md) §3.

#### D1 · Imenovani voditelji odjela: sada ili kasnije
- **Što je:** svaki odjel dobije ime osobe koja ga vodi („Odgovorna: Ivana Kovač", situacije se prepuštaju njoj), bez nove
  mehanike (~4–6 h). Prava mehanika (vještine, „prepusti cijeli odjel") je cigla L2a-5.
- **Preporuka:** imena **kad igra počne uzimati ljude iz banke** (K5), ne prije: voditelji dolaze iz banke, pa se imena ne
  pišu dvaput; do tada popravak temelja.
- **Inače:** sada, iz posebnog popisa. **Cijena:** ~4–6 h i imena koja banka poslije zamijeni.

#### D2 · Što znači „voditelj je odgovoran"
- **Preporuka:** (c) oboje, u L2a-5: vodi odluke odjela kad mu ih prepustiš **i** ocjena odjela je njegova ocjena (prema njoj
  ga nagrađuješ, zadržavaš ili mijenjaš).
- **Inače:** (a) samo vodi odluke ili (b) samo nosi ocjenu. **Cijena:** manje posla, ali pola lekcije o delegiranju.

#### D3 · Gdje student vidi tržište (koliko gostiju dolazi i zašto)
- **Što je:** igra već računa koliko je gostiju zainteresirano, koliko je smješteno i koliko odbijeno, ali student to vidi
  samo u rečenici. Prijedlog je graf kroz sezonu i rastav „sezona donosi X, tvoja cijena −8 %, ugled +3 %…".
- **Preporuka:** kartica „Tržište" na ekranu s pokazateljima (popunjenost, ADR, RevPAR su već ondje), bez nove stavke izbornika.
- **Inače:** zaseban ekran (više mjesta za graf, ~2 h više, nova stavka izbornika) ili Moj hotel (već pun).

#### D4 · Jedinica na grafu
- **Preporuka:** noćenja (hotelska mjera), uz broj gostiju u zagradi.
- **Inače:** samo gosti (razumljivije, ali nije mjera koju će koristiti u hotelu).

#### D5 · Prognoza prije odluke
- **Preporuka:** samo sezonska očekivanja („kolovoz: potražnja veća od broja soba"); kako cijena mijenja broj gostiju
  student uči iz rastava nakon kruga.
- **Inače:** i „ako postaviš cijenu X, očekuj oko Y noćenja". **Cijena:** igra postaje lakša (student isproba cijenu
  prije zatvaranja kruga), iako jasnije uči tu vezu.

### E — platforma i ostalo

#### E1 · Prijava: sesije umjesto JWT-a (L10)
- **Što je:** kako server pamti da je netko prijavljen. **Sesija:** server drži popis prijava, a preglednik samo ključ u
  kolačiću; odjava ili oduzimanje pristupa vrijedi odmah. **JWT:** potpisana potvrda koju nosi preglednik; teže ju je
  opozvati prije isteka.
- **Preporuka:** sesije. Vidim da upravo uređuješ `PLATFORMA/03-prijava.md`; ako tvoja izmjena već odgovara, napiši
  „E1 vidi 03".
- **Inače:** JWT. **Cijena:** sporija odjava i oduzimanje pristupa.
- **Odgovor ide u:** novi ADR i ispravak ARHITEKTURA §12 ([PLATFORMA dio 08](../architecture/PLATFORMA/08-faze-i-pitanja.md), L10).

#### E2 · Pitanja za profesore
- **Što je:** pismo profesorima (brojke, težine, AI) „spremno za slanje nakon tvog pregleda"; bilo je predviđeno 5.–11.10.
- **Preporuka:** ako još nije poslano: dopuni godinu studija uz potpis, odluči „Vi" (pojedinačno) ili „vi" (zajednički) i
  spominješ li zainteresiranog profesora, pa pošalji sada: pitanja 10–12 o brojkama hrane A10, A11 i L1-13.
- **Inače:** poslati uz verziju za profesore (~siječanj). **Cijena:** brojke L1-13 ostaju procjene.
- **Vidi:** [PITANJA_ZA_PROFESORE](../product/PITANJA_ZA_PROFESORE.md) „Za nas".

#### E3 · Plan platforme: šest pitanja ondje
- **Što je:** nacrt [plana platforme](./2026-11-platforma.md) (16.10.) ima vlastitih 6 pitanja s preporukom (§7): prijava za
  P3, kako profesor dobiva pristup za probu, Railway za P3 (~5–10 €/mj), ritam dvaju kolosijeka, redoslijed igre od
  studenog, vlastite situacije profesora. Pitanje o hostingu iz BACKLOG-a ondje je 3. pitanje.
- **Preporuka:** odgovori ondje, istim oblikom („P1 ok, P3 b"); ovdje se ne ponavljaju.

#### E4 · Potvrda arhitekture (Rust + TypeScript)
- **Što je:** [ADR-003](../records/DECISIONS/ADR-003-arhitektura-rust-jezgra-backend-typescript-sucelje.md) je još „nacrt":
  ARHITEKTURA kaže „prijedlog dok Leon ne potvrdi", a ROADMAP kaže da si je pregledao.
- **Preporuka:** potvrdi; sve je izgrađeno na njoj, promjena bi bila tjedni rada.
- **Inače:** rasprava ostaje otvorena. **Cijena:** dva dokumenta i dalje proturječe.

#### E5 · Inkubator (rok 16.11.)
- **Preporuka:** prijaviti HotelixSim (ne Sokrat Study): ima jasnog kupca (fakultet), pilot na FMTU-u i više tvog vremena.
- **Inače:** Sokrat Study. **Vidi:** [ROADMAP](./ROADMAP.md).

#### E6 · Vizija: što bira profesor
- **Što je:** tri otvorena pitanja iz tvoje vizije 10.10.: (1) je li „branša" odjel hotela ili vrsta objekta; (2) smišlja li
  AI situacije unaprijed uz profesorovo odobrenje ili u hodu; (3) bira li profesor duljinu igre ili ritam kruga.
- **Preporuka:** ne odlučivati sada; ide u nacrt profesorova ekrana (PLATFORMA) kao dokument s raspravom. Ako imaš prvi
  osjećaj, napiši ga. Voditeljev: (1) odjel hotela, (2) unaprijed uz odobrenje, (3) ritam (već tvoja odluka ADR-028 t. 5),
  a sezona ostaje cijela.

#### E7 · Stari koncept ekrana: zatvoriti
- **Što je:** iz DIZAJN §3 još visi: ocjena na portalima do 10 ili do 5; „1/4" mjeseca; kratki nazivi kriterija; avatari.
- **Preporuka:** zatvoriti: ocjena do 10 (kao Booking; mentor već tako piše), avatari su riješeni ciglom IZGLED, a ostalo
  provjeriš kad igraš sezonu u L1-14.
- **Vidi:** [DIZAJN](../product/DIZAJN.md) §3.

## Leonov pregled · F — tehničko (jedno „ok") i već riješeno

Dio indeksa [2026-10-17-leonov-pregled.md](./2026-10-17-leonov-pregled.md). Ovo su pretpostavke koje student ne osjeti:
kako je kod složen, kako se mjeri, kako rade agenti. Svaka ima zapisanu cijenu ako je kriva; mijenja se lokalno.

#### F1 · Tehničke pretpostavke (predlažem „F1 ok" bez čitanja)

| izvor | u jednoj rečenici |
|---|---|
| [ADR-008](../records/DECISIONS/ADR-008-odluke-iz-gradnje-kostura-koje-je.md) | sedam sitnica iz gradnje kostura (grana, brana ovisnosti, `cargo fmt`, pnpm skripta, README za Windows) |
| [ADR-013](../records/DECISIONS/ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md) t. 2–4, 6–9 | pravila prve igre (v1, zamrznuta): početni F, sezonci po planu, redoslijed obračuna, grupna cijena, drugi val |
| [ADR-015](../records/DECISIONS/ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md) osim t. 2 | igra kao dnevnik događaja, zaključavanje igre po naredbi, vlastiti red poslova, server ne mijenja bazu sam, tko što smije vidjeti |
| [ADR-017](../records/DECISIONS/ADR-017-odluke-voditelja-gradnje-u-koraku-3.md) | red poslova pamti tko je preuzeo posao, nepoznat profil = greška, provjera pristupa prije provjere sadržaja |
| [ADR-019](../records/DECISIONS/ADR-019-ai-sloj-arhitektonske-pretpostavke-koraka-4.md) osim t. 4–5 | ocjenu računa kod (AI samo kaže koji je kriterij ispunjen), AI ide kroz red poslova, odgođeni limit troška |
| [ADR-026](../records/DECISIONS/ADR-026-put-b-hotel-od-brojeva-postaje.md) t. 5 | v2 je zaseban paket uz zamrznuti v1 i uvijek daje i sažetak v1 (indeksi, mentor, ekrani rade dalje) |
| [ADR-029](../records/DECISIONS/ADR-029-sloj-1-pretpostavke-plana.md) P5, P7–P12, P14–P17 | kako se bira v2, budžet, kamo se knjiže troškovi, nazivi upita; robot Brižni planira sezonce unaprijed |
| [ADR-030](../records/DECISIONS/ADR-030-raspored-dokumentacije-granice-velicine-indeks-mapa.md) | pravila dokumentacije (indeks + mapa, granice veličine, ADR po datoteci); tvoje dopune 1000 i 1200 već su ✅ |
| [ADR-031](../records/DECISIONS/ADR-031-sloj-1-tjedni-krug-u-jezgri.md) t. 1, 3, 5 | odlasci u tjednu = dio mjesečnih, mjesec situacije, verzija jezgre se diže pri izdanju |
| [ADR-032](../records/DECISIONS/ADR-032-sloj-1-odjeli-u-obracunu-i.md) t. 1–8 | kako odjeli daju brojke cijelog hotela (plan sezonaca, troškovi, prosjeci, odlasci po odjelu) |
| [ADR-033](../records/DECISIONS/ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md) t. 1, 8 | kako balans mjeri i koliko sezona vrti CI |
| [ADR-039](../records/DECISIONS/ADR-039-brana-velicine-koda-upozorenje-600-granica.md) | datoteka koda: upozorenje iznad 600 redaka, greška iznad 1000 (za `.md` si rekao 1200; predlažemo 1000 za kod) |
| [ADR-051](../records/DECISIONS/ADR-051-pretpostavke-faze-b-situacije-i-mjerenje.md) t. 1, 4–13 | mjesečni krug kao jedan blok, granica odjela, kako se mjeri težina, popravak u odjelu situacije |
| [ADR-053](../records/DECISIONS/ADR-053-pretpostavke-16-10-oznake-testovi-b4.md) t. 1–5 | test „sastojak samo u najboljem", testovi čitaju oznake iz sadržaja, tekstove piše najjači model |
| [ADR-055](../records/DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md) t. 2, 4, 5 | voditelj sam popravi jedan redak testa, svako radno stablo svoja mapa prevođenja, mjerenje prije ankete |
| kartica IZGLED, rizik 1, 4, 5, 7, 8 | rod iz popisa imena, engleski nazivi u API-ju, stari izgled bez podataka ostaje isti, sidra S1–S4 ručno |
| nacrt VANJSKI, pretpostavke 1, 5–7, 9–13 | vanjski kreću odmah, polje ustanove, raspodjele, sesija banke, nazivi u kodu (`nacrt-vanjski-report.md`) |
| banka ljudi, provjere stroja | usporedba razine, dobi i roda unutar uloge (ćelija 40 zapisa; `k2-review.md` pitanje 5), pravila banke u šest dijelova |
| web | bočni izbornik na računalu može se preklopiti sa zaglavljem kod velikog teksta: voditelj popravi bez pitanja |

- **Preporuka:** F1 ok. Ako te jedan redak zanima, napiši „F1 ok osim ADR-039" i dobit ćeš ga objašnjenog.

#### F2 · Već riješeno: brisanje iz popisa „Otvoreno za Leona"

| stavka | zašto je zatvorena |
|---|---|
| „Put B — 12 odluka prije gradnje" | odgovorio si 10.10. ([ADR-028](../records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md)) |
| „Pravilo za zalijepljene odgovore (B13)" | odlučeno 16.10., sesija 7: neutralna 3 (ADR piše `/kraj`); ostaje samo A6 |
| „Redoslijed: faza C s izgledom likova prije L1-13?" | odlučeno 16.10., sesija 7: IZGLED sada usporedo s PRAVILO-om, plan dalje L1-13 → L1-15 → C |
| ADR-028: „ljude drugih hotela student ne vidi" | zamijenilo [ADR-037](../records/DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md): u razredu su tuđi radnici vidljivi i preotimaju se |
| ADR-053 t. 6: prag 65 za tri kvara | odlučeno 16.10.: prag 80 ([ADR-054](../records/DECISIONS/ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md)) |
| ciljna marža GOP 35–45 % (ADR-013 t. 5) | tvoj odgovor 10.10. (ADR-028, odgovor 9); ostaje samo profesorova potvrda (E2) |
| pretpostavke ADR-013, 015, 017, 019 i Sloja 1 kao zasebne stavke | pokrivene s A8, A13 i F1 |

- **Preporuka:** F2 ok; voditelj ih na `/kraj` briše iz BACKLOG-a i u CLAUDE.md skraćuje popis 🟨.
