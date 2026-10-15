# Bodovanje vlastitog odgovora — nacrt za raspravu

> **Status:** ✅ odlučeno 15.10.: opcija A ([ADR-047](../records/DECISIONS/ADR-047-vlastiti-odgovor-vrijedi-vise-od-klika.md)); nacrt (2026-10-15), anketa §6 odgovorena. Povod, Leon 15.10.: *„kada student
> daje svoj vlastiti odgovor on može biti bodovan više; ukoliko student ne priča s gostom, ne odgovori svojim riječima
> nego samo izabere najbolje rješenje, ne dobije maksimalan broj bodova — to trebamo riješiti."* Mjesto u planu:
> cigla **PRAVILO** u [planu popravka](../plan/2026-10-14-popravak-temelja.md) (Faza B). Odluka je u ADR-u.

## 1. Problem u dvije rečenice
Danas klik na najbolji ponuđeni odgovor donosi isto što i najbolji odgovor napisan vlastitim riječima, ocjenu 5 od 5,
pa student koji ne razgovara s gostom i ne piše ništa ne gubi ništa. Igra tako nagrađuje prepoznavanje točnog odgovora,
a ne vještinu koju učimo: sam smisliti i reći gostu rješenje.

## 2. Kako je danas (provjereno u kodu)
- **Ocjena (kvaliteta, 1–5)** po putu odgovora, `crates/game/src/resolve.rs`:
  - klik: kvaliteta tog odgovora (`:37-50`). Kvaliteta ponuđenog odgovora = broj kriterija rubrike koje ispunjava
    (`crates/content/src/templates.rs:295-299`). Svaki v2 predložak ima odgovor s 5 kriterija, pa najbolji klik = **5**;
  - vlastite riječi: mehanike (trošak, smjer) od sučeva `nearest_answer`, kvaliteta iz presude (`:51-81`). Presuda broji
    ispunjene kriterije, 0–1 → 1, najviše 5 (`crates/game/src/verdict.rs:52-56`). Najbolje napisano = isto **5**;
  - prepušteno voditelju: **2** (`DELEGATED_QUALITY`, `:12-13`, `:82-96`); šutnja: **1** (`:97-107`);
  - AI presuda ne stigne: neutralna **3** s mehanikama šutnje (`verdict.rs:17-18`).
- **Od ocjene do bodova:** jezgra množi korak `(q − 3)` s bodovima po koraku (`crates/sim-hotel/src/mechanic.rs:141-145`;
  oporavak usluge 200, zadržavanje 300, krizna komunikacija 300 — `content/hotel/catalog.json:42-44`). Jezgra prihvaća samo
  q 1–5 (`mechanic.rs:141`); grupa dobiva popust na cijenu tek od q ≥ 4 (`:208`). v2 to koristi preko
  `crates/sim-hotel-v2/src/resolve/situations.rs:39-49` i množi faktorom kruga (`situation_effect_bp` 350 tjedno / 325
  mjesečno, granica kruga ±800 bodova — `content/hotel/v2/periods.json:7-8, 15-16`).
- **Napredak** se računa samo iz pet pokazatelja (`crates/sim-hotel/src/score.rs:24-31`), pa ocjena u napredak ulazi
  samo preko tih bodova. Izvještaj kruga i mentor dobivaju `quality`, `free_text`, `delegated`
  (`crates/game/src/ai_input.rs:58-79`).
- **Sudac (presuda.v3) već vidi razgovor:** `AssessInput.transcript` (`ai_input.rs:25-33`, `:186-194`), u poruci kao
  `<razgovor>` (`crates/ai/src/request.rs:85-92`), a prompt kaže da kriterij smije biti ispunjen „i u razgovoru s likom"
  (`prompts/presuda.v3.md:4`). Ali razgovor **ne nosi zasebne bodove** i nitko ne provjerava je li tekst prepisan ponuđeni odgovor.
- **Razgovor:** najviše 6 studentovih poruka (`crates/game/src/talk.rs:16`), samo u situaciji sa slobodnim odgovorom
  (`talk.rs:61-63`), i samo dok situacija nije riješena (`:65-66`): nakon klika razgovora više nema.
- **Gdje se uopće može pisati:** slobodni odgovor postoji samo kad situacija ima lika
  (`crates/content/src/templates_instance.rs:343`): **40 od 55** v2 predložaka te sidra S1 i S2
  (`content/hotel/situations.json:48, 95`); S3, S4 i 15 predložaka nemaju polje za pisanje.

## 3. Opcije
Zajedničko svim opcijama (🟨 pretpostavke, ne pitam ih jer nemaju pravu alternativu):
- **Pravilo vrijedi samo gdje se može pisati.** U situaciji bez lika klik ostaje kao danas (5) — inače bi student gubio
  bodove bez načina da ih vrati.
- **Prepisani ponuđeni odgovor je klik.** Ako se tekst (mala slova, bez interpunkcije i kvačica) s nekim ponuđenim
  odgovorom poklapa u ≥ 70 % riječi, računa se kao klik tog odgovora. Bez toga bi svatko kopirao najbolji odgovor u polje.
- **Pravilo je postavka igre** (`GameSettings`, `crates/game/src/command.rs:27-31`, zapisano u `GameCreated` kao u B2a):
  stare igre i v1 zadržavaju staro pravilo i ostaju bajt-iste. Trošak odgovora ostaje isti (Leonova odluka 15.10.
  „trošak sada, ugled kasnije" se ne mijenja); mijenja se samo bodovni dio `(q − 3)`.

Brojke ispod su u **koracima kvalitete** `(q − 3)`. Primjer: oporavak usluge (200 bodova po koraku) danas za najbolji
odgovor daje +400 bodova zadovoljstva prije faktora kruga.

| put | danas | A | B | C |
|---|---|---|---|---|
| vlastiti, najbolji | +2 (q 5) | **+2** (q 5) | **+3** (q 6) | +2 (q 5, uz dobar razgovor) |
| klik, najbolji | +2 (q 5) | **+1** (q 4) | +2 (q 5) | +1 (q 4) |
| srednji (klik ili pisan) | 0 (q 3) | 0 | 0 | 0 |
| prepušteno | −1 (q 2) | −1 | −1 | −1 |
| šutnja | −2 (q 1) | −2 | −2 | −2 |
| razmak najbolji klik − šutnja | 4 | 3 (−25 %) | 4 | 3 |
| razmak najbolji vlastiti − šutnja | 4 | 4 | 5 (+25 %) | 4 |

### A — Klik najviše 4, vlastite riječi do 5 (preporuka)
- **Što student doživi:** uz ponuđene odgovore piše „Klikom dobivaš najviše 4 od 5. Za punih 5 odgovori gostu svojim
  riječima." Napisan odgovor koji ispuni 4 kriterija vrijedi kao najbolji klik; svih 5 vrijedi više.
- **Kod:** `crates/game/src/resolve.rs:42-49` (kvaliteta klika `min(q, 4)` kad situacija ima slobodni odgovor i pravilo je
  uključeno); `command.rs` (`GameSettings`, polje pravila); provjera prepisivanja u `crates/game/src/talk.rs`
  (`submit_response`) ili pri upisu presude; `DebriefSituation` dobiva `capped` da mentor zna objasniti
  (`ai_input.rs:58-79`); web: napomena u `apps/web/src/screens/situation/FreeResponse.tsx` i uz ponuđene odgovore;
  balans: novi put „klik najbolji" u B3a (`crates/balance/src/v2_answers.rs`) i paritet s igrom. **Jezgra se ne dira**
  (q ostaje 1–5), nema dizanja `CORE_VERSION`.
- **Balans i L1-13:** današnji put B3 „najbolji" postaje „vlastiti najbolji" (isti brojevi, q 5 i mehanike najboljeg),
  a umeće se „klik najbolji" (q 4). Lanac B3b: **vlastiti > klik najbolji > srednji > prepušteno > šutnja**. Popust
  grupe (q ≥ 4) ostaje i za klik. L1-13 kalibrira `situation_effect_bp` na put koji Leon izabere u pitanju 3.
- **Rizici:** student koji slabo piše dobije manje nego da je kliknuo (rizik je pošten, ali treba ga reći na ekranu);
  ako AI ne stigne, pisac dobije neutralnu 3 < klik 4 (pitanje 4); više pisanja = više poziva suca (Sonnet) i lika
  (Haiku) — trošak izmjeriti iz `AiMeta` prije pilota; brzi igrač gubi do 1 koraka po situaciji, temeljit dobiva.
- **Sati:** ~10–14 h [PROC] (čuvar + graditelj, jedna grana).

### B — Klik ostaje 5, vlastite riječi donose bonus (q 6)
- **Što student doživi:** klik daje „5 od 5" kao danas; najbolji vlastiti odgovor daje „5 + bonus".
- **Kod:** jezgra prihvaća q 1–6 (`crates/sim-hotel/src/mechanic.rs:141`) → **dizanje `CORE_VERSION`** i `probe-mutacija`;
  `verdict.rs:52-56` (q 6 kad je ispunjeno svih 5 i odgovor nije prepisan); prikaz ocjene na webu (`Verdict.tsx`,
  `FeedScreen.tsx`) i mentor (`prompts/mentor.v4.md`, sezonski prosjek) moraju znati za 6; balans kao u A.
- **Balans i L1-13:** razmak vlastiti − šutnja raste za 25 %. Ako klik ostane na ~20 %, najbolji pisac dolazi na ~25 %,
  na rub pojasa 15–25 % (ADR-042 t. 15).
- **Rizici:** „6 od 5" zbunjuje; svaka buduća provjera „q 1–5" (sadržaj, mjerilo, presuda) treba izuzetak; ne ispunjava
  Leonovu rečenicu doslovno — klik i dalje dobiva „maksimalnu" ocjenu ljestvice.
- **Sati:** ~14–20 h [PROC].

### C — Razgovor s gostom je dio ocjene (sudac ga ocjenjuje)
- **Što student doživi:** kao A, ali za punih 5 ne vrijedi samo sadržaj nego i *kako* je razgovarao: saslušao gosta
  (pitao ili potvrdio što mu treba) i sam mu ponudio konkretno rješenje. Kriteriji razgovora su isti za sve situacije.
- **Kod:** `presuda.v4` (dva zajednička kriterija razgovora, plus „pravopis se ne ocjenjuje"), `VerdictDraft`/presuda
  (`verdict.rs:21-27`), `ai_input.rs` (ništa novo — prijepis već ide sucu), JSON shema u `crates/ai`; mjerilo
  `content/hotel/mjerilo/*.json` dobiva primjere s razgovorom i **nove Leonove ocjene**.
- **Balans i L1-13:** brojke iste kao A, pa C ne traži novu kalibraciju — mijenja samo *tko* dobiva 5.
- **Rizici:** gomilanje poruka („ok", „hvala") da se razgovor „broji" — kriteriji moraju gledati sadržaj, ne broj
  poruka; student koji riješi u jednoj poruci ne smije biti kažnjen; granica 6 poruka mora ostati dovoljna (lik.v3 već
  zatvara razgovor do 6. poruke); student koji slabije piše hrvatski; sudac postaje stroži, pa mjerilo mora pokazati
  da se slaže s Leonom prije uključivanja. Kašnjenje i trošak suca isti kao danas (jedan poziv po odgovoru).
- **Sati:** ~18–26 h [PROC] + Leonove ocjene (~1–2 h njegova vremena).

## 4. Mjesto u planu
- **Prije L1-13**, jer A i B mijenjaju brojke koje L1-13 kalibrira i lanac koji čuva B3b. Cigla PRAVILO već je u
  tablici faze B: ovisi o B3b i ovom nacrtu, a L1-13 čeka B4c **i** PRAVILO.
- **Ovisi o:** B2a (`GameSettings` u `GameCreated` — pravilo je postavka igre), B3a (mjerač putova: dodaje se put „klik
  najbolji"), B3b (lanac brana: proširuje se jednim korakom). Ne ovisi o B1/B4.
- **Bez C3:** A i B rade s presuda.v3 (sudac već broji kriterije i čita razgovor).
- **Uz C3 (`presuda.v4` + mjerilo na v2 predlošcima):** dio C (kriteriji razgovora) i stroža provjera „vlastite riječi"
  od strane suca. Ne mijenja brojke, pa smije doći nakon L1-13.
- **Ime i veličina cigle** upisuju se u plan nakon ankete (🟨 grana `popravak/pravilo-vlastiti`, jedan dan).

## 5. Preporuka
**A sada, C kasnije u C3.** A doslovno ispunjava Leonovu rečenicu (bez vlastitih riječi nema maksimuma), ne dira jezgru
ni ljestvicu 1–5, ne traži novi AI i daje balansu jasan lanac od pet koraka prije L1-13. B je skuplji (jezgra,
`CORE_VERSION`, „6 od 5") za isti učinak. C je pravi cilj („priča s gostom"), ali bez novog prompta i Leonovih ocjena
ne znamo sudi li AI razgovor pošteno — zato ide u C3, na brojke koje A već postavi.

## 6. Anketa za Leona (4 pitanja)

**1. Koliko vrijedi klik na najbolji ponuđeni odgovor, a koliko najbolji odgovor svojim riječima?**
- **(preporučeno) Klik najviše 4 od 5, svojim riječima do 5.** Jednostavno i jasno studentu; ljestvica ostaje 1–5;
  ne mijenja jezgru igre.
- **Klik ostaje 5, svojim riječima „5 + bonus".** Nitko ne gubi u odnosu na danas, ali ocjena „iznad maksimuma" zbunjuje
  i traži promjenu jezgre.
- **Kao prvo, ali punih 5 traži i dobar razgovor s gostom koji ocjenjuje AI.** Najbliže onome što učimo, ali čeka novi
  prompt suca i tvoje ocjene primjera (C3), pa dolazi tek za nekoliko tjedana.

**2. Što treba za punih 5?**
- **(preporučeno) Dobar odgovor svojim riječima; razgovor s gostom nije obvezan.** Sudac već čita razgovor, pa što
  student kaže gostu pomaže mu ispuniti kriterije; nema „lažnih" poruka samo da se otključa bod.
- **Odgovor svojim riječima i barem jedna poruka gostu.** Tjera studenta da razgovara, ali jedna poruka „dobar dan"
  zadovolji pravilo dok sudac ne počne ocjenjivati razgovor (C3).

**3. Na što se odnosi „situacije nose ~20 % napretka"?**
- **(preporučeno) Na studenta koji sve odgovori najbolje svojim riječima.** To je najviše što situacije mogu dati; klik
  na najbolje tada nosi oko 15 %.
- **Na studenta koji sve klikne najbolje.** Klik ostaje na ~20 %, a pisanje diže situacije na ~27 %, iznad dogovorenog
  pojasa 15–25 %.

**4. Što kad AI ne stigne ocijeniti napisan odgovor (rijetko, nakon ponovljenih pokušaja)?**
- **(preporučeno) Student dobije 4, kao da je kliknuo najbolje.** Ne smije izgubiti zato što je pisao; profesor i dalje
  vidi da je presuda neutralna.
- **Ostaje 3 kao danas.** Jednostavnije, ali uz novo pravilo pisac tada prolazi lošije od onoga koji je kliknuo.
