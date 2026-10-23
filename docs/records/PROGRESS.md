# Dnevnik rada (PROGRESS)

Najnoviji unos na vrhu. Svaka sesija: što je napravljeno, što je provjereno, što slijedi. **Povijest, nikad izvor istine** — što vrijedi sad piše u [CLAUDE.md](../../CLAUDE.md).

---

## 2026-10-20…23 (13) — Sesija 13: video za profesora, igra spremna za prezentaciju, zapisi sesije 12

- **Video za profesora v2** (izvan repoa, `Documents/hotelixsim-video`): novi tekst glasa (živ, priča; „ocjenjuje **profesor ili AI**"; scena 7
  „grupa studenata", profesor upravlja svime), edge-tts Gabrijela +10 % / +3 Hz (nacrt dok Leon ne izabere), ime fonetski „Hoteliks Sim"; natpisi
  „Profesor ili AI ocjenjuje." i „CIJELA GRUPA". Render **bez glazbe** (Leon sam dostavlja MP3; nađena 3 CC0 kandidata): 96,1 s →
  `Desktop/HotelixSim-video-bez-glazbe.mp4`; stari v1 premješten u `hotelixsim-video/out/`. Na Drive ga Leon stavlja ručno (92 MB, MCP ne može).
- **Prezentacija 20.10.:** lokalni API je radio na starom kodu (26089c3, zdravlje s `events`, [BUGS](BUGS.md) B30) → ugašen, `cargo build -p server`,
  `migrate` (0005 `ai_spend`), `serve` na 9b6f047 (AI uključen, lokalno bez stropova). Proba u čistom pregledniku: nova tjedna igra, lik odgovara
  (~9 s), presuda 5/5 za 9 s, zatvaranje tjedna 4 s s mentorom i ljestvicom; „Za profesora" je samo stari model.
- **Zapisi sesije 12** (ovaj unos, 23.10.): [ADR-063](DECISIONS/ADR-063-pilot-jedna-adresa-lozinka-stropovi-ai-tri-igre.md) (pilot, dio 10 kartice),
  CHANGELOG, BACKLOG (minori Pilot-2/3, L14-1…4, pitanja pilota), BUGS B26–B30.
- **Plan je jedna datoteka do 1000 redaka** ([ADR-064](DECISIONS/ADR-064-plan-je-jedna-datoteka-do-1000-redaka.md), Leon 23.10.): `docs/plan/` s 60 na **5 datoteka**
  (ROADMAP i Leonov pregled spojeni; plan popravka samo s otvorenim L1-15, Pilot, C; gotove kartice i plan Sloja 1 u arhivu, dijelovi ≤ 1000 r.);
  brana vrsta `plan` = 1000 i pravilo 13 „PLAN U MAPI"; alat `razdvoji-plan` uklonjen; poveznice prepisane (143 sidra provjerena).
- **CI nakon pusha (488c108) crven:** neformatiran test (lint nije pokrenut prije commita) i nova sigurnosna upozorenja RUSTSEC-2026-0382/0383/0385/0386 →
  `cargo update` webpki-root-certs 1.0.11, rustls-webpki 0.103.16, ctutils 0.4.5, tokio 1.54.1; `cargo deny` čist, clippy i testovi radnog prostora.
- **Plan sesije 14:** tri read-only agenta složila podloge (L1-15 ~8–14 h, pilot 8–9, faza C); Leonovi odgovori →
  [ADR-065](DECISIONS/ADR-065-leonove-odluke-24-10-tempo-profesora-lokal-generatori.md) (profesor vodi tempo, lokal kao mehanika, mentor bez obećanja, generator događaja).
- **Gdje se vidi:** GitHub `model-v2/sloj1` (Leonov OK za push 23.10.); `main` 9b6f047.

---

## 2026-10-19 (12) — Sesija 12: priprema pilota (dijelovi 1–7) spojena, banka 930 ljudi, L1-14 započet, `sloj1` i `main` na GitHubu

- **Kartica pilota** [task-16](../plan/2026-10-14-popravak-temelja.md#task-pilot-priprema-pilota-na-railwayu) (opus, 10 dijelova, valovi 1–5; 367078a). Leonova anketa
  (4/4 preporučene): kolačić 120 dana s obnovom, `*.up.railway.app`, bez granice po IP-u, gradnja odmah usporedo s L1-14.
- **Spojeno u valovima** (svaki dio svoje stablo i `target/pilot-N`, tok čuvar → graditelj → pregled): dio 7 kopije (6c3acfd), dio 1 kolačić (fd09174),
  dio 5 Dockerfile, dio 6 CI `docker` + `deploy-pilot.yml`, dio 2 vrata (c7ee9f8; pregled opus: Changes requested 4 Important → runda 2 Approved),
  dio 3 strop AI (5c71a5a; opus: 3 Important → runda 2 Approved, sve cijene sa službene stranice), dio 4 tri igre po gostu (d793335; sukobi s dijelom 3
  riješeni ručno). Svi Rust dijelovi pilota spojeni; ostaju dio 8 (tekst privatnosti, Leon) i 9 (postava, nakon L1-15). Odluke → [ADR-063](DECISIONS/ADR-063-pilot-jedna-adresa-lozinka-stropovi-ai-tri-igre.md).
- **Sigurnosno pravilo:** slika se ne isporučuje dok dijelovi 2 i 3 nisu spojeni, a proba u dijelu 9 mora pokazati 401 bez lozinke prije dijeljenja adrese.
- **Provjereno:** cijeli CI nakon dijela 2 i nakon dijela 4 22× EXIT 0 (`cargo test` 1351/0), balans v1, v2 tjedno i mjesečno bajt-isti s
  `nakon-l113b/`; lint EXIT 1 samo na nacrtima banke u `.superpowers/` (B19). Nova osnovica `nakon-pilot-4/`.
- **Banka ljudi K7 spojena** (c51d49b): 930 ljudi (radnici 410, gosti 410, vanjski 110), serije 007 i 008 otvorene do Leonova uzorka. Leon tekstom:
  vanjski su zajednički za sve hotele, malo liječnika je dosta.
- **L1-14 (Leon igra tjednu sezonu):** L14-1 spora presuda → granica 8000 tokena + iskočni prozor (Leon, radi se u L1-15); L14-2 najam lokala poslije
  pilota; L14-3 ideja generatora ljudi → nacrt [BANKA_LJUDI §8](../architecture/BANKA_LJUDI/08-generator-ljudi.md) (9b6f047, 5 pitanja); L14-4 stručni
  pojmovi (celijakija). Mjesečna sezona još nije odigrana. Popis: `.superpowers/sdd/2026-10-14-popravak-temelja/l114-popis-izmjena.md`.
- **Pouke:** omotači `cargo run … serve` u brojanju slota (B26), zaključan `server.exe` (B27), „sve EXIT 0" po imenima koraka (B28), migracije na svježoj bazi (B29).
- **Push** (Leon: „želim samo dići puno commita na githubu"): `model-v2/sloj1` 7556d80..9b6f047, CI 39314933953 zelen (rust, web i prva gradnja slike).
  **Leon sam pushao `main`** 19f7b27..9b6f047 (ff, 429 commitova; spajanje L1-15 time učinjeno prije popravaka iz igre), CI main 39318523973 zelen.
- **Video za profesora v1** (izvan repoa): 78,7 s na Desktopu; Leon: glas „jako loš", i treba reći da ocjenjuje profesor **ili** AI → sesija 13.
- **`/kraj` nije odrađen** (kontekst prerastao); zapisi dovršeni u sesiji 13. **Gdje se vidi:** GitHub, `model-v2/sloj1` = `main` 9b6f047.

---

## 2026-10-19 (11) — Sesija 11: L1-13 dio 4 i cijela L1-13b spojeni i objavljeni (`CORE_VERSION` 2.6.0), prijedlog hostinga pilota

- **L1-13 dio 4, kalibracija, spojen** (575ef92 → e4ec659, `--no-ff`) uz Leonov OK u prvoj anketi (4/4 preporučene: tri svakodnevna kvara nose 0,6,
  održavanje B ostaje, P-GOST-POZLILO ostaje negostinjska, spoji i pushaj). `CORE_VERSION` 2.6.0. CI nakon spajanja: 22× EXIT 0 (sqlx je prvo pao jer
  se baza ugasila uz prekid sesije, [BUGS](BUGS.md) B24), a balans v2 je bajt-isti s ispisom graditelja. Nova osnovica `nakon-l113-4/`. Stabla
  `l113-4` i `l113-4d` s granama i `target/` obrisana. GitHub CI 39238997360 zelen.
- **L1-13b, cijela prije pilota, u valovima** (Leon, druga anketa): kartica [task-15](../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-l1-13b-izmiješani-odgovori-pojačano-održavanje-5-jedan-gumb-spremi-plan-opis-smještaja)
  (ebf28d9) nakon tri analize samo čitanjem. Val 1 usporedo u tri stabla, val 2 nakon dijela 1. Spojeno redom:
  **dio 2** jedan gumb „Spremi plan" (13db069; pregled: I1 i I2 popravljeni, Approved), **dio 3** opis Smještaja (4be1db4; graditelj opus ispravno
  BLOCKED na P-DOTRAJALO, čuvar r2 veže pragove uz sadržaj, Approved), **dio 4** pojačano održavanje +5 (bdbdebe; Approved, 4 Minor),
  **dio 1** izmiješani odgovori (7556d80; Approved, runda 2: zlatni test pravila, 409 za zatvoren krug, opisi).
- **Provjereno:** cijeli CI nakon L1-13b (`nakon-l113b/`, nova osnovica) 22× EXIT 0 + `deny-2` EXIT 0 ([BUGS](BUGS.md) B23); balans v2 bajt-isti s
  ispisom dijela 4, v1 bajt-isti. Zadani plan se nigdje ne mijenja (brane težine, klika i lanca iste). GitHub CI 39259381476 zelen.
- **Održavanje po hotelu** (tablica dijela 4, 1000 sezona): hotel A i dalje štedi (odgođeno najbolje), B i C ulažu (pojačano najbolje; B se okrenuo).
  Šest operativnih odjela je dosta, parking i lokal dodaju 0,02–0,05.
- **Prijedlog hostinga pilota** (opus, samo čitanje i web): Railway Hobby ~5–7 €/mj, AI za 40 studenata ~15–30 € po sezoni, ~20–27 h posla prije
  objave. Nađeno: kolačić gosta traje 7 dana bez obnove (student gubi hotel), a `POST /v1/demo` je otvoren svima. Leon (četvrta anketa): Railway Hobby,
  jedna lozinka za stranicu, stropovi AI 5 $/dan i 30 $/mj, 40 $/mj u konzoli, 3 igre po gostu.
- **Odluke i pretpostavke** → [ADR-062](DECISIONS/ADR-062-leonove-odluke-19-10-sesija-11.md) (i Dopuna ADR-042 t. 10).
- **Banka 11b se ugasila** nakon slanja četiri pisca K7. Ostala su četiri nacrta po 50 ljudi (radnici i gosti 007 i 008), bez izvještaja pisaca:
  sačuvani u `.superpowers/sdd/banka-ljudi-k1/nacrti/k7-sesija-11b/`. Banka u repou ostaje **730**; stablo ljudi e56aa1d čisto.
- **Gdje se vidi:** GitHub, `model-v2/sloj1` 7556d80 + docs-commit ovog kraja. Izvještaj: `.superpowers/sdd/2026-10-14-popravak-temelja/kraj-19-10-11.md`.

---

## 2026-10-19 (10) — Sesija 10 (18.–19.10.): banka 730 ljudi, L1-13 dijelovi 2 i 3 spojeni, dio 4 izgrađen i čeka Leonov OK

- **Banka ljudi spojena dvaput:** K4+K5 (19160c9, 530 ljudi, uloga `lijecnik` ×15) i K6 (ead0daf, 730 ljudi: radnici 310, gosti 310, vanjski 110;
  prvi radnici iz Egipta, Turske i Indonezije, vanjski govore hr i en). CI nakon oba spajanja sav EXIT 0, a balans v2 bajt-isti s prethodnom osnovicom.
  Bilješke sesija banke 9b i 10b prenesene su u docs (dc34c30, a56b53f). Sesija banke 10b je završila (stablo ljudi ea08847, samo `scripts/`, nije SPREMNO).
- **L1-13 dio 2, značaj** (fb3a563): stablo prekinutog čuvara je očišćeno, a novi čuvar napisao je 41 test. Razina se izvodi iz `weight`
  (ključna / važna / svakodnevna), a student je vidi kao tekst na kartici i uz naslov situacije. U repou su sve razine 1,0, pa su brojke i pečat isti
  (`CORE_VERSION` 2.5.0). Pregled opus: Approved, 5 Minor.
- **L1-13 dio 3, gosti** (c1ef169): ključ `guest_extra_slots` (isključen u repou). Mjesta kruga iznad „niske" pune samo situacije s gostima.
  Balans dobiva redak J-3 po popunjenosti. Rađen je usporedo s pregledom dijela 2, na drugim datotekama. Pregled opus: Approved, 5 Minor;
  kontrolirani test 3 (isti krug, različit broj mjesta) proglašen je prihvatljivim.
- **L1-13 dio 4, kalibracija — izgrađen, NIJE spojen** (575ef92 na `popravak/l113-4-kalibracija`). Tok: čuvar (ff41ba8) → graditelj BLOCKED
  (ni faktor 0 ne spušta tjednu težinu B na 25 %; lanac srednji < prepušteno) → dijagnoza opus u zasebnom stablu (`l113-4-dijagnoza.md`) →
  Leonova četvrta anketa (4/4 preporučene) → čuvar runda 2 (465cb18) → novi graditelj runda 2. Brojke: faktor kruga tjedno 110, mjesečno 325,
  klik 85 %, razine 1,5 / 1,0 / 0,6, kvar i popravak 3 077 / 6 154, sve uključeno, `CORE_VERSION` 2.6.0. Situacije nose oko petinu igre
  (tjedno prosjek 20,7 %, mjesečno 21,6 %), klik 63–73 % vlastitog. Pregled opus: **Approved**, 0 Critical/Important, 6 Minor; balans je ponovio sam (bajt-isti).
  Spaja se tek uz Leonov OK na dvije tablice (`l113-4-a2-tablica.md`, `l113-4-prije-poslije.md`).
- **Leonove odluke** (četvrta anketa 18.10.) i pretpostavke 🟨 → [ADR-061](DECISIONS/ADR-061-leonove-odluke-18-19-10-i-pretpostavke-sesije-10.md);
  dopuna [ADR-045](DECISIONS/ADR-045-trosak-odmah-ugled-kasnije-tjedni-krug.md) (u prvom tjednu šutnja je sada u medijanu bolja od najboljeg odgovora).
- **Pouke:** `git -C <repo> worktree add <relativni put>` napravio je stablo unutar glavnog repoa (B22); `cargo deny` pada bez mreže (B23).
  Leon je predložio BRZU GRADNJU (manje testnih binarnih, `rust-lld`, sccache…), ali to još nije odlučeno ([BACKLOG](BACKLOG.md)).
- **Gdje se vidi:** GitHub, `model-v2/sloj1` a56b53f + docs-commit ovog kraja; svako spajanje je pushano, GitHub CI zelen (39141923114 … 39152845046).
  Dio 4 je samo lokalno, u stablu `../hotelixsim-l113-4`. Izvještaj: `.superpowers/sdd/2026-10-14-popravak-temelja/kraj-19-10-10.md`.

---

## 2026-10-18 (9) — Sesija 9 (17.–18.10.): PRAVILO-c, IZGLED-c, L1-13 dijelovi 1 i 1b i banka (280 ljudi) spojeni i objavljeni

- **CI nakon PRAVILO-b dovršen:** ponovljeni `cargo test --workspace` (s bazom) i `corepack pnpm test` oba EXIT 0. Osnovica za dalje je `nakon-pravilo-b/`.
- **PRAVILO-c spojen** (8e39085): napomena uz ponuđene odgovore kad klik daje najviše 4, a presuda kaže je li odgovor prepisan ili zalijepljen.
  Pregled sonnet Approved, 2 Minor. CI: sve EXIT 0, a v2 tjedno i mjesečno bajt-isti s `nakon-pravilo-b/`.
- **IZGLED-c spojen** (879da8a + dopuna 69762f1): policajac, vatrogasac (s kacigom), djelatnik hitne (crvena odora, plava zvijezda života) i
  liječnik (nova uloga `lijecnik`, bijela kuta i stetoskop) nose odoru na avataru; `/likovi` ima red „Službe". Pregled sonnet Approved.
  Raspored na mobitelu popravio je voditelj (a833ec9). Prvi commit tog popravka nije uspio (`-am` uz `-F`), pa je trebalo drugo spajanje ([BUGS](BUGS.md) B17).
- **L1-13 kartica i dijelovi:** puna kartica s pet dijelova (d2be165). **Dio 1, kvar prostora** (7f22802): kvar skida 1 bod stanja prostora, a hitni
  popravak uz ocjenu 4–5 vraća 2. U repou je iza ključa, isključeno, pa je balans bajt-isti. Pregled opus Approved, 3 Minor.
  **Dio 1b, klik** (9813b8a): u situaciji s likom klik zadrži 85 % dobiti, a trošak je pun. Graditelj je bio BLOCKED (ni dobit 0 nije držala
  branu od 75 %). Dijagnoza opusa pokazala je zašto: situacije bez lika nose 39 % / 70 % Δ, a trošak klik smanjuje. Leon je zatim odlučio
  M1: brana mjeri samo situacije s likom. Pregled opus tražio je 8500 umjesto neodobrenih 9000 i presudio 5 izmjena testova čuvara (B18).
- **Banka ljudi spojena** (ee0f673, 280 ljudi: radnici 110, gosti 110, vanjski 60; pečat 000/001). Paralelna sesija banke 9b zatim je napravila
  K4 i K5 i ulogu `lijecnik` (530 ljudi, grana 106ede1 „SPREMNO ZA SPAJANJE"; spojena u sesiji 10 kao 19160c9, a K6 kao ead0daf, 730 ljudi).
- **Leonove odluke** (pet anketa i tekst): uniforme, klik ~75 % gdje se piše, razine značaja, „zadovoljan gost" ≥ 4, kvar = 9 predložaka, „s gostima" 27/55,
  **pilot odobren na kraju faze B**, a bez podrške fakulteta nema cijelog proizvoda do studenog → [ADR-060](DECISIONS/ADR-060-leonove-odluke-18-10-i-pretpostavke-sesije-9.md)
  (i pretpostavke 🟨 t. 17–20). Leon dao „ok za sve" za push `model-v2/sloj1`.
- **Railway:** na Leonov zahtjev obrisana su oba projekta KuhAI (`ajkuham` s Postgresom i `zoological-sparkle`). Railway briše s odgodom 48 h
  (`deletedAt` 2026-10-20 01:23 UTC), a do tada se mogu vratiti ([BUGS](BUGS.md) B21).
- **L1-13 dio 2 (značaj):** čuvar je zaustavljen usred rada na Leonov „idemo završit za danas". Ništa nije commitano, a necommitano je spremljeno
  u `l113-2-provjere/cuvar-prekinut-necommitano.diff`. Sesija 10 čisti stablo i šalje novog čuvara.
- **Gdje se vidi:** GitHub, `model-v2/sloj1` d8d991f (+ e700e7a predaja i docs-commit ovog kraja). Svako spajanje je pushano, a GitHub CI je zelen
  (39075464484 … 39096546459). `main` ostaje demo do kraja faze B. Izvještaj: `.superpowers/sdd/2026-10-14-popravak-temelja/kraj-18-10-9.md`.

---

## 2026-10-17 (8) — Sesija 8: Leonovih 43 odgovora u ADR-ove, IZGLED i PRAVILO-b spojeni, PRAVILO-c započet

- **Jutro (samo dokumenti):** Leonovi odgovori na 43 pitanja i razgovor → [ADR-056](DECISIONS/ADR-056-leonove-odluke-17-10-odgovori-na-43-pitanja.md),
  [ADR-057](DECISIONS/ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md), [ADR-058](DECISIONS/ADR-058-pretpostavke-sesije-7-izgled-vanjski-platforma-k2.md) (ec3ba48);
  kartica [L1-13](../archive/2026-10-14-popravak-temelja-kartice/dio-5.md#task-l1-13-jedna-kalibracija-brojki-kvar-i-popravak-klik-75--značaj-gosti-i-popunjenost-težina-20-), BANKA_LJUDI s nacrtom rizika C3 ([07-poznate-osobe-rizik](../architecture/BANKA_LJUDI/07-poznate-osobe-rizik.md)),
  VODITELJ_I_TRZISTE, ROADMAP (ee67b23); ispravak ADR-056 B2 (e71ecc4); sitnice BACKLOG-a (748bcab); dubinska provjera arhitekture E4
  ([PROVJERA_ARHITEKTURE](../architecture/PROVJERA_ARHITEKTURE_2026-10-17.md), e581849: „ok uz uvjete", 7 stavki, 5 pitanja za Leona).
- **CI nakon PRAVILO-a potvrđen zelen:** `cargo test --workspace` prvi put pao na linkeru (0xC0000142, okolina), ponovljen EXIT 0; balansi bajt-isti.
- **IZGLED (a+b) spojen** (872ff46): lik u situaciji i razgovoru dobiva rod, dobnu skupinu i podrijetlo; web crta lice iz tih podataka. Pregled opus DA
  (0 C / 1 I — sukob duga `check-kod.mjs` riješen pri spajanju / 7 M). CI nakon spajanja svi EXIT 0 (vitest „Timeout waiting for worker" prvi put, ponovljen 0); balans bajt-isti.
- **PRAVILO-b spojen** (ee6de80): balans dobiva put „klik najbolji" (najviše 4 gdje se piše vlastiti odgovor), lanac B3 s 4 koraka, paritet s igrom.
  Voditelj popravio dva testa (d8a7cc7, b0335d4): monotonija „klik ≤ vlastiti" vrijedi nad zbrojem, ne po sjemenu ni medijanu ([BUGS](BUGS.md) B14).
  Pregled opus DA (0 C / 2 I / 7 M). **Za Leona (I1):** klik nosi 92–95 % učinka vlastitog odgovora (gubi samo ocjenu, ne mehanike) → odluka u anketi L1-13.
- **CI nakon PRAVILO-b:** fmt, clippy, core-deps, probe-mutacija, git diff, sqlx, balansi v1/v2 svi EXIT 0; v1 bajt-isti, v2 tjedno i mjesečno razlikuju se
  samo u bloku klika (pogledano na `/kraj`); `cargo test --workspace` EXIT 127 (0xC0000142) — skripta zaustavljena na Leonov „zaustavi sve"; ponoviti u sesiji 9.
- **PRAVILO-c u tijeku:** čuvar 2c7af1a (7 RED testova); graditelj zaustavljen s necommitanim izmjenama u `../hotelixsim-pravilo-c`.
- **Paralelna sesija banke ljudi** (Leon 17.10.): pečat 000/001, vanjski 10 + 50, K3 serija-002, K4 u tijeku, rebase na ee6de80. Leon traži uniforme za
  avatare vanjskih (policija, vatrogasci, hitna) → nova cigla IZGLED-c u glavnoj sesiji.
- **Disk** pao na 45 GB dok su gradile tri stvari; obrisani stari `target/<stablo>` → 77 GB ([BUGS](BUGS.md) B16). Pretpostavke: [ADR-059](DECISIONS/ADR-059-pretpostavke-sesije-8-pravilo-b-izgled-k7-pravilo-c.md).
- **Gdje se vidi:** lokalno, `model-v2/sloj1` ee6de80 + docs-commit; **ništa danas nije pushano** (20 commita ispred `origin` s docs-commitom kraja). Main ostaje demo do kraja faze B.
  Izvještaj: `.superpowers/sdd/2026-10-14-popravak-temelja/kraj-17-10-8.md`.

---

## 2026-10-17 (7) — Sesija 7 (16.–17.10.): PRAVILO-a spojen i objavljen, IZGLED započet, paket od 43 pitanja

- **PRAVILO-a spojen** (9a275d8, `CORE_VERSION` 2.5.0) i uz Leonov OK pushan; GitHub CI zelen. Pregled opus našao lažno „zalijepljeno" na dobrom
  odgovoru → pravilo o parovima susjednih riječi ([BUGS](BUGS.md) B13).
- **Leonove ankete:** IZGLED odmah usporedo s PRAVILO-om, B13 = neutralna 3, stil lika crtić iz podataka; vanjski su treća vrsta banke ljudi; platforma od studenog
  → [ADR-057](DECISIONS/ADR-057-leonove-odluke-sesije-7-redoslijed-b13-vanjski-platforma.md).
- **Dokumenti:** nacrt [IZGLED_LIKA](../architecture/IZGLED_LIKA.md), kartica IZGLED, nacrt [plana platforme](../plan/2026-11-platforma.md),
  [SESIJA_BANKE](../workflow/SESIJA_BANKE.md), paket [43 pitanja](../plan/2026-10-17-leonov-pregled.md) (i Word na Leonovoj radnoj površini).
- IZGLED-a i -b napisani u stablu `../hotelixsim-izgled`, dovršeni u sesiji 8. `/kraj` sesije 7 nije pisan; ovaj unos je napisan na kraju sesije 8.

---

## 2026-10-16 (6) — Sesija 6: B4c dovršen, banka ljudi K1a + K1b, kartica PRAVILO

- **B4c spojen** (`model-v2/sloj1` f7ac7cc, CI 24/24): oznake 18 predložaka (najdulja ≤ 1,39× najkraće; dug `LabelLengths` 0 za svih 55) i uvjeti kvarova
  P-BRAVE, P-CURENJE-VODE, P-DIZALO = Smještaj ispod **80**; P-INSPEKCIJA vanjski događaj; dug `BreakdownCondition` 0. Težina tjedno A 41,9 → 41,6 %,
  B 48,9 %, C 41,2 %; mjesečno 25,7 / 33,8 / 20,3 %. Pregled opus: Needs fixes → runda 1 (P-ALERGIJA, nadskup u 5 predložaka, jezik) → Approved.
- **Prag 80 je Leonova odluka** (anketa uz izmjerenih 7 inačica): 50–70 ruše tjednu branu J-3 jer hotel A počinje na 75 → [ADR-054](DECISIONS/ADR-054-leonove-odluke-16-10-prag-80-kopiranje-odgovora.md) t. 1.
- **Banka ljudi K1a spojen** (adf5a4d): modul `ljudi` u `crates/content`, okviri `content/hotel/v2/ljudi/`, 42 testa; igra netaknuta, balansi bajt-isti. Pregled sonnet: Approved.
- **K1b** (de2dc35 čuvar, 440fc0c + 9c75d41 graditelj): serija-000 = 10 radnika + 10 gostiju; serije OTVORENE do Leonova pregleda (`pregled-20.md` i `pregled-20.html`).
  Pregled: gradovi u dva oblika i "tiskar" popravljeni; **rod po ulozi ide Leonu**.
- **Kartica PRAVILO** napisana (224 r.; podjela a/b/c, `CORE_VERSION` 2.5.0) → [plan](../plan/2026-10-14-popravak-temelja.md); pretpostavke u [ADR-055](DECISIONS/ADR-055-pretpostavke-16-10-sesija-6-pravilo-ljudi-target.md).
- **Novo: kvar B13** (Leon igrajući): zalijepljeni svi ponuđeni odgovori često daju 5; prepoznavanje u kodu, ne u AI-ju → [BUGS](BUGS.md), pravilo anketom u idućoj sesiji.
- **Pouka B12:** zajednički `CARGO_TARGET_DIR` među stablima daje zastarjelu biblioteku (lažno zeleno moguće) → svako stablo svoj target ([AGENTI](../workflow/AGENTI.md) t. 6).
- **Leonov smjer:** "faza C i dalje razvoj likova, u sučelju odmah njihov izgled" → nacrt i anketa o redoslijedu na početku iduće sesije ([SLJEDECA_SESIJA](../SLJEDECA_SESIJA.md)).
- **Gdje se vidi:** lokalno (`model-v2/sloj1`, **ništa nije pushano**, ~24 commita iza `origin`). Ledgeri: `.superpowers/sdd/2026-10-14-popravak-temelja/progress.md`,
  `.superpowers/sdd/banka-ljudi-k1/progress.md`; izvještaj `kraj-16-10-6.md`.

---

Stariji unosi (16.10. sesija 5 i ranije): [PROGRESS/2026-10.md](PROGRESS/2026-10.md).
