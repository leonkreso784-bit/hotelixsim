# ADR-023 — Mentor govori ljudski: pripremljene činjenice, četiri dijela, brojke samo s imenom i jedinicom
**Datum:** 2026-10-10 · **Status:** 🟨 ČEKA LEONOV PREGLED (Claude, „radi bez pitanja") · **Vezano:** [ADR-019](ADR-019-ai-sloj-arhitektonske-pretpostavke-koraka-4.md),
[ADR-022](ADR-022-ekrani-igraca-pretpostavke-plana-koraka-5.md) t. 7, ARHITEKTURA §8.5 · **Plan:** [2026-10-10-ekrani.md](../../archive/2026-10-10-ekrani.md) (Task 14)

**Kontekst.** Mentor je u srpnju napisao *„najviše je pao treći (s 4876 na 4269) … −600 uz osnovnih −453 i −1000 zbog
plaća … ugled je smanjio prodaju za 200 bp … 24 395 000 centi"*. Leon: *„indeksi ne smiju biti ‚tvoj hotel je pao za
−1000‘ … to mora biti dobro objašnjeno. Ovo nije dobro."* Uzrok: AI je dobivao sirovi `DebriefInput` (indeksi kao pet
brojeva u stotinkama, retci s `bp`, `cents`, šiframa situacija) i prepisivao ga.

**Odluka (pretpostavke):**
1. **AI nikad ne vidi internu jedinicu.** `game::debrief_facts` iz `DebriefInput` gradi činjenice samo s ljudskim
   vrijednostima i hrvatskim ključevima: indeksi po imenu (G R Z O F), novac u cijelim eurima s točkom za tisuće,
   bodovi cijelim brojem, ocjena portala s dvije decimale (ljestvica do 10), situacija kao uzrok nosi naslov.
   `debrief_body` šalje samo njih; upute su `prompts/mentor.v2.md` (v1 ostaje u povijesti).
2. **Indeksi cijelim brojevima** (pola od nule: 48,50 → 49), **promjena = razlika zaokruženih**, da se slaže s ekranom.
3. **Što je pokrenulo indeks** je podatak uz formulu (`sim_hotel::index_drivers` → `Driver`, testovi drže popis uz
   `raw_indices`). Pokretač je varijabla iz objašnjenja ili **opterećenje osoblja** (`Driver::StaffLoad`, nije
   varijabla, nosi 25 % Operative): činjenice ga pišu riječima („2,3 noćenja po zaposlenom na dan, osoblje je
   preopterećeno") iz opterećenja prije i poslije mjeseca; prag i kazna su `LOAD_LIMIT_HUNDREDTHS` i `load_penalty`
   u jezgri, jedna formula za indeks i mentora.
4. **Četiri dijela** s podnaslovom u retku: Što je išlo dobro · Što nije valjalo · Zašto · Što probati u <idući
   mjesec>; isto i mentor bez AI-ja (deterministički, iz istih činjenica).
5. **Brojke samo iz činjenica i uvijek s imenom i jedinicom** („indeks Zaposlenici s 49 na 43"), najviše četiri u
   tekstu, ukupno do ~170 riječi.
6. Faktori potražnje (`bp`) pišu se kao **„−2 % potražnje"**, ne „postotni bodovi": u jezgri je to množitelj potražnje
   (10 000 = 100 %), pa je −200 stvarno 2 % manje gostiju. Kod morala uzroci su **pritisci** prema cilju, a „ukupna
   promjena morala" je ono što se dogodilo; činjenice to kažu riječima. Retci koji zaokruženo daju nulu izostaju, a isti
   uzrok iste varijable spaja se u jedan redak. Ljudi se pišu riječima („zaposleno 6 osoba", „otišle 3 osobe").
7. **Odluke i odabrani odgovori su u činjenicama:** `odluke_u_planu` (cijena prema uobičajenoj za mjesec, marketing,
   sezonci, plaće kao % uobičajene, obuka, održavanje, obnova, odgovori na recenzije) i `odabrani_odgovor` uz svaku
   situaciju (oznaka ponuđenog odgovora ili „vlastiti odgovor"). Uputa traži vezu samo s onim što je u činjenicama i
   zabranjuje pogađanje; primjer u uputi je izmišljen (test pazi da ne spominje stvarnu situaciju iz sadržaja), a sav
   tekst u JSON-u je podatak, nikad uputa.
8. **Smjer retka i težina štete** (ekran „Zašto se indeksi pomaknuli" i „Zašto" mentora bez AI-ja, Task 16 i 19):
   jedno pravilo `push` u `mentor.rs` — `effect` je predznak guranja, `harm` je njegov negativni dio (`harm > 0 ⇔
   down`). Težina = udio člana u indeksu (%) × promjena člana u stotinkama: Operativa oprema 45, iskustvo 30,
   opterećenje 25; **Zaposlenici moral 60, zadržavanje 40**. Retci potražnje (cijena, ugled, marketing) pretvaraju
   postotak u prihod od noćenja i imaju težinu najmanje 1, da se smjer ne izgubi zaokruživanjem.
   **Zaposlenici:** indeks ne miče broj otišlih nego **stopa odlazaka prema prošlom mjesecu** (za lipanj: početno
   stanje), jer je zadržavanje = 10 000 − 12,5 · stopa (`sim_hotel::retention`); pokretač „Zaposleni" zato ima redak
   „udio osoblja koje je dalo otkaz ovaj mjesec: 5,44 %, više nego prošli mjesec" (više → dolje, manje → gore,
   isto → sivo) i usporedni redak prošlog mjeseca. U prvom mjesecu igre — kaže ga igra (`DebriefInput::prev_month`
   je `None`), ne kalendar — sve usporedbe u činjenicama (odlasci, opterećenje, sezonci) kažu „na početku igre". Kad su oba
   mjeseca na 8 % i više (zadržavanje na nuli), redak je siv i to kaže („…, ali od 8 % naviše taj je dio indeksa već
   na najnižem, pa se ne miče"). Broj zaposlenih i otišlih ostaje siva razina. Stopa nije zasebna poluga: slijedi moral na kraju mjeseca (3 % do 60 bodova morala, iznad toga raste,
   P21), pa redak to kaže riječima. **Ukupna promjena morala** nosi smjer samo kad je jedini prikazani redak morala;
   uz pritiske (plaće, opterećenje, obuka, otpuštanje, situacije) je zbroj i ostaje siva, da se ne broji dvaput.

**Cijena ako je krivo.** Tablice naziva i fraza (Rust, `crates/game/src/mentor.rs`) treba držati usklađene s
`packages/i18n` i formulama; promjena oblika je jedna datoteka plus nova verzija upute. Ekran „Mjesec završen"
prikazuje prijelome redaka mentora (`whitespace-pre-line`, web commit `481d696`). Ako se plan proširi novom
postavkom, `plan_facts` je treba dobiti, inače je mentor ne vidi.
