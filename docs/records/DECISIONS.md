# Arhitektonske i proizvodne odluke (ADR)

Svaka značajna odluka: kontekst → odluka → posljedice. Najnovija na vrhu. **Ovo je POVIJEST — što vrijedi sad piše u
`CLAUDE.md` i u `docs/product`, `docs/architecture`.**

---

## ADR-024 — Modeli: presuda na Sonnetu 5.5, lik na Haikuu 5.5, mentor na Sonnetu 5.5
**Datum:** 2026-10-10 · **Status:** ✅ **Leonova odluka 10.10.** · **Vezano:** [ADR-018](#adr-018) t. 4 (zamijenjeno),
[ADR-023](#adr-023)

**Kontekst.** Plan AI sloja imao je Leonovu „odluku 4": jači model (Opus 5.5) za presude, Sonnet za lika i mentora.
Leon 10.10.: *„stavi Sonnet i Haiku koristi, ne treba nam jače od Sonneta da procjenjuje situaciju"*; u anketi je
odabrao lika na Haikuu, mentor ostaje na Sonnetu.

**Odluka:**
1. **Zadani modeli** (`ai::Models::default()`): presuda `claude-sonnet-5-5` (effort `high`), lik `claude-haiku-5-5`
   (effort `low`), mentor `claude-sonnet-5-5` (effort `low`). Varijable okoline `HOTELIXSIM_AI_MODEL_PRESUDA`,
   `_LIK`, `_MENTOR` i dalje nadjačavaju — promjena modela je jedna varijabla.
2. **Razlog i trošak** (po milijunu tokena, ulaz/izlaz): Opus 4/20 $, Sonnet 2/10 $, Haiku 0,10/0,50 $. Presuda
   po rubrici sa strukturiranim izlazom ne traži više od Sonneta; lik je kratka replika.
3. **Haiku 5.5 nema poslužiteljski rezervni model.** Pravilo je po MODELU, ne po vrsti poziva:
   `request::supports_server_fallback(model)` je istina samo za `claude-opus-5-5`, `claude-sonnet-5-5`,
   `claude-opus-5` i `claude-fable-5-1`. Za ostale (svi `claude-haiku-*`, nepoznati) tijelo nema `"fallbacks"` i ne
   šalje se zaglavlje `anthropic-beta: server-side-fallback-2026-07-01` (`request::beta_header`).

**Cijena ako krivo:** presuda je slabije dosljedna. Provjera: alat `mjerilo` kad Leon ocijeni 40 odgovora
(`content/hotel/mjerilo/*.json`); ako treba, vratiti presudu na Opus varijablom okoline.

---

## ADR-023 — Mentor govori ljudski: pripremljene činjenice, četiri dijela, brojke samo s imenom i jedinicom
**Datum:** 2026-10-10 · **Status:** 🟨 ČEKA LEONOV PREGLED (Claude, „radi bez pitanja") · **Vezano:** [ADR-019](#adr-019),
[ADR-022](#adr-022) t. 7, ARHITEKTURA §8.5 · **Plan:** [2026-10-10-ekrani.md](../plan/2026-10-10-ekrani.md) (Task 14)

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

**Cijena ako je krivo.** Tablice naziva i fraza (Rust, `crates/game/src/mentor.rs`) treba držati usklađene s
`packages/i18n` i formulama; promjena oblika je jedna datoteka plus nova verzija upute. Ekran „Mjesec završen"
prikazuje prijelome redaka mentora (`whitespace-pre-line`, web commit `481d696`). Ako se plan proširi novom
postavkom, `plan_facts` je treba dobiti, inače je mentor ne vidi.

---

## ADR-022 — Ekrani igrača: pretpostavke plana koraka 5
**Datum:** 2026-10-10 · **Status:** 🟨 ČEKA LEONOV PREGLED (Claude, „radi bez pitanja") · **Vezano:** [ADR-021](#adr-021),
[DIZAJN.md](../product/DIZAJN.md), DEMO §1.4–1.5 · **Plan:** [2026-10-10-ekrani.md](../plan/2026-10-10-ekrani.md)

**Odluka (pretpostavke):**
1. **Vlastiti mali usmjerivač** (`src/router.tsx`, ~60 redaka, History API) umjesto `react-router` 8 (nova glavna
   verzija, a trebamo sedam ruta bez učitavača). Cijena ako zatreba više: zamjena jedne datoteke i `Link`-ova.
2. **Izgled samo kroz tokene:** boje, slova i zaobljenja su `@theme` varijable u `apps/web/src/styles.css`; ekrani
   koriste imena tokena. Ikone `lucide-react`. Ilustracija obale i „Jadran · 4★" iz koncepta izostaju (nema slike ni
   regije u API-ju); zaglavlje pokazuje broj soba.
3. **Gost:** `/` i `/demo` su isti ekran; tko nema igru vidi „Započni igru" (jedan klik → `POST /v1/demo`). Igra se ne
   stvara sama pri otvaranju, jer bi osvježavanje ili dvostruki efekt u Reactu stvorili dvije igre.
4. **Opis situacije** = prva rečenica lika (`opening`); sadržaj nema polje opisa. S3 (bez lika) pokazuje samo naslov i
   ponuđene odgovore; njezin naslov još nosi „(nacrt)" iz sadržaja — to je Leonov sadržaj, ekran ga ne skriva.
5. **Slobodan odgovor prvi, ponuđeni ispod** („ili izaberi jedan od ponuđenih"); jedno isključuje drugo (poslužitelj
   ionako vraća `already_answered`).
6. **Likovi:** izgled (`AvatarLook`) danas daje tablica poznatih likova u kodu ili deterministički izgled iz imena; poslije
   demoa seli u sadržaj (`character.look`) i profil igrača (API + baza), a uređivač mijenja ta polja. Boje lika su
   podatak (`src/avatar/palette.ts`), ne tema; novi element palete ide na kraj niza da stari izgledi ostanu isti.
7. **Promjene indeksa:** na „Moj hotel" od **početka** (ocjena je napredak od početka), na „Mjesec završen" **u tom
   mjesecu** (prema prethodnom mjesecu, za lipanj prema početku); oba su označena riječima. Uzroci se ne vežu uz
   indekse (API ih ne veže) nego se prikazuju po varijabli („Prodane noći: cijena −120…").
8. **Hrvatski tekstovi uzroka, varijabli i grešaka** (`cause.*`, `var.*`, `problem.*`) napisao je Claude.
9. **Ispitivanje umjesto SSE-a:** razgovor svakih 1,5 s dok lik piše, presuda svake 2 s dok je `pending`, obračun
   svake 2 s dok je runda `closing`; nakon 150 s obračuna ekran kaže da traje dulje nego inače.
10. **Provjera u pregledniku** je ručna (Playwright MCP, 390 px) na pravom poslužitelju i bazi; e2e u CI-ju traži
   Postgres i poslužitelj u poslu weba — BACKLOG `[ekrani]`.

---

## ADR-021 — Koncept ekrana v2 i odluke o prikazu
**Datum:** 2026-10-10 · **Status:** ✅ ODLUČIO LEON (poruke i anketa 10.10.) · **Vezano:** [ADR-020](#adr-020),
[DIZAJN.md](../product/DIZAJN.md)

**Odluka.** Leon: *„pusti sada logo i dizajn jer mijenjamo ga"*, zatim *„dao sam ti pood new UI izgled"* — novi koncept
(v2) zamjenjuje prvi u `docs/product/dizajn/koncept-ekrana.png` (logo isti; nove boje pijesak `#F4E8D5` i menta
`#D8F0EE`; inicijali umjesto avatara; „50 = prosjek" i ▲/▼ uz indekse). Anketa: (1) ocjena na portalima **„8,4/10"**,
bez preračuna; (2) demo ima **dva mjeseca**, oznaka „1/2"; (3) AI obrazloženje pokazuje **cijelu rečenicu kriterija**
s ✓/✗ i razlogom; (4) plan mjeseca pokazuje **svih osam postavki**, cijenu, sezonce i održavanje na vrhu, ostalo pod
„Više postavki". Zatim Leon: *„trebat cemo napravit razlicite likove u igrici izgledno i to na nacin da cemo kasnije moci
ih customizat isto kao sto bit life ima"* → anketa (5): **temelj sad, uređivač poslije demoa** — likovi se crtaju iz
slojeva, izgled je podatak. Zatim *„trebat ce prosirit likove"* → anketa (6): više izgleda (nove osobine i opcije),
više likova u igri (svako lice u ekranima) i uređivač lika; „dio odmah". Claude: prije demoa izgled i likovi (Task 11–12
plana), uređivač kao zaseban plan odmah poslije, jer izgled mora na poslužitelj (ADR-022 t. 6).
**Posljedice.** DIZAJN.md §3 zatvoren; DEMO §1.1 kaže dva mjeseca.

---

## ADR-020 — Sučelje: Tailwind i Leonov koncept dizajna; gradnja ekrana u sljedećoj sesiji
**Datum:** 2026-10-10 · **Status:** ✅ ODLUČIO LEON · **Vezano:** [DIZAJN.md](../product/DIZAJN.md), ROADMAP korak 5

**Odluka.** Leon: *„mislim da je kreiaranje sucelja za novu sesiju iskreno. koristit cemo tailwind za dizajn ja cu ti
poslat u ovoj sesiji plan dizajna pa cemo u sljedecoj sesiji krenuti sa izradom"*. Dizajn sučelja je Leonov logo i
koncept tri ekrana (Moj hotel, Situacija, Mjesec završen) u `docs/product/dizajn/`; boje `#246BFD`, `#151B2B`,
`#FFFFFF`; stilovi se pišu Tailwindom.
**Posljedice.** Plan ekrana (korak 5) gradi po [DIZAJN.md](../product/DIZAJN.md); šest neslaganja koncepta i igre
(§3 tamo) rješava plan ili Leon prije gradnje.

---

## ADR-019 — AI sloj: arhitektonske pretpostavke koraka 4
**Datum:** 2026-10-09 · **Status:** 🟨 ČEKA LEONOV PREGLED (Claude, „radi bez pitanja") · **Vezano:** [ADR-018](#adr-018),
[ADR-015](#adr-015), ARHITEKTURA §8.3–8.4, §9.4, §10 · **Plan:** [2026-10-09-ai-sloj.md](../archive/2026-10-09-ai-sloj.md)

**Kontekst.** Korak 4 dodaje AI (presuda, lik, mentor) na igru iz koraka 3. ARHITEKTURA opisuje cilj, ali ne i kako se
to uklapa u dnevnik, red poslova i rok demoa ~14.10. Ovo su pretpostavke s kojima plan gradi.
**Odluka (pretpostavke):**
1. **Otisak pravila nad kanonskim `Value`** spremljenog snimka; igre iz koraka 3 prolaze preko starog otiska
   (`legacy_fingerprint`). Nova polja sadržaja uvijek uz `serde(default, skip_serializing_if)`. `core_version` se
   provjerava pri obračunu (druga verzija jezgre = 500, ne tihi obračun). Cijena: jedna dodatna grana u foldu.
2. **Kvalitetu računa kod, ne AI:** AI za svaki od pet kriterija kaže ispunjen/ne uz razlog i bira najbliži ponuđeni
   odgovor; kvaliteta = `max(1, broj ispunjenih)`, kao kod ponuđenih (DEMO §7). Presuda bez pet obrazloženih kriterija
   je nevaljana i ponavlja se.
3. **Sve AI uloge idu kroz postojeći red poslova** (lik, presuda, mentor), ne u HTTP zahtjevu: upis poruke i posao u
   istoj transakciji, AI se zove izvan zaključavanja igre, rezultat se upisuje kroz `decide` koji ponovno provjerava
   stanje runde. Cijena: ekran mora pitati za odgovor lika (202 pa čitanje), sekunda-dvije kašnjenja.
4. **Zadnji pokušaj (5.) piše rezervu:** lik napisanu rečenicu, presuda neutralnu (kvaliteta 3, mehanike `unanswered`,
   „na pregled profesoru"), mentor tekst iz brojki. Igra se nikad ne zaglavi zbog AI-ja.
5. **„Zatvori mjesec" čeka presude najviše 120 s** (posao obračuna se odgađa po 5 s), zatim neutralna presuda. Presuda
   koja stigne kasnije zapisuje se kao `VerdictLate` i **ne mijenja obračun**; ponovni obračun / kompenzacija (§8.3 t. 6)
   i profesorova ispravka su korak 6 / drugi val.
6. **Red poslova dobiva odgodu** (`run_after`): neuspio posao čeka 2, 4, 8, 16 s (najviše 60 s); zakup 180 s jer presuda
   može trajati desetke sekundi. Probni test apalis/pgmq i `Idempotency-Key` ostaju u BACKLOG-u (demo ima prirodnu
   idempotenciju: jedan odgovor po situaciji, poruka tek nakon odgovora lika).
7. **Novi crate `ai`** (tanki HTTP klijent, `reqwest` + rustls) implementira port `AiPort` iz `app`; `game` ostaje čist.
   Upute su datoteke `prompts/*.v1.md`, ugrađene pri prevođenju; uz svaki izlaz u dnevnik idu model, verzija upute i
   tokeni. Bez ključa server radi s `DisabledAi` (sve rezerve).
8. **Odgođeno:** tablica `ai_usage` i tvrdi limit troška po igri (prije Railwaya), zaključavanje modela i uputa u snimak
   pravila (prije pilota), AI koji smišlja situacije (pilot).
9. **Likovi (Petra Horvat, gošća; Marko Babić, šef kuhinje), njihovo ponašanje i 40 nacrta odgovora za mjerilo** su
   nacrt; ocjene u mjerilu upisuje Leon.
10. **AI izlaz koji ulazi u drugu uputu (replike lika u presudi) je također podatak, ne uputa** — ide kroz istu zaštitu
   oznaka kao studentov tekst (svaka replika u vlastitoj oznaci, `<` i `>` razoružani, novi red kao `⏎`); upute presude
   zato postaju `presuda.v2` (v1 ostaje u repou radi povijesti).
**Posljedice.** Novi crate `ai`, migracija `0004`, šest novih događaja (`schema_ver` ostaje 1). Ako Leon odbije t. 3,
lik prelazi na izravan poziv u zahtjevu (mijenja se `app/talk.rs` i ruta); ako odbije t. 5, mijenja se jedna konstanta.

---

## ADR-018 — Leonove proizvodne odluke za AI sloj: gdje slobodan tekst, odakle brojke, razgovor, pružatelj
**Datum:** 2026-10-09 · **Status:** ✅ ODLUČIO LEON (anketa 09.10.) · **Vezano:** [ADR-019](#adr-019), DEMO §1.5, §7, §11 t. 7

**Odluka.**
1. **Slobodan tekst samo u S1 i S2**, uz ponuđene odgovore; S3 (agencija, brojke) ostaje biranje.
2. **Brojke slobodnog odgovora iz najbližeg ponuđenog odgovora:** AI ocijeni kvalitetu po rubrici i izabere najbliži
   ponuđeni odgovor; trošak i ostale brojke dolaze od njega. AI nikad ne izmišlja novac.
3. **Razgovor s likom: najviše 6 poruka studenta po situaciji;** presuditelj vidi cijeli razgovor uz konačni odgovor.
4. **Pružatelj: Anthropic (Claude)** — jači model za presude, jeftiniji za lika i mentora; ključ u `.env`.
   *Zamijenjeno [ADR-024](#adr-024): presuda Sonnet 5.5, lik Haiku 5.5, mentor Sonnet 5.5.*

---

## ADR-017 — Odluke voditelja gradnje u koraku 3 (red poslova, greške, putanje, javne funkcije)
**Datum:** 2026-10-05 · **Status:** 🟨 ČEKA LEONOV PREGLED (Claude, „radi bez pitanja") · **Vezano:** [ADR-015](#adr-015), [ADR-016](#adr-016)

**Kontekst.** Tijekom gradnje koraka 3 voditelj je presudio više tehničkih pitanja umjesto Leona. Svako je malo i
mijenja se lokalno; zapisano da se ne izgube.
**Odluka (pretpostavke):**
- **Red poslova nosi identitet preuzimanja** (`JobClaim{id, attempts}`): `fail` i `finish` djeluju samo ako se poklapaju
  i posao je `running`; istek zakupa na zadnjem pokušaju → `failed`. Razlog: zakašnjeli radnik ne smije pokvariti posao
  koji je već preuzeo drugi. Cijena ako je krivo: jedan dodatni parametar u `jobs.rs`.
- **Nepoznat profil ili strategija u API-ju = zabilježen 500**, ne tihi zamjenski izbor. Razlog: tihi izbor skriva
  pokvaren dnevnik. Cijena: pokvarena igra se vidi odmah (što je i cilj).
- **Provjere pristupa idu prije provjere tijela** (401 → 404 → 403 prije 422). Razlog: neprijavljen ne smije saznati
  oblik tijela ni postojanje igre. Cijena: nijedna.
- **Odgovor na situaciju nema mjesec u putanji**, jer su šifre situacija jedinstvene po igri. Ako sadržaj ikad ponovi
  šifru kroz mjesece, mjesec mora ući u putanju (nova verzija rute). Cijena ako je krivo: promjena rute i klijenta.
- **`sim_hotel::{euro_bounds, seasonal_range}` postaju javne** da ih `game` koristi umjesto kopije. Cijena: veće javno
  sučelje jezgre.
- **`crates/game` dobiva istu zabranu floatova, sata i `HashMap` kao jezgra** (`clippy.toml` + `check-core-deps`).
  Razlog: obračun runde mora ostati ponovljiv. Cijena: stroža pravila pri pisanju.
- **Pravilo oblika snimka** (nova polja content DTO-a samo uz `#[serde(default)]`; ispravak otiska nad kanonskim
  `Value`) dokumentirano u [ADR-015](#adr-015) t. 3, a kodni ispravak je dug za korak 4 (BACKLOG `[korak 4]`).

---

## ADR-016 — Leonove proizvodne odluke za korak 3: gost, zatvaranje mjeseca, tko zatvara, profili
**Datum:** 2026-10-05 · **Status:** ✅ ODLUČIO LEON (ankete 05.10.) · **Vezano:** [ADR-015](#adr-015), [ADR-012](#adr-012), DEMO §1

**Odluka.**
1. **Gost preko poveznice:** pozadina (kolačić, bez lozinke) sad u koraku 3, ekran u koraku 5; ograničenje po IP-u i
   brisanje nakon 7 dana u drugom valu.
2. **„Zatvori mjesec" uz neodgovorenu situaciju:** upozori, pa dopusti; neodgovorena = kvaliteta 1 (DEMO §1.2).
3. **Tko zatvara mjesec u demu:** svatko u igri. Leon je dodao: „moze ovo sta si predozio ali uzmi u obzir da moramo
   igricu napraviti kao bitlife slicnu tako imam neku viziju". Posljedica: plan se može mijenjati bilo kad, hotel ima
   feed kao vremensku crtu, svaka promjena nosi uzrok.
4. **Profili:** hoteli kreću s različitim profilima — igrač iz sjemena, roboti A, B, C.

**Posljedice.** Pretpostavke koje to razrađuju su u [ADR-015](#adr-015) (t. 2, 10, 13, 14). Otvoreno za Leona ostaje
samo ishod neodgovorenih situacija S1–S4 (nacrt) u BACKLOG-u.

---

## ADR-015 — Dnevnik, runde i red poslova: arhitektonske pretpostavke koraka 3
**Datum:** 2026-10-05 · **Status:** 🟨 ČEKA LEONOV PREGLED (Claude, „radi bez pitanja") · **Vezano:** [ADR-012](#adr-012),
[ADR-013](#adr-013), ARHITEKTURA §6.2, §7.2, §8–§9, §11–§12 · **Plan:** [2026-10-05-dnevnik-runde.md](../archive/2026-10-05-dnevnik-runde.md)

**Kontekst.** DEMO korak 3 pretvara jezgru u igru: gost, tri robota, runde koje zatvara gumb, obračun u redu poslova,
sve kao dnevnik događaja. Leon je 05.10. odlučio proizvodno (gost samo kao pozadina; zatvaranje s neodgovorenom
situacijom dopušteno uz ocjenu 1; zatvara svatko u igri; „kao BitLife": plan se mijenja bilo kad, feed hotela, svaka
promjena nosi uzrok). Arhitektura toga nije bila zapisana; ovo su pretpostavke s kojima plan gradi.
**Odluka (pretpostavke):**
1. **Novi čisti crate `game`** (ovisi o `engine`, `sim-hotel`, `content`, serde, uuid; nikad tokio/sqlx/axum — čuva
   `check-core-deps`): događaji, serde DTO-i, projekcija (fold), pravila runde, potezi robota, obračun runde. Jezgra
   ostaje bez serdea. **Snimak pravila** je stablo content DTO-a (`content::Snapshot`), a `ruleset_id` = SHA-256
   kanonskog JSON-a tipiziranog snimka (ne ovisi o redoslijedu ključeva koji JSONB mijenja). Cijena alternative (serde u
   jezgri): jezgra dobiva vanjsku ovisnost i gubi branu ADR-009.
2. **Neodgovorena situacija = kvaliteta 1 + mehanike iz novog polja `unanswered`** u `situations.json` (jezgra dobiva
   polje `SituationTemplate.unanswered` i `SituationOutcome::unanswered`). Odstupanje od „jezgra se ne dira": kvaliteta 1
   bez mehanike nema nikakav učinak, pa bi šutnja bila bolja od lošeg odgovora. Ishodi za S1–S4 su nacrt (S3: ponuda
   propada bez kazne) — Leonova provjera, kao i rubrike.
3. **Pet događaja** (`GameCreated` s punim snimkom, sjemenom, verzijom jezgre i hotelima · `PlanSet` · `AnswerChosen` ·
   `RoundClosed` · `RoundResolved` s punim izvještajem po hotelu), `schema_ver = 1`. Obračun uvijek iz snimka u
   `GameCreated`, nikad iz trenutnih datoteka. `actor` je tekst: `guest:<uuid>` · `robot:<strategija>` · `system`.
   **Pravilo oblika snimka:** otisak se danas računa iz TIPIZIRANOG snimka pri svakom foldu, pa promjena oblika
   content DTO-a (novo, preimenovano ili uklonjeno polje) čini sve postojeće igre nečitljivima (`Corrupt`). Do ispravka
   u koraku 4 vrijedi: nova polja DTO-a samo uz `#[serde(default)]`, a promjena oblika = nova verzija `GameCreated`
   s podizanjem. Ispravak (prije prve promjene content DTO-a): otisak računati nad kanonskim `serde_json::Value`
   spremljenog snimka, ne nad tipiziranim ponovnim zapisom (BACKLOG `[korak 4]`).
4. **Jedan tok po igri `game:<id>`, a svaka naredba je jedna transakcija**: `pg_advisory_xact_lock` po igri → učitaj tok
   → odluči (čista funkcija) → upiši. `UNIQUE(stream, stream_seq)` ostaje zadnja brana. Odstupanje od „optimistička
   konkurentnost s ponavljanjem": zaključavanje uklanja petlje ponavljanja i utrku između provjere i upisa; cijena je
   serijalizacija naredbi jedne igre, zanemariva za 4 hotela.
5. **Nepromjenjivost dnevnika jamči baza**: okidač odbija `UPDATE/DELETE/TRUNCATE` (SQLSTATE 42501), `CHECK` na oblik
   `actor`, indeks `(game_id, id)`. Uloga bez prava za Railway → BACKLOG.
6. **Platformske tablice** `guests` (u bazi samo SHA-256 tokena iz kolačića `httpOnly`), `games` (redoslijed `seq`),
   `hotels` (hotel → igra, gost → zadnja igra). Istina o igri ostaje u `events`.
7. **Roboti igraju istim naredbama**: njihovi `PlanSet`/`AnswerChosen` (actor `robot:<strategija>`) upisuju se u
   transakciji koja otvara rundu (uz `GameCreated` za lipanj, uz `RoundResolved` za srpanj); kocka
   `Rng::derive(sjeme, [svrha, mjesec, slot(, situacija)])`.
8. **Red poslova: vlastita tablica `jobs`** (jedinstveni `job_key` `resolve:<igra>:<mjesec>`, zakup 30 s,
   `FOR UPDATE SKIP LOCKED`, najviše 5 pokušaja, bez odgode). Gumb upisuje `RoundClosed` i posao u istoj transakciji
   (202); `RoundResolved`, potezi robota i „posao gotov" također u jednoj transakciji, idempotentno po stanju runde.
   Radnik je tokio zadatak u `serve`. Probni test apalis/pgmq, odgoda i zakašnjeli AI rezultat → korak 4.
9. **Projekcije se računaju foldom pri čitanju**, bez projekcijskih tablica (4 hotela × 2 mjeseca). Ponovljivost se
   provjerava tipizirano (bajt-isti `RoundResolved` nakon čitanja iz JSONB-a).
10. **Idempotencija prirodna po naredbi**: plan „zadnji vrijedi", odgovor jednom (409), zatvaranje već zatvorene runde
    vraća stanje (200 umjesto 202); **mjesec je u putanji** plana i zatvaranja, pa zastario ekran dobiva 409 umjesto
    tihog upisa u idući mjesec. Zaglavlje `Idempotency-Key` → BACKLOG.
11. **API ima vlastite DTO-e** (odvojene od DTO-a događaja): cijena je zrcaljenje tipova, dobitak je da se API mijenja
    bez nove verzije događaja. Kodovi grešaka i sve vrijednosti su enumi u OpenAPI-ju; kvaliteta i mehanike ponuđenih
    odgovora nikad ne izlaze. Rute po ARHITEKTURA §11 (`/v1/hotels/{id}/…`, `/v1/games/{id}/rounds/{mjesec}/close`).
12. **Server pri startu učitava i provjerava sadržaj** (`HOTELIXSIM_CONTENT`, zadano `content/hotel`) i ne kreće s
    pokvarenim katalogom; **migracije se pri startu ne pokreću** (`migrate` ostaje zasebna naredba, kasnije korak
    isporuke na Railwayu), jer automatska migracija s više procesa vodi u utrku.
13. **Profil: hoteli kreću s različitim profilima** (Leon 05.10., anketa): igrač dobiva profil iz sjemena, tri robota
    dobivaju A, B i C u rasporedu iz sjemena. Demo tako pokazuje bit ocjene — napredak od vlastitog početka, ne zarada.
    Imena: „Tvoj hotel", „Štediša", „Brižni", „Lovac na popunjenost".
14. **Pravo pristupa u `app`**: član igre = gost koji vodi hotel u njoj; vlasnik vidi stvarne brojke, plan, situacije i
    feed; član vidi tuđe samo kroz ocjenu i pet indeksa; ostali 403, bez kolačića 401. Test po ulozi.
**Posljedice.** Novi crate `game` i proširena brana čistoće; jezgra dobiva jedno polje i jednu funkciju; baza dobiva
dvije migracije. Ako Leon odbije t. 13, mijenja se jedna funkcija (`new_game`); ako odbije t. 2, mijenja se samo sadržaj
`unanswered`. Otvorena proizvodna pitanja (t. 2 i t. 13) su u [BACKLOG.md](BACKLOG.md) → „Otvoreno za Leona".

---

## ADR-014 — Kolovoz rasprodan (lekcija S3) i blagajna bez gornje granice u financijama
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon, anketa) · **Vezano:** mijenja [ADR-013](#adr-013) t. 1 (formula F) i
kalibraciju · **Plan:** [2026-10-04-jezgra.md](../archive/2026-10-04-jezgra.md) Task 13 i 14

**Kontekst.** Završni pregled jezgre izmjerio je dvije strukturne greške. (1) Lekcija S3 radi naopako: prihvatiti grupu
od 175 soba-noći uz 25 % popusta u kolovozu donosi profilu C +25 598 € GOP-a, a u lipnju +19 184 € — kolovoz nije pun pa
grupa nikoga stvarno ne istiskuje. (2) Obnova od 150 000 € pred kraj sezone ne košta ništa: likvidnost je ograničena na
100, a do rujna je blagajna svima toliko puna da i hotel i referentni hotel stoje na 100, pa razlika nestane.
**Odluka (Leon, anketa):**
1. **Kolovoz je rasprodan:** potražnja u kolovozu veća je od broja soba (kao stvarni 4* u Opatiji u vrhu sezone), pa grupa
   s popustom istiskuje goste koji plaćaju punu cijenu. Samo podaci; kalibracija se ponavlja (više osoblja u kolovozu da
   marža ostane ≤ 50 %). Alat za balans dobiva provjeru: prihvaćanje S3 u kolovozu donosi manje od prihvaćanja u lipnju.
2. **Likvidnost se ne ograničava odozgo prije usporedbe s referentnim hotelom:** sirovi F = 0,7 × GOPPAR-indeks +
   0,3 × likvidnost, gdje likvidnost = blagajna / (2 × (fiksni + trošak rada)) × 100 **bez gornje granice** (negativna
   blagajna → 0); granica 0–100 vrijedi samo za konačni, centrirani indeks. Potrošen novac tako ostaje vidljiv do kraja sezone.
**Posljedice.** Mijenja se jedna formula (`indices.rs`) i `scripts/oracle-jezgra.mjs`; zlatne vrijednosti F referentne
tablice od srpnja i početni F profila B mijenjaju se. DEMO §4 i §12 (P30) se usklađuju.

---

## ADR-013 — Pretpostavke jezgre: centrirani indeksi, sezonci po planu, redoslijed obračuna, drugi val
**Datum:** 2026-10-04 · **Status:** 🟨 ČEKA LEONOV PREGLED (Claude, „radi bez pitanja") · **Vezano:** [ADR-012](#adr-012),
[ADR-009](#adr-009) · **Plan:** [2026-10-04-jezgra.md](../archive/2026-10-04-jezgra.md) · **Model:** `scripts/oracle-jezgra.mjs`

**Kontekst.** Revizija 04.10. našla je rupe u [DEMO.md](../product/DEMO.md) (BACKLOG „Rupe u specu"): profil C po §4
ne kreće s 50 (G 70, Z 54, O 70), prosječan hotel bez odluka kroz sezonu gubi R i dobiva F (samo zato što ocjena
klizi, a blagajna raste), nije rečeno kako se računa početni F ni što je „zadano" za sezonce, a s brojkama iz §2.1
kolovoz nije pun pa grupa iz S3 nikoga ne istiskuje. Plan jezgre ne može napisati prvi test dok to nije presuđeno.
**Odluka (pretpostavke, svaka zamjenjiva podacima ili jednom funkcijom):**
1. **Svih pet indeksa mjeri se prema referentnom hotelu iste kategorije u istom mjesecu:**
   `indeks = 50 + (sirova vrijednost − sirova vrijednost referentnog hotela)`, ograničeno 0–100. Sirove vrijednosti su
   formule iz DEMO §4; referentni hotel je profil C koji igra zadani plan bez situacija. Posljedica: C bez odluka
   stoji na **točno 50 u svakom indeksu cijelu sezonu** („ništa ne gori, ništa ne blista"), a napredak mjeri samo ono
   što su odluke promijenile. Širi KALIBRACIJA opciju 1 (centriranje) s G, Z, O na svih pet i na svaki mjesec.
2. **Početni F** = F iz prognoze lipnja (zadani plan, bez situacija) i likvidnosti uz trošak rada bez zapošljavanja.
3. **Sezonci:** katalog ima plan sezonaca po mjesecu; zadani plan ga slijedi (najviše ±10 mjesečno). Radnje
   `HireSeasonal`/`ReleaseSeasonal` postaju jedna: ciljani broj sezonaca za mjesec.
4. **Redoslijed obračuna mjeseca:** odgođeni učinci prošlog mjeseca → osoblje → tržište → ljudi → novac → oprema i
   gosti. Fluktuacija se računa iz novog morala; automatska zamjena odlazaka košta kao zapošljavanje (1 500 €) i
   ulazi u „mješavinu" iskustva.
5. **Cilj marže** je sezonski (VI–IX) **35–45 %**, i nijedan mjesec iznad 50 % (kolovoz je bio 55 %).
6. **Grupna cijena** = referentna cijena ciljnog mjeseca uz popust; grupa se smije ugovoriti za tekući ili kasniji mjesec.
7. Parametri mehanika i **ponuđeni odgovori za S3 i S4** (DEMO ih nema) napisani su kao nacrt u `content/hotel/`
   i čekaju Leonovu provjeru, kao i rubrike S1b i S5–S8.
8. **Drugi val (poslije demoa 14.10.):** splet okolnosti i tragovi (§9), S1b i S5–S8, naplata otvorenih tragova na
   kraju sezone, različit učinak S4 po stanju hotela, tablice za 3* i 5*, JSON Schema kataloga.
9. Pravila cjelobrojnog računa P1–P32 iz DEMO §12 vrijede kako su napisana.
**Posljedice.** Kalibracija (Task 11 plana) mora postići i da kolovoz bude dovoljno pun da grupa iz S3 istiskuje
goste. Ako Leon odbije točku 1, mijenja se jedna funkcija (`centred`) i referentna tablica; ostale točke su podaci.

---

## ADR-012 — Ciljani rok demoa ~14.10.; prvo se gradi profesorovih 15 minuta
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon, anketa) · **Vezano:** [ADR-009](#adr-009) · mijenja redoslijed
unutar [DEMO.md](../product/DEMO.md) §1, ne opseg

**Kontekst.** Demo nije imao datum (10.10. pomaknut). Leon: *„pitanja za profesore u postavit tokom ovog nadolazeceg
tijedna bilo bi lijepo da je demo gotov za nekih 10 dana"*. Za 10 dana ostaje pet koraka gradnje (jezgra, runde, AI,
ekrani, profesorov pogled) — cijeli DEMO.md ne stane bez rezanja provjera.
**Odluka.** Ciljani rok **~14.10.** (želja, ne tvrdi rok). Prvo se gradi točno put iz DEMO §1.5 („što profesor vidi u
15 minuta"): lipanj i srpanj s njihovim situacijama, izbornik radnji, „Zatvori mjesec", mentor, tri robota, profesorov
pogled. **Temelji se ne režu** (brane determinizma kao Task 1, dnevnik, testovi, pregledi). Odmah poslije: kolovoz i
rujan, S5–S8, limiti i brisanje gosta, širi alat za balans. Pitanja profesorima šalju se tijekom tjedna 5.–11.10.
**Posljedice.** Plan jezgre bira mehanike i situacije koje trebaju lipanj i srpanj; ostalo ostaje u DEMO.md kao spec
za drugi val. Ako 14.10. ne stoji, reže se drugi val, ne provjere.

---

## ADR-011 — Dokumentacija po Sokratovom sustavu: mape po ulozi, jedan indeks, brana, komande
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon, „kasnije da" na prijedlog) · **Vezano:** [ADR-010](#adr-010),
[ADR-001](#adr-001)

**Kontekst.** Popis 04.10. (20 .md): ista činjenica živi na 3–6 mjesta (stanje CI-ja, push, baza 5433, pravila rada),
nema indeksa ni numeriranih odluka, a `SLJEDECA_SESIJA` se prepisuje pa gubi povijest. Leon: *„ne sviđa mi se kako imamo
dokumentaciju posloženu, moramo imat isti pristup kao i u Sokrat Studyju pošto će ovo biti veliki razvoj, mislim da
dokumentacije ne idu dobro kroz sesiju."*
**Odluka.** Sokratov raspored 1:1: mape po ulozi dokumenta (`product`, `architecture`, `plan`, `workflow`, `records`,
`research`, `archive`); jedini indeks `docs/README.md`; jedna činjenica jedno mjesto; najviše jedan aktivni plan
(`ROADMAP.md` se ne broji); brana `scripts/check-docs.mjs` (mrtve poveznice, .md izvan indeksa, >1 aktivni plan,
>3 ISO datuma u `product/`, budžet `CLAUDE.md`) s vlastitim testom i korakom u CI-ju; komande `/newsession` i nova
`/kraj`. Poboljšanje nad Sokratom: predaja `docs/SLJEDECA_SESIJA.md` ostaje u repou, ne u memoriji, pa ne zastarijeva.
**Posljedice.** Datoteke premještene s `git mv` (povijest ostaje), poveznice ispravljene. Povijest odluka je ovdje,
dnevnik sesija u `PROGRESS.md`, odgođeno u [BACKLOG.md](BACKLOG.md). Provedeno 04.10. (unos u [PROGRESS.md](PROGRESS.md)).

---

## ADR-010 — Rad u Claude Code dodatku za VS Code (panel), ne u terminalu
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-011](#adr-011)

**Kontekst.** Leonu je VS Code okruženje prirodnije (Explorer, preglednik uz kod, slike povlačenjem).
**Odluka.** Leon: *„nećemo mi više radit u terminalu… ostat ćemo u VS Code okruženju."* Upute za prečace i slike daju
se za panel dodatka (otvaranje: iskrica / `Ctrl + Esc`; slika: `Ctrl + V` ili povlačenje u polje za poruku), ne za terminal.
**Posljedice.** Za stvari koje treba vidjeti, zaslon se može snimiti i izravno, ali vidi se samo ono što je otvoreno.
Ne mijenja ni kod ni arhitekturu.

---

## ADR-009 — Ojačavanje brana determinizma je Task 1 plana „jezgra i alat za balans", prije ijedne linije jezgre
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Claude, nakon završnog pregleda kostura; Leon ne mora potvrđivati) ·
**Vezano:** [ADR-003](#adr-003), [ADR-008](#adr-008) · **Izvori:** [BRANE_DETERMINIZMA](../research/BRANE_DETERMINIZMA_2026-10-04.md),
[ledger kostura](../archive/2026-10-04-kostur-ledger.md) (Final #3)

**Kontekst.** Završni pregled kostura: brane determinizma slabije su od ARHITEKTURA §6.2/§7.2 (`indexmap`, `RandomState`,
`as` na float/u16, `#[allow]` zaobilazi `deny`, `env::var`). Jezgra je još prazna, pa ništa ne puca.
**Odluka.** Nalaz ne blokira spajanje kostura, ali je OBAVEZAN prvi task sljedećeg plana: allowlist ovisnosti,
`RandomState` u `clippy.toml`, `forbid` + `as_conversions`/cast lintovi, zabrana `f32`/`f64`, probe mutacija kao skripta
u CI-ju. Pokus izvedivosti je izmjeren (npr. `forbid` radi samo pod `cargo clippy`, pa CI mora vrtjeti clippy).
**Posljedice.** Cijena ako je krivo: kod jezgre napisan prije jačih brana mogao bi proći s nedeterminizmom. Zato prvo
brane, pa jezgra. Plan se piše na temelju istraživanja iznad.

---

## ADR-008 — Odluke iz gradnje kostura koje je Claude donio umjesto Leona
**Datum:** 2026-10-04 · **Status:** 🟨 ČEKA LEONOV PREGLED · **Vezano:** [ADR-005](#adr-005), [ADR-007](#adr-007),
[ADR-009](#adr-009) · **Izvor:** [SLJEDECA_SESIJA.md](../SLJEDECA_SESIJA.md) §4

**Kontekst.** Po pravilu „radi bez pitanja" ([ADR-005](#adr-005)) Claude je tijekom gradnje kostura sam odlučivao sitnije
stvari. Sve su u gitu i mogu se vratiti. (Točka 2 je zaseban [ADR-007](#adr-007), točka 8 je [ADR-009](#adr-009).)
**Odluka (pretpostavke, s cijenom ako su krive):**
1. Gradnja na grani `gradnja/kostur` u istom direktoriju, ne u worktreeu. Cijena: nikakva.
2. Brana ovisnosti mjeri i build-ovisnosti, sve targete i značajke. Cijena: sporija brana.
3. `cargo fmt` nad kodom iz plana (plan je pisan preširoko). Cijena: jedan commit formatiranja.
4. `scripts/pnpm.mjs` jer `pnpm` nije na PATH-u (radi lokalno i u CI-ju). Cijena: zamijeniti ako se uvede samostalni pnpm.
5. Ekran pokazuje grešku indeksa; ključevi i interpolacija u i18n su provjereni (popravak nakon pregleda).
6. README i `.gitattributes` popravljeni za svježi Windows stroj (LF svugdje, `SQLX_OFFLINE`).
7. Sitni nalazi pregleda (~40) zapisani kao ulaz za sljedeće planove, ne popravljeni odmah ([BACKLOG](BACKLOG.md)).
**Posljedice.** Leon pregledava; ono što odbije, vraća se iz gita. Do tada vrijede kao radna pretpostavka.

---

## ADR-007 — Lokalna baza: zaseban PostgreSQL 16 klaster na priključku 5433
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Claude kao pretpostavka; Leon pregledava s [ADR-008](#adr-008)) ·
**Vezano:** [ADR-004](#adr-004), [ADR-008](#adr-008)

**Kontekst.** Sistemski PostgreSQL na 5432 već postoji, a Leon ne zna njegovu lozinku. Treba lokalna baza za razvoj i demo.
**Odluka.** Zaseban klaster PostgreSQL 16 u `C:\Users\leonk\AppData\Local\hotelixsim\pgdata`, priključak **5433**.
Sistemski na 5432 se ne dira. Umjesto resetiranja njegove lozinke.
**Posljedice.** Ne pali se sam nakon restarta: `bash scripts/baza.sh start`. Cijena ako je krivo: obrisati `pgdata`.
Pokretanje svega opisano je u `README.md` („Lokalni razvoj").

---

## ADR-006 — Arhitektura kroz dokument i raspravu, ne kroz ankete; ankete samo za jednostavne proizvodne odluke
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-003](#adr-003), [ADR-005](#adr-005) ·
**Izvor:** [RASPRAVA_ARHITEKTURE](../architecture/RASPRAVA_ARHITEKTURE.md)

**Kontekst.** Na anketu o obliku sustava Leon je odgovorio ljutito (*„ovo je ozbiljno, ovo nije igraonica"*), a jedna od
ponuđenih opcija bila je lažna, ubačena radi kontrasta. Isti dan je odbio da se stack bira prema onome što on zna:
*„ne razmišljaj o tome što ja znam, razmišljaj široko… Rust, drugi jezici, budi otvoren i kreativan"*.
**Odluka.** Arhitektura se radi u [ARHITEKTURA.md](../architecture/ARHITEKTURA.md): za svaku odluku razlog, razmotrene
alternative i cijena. Leon čita, raspravlja, potvrđuje (*„sada ćemo raspravljati o arhitekturi, to još nije to"*).
Kandidati se biraju po zahtjevima sustava, ne po Leonovom iskustvu; nikad lažna opcija. Ankete (AskUserQuestion) ostaju
za jednostavne proizvodne odluke.
**Posljedice.** Dokument je polazište rasprave, nije gotov dok Leon ne kaže. Cilj je razina seniora („arhitektura 10/10").

---

## ADR-005 — Radi bez pitanja: pretpostavke u dokument; push i deploy samo uz Leonov izričit OK
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-006](#adr-006), [ADR-008](#adr-008)

**Kontekst.** Čekanje potvrde svakog koraka usporava Leona; pravilo „nijedan korak ne počinje dok Leon ne potvrdi
prethodni" nadjačano je za tok rada.
**Odluka.** Leon usred ugradnje arhitekture v2: *„automatski radi, nemoj me pitati"*. Claude ide korak po korak bez anketa
i bez „smijem li nastaviti", a odluke koje bi bile upit donosi sam i zapisuje kao **pretpostavku s razlogom**. Ne vrijedi
za push i deploy: za njih treba izričit OK, a odobrenje jedne radnje ne proteže se na sljedeću. Lokalni commit je u redu.
**Posljedice.** Stvari koje zna samo Leon (npr. koje besplatne AI ključeve ima) ostaju otvorene, a gradi se tako da ne
blokiraju (AI iza porta s lažnim pružateljem). Nagomilane pretpostavke vidi [ADR-008](#adr-008).

---

## ADR-004 — Sve se gradi i pokazuje lokalno; Railway tek za pilot
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon) · **Vezano:** [ADR-003](#adr-003), [ADR-007](#adr-007) ·
**Izvor:** [ARHITEKTURA](../architecture/ARHITEKTURA.md) §14

**Kontekst.** Demo se pokazuje profesoru uskoro; hosting nije potreban da bi se dokazala igra.
**Odluka.** Leon: *„za sada radimo sve lokalno, Railway je za kraj"*. Razvoj i demo idu lokalno; Railway (jedan servis +
Postgres) dolazi u PILOTU.
**Posljedice.** Ništa u kodu ne smije ovisiti o Railwayu: postavke dolaze iz varijabli okoline, a isporuka je zaseban
završni korak. Deploy i dalje samo uz izričit OK ([ADR-005](#adr-005)). Vidi [ROADMAP](../plan/ROADMAP.md).

---

## ADR-003 — Arhitektura: Rust (jezgra + backend) + TypeScript (sučelje), v2
**Datum:** 2026-10-04 · **Status:** 🟨 NACRT v2, čeka Leonovu potvrdu · **Vezano:** [ADR-006](#adr-006),
[ADR-004](#adr-004), [ADR-009](#adr-009) · **Dokument:** [ARHITEKTURA.md](../architecture/ARHITEKTURA.md),
[RASPRAVA_ARHITEKTURE.md](../architecture/RASPRAVA_ARHITEKTURE.md)

**Kontekst.** Nacrt v1 pisan je po zahtjevima sustava (determinizam, ponovljivost, pravednost profila), ne po Leonovu
znanju ([ADR-006](#adr-006)). Krug 1 rasprave ugrađen je u v2: ispravka presude, puni snimak pravila, pravednost profila
i AI-ja, red poslova, što tko vidi, slobodan potez, obrazloženje za Rust.
**Odluka.** Jezgra simulacije i backend u **Rustu**, sučelje u **TypeScriptu**. Domena i jezici dogovoreni su 04.10.,
ostalo je prijedlog.
**Posljedice.** **Status je izrijekom nepotvrđen:** `ARHITEKTURA.md` (zaglavlje) kaže „prijedlog dok Leon ne potvrdi", dok
`ROADMAP.md` navodi da ju je Leon pregledao; do izričite Leonove potvrde vrijedi stroža formulacija (nacrt). Kostur
(Rust workspace, API, jedan ekran) već je izgrađen na toj osnovi, pa je promjena smjera skupa. Brane determinizma: [ADR-009](#adr-009).

---

## ADR-002 — Ime „HotelixSim", nikad skraćeno na „Hotelix"
**Datum:** 2026-10-04 · **Status:** ✅ ODLUČENO (Leon izabrao sam) · **Vezano:** [ADR-001](#adr-001)

**Kontekst.** Prijedlog „Xenokrat" bio je odabran 03.10. pa vraćen (revert) isti dan. Leonovi kriteriji: kratko (najviše
dvije riječi), odmah vidljivo da je hotelski simulator ili igra, i da ne postoji. Nakon više krugova odbijenih prijedloga
Leon je sam izabrao HotelixSim.
**Odluka.** Ime je **HotelixSim**. Provjereno: `hotelixsim.com` slobodan 04.10., na webu nema ničega s tim imenom.
„Hotelix" (bez Sim) postoji u istoj branši (AI softver za upravljanje hotelima i studio za hotelske stranice), pa se ime
**nikad ne skraćuje** na „Hotelix".
**Posljedice.** Prije javnog izlaska provjeriti žig (EUIPO/DZIV). Folder je `Documents\hotelixsim`.

---

## ADR-001 — HotelixSim je zaseban projekt, odvojen od Sokrat Studyja
**Datum:** 2026-10-03 · **Status:** ✅ ODLUČENO (Leon, sesija planiranja) · **Vezano:** Sokratov ADR-040 (vanjski repo) ·
**Vizija:** [VIZIJA.md](../product/VIZIJA.md) · **Plan:** [ROADMAP.md](../plan/ROADMAP.md)

**Kontekst.** Leon (03.10.): *„mijenjamo cijeli smjer razvoja"*. Profesor s FMTU-a (Opatija, Hospitality Management)
zainteresiran je za simulaciju vođenja hotela, ali prvo želi demo. Sokrat je na produkciji i dovršava se u užem opsegu.
**Odluka.** HotelixSim je zaseban proizvod u zasebnom repozitoriju (`Documents\hotelixsim`, privatni GitHub
`leonkreso784-bit/hotelixsim`). Ne nosi Sokratov kod ni njegova pravila; smije posuditi znanje i obrasce. Od 2026-10-03
je glavni projekt: Leon radi paralelno, ali više vremena daje ovome.
**Posljedice.** Ništa iz simulacije ne ide u Sokratov repo i obrnuto. Izvorna ideja je u
[archive](../archive/HOTEL_SIM_2026-08-22.md). Ocjenjuje se napredak u odnosu na početak, ne zarada.
