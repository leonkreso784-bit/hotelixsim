# BANKA_LJUDI §8 · Generator ljudi umjesto (ili uz) banku — načini i cijena

Dio indeksa [BANKA_LJUDI.md](../BANKA_LJUDI.md). **Status:** 🟨 NACRT ZA RASPRAVU, ČEKA LEONA (19.10.2026.). Arhitektura ide kroz
dokument i raspravu ([ADR-006](../../records/DECISIONS/ADR-006-arhitektura-kroz-dokument-i-raspravu-ne.md)); ovdje se ništa ne
gradi. Povod, Leon 19.10. (dok igra): *„Ja mislim da bi bilo najbolje da umjesto da pišemo sve ljude zasebno da napravimo
generator ljudi odmah u igri."*

**Riječi.** *Generator* = program koji lika složi sam, kad zatreba. *Sjeme* = broj iz kojeg igra izvodi sve „slučajno"; isto sjeme
daje iste ljude, pa se igra može ponoviti i provjeriti. *Token* = komadić teksta koji AI čita ili piše; po njima se plaća.

## 8.1 Što je danas

- **Banka:** 930 ljudi u repou — radnici 410, gosti 410, vanjski 110 (serije 000 i 001 zapečaćene; ledger banke, spajanje K7
  `c51d49b`). JSON datoteke u `content/hotel/v2/ljudi/`, ne baza (§4.1).
- **Kako nastaju:** serija od 50 = pisac (AI) → provjera imena webom → pregled (AI) → voditelj → Rustova brana → cijeli CI.
  Izmjereno u K7: **~0,6–0,7 M tokena i ~1 h agenata po seriji**, plus ~45 min CI-ja po točki od 2–4 serije (ledger banke,
  „Trošak serije"). Pregled nađe 10–30 ispravaka teksta po seriji („isti život", kalupi), 0 kritičnih.
- **Što banka jamči:**
  1. **Provjerena imena.** Svako ime traženo webom; zamijenjeno je **23 od 200** u K6 i **29 od 200** u K7 (javne osobe ili
     sudar s bankom). Dakle i pažljivo pisana imena u ~12–15 % slučajeva pogode stvarnu ili već zauzetu osobu.
  2. **Pregledan opis** (ton, rod, padež, prirodan hrvatski, bez stereotipa u frazi — §3.6, stupac „čovjek").
  3. **Raznolikost po pravilima** koje stroj broji (§2, §3.3). Pravilo stereotipa (razina, dob i rod ne smiju ovisiti o podrijetlu)
     prvi put se uključilo na 410 radnika: ćelija sobar/lokalno n = 54 pala je za iskusne (+554 bp) i morala se ispraviti parom
     zapisa; upozorenje da sobar/inozemstvo (n = 35) padne kad prijeđe 40 (ledger banke, sesija 12b).
  4. **Bez ćirilice, zabranjenih osoba, hotela i marki** (§3.5), stroj pri svakom `cargo test -p content`.
- **Što igra stvarno koristi:** **ništa iz banke još** (K5 nije građen; „ljudi:: izvan content 0", ledger banke). Likovi situacija
  uzimaju ime iz `content/hotel/v2/names.json` (120 imena): popis po izvoru i rodu `crates/sim-hotel-v2/src/situations/fill.rs:90`,
  izbor `fill.rs:99`. AI lik dobiva „ime, opis uloge iz predloška" (`crates/ai/src/request.rs:135`). Izgled lica se pogađa
  po tome u kojem je popisu ime (`crates/app/src/look.rs:135-160`), pa ime izvan `names.json` danas nema lice.
- **Ovisi li balans o ljudima?** Ne o banci (igra je ne čita). Ime lika situacije bira se istim tokom kocke kao brojke te situacije
  (`fill.rs:83`, ime `:99`, brojke `:128`); kocka zbog „odbijanja" (`crates/engine/src/rng.rs:56-62`) u teoriji smije potrošiti
  drukčiji broj bacanja kad se promijeni veličina popisa imena, u praksi skoro nikad. Banka po načelu daje samo **tko je osoba**,
  a brojke (vještine, plaća) računa jezgra (§1.1) — zato ni banka ni generator ne bi smjeli pomaknuti balans; jedan `--check`
  ostaje dokaz. **Usporedivost razreda** ne ovisi o tekstu lika, nego o tome da isto sjeme daje iste ljude — to može i generator.
- **Pravilo koje Leonova ideja dira:** §4.2 kaže *„Ime se nikad ne slaže iz dijelova"* (`04-gdje-zivi.md:38`).

## 8.2 Koliko ljudi treba — i zašto je to pravo pitanje

Novo (Leon banci 19.10.): gosti se vide i u **recenzijama**. Sesija banke je izračunala dvije mogućnosti (ledger banke):
**R-A** svaki recenzent jedinstven u razredu (~10 500 gostiju) i **R-B** recenzent se ne ponavlja samo unutar hotela (~1 500
gostiju; preporuka banke). Leon još nije odabrao. Uz radnike ~3 000 (§2.1, M3) i vanjske **110** (Leon 19.10.: vanjski su
zajednički za sve hotele, postojećih dosta; sesija banke računa s 600 pa dobije ~5 000 i ~14 000):

| | cilj | novih (− 930) | serija po 50 | tokeni (× 0,6–0,7 M) | agenti | CI (točka = 4 serije) | Leonovi uzorci (10/seriji) |
|---|---|---|---|---|---|---|---|
| **R-B** | 3 000 + 1 500 + 110 = 4 610 | 3 680 | **74** | **44–52 M** | ~74 h | ~19 × 45 min ≈ 14 h | ~740 |
| **R-A** | 3 000 + 10 500 + 110 = 13 610 | 12 680 | **254** | **152–178 M** | ~254 h | ~64 × 45 min ≈ 48 h | ~2 540 |

Tokeni su kvota Leonove pretplate Claude Codea, ne račun API-ja (🟨 pretvorba u € nije izračunata). Usko grlo nije samo kvota
nego i **Leon**: uzorci K3–K7 još nisu svi pregledani (ledger banke). R-A ručnom bankom nije razuman; **recenzent pokazuje samo ime
i zemlju** (§2.1), a za to ne treba ni opis ni pregled — tu je generator očito jači.

## 8.3 Četiri načina

### A · Generator u igri iz dijelova, po sjemenu, bez AI-ja
Igra složi lika iz popisa: ime × prezime (po zemlji i rodu) × mjesto × uloga × dob × razina × jezici × rečenice opisa iz
gotovih dijelova („Prije je radio …", „Na Kvarner dolazi …"). Sjeme igre (i utor hotela) određuje izbor.
- **Student vidi:** puno ljudi, nikad isti razred dvaput isti; opisi kratki i **sličnog kroja** (pregledi banke svaku seriju
  bore se baš protiv kalupa, a generator je sam kalup).
- **Raznolikost:** raspodjele (§2) i pravilo stereotipa **točne po građi** — podrijetlo se ne smije ni čitati pri izboru razine,
  dobi i roda. Ćirilica nemoguća (dijelovi su provjereni jednom).
- **Rizici:** spoj dva obična dijela daje ime poznatog političara ili sportaša (oba dijela česta, spoj slavan); web provjera svakog spoja u igri nije
  moguća → treba **popis zabranjenih punih imena** javnih osoba (🟨 nekoliko stotina, ručno) i ostaje preostali rizik. Ime iz
  dijelova traži i ispravan rod prezimena (slovački `-ová`, §3.2) za svaku zemlju.
- **AI trošak:** 0 $. **Rad:** 🟨 15–25 h (dijelovi i njihov pregled, Rustov modul, testovi determinizma, vlastiti tok kocke
  npr. `ljudi:gen`) + Leon ~1–2 h za pregled dijelova. Dijelovi se mogu izvući iz 930 provjerenih imena banke.
- **Jezgra i balans:** novi tok kocke, postojeći se ne diraju (§4.2); `CORE_VERSION` raste, stare probne igre daju 409 kao i
  danas. Popisi dijelova postaju zapečaćen sadržaj (promjena dijela = drugi ljudi u staroj igri, osim uz verziju).
- **930 postojećih:** postaju izvor dijelova ili se ne koriste. **Sesija banke:** staje.

### B · AI piše lika u igri (pri ulasku u igru ili kad se lik prvi put pojavi)
- **Student vidi:** najživlje opise; ali **čeka** 🟨 3–6 s po pozivu (10 likova po pozivu) ili dulje na početku igre.
- **Kvaliteta:** nitko je ne pregleda prije studenta; Rust može provjeriti svaki zapis (oblik, ćirilica, zabranjene riječi i
  imena), ali **ne** raspodjelu ni stereotip (to je svojstvo cijele banke) ni „isti život". Imena bez web provjere — u banci
  pažljiv pisac s provjerom i dalje treba zamjenu za ~12–15 % imena (§8.1).
- **AI trošak** (cijene iz pilot-3: Haiku 5.5 0,10 $ ulaz / 0,50 $ izlaz, Sonnet 5.5 2 $ / 10 $ po milijunu tokena;
  🟨 pretpostavka: poziv za 10 likova = 3 000 tokena ulaza + 1 200 izlaza):
  Haiku 0,0009 $ / 10 likova, Sonnet 0,018 $ / 10 likova. Igra treba **~110** likova (studentov hotel: ~65 radnika + ~31 gost +
  ~12 vanjskih, §2) do **~900** (cijela igra za vježbu s burzom, §2.1):

  | | po igri | pilot: 40 studenata × 3 igre = 120 igara | prema stropu 30 $/mj, 5 $/dan |
  |---|---|---|---|
  | Haiku | 0,01–0,08 $ | **1,2–9,7 $** | stane, ali uzme do ~⅓ mjesečnog stropa |
  | Sonnet | 0,20–1,62 $ | **24–194 $** | ne stane |

- **Rizici:** stvarne i poznate osobe (§7) bez ikakvog filtra prije studenta; javni repo ih ne bilježi, ali student ih vidi.
- **Determinizam:** tekst AI-ja nije ponovljiv → mora u dnevnik događaja (kao presuda), a **balans i roboti ne zovu AI** pa im
  treba rezerva — **a to je opet A ili banka**. Kad strop troška padne, nova igra bez rezerve nema ljude.
- **Rad:** 🟨 20–30 h (uputa, posao u redu, događaj u dnevniku, rezerva, ekran čekanja) **plus** rezerva iz A. **Sesija banke:** staje.

### C · Hibrid: banka za „lica s pričom", generator za „mase"
- **Banka** (kao danas, provjerena i pregledana) daje one s kojima student **razgovara ili ih bira**: likove situacija (gosti,
  vanjski), početno osoblje hotela. **Generator A** daje one koje student vidi samo kao **ime i zemlju**: recenzente,
  🟨 kandidate burze i agencije dok ih student ne otvori, ljude robotskih hotela u igri za vježbu.
- **Student vidi:** priče tamo gdje se čitaju, mnoštvo tamo gdje je mnoštvo. Recenzije više nisu pitanje banke (R-A i R-B se
  svode na postavku generatora).
- **Koliko banke:** radnici ~2 250 (početni 1 950 + ~15 % zalihe, §2.1, bez burze i agencije) + gosti ~1 000 (M2) + vanjski 110
  ≈ **3 360 → ~49 serija ≈ 29–34 M tokena**, ~49 h agenata, ~12 točaka CI-ja (≈ 9 h) — oko ⅓ manje od R-B i ~⅕ od R-A.
- **Rizici:** generator samo za imena (bez opisa) → manji rizik kalupa; poznata osoba iz spoja ostaje (popis zabranjenih, kao A).
- **AI trošak u igri:** 0 $. **Rad:** generator samo imena 🟨 8–12 h (lakši od A: bez rečenica opisa) + K5 (koji treba svaki način).
- **930 postojećih:** ostaju i koriste se. **Sesija banke:** nastavlja, s nižim ciljem.

### D · Generator izvan igre (alat koji brže puni banku)
Skripta u `scripts/` složi **kostur** zapisa (uloga, dob, razina, podrijetlo, zemlja, jezici) točno po raspodjelama — stereotip
i raspodjela prolaze po građi; AI pisac piše samo ime i opis; provjera imena i pregled ostaju.
- **Student vidi:** isto kao danas (sve iz banke, sve pregledano). **Igra i jezgra:** ništa se ne mijenja.
- **Ušteda:** 🟨 pisac 250–400k → ~100–150k po seriji; provjera imena (~120k) i pregled (~135k) ostaju → ~0,35–0,45 M po
  seriji, oko **35–45 % manje**; Leonovi uzorci isti.
- **Rad:** 🟨 4–6 h alat. **Sesija banke:** nastavlja brže. Kombinira se s C.

## 8.4 Usporedba

| | A u igri | B AI u igri | C hibrid | D alat |
|---|---|---|---|---|
| AI $ po pilotu | 0 | Haiku 1–10 $, Sonnet 24–194 $ | 0 | 0 |
| tokeni banke do cilja | 0 | 0 | ~29–34 M (s D ~17–22 M 🟨) | R-B ~26–33 M, R-A ~89–114 M 🟨 |
| sati gradnje 🟨 | 15–25 | 20–30 + A | 8–12 | 4–6 |
| imena provjerena webom | ne (popis zabranjenih) | ne | da za lica s pričom | da |
| opis pregledan | dijelovi jednom | ne | da za lica s pričom | da |
| stereotip i raspodjela | točno po građi | ne kontrolira se | banka + građa | po građi + brana |
| mijenja jezgru | da | da (+ dnevnik, red) | da (samo imena) | ne |

**Što vrijedi za svaki način:** igra mora najprije naučiti čitati ljude (**K5**: lik nosi rod, dob i podrijetlo, a lice se više
ne pogađa iz `names.json`, `look.rs:135`). **Jezgra se za trajanje pilota zamrzava** (CLAUDE.md), pa A, B i C u igri dolaze **tek
nakon pilota**; pilot igra s 120 imena kao danas. D se smije odmah (dira samo banku).

## 8.5 Preporuka voditelja

**C + D**: generator u igri za mase (recenzenti, burza, robotski hoteli), banka za lica s pričom, a banku dalje puni alat s
kosturom. Razlozi:
1. Leonova ideja dobiva ono što najviše štedi: R-A bi ručnom bankom bio ~254 serije; uz C recenzenti koštaju 0.
2. Ono što student **čita** ostaje provjereno: ~12–15 % imena koje je AI dao trebalo je zamijeniti nakon web provjere (§8.1);
   bez te provjere (A za opise, B) to dolazi studentu i profesoru, u nastavni materijal javnog fakulteta (§7.4).
3. B nije jeftiniji ni sigurniji: treba rezervu (opet A ili banku), čekanje, nadzor koji ne postoji, i stane u strop samo s Haikuom.
4. Stereotip i raspodjele generator dobiva besplatno, po građi — to vrijedi i za kostur iz D.

**Cijena preporuke** 🟨: D 4–6 h odmah; C 8–12 h + K5 nakon pilota; banka ~49 serija (s D ~17–22 M tokena umjesto ~29–34 M).
**Ako je krivo:** C → A kasnije je proširenje generatora na opise (banka ostaje kao izvor dijelova); A → C nakon objave znači
vraćati provjeru imena za sve što su studenti već vidjeli.

## 8.6 Što Leon odlučuje

1. **Način:** C + D (preporuka), samo A, samo B, samo D ili ostaje banka kao danas?
2. **Recenzije:** smije li recenzent biti lik iz generatora (samo ime i zemlja)? Ako ne — R-A ili R-B?
3. **Ime iz dijelova:** smije li generator slagati ime (mijenja §4.2 „ime se nikad ne slaže iz dijelova"), uz popis zabranjenih
   javnih osoba?
4. **Kada:** generator u igri tek nakon pilota (jezgra zamrznuta), alat D odmah — slažeš li se?
5. **Sesija banke do odluke:** nastavlja K8 (radnici + gosti, trebaju u svakom načinu osim čistog A i B) ili čeka?
