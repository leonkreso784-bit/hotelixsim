### Task B4: Prepis oznaka odgovora (55 predložaka + sidra S1–S4) i uvjeta kvarova — podijeljeno B4a / B4b / B4c

**Veličina:** B4a M, ~6 h · B4b M, ~4 h · B4c M, ~4 h (zbroj ~14 h; cijela B4 ne stane u jedan dan, vidi „Podjela")
· **Ovisi o:** A2 (načela, popis duga), **B1b** (`condition_of` za uvjete kvarova u B4c; voditelj 15.10.: B4 ide nakon B1b) · B4b i B4c ovise o B4a (alat i čuvarski testovi) · **Nije paralelno
s:** B1, B2, B3 (red faze B: B2a → B2b → B3a → B3b → B1a → B1b → **B4a → B4b → B4c** → L1-13) · **Model:** B4a sonnet (kod + tekst), B4b/B4c sonnet (tekst) · **Prevodi Rust:** B4a da (testovi), B4b/B4c samo
testovi sadržaja (`cargo test -p content`).

**Cilj:** Student ne smije prepoznati najbolji odgovor po duljini, redoslijedu rečenice ni broju koraka. Zato se prepisuju
**oznake** (`answers[].label`) svih 55 predložaka (213 oznaka) i sidara S1–S4 (16 oznaka) tako da su unutar 1,5× (P3) i da dobri
sastojci ne stoje samo u najboljem odgovoru. Kvarovi P-KLIMA i P-PRAONICA (B4a) te P-BRAVE, P-CURENJE-VODE i P-DIZALO (B4c, preko B1b `condition_of`) dobivaju uvjet lošeg stanja Smještaja (P6, ADR-042 t. 7); P-INSPEKCIJA postaje vanjski događaj (`external_cause`, Leon 15.10.).
Dug A2 za duljine pada 55 → 0, a za uvjete kvarova na 0 (`SilenceNotBetter` 16 nije u B4, ide u L1-13). Kad bude uveden prikaz s izmiješanim odgovorima (L1-13b, ADR-042 t. 10), miješanje je sigurno.

**Zašto (nalaz):** [REVIZIJA_TEMELJA](../../architecture/REVIZIJA_TEMELJA.md) §6 B4 · [faza-a-review.md]
(`.superpowers/sdd/2026-10-14-popravak-temelja/faza-a-review.md`) tablica trijaže: A2 „dug 55/55 duljina", m1, m4,
`DeptPlaceholder` u `character.*`, Review Focus 5 (sidra), m5 (A6 bira `answer: 0`). Brojke danas (provjereno čitanjem JSON-a):
najbolji odgovor je **najdulji u 54 od 55** predložaka i ima **najviše zareza i veznika „i” u 55 od 55**; omjer duljina: min 2,52 (P-INSPEKCIJA), medijan
4,28, max 7,44 (P-SLOBODNI-DANI); sidra S1 3,1× · S2 5,0× · S3 10,3× („Odbiti" = 6 znakova) · S4 3,4×. U 53 od 55 predložaka
kvaliteta ide silazno (iznimke P-VJENCANJE, P-KRIZA-OSOBLJA). Kvaliteta odgovora = broj kriterija (`TemplateAnswer::quality`,
`templates.rs:295`), a svaki predložak ima točno jedan odgovor kvalitete 5.

## Što B4 mora jamčiti da je miješanje kasnije sigurno (za L1-13b)

1. **Duljina ne odaje:** svi odgovori jednog predloška i sidra unutar 1,5× (kod: `LABEL_RATIO_PCT`, `templates_principles.rs:17`),
   a graditelj cilja ≤ 1,40× (rezerva jer se broji sirovi tekst s `{sobe}`, `{ponuda_pct}`…; prikazani je kraći ili dulji za nekoliko znakova).
2. **Građa ne odaje:** najbolji nije pojedinačno najdulji ni s pojedinačno najviše zareza u više od 50 % ( 🟨 prag) očišćenih
   predložaka (test T4). Ne smije biti sastojka koji ima samo najbolji (npr. „sutra nazvati", „obrazložiti", „provjeriti kasnije").
3. **Redoslijed i kvaliteta se ne diraju:** B4 ne mijenja ni indeks odgovora, ni `criteria`, ni `mechanics`, ni `unanswered` (događaji i roboti
   pamte izvorni broj). Danas je najbolji u svim predlošcima i sidrima **na indeksu 0**; to je istina do L1-13b.
4. **Točno jedan odgovor kvalitete 5** u svakom predlošku i sidru (danas vrijedi; test T5) — jedino tako „izvorni broj najboljeg" postoji.
5. **Napomena za A6 scenarij (a):** `crates/api/tests/student_sees.rs:121` šalje `"answer": 0` jer je to najbolji danas. Kad L1-13b izmiješa
   prikaz, indeks 0 postaje nasumičan i test bi tiho mjerio drugo. L1-13b mora scenariju (a) dati **izvorni broj najboljeg odgovora**: API ne
   smije otkriti kvalitetu, pa test taj broj čita iz sadržaja (`Snapshot::read_v2(&repo_dir())`, odgovor sa 5 kriterija za šifru
   predloška bez nastavka kruga, npr. `P-KLIMA.t03` → `P-KLIMA`) i šalje ga kao `answer`. B4 to **ne** mijenja (zabranjeno), samo to ovdje piše.

## Podjela (zašto i kako)

Jedna cigla = ~230 oznaka, pristojan tekst se ne piše brže od ~1 oznake u 3–4 minute, a Leon mora pregledati tekstove. Grana ≤ 1 dan i
Leonov pregled po porciji (≤ ~85 oznaka) traže tri cigle. Dug duljina pada stepenasto (ceilings u `v2_principles.rs` test 2 ide dolje).

| cigla | sadržaj | predlošci | oznaka | dug duljina poslije |
|---|---|---|---|---|
| **B4a** | alat + testovi + sidra S1–S4 + uvjeti P-KLIMA i P-PRAONICA + prva porcija: odjeli `rooms` i `*guest_facing` | 15 | 57 + 16 sidra | 55 → 40 |
| **B4b** | `hotel`, `*staffed`, `admin`, `sales`, `maintenance`, `parking` | 22 | 85 | 40 → 18 |
| **B4c** | `restaurant`, `pool_bar`, `spa`, `rooftop_bar`, `lobby_bar` + uvjeti kvarova P-BRAVE, P-CURENJE-VODE, P-DIZALO (njihove oznake su u B4b) + P-INSPEKCIJA `external_cause` | 18 | 71 | 18 → 0 |

Šifre porcija (jedino mjesto gdje su popisane):
- **B4a:** P-KLIMA, P-BOJLER, P-KRIZA-OSOBLJA, P-RANI-DOLAZAK, P-NESTALA-STVAR, P-LJUBIMAC, P-BUCNA-SOBA, P-OVERBOOKING, P-RED-NA-RECEPCIJI,
  P-PRAONICA, P-UCJENA-RECENZIJOM, P-DOTRAJALO, P-POHVALA, P-PRISTUPACNOST, P-NETOCNA-RECENZIJA.
- **B4b:** P-NOVINAR, P-KONGRES, P-VAL-RECENZIJA, P-TOPLINSKI-VAL, P-OLUJA, P-REGATA, P-GOST-POZLILO, P-DIZALO, P-NESTANAK-STRUJE,
  P-DIJETE-IZGUBLJENO, P-POVISICA, P-BOLOVANJE, P-SLOBODNI-DANI, P-OSPORENA-NAPLATA, P-RACUN-GRESKA, P-TERMINAL, P-PREPORUKA, P-LAST-MINUTE,
  P-RADNICI-TVRTKE, P-BRAVE, P-CURENJE-VODE, P-PARKING-STETA.
- **B4c:** P-NOVI-LJUDI, P-INSPEKCIJA, P-ALERGIJA, P-POSEBNA-PREHRANA, P-MANJAK-INVENTURA, P-DOBAVLJAC, P-PIJANI-GOST, P-PROSIDBA,
  P-BAZEN-PUMPA, P-LEZALJKE, P-SUSJEDI-BAZEN, P-SPA-TERMIN, P-SPA-TECAJ, P-WELLNESS-PAKET, P-VJENCANJE, P-ROOFTOP-BUKA, P-LOBBY-SUKOB, P-LOBBY-KAVA.

## Files

**Create (B4a):**
- `crates/content/tests/v2_b4_labels.rs` — testovi T1–T7 (dolje)
- (privremeno, ne ulazi u git) skripta za pregledni dokument u scratchpadu; izlaz `.superpowers/sdd/2026-10-14-popravak-temelja/b4a-pregled-tekstova.md`
  (`b4b-…`, `b4c-…`); `.superpowers/` je u `.gitignore`, dokument se ne commitira nego Leonu preda voditelj.

**Modify:**
- B4a: `content/hotel/situations.json` (samo `answers[].label` u S1–S4); `content/hotel/v2/situation_templates.json` (oznake porcije +
  `when.dept_condition_pts` za P-KLIMA i P-PRAONICA); `crates/content/src/templates_principles_debt.rs` (makni 15 šifara iz `LabelLengths`,
  P-KLIMA i P-PRAONICA iz `BreakdownCondition`); `crates/content/src/templates_principles.rs:112-130` (m4, vidi T6) i
  `crates/content/src/lib.rs:21` (izvezi `LABEL_RATIO_PCT` ako nije izvezen — T1 ga koristi); `crates/balance/src/main.rs:60`
  (`Snapshot::read_v2(dir)` → `Snapshot::read_v2_checked(dir)`, m1; poruke brana ostaju doslovno iste);
  `crates/content/tests/v2_principles.rs:56-60` (stropovi test 2: `LabelLengths` 55 → 40, `BreakdownCondition` 6 → 4; u B4c → 0).
- B4b: `situation_templates.json` (22 predloška, samo oznake); debt `LabelLengths` −22; strop 40 → 18.
- B4c: `situation_templates.json` (18 predložaka, oznake; uz to `condition_of` + `when.dept_condition_pts` za P-BRAVE, P-CURENJE-VODE, P-DIZALO i `external_cause: true` za P-INSPEKCIJA); debt `LabelLengths` −18 (lista postaje prazna: `&[]`), `BreakdownCondition` −4 (3 uvjeta + P-INSPEKCIJA), stropovi → 0.

**Ne dirati:** `criteria`, `mechanics`, `unanswered`, `effect_band`, redoslijed `answers`, `title`, `description`, `rubric`, `character.*`,
`params`, `chance`, `repeat`, `external_cause` (pet već postavljeno u A2; jedini novi je P-INSPEKCIJA u B4c); `crates/sim-hotel*`, `crates/game`, `crates/engine`; `CORE_VERSION`;
`crates/*/tests/fixtures*/**` i `crates/game/tests/fixtures_v2/**` (zamrznuti snimci nose stare oznake — i treba tako ostati); `prompts/`; web.
Ako oznaka ne može biti poštena bez izmjene opisa/rubrike, graditelj to **prijavi** u izvještaj, ne mijenja.

## Interfaces
**Consumes:** `content::{Snapshot, Principle::*, principle_violations, check_principles_with, KNOWN_VIOLATIONS, fingerprint_value, repo_dir}`;
`TemplateAnswer::quality`. **Produces:** isti javni tipovi; novo samo `pub use templates_principles::LABEL_RATIO_PCT` (ako treba). Format JSON-a se ne mijenja.

## Testovi čuvara (`crates/content/tests/v2_b4_labels.rs`; pomoćne funkcije kopirati iz `v2_principles.rs:7-37`)
Čuvar prvo (korak 1) iz **neizmijenjenih** datoteka izračuna konstantu `SKELETON_SHA` ispod i upiše je u test.

- **T1 `anchors_labels_within_ratio`** — za S1–S4 iz `Snapshot::read_v2(&repo_dir())["situations"]`:
  `max·100 <= min·LABEL_RATIO_PCT` (znakovi: `chars().count()`). **RED** (3,1 / 5,0 / 10,3 / 3,4). Zeleno nakon B4a.
- **T2 `only_labels_and_listed_conditions_change`** (karakterizacijski, zelen od početka, ostaje zelen kroz B4a–c):
  ```rust
  fn skeleton(v: &Value) -> String { // kostur bez oznaka i bez dopuštenih uvjeta
      let mut v = v.clone();
      for a in v["situations"].as_array_mut().unwrap().iter_mut()
          .flat_map(|s| s["answers"].as_array_mut().unwrap().iter_mut()) { a.as_object_mut().unwrap().remove("label"); }
      for t in v["v2"]["situation_templates"]["templates"].as_array_mut().unwrap() {
          for a in t["answers"].as_array_mut().unwrap() { a.as_object_mut().unwrap().remove("label"); }
          if matches!(t["code"].as_str(), Some("P-KLIMA" | "P-PRAONICA")) {
              t["when"].as_object_mut().unwrap().remove("dept_condition_pts"); }
      }
      content::fingerprint_value(&v)
  }
  assert_eq!(skeleton(&v2_value()), SKELETON_SHA, "B4 smije mijenjati samo oznake, uvjete kvarova (5 predložaka) i `external_cause` P-INSPEKCIJE");
  ```
  🟨 ključ `"situations"` u `Snapshot::to_value()` provjeri čuvar. Test se briše na početku L1-13 (kalibracija mijenja brojke).
- **T3 `batch_labels_have_no_debt`** — tri testa `b4a_…`, `b4b_…`, `b4c_…` s konstantnim popisom šifara porcije (gore):
  `repo_codes(LabelLengths)` ne smije sadržavati nijednu šifru porcije. **RED** do kraja pripadne cigle. Uz to test 2 sa spuštenim stropovima.
- **T4 `best_answer_is_not_a_tell`** — za sve predloške koji **nisu** u `KNOWN_VIOLATIONS[LabelLengths]` (očišćeni) i za sidra, ako su očišćena
  (T1 zelen): udio predložaka u kojem je najbolji odgovor strogo najdulji (znakovi) ≤ 50 %, i udio u kojem ima strogo najviše zareza ≤ 50 %.
  Kad nema očišćenih, test prolazi trivijalno. **RED** tek kad čuvar u test unese sidra kao „očišćena" nakon B4a; za porcije pada kad graditelj
  preslika staru građu. 🟨 prag 50 % (slučajno bi bilo ~25–35 %).
- **T5 `exactly_one_best_answer_per_template_and_anchor`** — karakterizacijski: točno jedan odgovor s 5 kriterija (sidra: `quality == 5`).
- **T6 `duplicate_name_in_a_list_is_counted_once`** (m4) — u `v2/names.json` snimku `guest.f` s 20 unosa od kojih jedan duplikat → `NamesPerList`
  za `names.guest.f` s „19 različitih". **RED** (`principle_violations` danas broji `v.len()`). Popravak: brojati `BTreeSet`. Repo imena su
  svi različiti (provjereno: 6 popisa × 20 jedinstvenih), pa repo ostaje zelen.
- **T7 `dept_placeholder_checked_in_character_fields`** — `{odjel}` u `character.brief`/`opening`/`fallback_reply` bez „odjel*" ispred ili zagrade → `DeptPlaceholder`.
  Karakterizacijski (kod `dept_placeholder_misuse` to već radi, test nedostaje).
- **T8 (B4a) `klima_and_praonica_need_bad_rooms_state`** — `when.dept_condition_pts.below` postoji; `BreakdownCondition` ne krši ni jedan; ostala četiri
  (P-INSPEKCIJA, P-DIZALO, P-BRAVE, P-CURENJE-VODE) ostaju u dugu do B4c. **RED** do B4a.
- **T9 (B4c) `breakdown_conditions_via_condition_of`** — P-BRAVE, P-CURENJE-VODE i P-DIZALO imaju `condition_of` i `when.dept_condition_pts.below`, P-INSPEKCIJA ima `external_cause`; `BreakdownCondition` nema nijednu šifru u dugu. **RED** do B4c.

## Koraci
**B4a**
1. **Čuvar:** izračuna `SKELETON_SHA`, piše T1–T8, spušta stropove (test 2). `cargo test -p content --test v2_b4_labels --test v2_principles` → RED samo T1, T3a, T4*, T6, T8, test 2.
2. **Graditelj, kod:** T6 (m4, `BTreeSet`), m1 (`balance/main.rs:60`), po potrebi izvoz `LABEL_RATIO_PCT`.
3. **Graditelj, tekst:** prvo sidra pa porcija (pravila ispod). **Commit 1: samo oznake** (sidra + 15 predložaka). Balans mora biti **bajt-isti** (`cmp`).
4. **Graditelj, uvjeti — commit 2 zasebno:** `when.dept_condition_pts: {"below": 6500}` na P-KLIMA i P-PRAONICA (🟨 prag kao P-BOJLER, ista skupina `rooms`;
   P-KLIMA već ima pojačanje `below 5000` ×3 — ostaje). Ovdje se brojke v2 **mijenjaju** (rjeđi kvar) pa se `cmp` ne radi, nego brane `--check`.
5. **Pregledni dokument** za Leona (format niže), pa izvještaj; voditelj ga pokaže Leonu.
**B4b / B4c:** čuvar upiše porciju (T3) i spusti strop; graditelj piše oznake po pravilima, pa dokument; balans **bajt-isti** prema stanju poslije B4a (commit oznaka).
**B4c, commit 2 zasebno (uvjeti):** `condition_of` + `when.dept_condition_pts: {"below": 6500}` 🟨 za P-BRAVE, P-CURENJE-VODE, P-DIZALO i `external_cause: true` za P-INSPEKCIJA; brojke v2 se mijenjaju, pa umjesto `cmp` vrijede brane `--check` (kao B4a korak 4).

## Pravila pisanja oznaka (graditelj; ovo je i sadržaj Leonova pregleda)
- Svi odgovori istog predloška: ≤ 1,40× najkraćeg, a najkraći ≥ 55 znakova (da nije „Odbiti"). Praktično 55–100 znakova; S3: „Odbiti" → rečenica jednake težine.
- Slabiji odgovori su **vjerodostojni**: konkretna radnja s pravom manom u sadržaju (vratiti cijeli novac, čekati tehničara), ne karikatura; bez riječi koje
  samo označavaju lošu ocjenu („samo", „bez objašnjenja", „ignorirati"). Najbolji ne smije imati posebne markere dobrote.
- Sastojke „iskrena isprika / rješenje odmah / naknada / uzrok / provjera kasnije" **raspodijeli** na barem dva odgovora; broj zareza razlikuje se najviše za 1.
- Isti oblik rečenice (glagol u infinitivu, ista razina formalnosti). Ništa novo u `{…}`; postojeća mjesta (`{sobe}`, `{noci}`, `{ponuda_pct}`) ostaju.
- **Značenje se mora slagati s mehanikom** (hitan popravak ⇒ „odmah pošalje tehničara"; `compensation: 0` ⇒ ne obećava naknadu). Dokument ih stavlja jedno do drugog.
- Kvaliteta ostaje: `criteria` se ne pomiču. Kriteriji koje najbolji odgovor „pokriva" ostaju u `rubric` (presuda ih čita odande), pa kraći tekst ne gubi ocjenu.

## Leonov pregled tekstova — što graditelj predaje i što blokira spajanje
**Datoteka** `b4X-pregled-tekstova.md` (X = a/b/c): za svaki predložak `### P-KLIMA — naslov (odjel)` pa tablica **slijepog pregleda**: slova A–D redom
od stalnog nasumičnog redoslijeda (sjeme = šifra), stupci: slovo · tekst · znakova; ispod redak `Moj izbor najboljeg: __` i `Odluka: __` (✔ ili ✎ + zamjena).
Na kraju odjeljak **Ključ**: slovo · stari tekst · novi tekst · broj kriterija (5/3/1/0) · mehanika u jednoj rečenici · omjer duljine prije → poslije;
sažetak: koliko predložaka je ≤ 1,40×, koliko je najbolji još najdulji, imena predložaka bez prijavljenog problema. Skripta provjeri da su prisutne **sve** šifre porcije.
**Što blokira:**
- **Spajanje cigle u `model-v2/sloj1`** blokiraju samo automatske provjere (niže) + postojanje dokumenta sa svim šiframa porcije. Leonov pregled **ne blokira**
  (pretpostavka 🟨: grana nije `main`, ispravak je jeftin jer T2 jamči „samo oznake"). Predložak sa zapisom 🟨 TEKST ČEKA PREGLED ostaje u `docs/records/` (voditelj).
- **L1-14 (Leon igra)** čeka da je B4a–c spojena; **L1-15 (`sloj1` → `main`)** blokira svaki prazan `Odluka:` u trima dokumentima.
- Leonovi ispravci dolaze kao jedna mala cigla „B4-ispravci" (samo oznake, ista pravila, T2 i T4 zeleni).

## Provjere (svaka zasebno, `; echo EXIT=$?`)
- Prije prve izmjene spremi u scratchpad izlaz tri balansa (osnovica B4a): v1 `--seeds 100 --check`, v2 tjedni i mjesečni `--seeds 1000 --check`.
- `cargo test -p content --test v2_b4_labels --test v2_principles --test v2_templates` → `ok`, 0 failed
- `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `node scripts/check-core-deps.mjs` · `node scripts/probe-mutacija.mjs`
  · `git diff --exit-code` (nakon prevođenja) · `cargo sqlx migrate run --source crates/store/migrations` pa `cargo sqlx prepare --workspace --check`
  (kao CI) · `cargo test --workspace` (uključuje `v2_sealed`, `v1_frozen`, `fixtures_sealed`: moraju ostati zeleni jer se zamrznuti snimci ne diraju)
- `cargo run -q -p balance --release -- --seeds 100 --check`, `… --model v2 --round week --seeds 1000 --check`, `… --model v2 --round month --seeds 1000 --check`
- **`cmp` s osnovicom:** v1 balans **uvijek** bajt-isti (oznake nisu u obračunu; v1 je zamrznut). v2 (tjedni i mjesečni): **bajt-isti poslije commita 1 u B4a i u cijelim
  B4b/B4c**; **poslije commita 2 u B4a brojke se mijenjaju** — tada nema `cmp`, nego obje brane `--check` zelene i nove brojke u izvještaj.
- `cargo deny` samo CI · `corepack pnpm check:docs` · `test:docs` · `check:kod` · `check:api` · `lint` · `typecheck` · `test` · `corepack pnpm --filter @hotelixsim/web build`
  (web testovi imaju vlastite stare oznake u `apps/web/src/test/fixtures.ts`; ne mijenjaju se)
- Commit B4a: „Popravak B4a: oznake sidara i prve porcije unutar 1,5×, uvjet kvara za klimu i praonicu" (dva commita, vidi korak 3–4); B4b/B4c analogno.
- **Živi AI test i mjerilo** (`content/hotel/mjerilo/S1.json`, `S2.json`, presuda bira najbliži odgovor po oznaci) tek uz Leonov izričit OK; ne radi agent.

## CORE_VERSION i pečat (iz koda, ne iz plana)
- Pečat `crates/game/tests/v2_sealed.rs` igra nad **zamrznutim** snimkom `tests/fixtures_v2/v2-snapshot.json` (test 2 čuva njegov otisak), pa izmjena
  `content/hotel/**` **ne ruši** pečat i **ne traži** novi. `CORE_VERSION` (`crates/sim-hotel-v2/Cargo.toml`; do B4 je 2.4.0 iz B1a) se bira iz koda obračuna.
- Svaka igra nosi svoj snimak sadržaja u `GameCreated` (`event.rs:25`, otisak provjerava `fold`), pa stare igre i dalje igraju stare oznake i stare uvjete bez 409.
- Zaključak: B4 **ne diže** `CORE_VERSION` i **ne obnavlja** pečat. Plan (Global Constraints) kaže „svaka promjena koja mijenja brojke v2 igre diže verziju" — čitam to
  kao promjenu koda obračuna/izvlačenja; promjena sadržaja (uvjeti kvarova) mijenja brojke samo **novih** igara. **Ako voditelj to čita drukčije**, B4a diže na sljedeću verziju (2.5.0)
  i pečati novi otisak (`HOTELIXSIM_SEAL_V2=1 cargo test -p game --test v2_sealed -- --ignored seal_v2_digest`), Leonove probne igre tada daju 409. 🟨 za ADR.
- Zajednička kalibracija L1-13 ionako diže verziju; B4 je ne troši.

## Rizik i odluke (🟨)
- 🟨 **Sidra su u v1 datoteci** (`content/hotel/situations.json`) koju v2 koristi za S1–S4 (`Snapshot::read_v2` čita istu datoteku), pa se nove oznake vide i u v1. Brojke v1
  se ne mijenjaju (oznake nisu u obračunu; zamrznuti v1 fixturei imaju vlastitu kopiju), a `cmp` v1 balansa to dokazuje. Alternativa (zasebna v2 kopija sidara) traži izmjenu učitavanja
  i nije u B4. Leon 15.10.: zajednička datoteka.
- 🟨 **Prag poštene oznake** 1,40× (rezerva do 1,5×) i najkraći ≥ 55 znakova; 🟨 prag T4 50 %.
- 🟨 **Uvjet 6500** za P-KLIMA/P-PRAONICA; kalibracija ga može pomaknuti u L1-13. Dva predloška više ne dolaze kod dobro održane sobe: `DrawStats` brane J-3 (m3 iz faza-a-review)
  postaju djelomično ovisne o planu robota. Ako J-3 padne, graditelj **ne** popravlja broj, nego javlja (ovisnost se zapisuje uz branu, kao u B1).
- **P-DIZALO (`hotel`), P-BRAVE i P-CURENJE-VODE (`maintenance`) dobivaju uvjet u B4c:** `templates_check.rs:312` odbija uvjet odjela za `hotel`, a za `maintenance`
  bi gledao stanje održavanja, ne Smještaja (ADR-042 t. 7 kaže Smještaj). Zato B4 ide nakon B1b i koristi njegov „čijeg prostora" (`condition_of`). Dug `BreakdownCondition` (4 s P-INSPEKCIJOM) pada na 0 u B4c;
  P-INSPEKCIJA ide kao `external_cause` (Leon 15.10.).
- **Dug `SilenceNotBetter` (16) NIJE u B4:** nije tekst nego brojke (trošak/bodovi odgovora i šutnje); vlasnik P4 → L1-13 (voditelj 15.10.). „Dug pada na 0" u B4 znači oznake i uvjete kvarova, ne šutnju. Nakon B4c:
  `LabelLengths` 0, `BreakdownCondition` 0, `SilenceNotBetter` 16 (pada na 0 na kraju L1-13).
- Predložak s jednakim iznosima kvalitete (P-POHVALA [5,4,1]; P-VJENCANJE, P-KRIZA-OSOBLJA ne idu silazno) — redoslijed ostaje, miješanje to rješava.
- Vrijeme pisanja ovisi o sastavljanju 229 tekstova; ako graditelj prekorači porciju, B4c se dijeli po odjelu (restoran | bar/spa/lobby).

## Leonov dio (odgovoreno 15.10.)
1. **Duljina (P3):** najviše **1,5×** (Leon potvrdio; najbolji odgovor ~90 znakova umjesto ~230, detalji ostaju u rubrici).
2. **Sidra S1–S4:** prepisuju se u **zajedničkoj datoteci** (v1 i v2 vide nove tekstove, brojke iste); posebna v2 kopija otpada.
3. **P-INSPEKCIJA:** vanjski događaj, `external_cause: true` (opcija a); ide u B4c.
Preostaje samo pregled tekstova: pregled tekstova (niže) nije anketa nego dokument: A/B/C/D slijepi izbor najboljeg + `Odluka`.

## Review Focus kandidati
- Je li ijedna oznaka promijenila značenje u odnosu na mehaniku (npr. „odmah" uz `urgent: false`)? Dokument slijepog pregleda to ne hvata strojno; pregledati uzorak od 10 po porciji.
- Je li T2 stvarno ugrizao (probno promijeniti `criteria` jednog odgovora; test mora pasti) i T4 (vratiti staru oznaku na jedan predložak)?
- Vide li roboti/balans oznake igdje? (provjereno: `label` u `balance`, `sim-hotel*` ne postoji; ide samo u `ai_input`/`feed`/`mentor`.)
- Presuda (`presuda.v3`) bira najbliži odgovor po oznaci: kraći najbolji tekst može promijeniti odabir slobodnog odgovora; mjerilo S1/S2 prije L1-14 uz Leonov OK.
- A6 (a) i dalje zelen (`answer: 0` najbolji dok miješanje ne stigne); L1-13b mora promijeniti taj redak (točka 5 gore).
