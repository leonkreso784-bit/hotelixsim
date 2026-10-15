# Dnevnik rada (PROGRESS)

Najnoviji unos na vrhu. Svaka sesija: što je napravljeno, što je provjereno, što slijedi. **Povijest, nikad izvor istine** — što vrijedi sad piše u [CLAUDE.md](../../CLAUDE.md).

---

## 2026-10-15 (4) — faza B: B2a, B2b, B3a, B3b, B1a spojeni u sloj1

- **Spojeno lokalno u `model-v2/sloj1`** (svaka cigla: čuvar testova → graditelj → pregled opus → spajanje `--no-ff`): B2a 143f593
  (broj situacija u postavkama, mjesečno 4/5/6, `CORE_VERSION` 2.3.0; [BUGS](BUGS.md) B3 popravljen), B2b 635e647 (`--situations L,M,H` u balansu),
  B3a 76271d1 (mjera udjela situacija, pet putova), B3b b94264a (brane B3 u `failures_v2`), B1a 39e5f25 (popravak diže stanje
  svog odjela, 2.4.0). Nakon svakog spajanja **cijeli popis iz `ci.yml`** (23–24 provjere EXIT 0); v1 balans bajt-isti kroz sve cigle.
- **Leonove odluke:** anketa B3 („~20 % mjeri se od ravnala", mjesečni dug redoslijeda) → [ADR-049](DECISIONS/ADR-049-situacije-dvadeset-posto-mjeri-se-od-ravnala.md);
  anketa B1a („mehanizam sad, iznos u L1-13", privremeno +1) → [ADR-050](DECISIONS/ADR-050-popravak-podize-odjel-iznos-u-l1-13.md);
  pretpostavke voditelja 🟨 → [ADR-051](DECISIONS/ADR-051-pretpostavke-faze-b-situacije-i-mjerenje.md).
- **Važni nalazi pregleda:** B3a C1 (medijan se računao kao razlika medijana, ne po sjemenu; popravljeno 9f98a5f); B3b: čuvarov test
  „negativan korak pada" protivio se Leonovoj odluci o dugu → strop −25 umjesto „točno 0"; B1a: bonus +10 bez faktora kruga rušio je
  težinu (tjedno do 72 %) → Leon: mehanizam sad, iznos u L1-13, privremeno +1; dug redoslijeda time otplaćen.
- **Smetnje:** `check:api` pada kad drugo stablo istodobno prevodi (zajednički `target`) → nakon spajanja vlastiti `CARGO_TARGET_DIR`
  ([AGENTI](../workflow/AGENTI.md)); vitest mentor-poll testovi (SeasonScreen) nestabilni pod opterećenjem ([BACKLOG](BACKLOG.md)).
- **Rizik za dalje:** tjedna težina B 48,9 % uz granicu 50 % — B4 mora izmjeriti `--check` 1000 tjedno i mjesečno prije spajanja.
- **Gdje se vidi:** lokalno (`model-v2/sloj1` 39e5f25); stare probne v2 igre daju 409 (jezgra 2.4.0). Ledger:
  [arhiva](../archive/2026-10-15-popravak-temelja-faza-b-ledger.md). Sljedeće B1b → B4a–c → PRAVILO → L1-13.

## 2026-10-15 — Faza A popravka gotova; kartice faze B; novo pravilo bodovanja

- **A4–A6 spojeni u sloj1** (760dbec, 8f931fc, 024160a): A4 pregled i runda popravka (JS provjere i cmp balansa), A5 `lik.v3` i
  provjera mentora kruga (živi AI test 6/6 uz Leonov OK), A6 pet + t05 scenarija „što student vidi" (probe ugriza vraćene).
  Cijeli `ci.yml` popis na spojenom sloju: 24/24 EXIT 0, balans bajt-isti kao prije.
- **Nalaz A6 i Leonova odluka:** tjedni krug, odgovor u t01 gubi od šutnje (trošak odmah, ugled kasnije); Leon: igra ostaje, objasniti
  ([ADR-045](DECISIONS/ADR-045-trosak-odmah-ugled-kasnije-tjedni-krug.md)).
- **Završni pregled faze A** (opus): „S popravcima", 6/7 uvjeta; I1–I4 ugrađeni u kartice B. Kopija grana `hotelixsim-kopije/2026-10-15`.
- **Leon odigrao mjesečnu v2 igru:** „puno bolja je igrica" (7. uvjet ispunjen); želje i opažanje o moralu u [BACKLOG](BACKLOG.md).
  Push `model-v2/sloj1` 024160a uz njegov OK; GitHub CI success (rust + web).
- **Kartice faze B** napisane i usklađene s planom (B1a/b, B2a/b, B3a/b, B4a–c); Leonove odluke:
  [ADR-046](DECISIONS/ADR-046-leonove-odluke-15-10-za-fazu-b.md). Novo pravilo igre, vlastiti odgovor vrijedi više (opcija A):
  [ADR-047](DECISIONS/ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md), nacrt
  [BODOVANJE_VLASTITOG_ODGOVORA](../architecture/BODOVANJE_VLASTITOG_ODGOVORA.md). 🟨 [ADR-048](DECISIONS/ADR-048-lik-v3-redni-broj-poruke-i-zakljucak.md) (`lik.v3`).
- **Gdje se vidi:** lokalno i GitHub (`model-v2/sloj1` 024160a); bilješke iz gitignoriranog `za-dokumentaciju.md` prenesene (I4).

## 2026-10-14 (3) i 15.10. — revizija temelja, odluka „popravak", faza A (A1–A3, A7), GitHub za kolegu, disk

- **Revizija temelja** (šest revizora, samo čitanje, nad `sloj1-l111`): temelj stoji (dnevnik, determinizam, snimak pravila,
  baza, red poslova, AI iza sučelja); puca na v1 kalupu ispod v2, branama koje mjere robota i dugim granama →
  [REVIZIJA_TEMELJA.md](../architecture/REVIZIJA_TEMELJA.md). Leon: **popravak, ne ispočetka**, samo to dok nije gotovo
  ([ADR-043](DECISIONS/ADR-043-popravak-temelja-ne-gradnja-ispocetka.md)); plan Sloja 1 pauziran, novi
  [plan popravka](../plan/2026-10-14-popravak-temelja.md) (7 kartica faze A, obris B i C).
- **Spojeno u sloj1** (čuvar → graditelj → pregled, svaka cigla): A1 brane modela ruše `--check`, mjere robota se ispisuju,
  situacije ušle bez kalibracije (L1-11 ✅); A2 načela igre kao provjere sadržaja pri startu servera (dug 55/6/16 smije
  samo padati); A3 pečat sezone v2 uz `CORE_VERSION` 2.2.0 (grize: 840→841 pada); A7 `corepack pnpm grane` i pravila kratkih
  grana. Nakon svakog spajanja balans v2 tjedno/mjesečno i v1 bajt-isti, pečat zelen. A4 (igra = balans) čeka pregled, A5 graditelja.
- **GitHub** (Leonov OK, kolega pregledava 16.10.): `sloj1` pushan; `main` dobio README s cijelom idejom i `LICENSE` sva prava
  pridržana ([ADR-044](DECISIONS/ADR-044-licenca-sva-prava-pridrzana.md)) i popravak `rustls`. CI zelen na obje (nakon B12).
- **Disk** pao na 12 GB (B11): ugašeni stari pregledi grana, obrisana stabla spojenih grana, napuštene mape i stari `target/`
  → >120 GB; pravilo: jedan zajednički `CARGO_TARGET_DIR`, stablo se briše isti dan nakon spajanja.
- **Sljedeće:** [SLJEDECA_SESIJA.md](../SLJEDECA_SESIJA.md) — pregled A4, A5, A6, kraj faze A.

---

## 2026-10-14 (2) — situacije do kraja (G-2, A-1, W-1), Leonov pregled igre, obnova spojena u sloj1

- **Grana situacija** (`../hotelixsim-s1-sit` ea19d54), svaka cigla čuvar → graditelj → pregled: G-2 popravak 1 (e95393b, Approved);
  A-1 API situacija + naredba „prepusti voditelju" po P4 (6f44e25, Approved; balansi bajt-isti); W-1 ekran situacija (33353fb) i
  popravak 1 (ea19d54: obvezne samo uz voditelja, sidra bez „Cijeli hotel"); UI čuvar 2 SPREMNO DA. Živi testovi s ključem (G-2, KS-3) ok.
- **Leon igrao** obje grane (pregled na 5182/5183, zasebne baze) i našao: najbolji odgovor uvijek prvi (55/55), kvarovi se vraćaju
  (popravak ne diže stanje, 11/15 kvarova bez uvjeta), ista situacija dvaput u mjesecu, lik ne završi u 6 poruka, izgled lika
  ne prati rod/podrijetlo, previše situacija u mjesecu, nema plaća po radniku, „Spremi" po odjelu. Sve odluke:
  [ADR-042](DECISIONS/ADR-042-leonove-odluke-iz-pregleda-14-10.md) t. 1–15 (među njima mjesečno 4/5/6, profesor bira broj, situacije ~20 % napretka).
- **Banka ljudi:** nacrt [BANKA_LJUDI](../architecture/BANKA_LJUDI.md) (~3000 radnika + 1500 gostiju u tri koraka), Leonovi odgovori u ADR-042 t. 8.
- **L1-10** obnova (s KS-3) spojena u sloj1 (24b4e2a): bez sukoba, 703/0, balansi prolaze. **L1-11** situacije: sukobi riješeni
  na grani `model-v2/sloj1-l111` (b1bf446, 879/0), ali brana „udio situacija" pada (B Brižni 10,2 % tjedno) jer obnova diže napredak
  bez situacija — ne ulazi u sloj1 dok se ne kalibrira.
- **Nova datoteka** [BUGS.md](BUGS.md): pogreške i pouke (B1–B10). Leonova zabrinutost „gradimo ciglenu nadogradnju na bungalov" →
  sljedeća sesija je **revizija temelja** (samo čitanje).
- **Gdje se vidi:** sve lokalno; ništa nije pushano (sloj1 ahead 24 + docs).

## 2026-10-14 — G-1, SB-1, G-2 i RB-1 na grani situacija, redak „Obnova" popravljen, Faza 0 zatvorena

- **K0-1 dokazan:** CI za `b495c8d` zelen (rust + web) na `ubuntu-24.04`; plan Faze 0 ide u
  [arhivu](../archive/2026-10-13-temelj-rada.md) (K0-4/5/6 nakon spajanja grana, K0-8 Leonova sjednica ostaju u ROADMAP-u).
- **Grana situacija** (`../hotelixsim-s1-sit`, lokalno 252fe8b), svaka cigla čuvar testova → graditelj → pregled:
  G-1 situacije iz predložaka u igri (707d524; pregled našao da kolektivna situacija može probiti granicu grupa i broj
  situacija → popravak 0bbec33: jedna kolektivna za cijelu igru, Approved); SB-1 osnovica ostalih odjela bruto (ca31005,
  ADR-040 t. 2); G-2 AI razgovor i presuda za situacije s likom (c2ed74e; pregled Needs fixes — razgovor bez AI-ja ostane
  zaključan, prazna rubrika presude; popravak 1 započet, nedovršen); RB-1 (252fe8b). Zadnje: 736 passed / 0 failed,
  balansi bajt-isti osim namjerne promjene RB-1.
- **Tjedan ≈ mjesec** (ADR-040 t. 3): dijagnoza s pokusima našla uzrok — mjesečni Brižni nije otpuštao prije pada kao tjedni
  (ZP-6). Ocjena po cijelom zadnjem mjesecu isprobana i odbačena (razlika 2,96 → 4,50). Leon anketom:
  [ADR-041](DECISIONS/ADR-041-mjesecni-brizni-otpusta-prije-pada.md) → B Brižni 2,96 → 0,18. Leon usput: roboti su samo za demo.
- **Grana obnove** (`../hotelixsim-s1-obnova`, f76905c): redak „Obnova" na Kraju sezone čita v2 obnove (OKS-1), isto za
  robote; UI čuvar 2: SPREMNO DA (360–1280 px, 0 prekoračenja).
- **Leonove ideje 13.10.** (voditelj odjela koji upravlja i odgovara za odjel; graf tržišta: zainteresirani, smješteni,
  odbijeni gosti) → nacrt za raspravu [VODITELJ_I_TRZISTE](../architecture/VODITELJ_I_TRZISTE.md), 5 pitanja Leonu.
- **Gdje se vidi:** sve lokalno (grane situacija i obnove, dokumentacija u `model-v2/sloj1`); ništa nije pushano.

## 2026-10-13 (večer) — raspored do pilota, novi način rada, Faza 0 TEMELJ RADA, završetak Sloja 1 u tijeku

- **Analiza prije prijedloga** (Leon: *„nisi analiziro projekt"*): tri paralelna izvidnika našla su ono što predaja nije
  rekla (O-5 i KS-3 pišu istu uputu mentora; obnova i situacije dijele 14 datoteka jezgre; sloj1 je već bio pushan).
- **Raspored do pilota** ([ROADMAP](../plan/ROADMAP.md)): dva kolosijeka po datotekama (IGRA, PLATFORMA) + kvaliteta,
  prekretnice P1 (listopad) – P4 (ožujak, ljetni semestar), do P3 ~525 h [PROC], linije reza. Odluke:
  [ADR-037](DECISIONS/ADR-037-razred-studenti-jedni-protiv-drugih-obracun.md) (razred = jedna igra, obračun po hotelu,
  lista kolega, vidljivi radnici i preotimanje, sat i semestar, računi studenata pa AAI, engleski kasnije),
  [ADR-038](DECISIONS/ADR-038-nacin-rada-dva-kolosijeka-faze-i.md) (faze i cigle; čuvar testova → graditelj →
  provjeravatelj → čuvar dokumentacije; staje se na kraju faze), [ADR-040](DECISIONS/ADR-040-leonove-odluke-iz-pregleda-situacija-s.md).
  Tvrda granica `.md` s 1000 na 1200 (Leon).
- **Faza 0 TEMELJ RADA** ([plan](../archive/2026-10-13-temelj-rada.md)): CI pribijen na ubuntu-24.04 i akcije v5 (K0-1);
  područja planova igra/platforma/kvaliteta (K0-2); nova brana `check:kod` 600/1000 s popisom duga (K0-3,
  [ADR-039](DECISIONS/ADR-039-brana-velicine-koda-upozorenje-600-granica.md) 🟨); arhitektura platforme u
  [PLATFORMA.md](../architecture/PLATFORMA.md) (K0-7). Svaka cigla: testovi prije koda, provjeravatelj; završni pregled
  (najjači model) našao 5 nalaza (brana koja šuti, lažna crvena…) — svi popravljeni, ponovni pregled čist; `test:docs` 102/0.
- **Završetak Sloja 1:** S-4 odobren (686/0, probe-mutacija 14/14); KS-3 spojen u granu obnove; O-5 gotov i odobren
  (703/0); G-1 kod gotov ali necommitan (jedan kriv test); UI čuvar obnove: SPREMNO NE (redak „Obnova" na Kraju sezone).
- **Gdje se vidi:** GitHub `model-v2/sloj1` (pushano uz Leonov OK, f5871aa); grane situacija i obnove lokalno.

## 2026-10-13 — tvrda granica 1000 redaka, raspored dokumentacije spojen, pretinac prenesen

- Leon: *„nijedan dokument (.md) u repou nema više od 1000 redaka … i brana pazi da se to ne vrati"*. Prije: 6 datoteka
  preko 1000 na sloj1 (najveća 4733), 2 na grani dokumentacije (plan v2-sloj1 4104, ekrani task-04 1037). **Poslije: 0**;
  najveći `.md` je MODEL_V2 s 843 retka.
- **Brana:** `check:docs` pravilo 12 — svaki `.md` u repou ≤ 1000 redaka, bez prijelaznih iznimaka (izuzeci samo
  izrijekom, danas nijedan); arhiva 1100 → 1000. Dokaz: dokument produljen na 1001 redak → EXIT=1, vraćen → EXIT=0; isto
  za `prompts/` izvan `docs/`. `test:docs` 93/93.
- **Razdvojeno po smislu:** plan v2-sloj1 → pregled + 22 taska (alat, spoj = izvornik bajt po bajt); ekrani Task 4 →
  4 (Step 1–7) i 4b (Step 8–15). Jednokratna provjera sidara (`#…`): 0 slomljenih (prije spajanja 148, sve
  `DECISIONS.md#adr-…`).
- **Spajanje (Leonov OK):** `dokumentacija/raspored` → `main` (f38755b); `main` → `sloj1` (66b2ef4) u zasebnom stablu —
  jedini sukob u planu Sloja 1: tekst sa sloj1, plan ponovno razdvojen. Leonov demo (8080/5173) nije diran.
- **Pretinac** `za-dokumentaciju.md` prenesen za spojeni kod: ADR-029, 031–036 (🟨 osim 034 ✅), dopune ADR-002/023/024,
  BACKLOG, CHANGELOG, MODEL_V2, UI_PRAVILA, ARHITEKTURA, nova [IDEJE.md](../product/IDEJE.md). Ostaju Obnova, Situacije, KS-3.
- **Sokratis** `docs`: 0/100 — mjeri staro pravilo indeksa (BACKLOG [alati]); `check:docs` zelen.
- **Gdje se vidi:** lokalno (`main` f38755b, `sloj1`); **ništa nije pushano**. Kod > 1000 redaka samo zapisan (BACKLOG).

## 2026-10-10 (večer) — raspored dokumentacije (paralelna sesija)

- Leon: *„md datoteke imaju previše informacija … preko 3000 linija je JAKO JAKO loše"* → preuređenje na grani
  `dokumentacija/raspored` (radno stablo uz sesiju koda, njezin plan, MODEL_V2 i predaja nedirnuti). Rezultat: nijedan
  dokument nije preko granice osim dvije prijelazne iznimke (plan i MODEL_V2 sloja 1).
- **Alati i brana (T1–T3, T7):** mjere i granice u `scripts/docs-granice.mjs`, zajednički alati u `scripts/docs-alati.mjs`,
  `docs:plan` (razdvaja plan na pregled + taskove), `docs:adr` (generira indeks ADR-ova); `check:docs` sada pazi
  granice, podindekse, područje plana i svježinu indeksa ADR-ova. Svako razdvajanje ima provjeru „spoji natrag = izvornik".
- **Razdvojeno (T4–T6, T8):** ARHITEKTURA i DEMO po dijelovima (§ brojevi u naslovima ostaju), pet arhiviranih planova
  na pregled + taskove (sadržaj doslovan), 28 ADR-ova u zasebne datoteke s generiranim indeksom
  [DECISIONS.md](DECISIONS.md) i poveznicama ravno na odluke. T9: pravila i tok rada zapisani, pregled popravljen.
- **Brojke prije → poslije** (`node scripts/check-docs.mjs --popis`): najveća datoteka 4733 → 4104 redaka, a bez dviju
  iznimki 4733 → 1037 (arhivski task; granica arhive 1100), najveći živi dokument izvan iznimki 772 → 263 (plan ove
  sesije; granica 300); `.md` pod `docs/` 36 → 136; preko granice 10 → 2 (samo iznimke).
- **Provjere:** `check:docs`, `test:docs` (90 testova), `adr.mjs indeks --provjeri` i `lint` — sve izlaz 0.
- **Gdje se vidi:** samo lokalno, grana `dokumentacija/raspored` — **nije pushana ni spojena**.
- **Odluka:** [ADR-030](DECISIONS/ADR-030-raspored-dokumentacije-granice-velicine-indeks-mapa.md) 🟨 čeka Leonov pregled.
- **Slijedi:** završni pregled grane; uz Leonov OK push grane, CI, spajanje u `main` (novi OK za push). Zatim sesija koda
  spaja `main` u svoju granu i prenosi pretinac. Taskovi 11–13 (MODEL_V2 po dijelovima, zatvaranje) tek kad je sloj 1 u `main`;
  plan je do tada ⏸️ pauziran.
- AI trošak ove sesije: 0 $.

## 2026-10-10 (popodne) — demo na GitHubu, računalo, sloj 0, dizajn puta B, UI čuvar

- Završni pregled grane ekrana (opus) → val popravaka (presuda bez internih brojeva `presuda.v3` + brana u kodu
  [ADR-025](DECISIONS/ADR-025-presuda-ne-spominje-redne-brojeve-ponudenih.md) 🟨, utrka „Moj hotel" nakon zatvaranja, jedinice, tekstovi) → ponovni pregled → **spojeno i
  pushano**; CI je jednom pao na nestabilnom testu („▲ +6" dvaput) — reproducirano i popravljeno, CI zelen.
- **Računalo** ([ADR-027](DECISIONS/ADR-027-raspored-za-racunalo-prije-demoa.md) 🟨): bočni izbornik, dva stupca; **sloj 0 puta B**: 15 pokazatelja (ADR,
  RevPAR, GOPPAR, točka pokrića…), ruta `/kpi`, ekran `/pokazatelji`; oba pregledana, u `main` i na GitHubu. Kalibracija:
  sloj 0 procijenjen 6–10 h, agent ga je napravio za 28 min (+14 min popravci).
- **Put B** (Leon: „kao BitLife, opsežno do bola"): katalog [PROSIRENJA.md](../product/PROSIRENJA.md), arhitektura
  [MODEL_V2.md](../architecture/MODEL_V2.md), [ADR-026](DECISIONS/ADR-026-put-b-hotel-od-brojeva-postaje.md)/[ADR-028](DECISIONS/ADR-028-leonove-odluke-za-put-b-ljudi.md) (Leonove odluke:
  ljudi kao u Simsima, burza rada, agencija, tjedni ili mjesečni krug, AI razgovori; pravilo: nacionalnost određuje samo
  činjenice 🟨), istraživanje [JEZICI_GOSTIJU.md](../research/JEZICI_GOSTIJU.md), plan sloja 1 (22 taska) s preflightom (24 nalaza,
  presude u ledgeru). Izvedba sloja 1 započeta: Task 1 i 2 paralelno.
- **UI čuvar** (Leon: „pripazi na frontend i cijeli UI"): pregled svih ekrana 360–1920 px (2 blokera, 12 važnih, 22 sitna),
  pravila `docs/product/UI_PRAVILA.md` (na grani); val popravaka na grani `gradnja/ui-popravci`.
- Leon: dokumentacija je „srce operacija", planovi od 4000+ redaka su loši → plan preuređenja za paralelnu sesiju
  (`docs/plan/2026-10-10-dokumentacija.md`, PAUZIRAN dok ta sesija ne krene).
- AI trošak ove sesije: 0 $ (sve provjere bez ključa).

## 2026-10-10 — Ekrani igrača (DEMO korak 5), mentor govori ljudski, modeli

- Grana `gradnja/ekrani` (lokalno, **nije pushana**), plan [2026-10-10-ekrani.md](../archive/2026-10-10-ekrani.md), ledger u
  `.superpowers/sdd/2026-10-10-ekrani/progress.md` (Rulings 12–23). Svaki task pregledan: 9 (Mjesec završen, + Leonov zahtjev:
  indeksi s kraja mjeseca, Moj hotel se osvježi nakon obračuna), 10 (tijek, množina), 12 (lica), 13a (dorada izgleda, cijeli
  brojevi), 13c (README, BACKLOG).
- Leon: mentor je pisao sirove brojke („4876 na 4269“, „200 bp“, „centi“) → Task 14: činjenice u ljudskim jedinicama, odluke iz
  plana, izabrani odgovor, četiri dijela, čuvar u testovima ([ADR-023](DECISIONS/ADR-023-mentor-govori-ljudski-pripremljene-cinjenice-cetiri.md) 🟨). Task 16: ruta objašnjenja
  mjeseca i kartica „Zašto se indeksi pomaknuli“ iz istih činjenica; Task 17: detalj svakog pokazatelja (`/pokazatelj/<kod>`);
  Task 19: Zaposlenici objašnjeni prema formuli, prvi mjesec „na početku igre“.
- Nađeno u pregledniku: lažno „Bez odgovora: 1“ (dva pravila brojanja) → Task 15; dvostruki React ključevi u tijeku → Task 16.
- Leon: presuda na Sonnetu, lik na Haikuu, mentor na Sonnetu ([ADR-024](DECISIONS/ADR-024-modeli-presuda-na-sonnetu-5-5.md) ✅) → Task 18.
- Provjereno: Rust 310 / web 130 testova, balans 1000 sjemena prolazi, preglednik 390 i 360 px bez vodoravnog pomicanja i bez
  grešaka u konzoli. Jedan prolaz s pravim AI-jem (lipanj → srpanj): lik, presuda i mentor rade; mentor piše ljudski.
- Trošak AI-ja ove sesije (dnevnik događaja, od 05:51): Haiku 9 poziva 7.654/748 tokena ≈ 0,001 $; Sonnet 5 poziva
  16.886/2.451 ≈ 0,058 $; **ukupno ≈ 0,06 $** (dio poziva poslije 05:56 vjerojatno je Leonova igra). Po igri sada ≈ 0,05 $:
  presuda ≈ 0,008 $, mentor ≈ 0,015 $ po mjesecu (veći ulaz — činjenice), lik ≈ 0,0001 $ po odgovoru.
- Otvoreno: završni pregled cijele grane (opus) + val popravaka (među njima: presuda piše „ponuđenom odgovoru 0“); Leonova vizija
  (profesor kreira igru, kod kao u Kahootu, novi likovi, više situacija i mjeseci) u [BACKLOG.md](BACKLOG.md).

## 2026-10-09/10 — AI sloj (DEMO korak 4) i Leonov dizajn

- Leon kroz anketu: slobodan tekst u S1 i S2, brojke iz najbližeg ponuđenog odgovora, do 6 poruka liku, Anthropic ([ADR-018](DECISIONS/ADR-018-leonove-proizvodne-odluke-za-ai-sloj.md) ✅); pretpostavke [ADR-019](DECISIONS/ADR-019-ai-sloj-arhitektonske-pretpostavke-koraka-4.md) 🟨. Plan s 11 taskova ([archive/](../archive/2026-10-09-ai-sloj.md), ledger uz njega).
- Izgrađeno kroz subagente (sonnet izvođači i pregledi, opus završni pregled): otisak pravila nad kanonskim `Value` (stare igre i dalje rade — zamrznut hash koraka 3 provjeren na lokalnoj bazi), rubrike i likovi, razgovor, slobodan odgovor, presuda (kvalitetu računa kod), neutralna i zakašnjela presuda, mentor, odgoda u redu poslova, crate `ai` (Claude), rute, mjerilo.
- Završni pregled: „spremno uz popravke" — 4 važna (injekcija preko replika lika → `presuda.v2`, timeout klijenta, izlazni kod mjerila, e2e komentar) i 4 sitna; jedan val popravaka, ponovni pregled čist.
- Provjereno: 262 testa / 0 palih, fmt, clippy, brana jezgre, balans, check:docs, check:api, lint, typecheck — na grani i na spojenom `main`. Živi test s pravim ključem prošao (presuda ~8 s). Spojeno u `main` (`156e6df`), pushano uz Leonov OK, CI zelen (run 38003510045).
- Leon je zalijepio ključ u `.env.example` (datoteka u gitu) — premješten u `.env` prije ikakvog commita; ništa nije otišlo na GitHub.
- Leon: ekrani u sljedećoj sesiji, Tailwind, njegov logo i koncept ekrana ([ADR-020](DECISIONS/ADR-020-sucelje-tailwind-i-leonov-koncept-dizajna.md), [DIZAJN.md](../product/DIZAJN.md)); nađeno 6 neslaganja koncepta i igre.
- Otvoreno: Leonove ocjene 40 odgovora za mjerilo; likovi i ADR-019 na pregled; odgođeni nalazi u BACKLOG-u `[ai-sloj]`.

## 2026-10-05 — Dnevnik, runde i red poslova (DEMO korak 3)

- Plan s 12 taskova ([archive/](../archive/2026-10-05-dnevnik-runde.md), ledger uz njega) i pretpostavke [ADR-015](DECISIONS/ADR-015-dnevnik-runde-i-red-poslova-arhitektonske.md) 🟨. Leon je kroz ankete odlučio gosta, zatvaranje s neodgovorenim, tko zatvara i profile ([ADR-016](DECISIONS/ADR-016-leonove-proizvodne-odluke-za-korak-3.md) ✅); tehničke presude voditelja su u [ADR-017](DECISIONS/ADR-017-odluke-voditelja-gradnje-u-koraku-3.md) 🟨.
- Izgrađeno kroz subagente na grani `gradnja/dnevnik-runde` (izvođač + pregled po tasku; popravni krugovi na taskovima 6, 9, 10, 12). Završni pregled na opusu: „spremno uz popravke" → jedan val popravaka (radnik preživi paniku, pravilo otiska snimka, BACKLOG) → ponovni pregled čist.
- Što radi: cijela igra preko API-ja na pravom Postgresu — gost bez lozinke, igrač + 3 robota, lipanj i srpanj, „Zatvori mjesec" (202), radnik u serveru, ljestvica i feed hotela. Nepromjenjiv dnevnik (okidač u bazi), red poslova s `SKIP LOCKED`, novi crate `game`.
- Provjereno: sve CI provjere lokalno zelene; E2E na Postgresu provjerava stvarne ishode; ručni prolaz curlom na pravom serveru. Spojeno u `main` (`894d023`) i pushano uz Leonov OK; CI run 37250763627: rust i web uspjeh. Ekrana još nema.
- Otvoreno: prije prve promjene content DTO-a popraviti otisak snimka (BACKLOG `[korak 4]`); ishodi neodgovorenih S1–S4 su nacrt; ADR-015 i ADR-017 čekaju Leonov pregled.


## 2026-10-04 — Jezgra igre i alat za balans (DEMO korak 2), rok demoa ~14.10.

- Leon: rok demoa ~14.10., prvo profesorovih 15 minuta ([ADR-012](DECISIONS/ADR-012-ciljani-rok-demoa-14-10-prvo.md)); pitanja za profesore pregledana i spremna za slanje (15 + 13).
- Plan jezgre s 12 taskova + 2 nakon završnog pregleda (sad u [archive/](../archive/2026-10-04-jezgra.md), ledger uz njega). Pretpostavke [ADR-013](DECISIONS/ADR-013-pretpostavke-jezgre-centrirani-indeksi-sezonci-po.md) 🟨; zlatne vrijednosti iz neovisnog modela `scripts/oracle-jezgra.mjs`.
- Izgrađeno kroz subagente (sonnet izvođači i pregledi, opus kalibracija i završni pregled), svaki task pregledan: brane determinizma s mutacijskim probama u CI-ju, `engine::calc`, obračun mjeseca, mehanike S1–S4, centrirani indeksi i ocjena, crate `content`, roboti, crate `balance`, kalibracija.
- Završni pregled našao: lekcija S3 radi naopako, obnova besplatna. Leon (anketa): kolovoz rasprodan, likvidnost bez gornje granice ([ADR-014](DECISIONS/ADR-014-kolovoz-rasprodan-lekcija-s3-i-blagajna.md)). Sad S3: kolovoz −9 953 €, lipanj +19 184 €.
- Provjereno: 106 testova, `balance --seeds 1000 --check` „Sve provjere prolaze", clippy, fmt, ovisnosti jezgre, mutacijske probe (0 proboja), check:docs, lint, check:api, typecheck, web test — sve EXIT=0, lokalno i na spojenom mainu. Spojeno u `main` i pushano (Leonov OK).
- Otvoreno za Leona: [BACKLOG](BACKLOG.md) „Otvoreno za Leona" (sezonci 50, B fiksni, A opterećenje, obnova za A/C, S3/S4 odgovori, ADR-013).

## 2026-10-04 — Preslagivanje dokumentacije po Sokratovom sustavu (ADR-011)

- Povod: Leon nije zadovoljan slaganjem dokumentacije („dokumentacije ne idu dobro kroz sesiju") pa dokumenti idu po ulozi u mape `product`, `architecture`, `plan`, `workflow`, `records`, `research`, `archive` ([ADR-011](DECISIONS/ADR-011-dokumentacija-po-sokratovom-sustavu-mape-po.md)). Premješteno s `git mv` (povijest ostaje), 33 relativne poveznice prepisane skriptom.
- Novo: jedini indeks [docs/README.md](../README.md); zapisi `PROGRESS.md`, [CHANGELOG.md](CHANGELOG.md), [DECISIONS.md](DECISIONS.md) (ADR-001…011), [BACKLOG.md](BACKLOG.md) (46 odgođenih nalaza + pitanja za Leona + rupe u specu); [AGENTI.md](../workflow/AGENTI.md); komanda `/kraj`; `/newsession` usklađen; `.sokratis/profile.json`.
- CLAUDE.md sveden na pravila, trenutno stanje i pokazivače (3 993 znaka, budžet 12 000); `SLJEDECA_SESIJA.md` skraćen na cilj, prvi korak i pitanja; ROADMAP više ne nosi pitanja (žive u BACKLOG-u); arhiva dobila pečate `✅ ISPUNJEN`.
- Brana `scripts/check-docs.mjs` (mrtve poveznice, indeks, jedan aktivni plan, datumi u `product/`, budžet CLAUDE.md, ćirilica, arhiva proglašena aktivnom) + 11 testova (`node:test`), svaka provjera dokazana mutacijom; korak u CI-ju (`web`). Usput nađeno: Sokratov regex `AKTIVN(O|I|A)` nikad ne hvata „AKTIVAN" — ovdje ispravljeno.
- Provjereno lokalno: `check:docs` EXIT=0 (26 .md, 158 poveznica), `test:docs` 11/11, `lint` EXIT=0, ručna mutacija mrtve poveznice → EXIT=1. CI na GitHubu nije viđen (nije pushano).
- Obrisana spojena grana `gradnja/kostur`.

## 2026-10-04 — Predaja: VS Code panel i plan preslagivanja dokumentacije

- Dogovoreno da se radi u Claude Code dodatku za VS Code (panel, ne terminal) — [ADR-010](DECISIONS/ADR-010-rad-u-claude-code-dodatku-za.md).
- Leon zatražio preslagivanje dokumentacije po Sokratovom sustavu; plan i dijagnoza (ista činjenica na 3–6 mjesta, nema dnevnika ni numeriranih odluka) upisani u [SLJEDECA_SESIJA](../SLJEDECA_SESIJA.md) §2b.
- Provjereno: popis 20 `.md` datoteka, samo 2 mrtve poveznice.
- Otvoreno: sam posao preslagivanja; zatim plan jezgre i alata za balans.

## 2026-10-04 — Push i prvi CI zelen

- Uz Leonov izričit OK pushano na GitHub (`leonkreso784-bit/hotelixsim`, privatni) — [ADR-005](DECISIONS/ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md).
- Provjereno: CI run 37167540215 zelen, `rust` 4m51s, `web` 1m44s; gledano preko `gh` (nije na PATH-u Bash alata, putanja u SLJEDECA_SESIJA §5).
- Upozorenje: akcije `checkout@v4` i `setup-node@v4` na Node 20 zastarjele; `ubuntu-latest` prelazi na Ubuntu 26 od 19.10.
- Revizija stanja (4 agenta) pa 7 paralelnih agenata: nađene rupe u specu [DEMO](../product/DEMO.md) (profil C ne kreće s 50, kalibracija bez redovnog održavanja, nema pravila zaokruživanja); upisane kao ulaz za plan jezgre. Ankete samo za proizvodne odluke — [ADR-006](DECISIONS/ADR-006-arhitektura-kroz-dokument-i-raspravu-ne.md).
- Dodana komanda `/newsession`.

## 2026-10-04 — Gradnja kostura od kraja do kraja (8 taskova)

- Po planu [kostur](../archive/2026-10-04-kostur.md): Rust workspace (`engine`, `sim-hotel`, `app`, `store`, `api`, `server`) → `openapi.json` → generirani TS klijent → i18n → jedan React ekran.
- Svaki task s pregledom, završni pregled cijele grane i jedan krug završnih popravaka (LF za sve datoteke, `SQLX_OFFLINE`, dozvole CI-a). Odluke koje je kostur donio umjesto Leona: [ADR-008](DECISIONS/ADR-008-odluke-iz-gradnje-kostura-koje-je.md); lokalna baza na 5433: [ADR-007](DECISIONS/ADR-007-lokalna-baza-zaseban-postgresql-16-klaster.md); brane determinizma kao Task 1 sljedećeg plana: [ADR-009](DECISIONS/ADR-009-ojacavanje-brana-determinizma-je-task-1.md).
- Provjereno lokalno: testovi Rust 28, sučelje 10; zdravlje 200, profil D → 404, kriva lozinka baze → 503 bez pada, Vite proxy.
- Odgođeni nalazi pregleda (47 stavki) zapisani u [BACKLOG](BACKLOG.md).
- Otvoreno: ojačati brane determinizma (allowlist ovisnosti, mutacijski probni testovi).

## 2026-10-04 — Ime, arhitektura v2 i dizajn demoa

- Ime odlučeno: HotelixSim — [ADR-002](DECISIONS/ADR-002-ime-hotelixsim-nikad-skraceno-na-hotelix.md). Pokušaj „Xenokrat" vraćen (revert) prije toga, 03.10. navečer.
- Arhitektura nacrt v1 (Rust + TypeScript) i rasprava, zatim v2 s ugrađenim krugom 1 rasprave — [ADR-003](DECISIONS/ADR-003-arhitektura-rust-jezgra-backend-typescript-sucelje.md); sve lokalno, Railway tek za pilot — [ADR-004](DECISIONS/ADR-004-sve-se-gradi-i-pokazuje-lokalno.md). Vidi [ARHITEKTURA](../architecture/ARHITEKTURA.md).
- Istraživanja: hotelski pokazatelji, postojeće simulacije, Rust alati ([research](../research/HOTELSKI_POKAZATELJI.md)).
- [DEMO](../product/DEMO.md): opseg demoa i dizajn igre; kasnije kategorija hotela (3*/4*/5*), demo svi 4*. Pitanja za profesore napisana ([PITANJA_ZA_PROFESORE](../product/PITANJA_ZA_PROFESORE.md)).
- Arhitektura se vodi kroz dokument, ne ankete — [ADR-006](DECISIONS/ADR-006-arhitektura-kroz-dokument-i-raspravu-ne.md); od 04.10. radi bez pitanja — [ADR-005](DECISIONS/ADR-005-radi-bez-pitanja-pretpostavke-u-dokument.md).
- Otvoreno: Leonova provjera domene (pitanja 2–6 u DEMO-u).

## 2026-10-03 — Početak: HotelixSim kao zaseban projekt

- Sesija planiranja u Sokratu: HotelixSim postaje glavni projekt, zaseban repo bez veze s kodom Sokrat Studyja; Sokrat ide u održavanje — [ADR-001](DECISIONS/ADR-001-hotelixsim-je-zaseban-projekt-odvojen-od.md).
- Prvi commit repoa: [VIZIJA](../product/VIZIJA.md) (dobiveni hotel, fer start, pet osi situacija, AI u četiri uloge, relativna ocjena), [ROADMAP](../plan/ROADMAP.md) (tada PLAN: prototip, prezentacija, pilot, vizija), izvorna ideja u [archive](../archive/HOTEL_SIM_2026-08-22.md), CLAUDE.md.
- Pokušano ime „Xenokrat" (xenia + -krat), isti dan vraćeno.
- Otvoreno: GitHub repo još ne postoji (nema `gh`); backend simulacije; vrijeme „1–7 dana" u simulaciji.
