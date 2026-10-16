# Leonov pregled · A — pravila igre i brojke

Dio indeksa [2026-10-17-leonov-pregled.md](../2026-10-17-leonov-pregled.md). Odgovor: broj + `ok` / `ne` / `napomena`.
A1–A4 koče L1-13 (jednu zajedničku kalibraciju brojki), A5–A6 koče spajanje PRAVILO-a; ostalo je redom vrijednosti.

### A1 · Popravak u situaciji: „oko +10" ili manje
- **Što je:** kad u situaciji izabereš hitni popravak (klima, bojler…), stanje tog odjela (0–100) poraste. Ti si 14.10.
  rekao „oko +10" i 15.10. „mehanizam sad, iznos u L1-13".
- **Pretpostavili smo:** privremeno **+1**, jer već +2 ruši provjeru težine situacija (tjedno hotel B 57,6 % uz granicu 50 %).
- **Preporuka:** cilj „oko +10" ostaje; L1-13 ga uklapa uz rjeđe kvarove (A2) i slabiji učinak po situaciji (A3) i donosi
  ti tablicu s 3–4 izmjerene inačice prije spajanja. Uvjet: ekran odjela popravak pokazuje kao popravak, ne kao „održavanje".
- **Inače:** iznos koji stane bez drugih promjena (+1 do +2). **Cijena:** student popravak ne osjeti.
- **Vidi:** [ADR-050](../../records/DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md), plan popravka red L1-13.

### A2 · Koliko često dolaze kvarovi
- **Što je:** kvarovi (klima, bojler, praonica, brave, curenje vode, dizalo) su situacije; javljaju se samo dok je stanje
  Smještaja ispod 65 odnosno 80. Što češće dolaze, to veći dio napretka nose situacije.
- **Pretpostavili smo:** čestoću u predlošcima (koliko puta po sezoni, koliko vjerojatno) pisali su agenti; nitko je nije
  usporedio s praksom.
- **Preporuka:** napiši grubu brojku iz prakse: koliko ozbiljnih kvarova ima hotel 4\* od 120 soba od lipnja do rujna,
  (a) dobro održan, (b) zapušten. L1-13 je uzima kao cilj.
- **Inače:** „ne znam" → L1-13 bira čestoću koja drži težinu u 15–25 %. **Cijena:** brojka nije iz prakse.
- **Vidi:** [ADR-052](../../records/DECISIONS/ADR-052-leonove-odluke-16-10-kvarovi-banka-ljudi-usporedo.md) t. 1,
  [ADR-054](../../records/DECISIONS/ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md) t. 1.

### A3 · Kako spustiti težinu situacija na 15–25 %
- **Što je:** težina = koliki dio razlike između dobre i loše sezone dolazi od odgovora na situacije, a ne od plana
  (cijena, ljudi, održavanje). Ti si odlučio ~20 %, pojas 15–25 % (ADR-042 t. 15).
- **Danas:** tjedna igra 41,6 / 48,9 / 41,2 % (hoteli A / B / C), mjesečna 25,7 / 33,8 / 20,3 %. Tjedna igra ima 3–5
  situacija po tjednu, oko 65 po sezoni.
- **Preporuka:** **broj situacija ostaje, slabi učinak svake** (manje novca i bodova po odgovoru): student i dalje svaki
  tjedan vježba razgovor s gostom i zaposlenikom.
- **Inače:** manje situacija po tjednu (npr. uvijek 3). **Cijena:** manje vježbe; ili oboje pomalo.
- **Vidi:** [ADR-049](../../records/DECISIONS/ADR-049-situacije-dvadeset-posto-mjeri-se-od-ravnala.md),
  `.superpowers/sdd/2026-10-14-popravak-temelja/b4c-anketa-brojke.md`.

### A4 · Samo najbolji odgovor troši novac
- **Što je:** u 11 od 12 takvih predložaka iz B4b i 4 od 6 iz B4c samo najbolji odgovor gostu nešto daje (naknadu, piće,
  popust). Pravilo „biraj jedini koji gostu nešto daje" pogodilo je najbolji 4 od 5 puta u pokusu.
- **Pretpostavili smo:** ništa; tekst to ne može sakriti, to je mehanika.
- **Preporuka:** **jednom slabijem odgovoru dati trošak** (npr. pretjeranu naknadu, kao drugi odgovor u P-KLIMA): uči da
  novac nije isto što i dobar postupak. Radi se u L1-13 (sadržaj i balans).
- **Inače:** ostaviti. **Cijena:** pametan student pogađa najbolji bez čitanja, i nakon miješanja odgovora.
- **Vidi:** [BACKLOG](../../records/BACKLOG.md) „Otvoreno za Leona", `b4b-pregled-tekstova.md` (Napomene voditelja).

### A5 · Pravilo vlastitog odgovora: pet pretpostavki (jedno „ok")
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
- **Vidi:** [ADR-055](../../records/DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md) t. 1,
  [kartica PRAVILO](../2026-10-14-popravak-temelja/task-12-pravilo-vlastiti-odgovor.md) „Rizik i odluke".

### A6 · Zalijepljeni odgovori: kada se smatra da je zalijepio
- **Što je:** ti si 16.10. (sesija 7) odlučio: tko u polje zalijepi više ponuđenih odgovora dobije neutralnu 3, bez učinka
  (kao šutnja) i poruku „to nije tvoja odluka". Ostaje odrediti granicu.
- **Pretpostavili smo:** tekst pokriva **barem dvije** ponuđene oznake, svaku s barem 70 % njezinih riječi; ta provjera ide
  prije provjere „prepisao jedan".
- **Preporuka:** ok.
- **Inače:** već jedna cijela oznaka i pola druge. **Cijena:** student koji citira dio odgovora pa ga objasni može biti
  lažno uhvaćen.
- **Vidi:** kartica PRAVILO (Cilj, test 13), [BUGS](../../records/BUGS.md) B13.

### A7 · Broj situacija ne pada kad je hotel prazan
- **Što je:** u krugu dolazi 3–8 situacija (prve 3 su obvezne); danas tjedno 3–5, mjesečno 4–6.
- **Pretpostavili smo:** broj **ne ovisi o popunjenosti** (izvedeno, nisi pitan); barem jedna prilika (ponuda, pohvala) u
  svakom krugu, u mjesečnoj igri od srpnja.
- **Preporuka:** ok: svaki krug jednako vježbe; puniji hotel ionako ima više kvarova kroz uvjete stanja.
- **Inače:** broj raste s popunjenošću. **Cijena:** srpanj i kolovoz teži, lipanj i rujan prazniji; ponovni balans.
- **Vidi:** [ADR-051](../../records/DECISIONS/ADR-051-pretpostavke-faze-b-situacije-i-mjerenje.md) t. 2–3.

### A8 · Šutnja (situacija bez odgovora)
- **Što je:** situacija bez odgovora daje ocjenu 1 i posljedice „nitko nije reagirao".
- **Pretpostavili smo:** tako od 05.10.; ponuda agencije (S3) bez odgovora propada bez kazne. Pravilo „šutnja nikad nije
  bolja od odgovora" danas krši 16 predložaka; to popravlja L1-13.
- **Preporuka:** ok, uz L1-13 koji tih 16 popravlja.
- **Inače:** i S3 bez odgovora nosi kaznu. **Cijena:** jedna izmjena sadržaja.
- **Vidi:** [ADR-015](../../records/DECISIONS/ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md) t. 2.

### A9 · Radnici iz Azije: plaća i obuka
- **Što je:** tvoja želja 16.10.: radnici iz Azije (Nepal, Indija, Filipini) s plaćom 800–900 €, bez pada morala ostalima,
  i puno teža obuka. Sudara se s [MODEL_V2](../../architecture/MODEL_V2.md) §5.4 („ista plaća za istu vještinu"; razlika
  po podrijetlu u Hrvatskoj je protuzakonita) i s 🟨 pravilom iz ADR-028: podrijetlo određuje samo činjenice (jezike,
  trošak agencije, smještaj, dozvolu), nikad vještinu ni plaću.
- **Pretpostavili smo:** ništa nije građeno (dolazi s burzom rada L2a-6 i agencijom L2b-3).
- **Preporuka (voditelj):** isti ishod bez pravila po podrijetlu: plaća po **ulozi i razini** (agencijski početnici na
  najnižoj plaći uloge, ≈ 800–900 €), obuka sporija po **znanju hrvatskog i engleskog** (Azija 0–1, BiH, Srbija, Crna Gora 4),
  moral ostalih se ne mijenja jer se njihova plaća ne mijenja; plaća ispod najniže za ulogu = odluka s rizikom inspekcije.
  Student vidi istu stvarnost, a igra ne uči da je plaća po zastavi pravilo.
- **Inače:** pravilo po podrijetlu kako si rekao. **Cijena:** nastavni materijal fakulteta pokazuje nejednaku plaću po
  nacionalnosti kao pravilo igre; to bi trebao odobriti i profesor.
- **Vidi:** [ADR-028](../../records/DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md) (pravilo voditelja), BACKLOG.

### A10 · Koliko ljudi treba hotelu 4\*
- **Što je:** raspon ljudi na 100 soba po odjelu; igra provjerava da su početni hoteli u tom rasponu.
- **Pretpostavili smo [procjena]:** Smještaj 17–25, Uprava 5–7, Prodaja i marketing 1,5–3, Održavanje 3–6. Hoteli 3\* i 5\*
  još nemaju brojke.
- **Preporuka:** potvrdi ili ispravi iz prakse; to znaš bolje od bilo kojeg izvora koji imamo.
- **Inače:** pitati profesore (E2). **Cijena:** čeka odgovor.
- **Vidi:** [ADR-032](../../records/DECISIONS/ADR-032-sloj-1-odjeli-u-obracunu-i.md) t. 9, `content/hotel/v2/departments.json`.

### A11 · Brojke odjela koje student vidi u računu dobiti
- **Pretpostavili smo [procjena, hotel 4\* na moru]:** restoran: prosječni račun 35 €, nabava hrane i pića 30 % prihoda;
  wellness: plaća terapeuta 2.500 €, materijal 5 € po tretmanu, nabava 15 %; radnik Smještaja košta hotel 2.500 € mjesečno
  (bruto ≈ 1.750 € + doprinosi + smještaj i hrana sezonaca); Uprava 3.500 €, Prodaja 3.300 €, Održavanje 2.900 € po osobi;
  ostali prihodi (hrana, piće, usluge) 25–35 % prihoda od soba; u tjednoj igri najviše ±3 sezonska radnika po tjednu.
- **Preporuka:** potvrdi ili ispravi one koje znaš (npr. „A11 restoran 40 €"); ostale ostaju procjena.
- **Vidi:** [ADR-033](../../records/DECISIONS/ADR-033-sloj-1-balans-i-kalibracija-sadrzaja.md) t. 2–7.

### A12 · Stare brojke v1 (demo od 04.10.)
- **Što je:** BACKLOG još drži 9 pitanja o brojkama prve igre (radnik 2.200 €, 30 stalnih + 20 sezonaca, najviše 50 sezonaca,
  25 % popusta agenciji u S3, sezonska marža 35,8 %, fiksni troškovi hotela B, priče hotela A i B, obnova 150.000 €,
  ostali prihod 30 %).
- **Pretpostavili smo:** v1 je zamrznut (stare igre se igraju do kraja, nova igra je v2 i ne mijenja se odgovorom na v1).
- **Preporuka:** **zatvoriti** ih za v1; iste brojke za v2 su A10 i A11, a priče hotela A i B i obnovu provjeriš kad igraš
  sezonu u L1-14 („prepoznajem li ovaj hotel?").
- **Inače:** odgovoriti i za v1. **Cijena:** ništa se u igri ne bi promijenilo.
- **Vidi:** BACKLOG „Otvoreno za Leona", [DEMO §12](../../product/DEMO/12-cjelobrojni-racun.md) „Za Leonovu provjeru".

### A13 · Ono što već igraš (jedno „ok")
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
