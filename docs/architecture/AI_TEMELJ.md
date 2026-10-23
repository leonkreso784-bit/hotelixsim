# AI temelj — rok određuje cijenu (dizajn, 24.10.2026.)

> Status: 🟨 **NACRT, ČEKA LEONOV PREGLED** · Povod: [ADR-065](../records/DECISIONS/ADR-065-leonove-odluke-24-10-tempo-profesora-lokal-generatori.md) t. 12–14
> (AI puno jeftiniji, opterećenje nije problem, temelj kreativan) i [ADR-066](../records/DECISIONS/ADR-066-pilot-s-profesorovim-tempom-grupa-igara.md) (pilot s profesorovim tempom).
> Troškovi za fakultet: [TROSKOVI_I_AI.md](./TROSKOVI_I_AI.md). Podloge (sesija 14): `.superpowers/sdd/2026-10-14-popravak-temelja/`
> `ai-trosak-istrazivanje-24-10.md`, `ai-cijene-provjera-24-10.md`, `temelj-opterecenja-24-10.md`, `ai-temelj-ideje-24-10.md`, `revizija-testova-24-10.md`.

## 1. Zašto

Danas AI sloj radi za jednog studenta, ali ne za razred:
- **jedan serijski radnik** i jedan FIFO red za sve vrste poslova (`crates/server/src/main.rs:145-170`, `crates/store/src/jobs.rs:29`);
- rok presude teče **od nastanka obračuna** (120 s, `crates/app/src/jobs.rs:25`). Kad 40 studenata zatvori krug, presude iza
  reda postaju neutralne 3/5 — gužva **mijenja ocjenu**, ne samo čekanje;
- presuda ide na Sonnet s `effort: high` i `max_tokens` 16000 (`crates/ai/src/request.rs:106`), a opis situacije i rubrika stoje u
  korisničkoj poruci, pa se ne dijele među studentima (predmemorira se samo sustavna uputa suca, `request.rs:107`);
- lik se **uopće ne predmemorira** (`character_body` bez `cache_control`, `request.rs:152-160`), a tijek razgovora upisuje se u sustavnu
  uputu svakom porukom (`request.rs:137-139`), pa se ni ona ne bi mogla dijeliti; mentor već radi s `effort: low` (`request.rs:174`, `:189`);
- 429 je obična greška koja troši pokušaj (`crates/app/src/jobs.rs:156-158`);
- procjena 0,45–0,85 $ po tjednoj igri → 40 studenata ≈ 34 $, iznad stropa 30 $/mj.

Leon (24.10.): AI mora biti **puno jeftiniji**, opterećenje **ne smije biti problem**, a temelj takav da inženjer kaže „ja se ne bih
sjetio ovoga".

## 2. Glavna ideja u jednoj rečenici

**Razred dijeli jedan upit, svaki posao nosi rok, a jeftini sudac radi pod stalnim nadzorom skupoga.**

1. **Razredni prefiks:** 40 studenata u istom krugu ima iste situacije, pa je 90 % svakog upita isto. Složimo upit od zajedničkog prema
   osobnom: prvi student plati zajednički dio, ostalih 39 ga čita iz predmemorije po 10 % cijene — a čitanje iz predmemorije **ne broji se
   u granicu ulaznih tokena po minuti**. Isti potez spušta trošak i uklanja najveće usko grlo naleta.
2. **Rok umjesto reda:** posao lika ima rok od 10 s, presuda minutu, mentor minute, sezonski mentor i provjere dan. Radnici uvijek uzimaju
   najbliži rok; što ima vremena ide najjeftinijim putem (skupna obrada, −50 %), a gužva nikad ne pretvara presudu u neutralnu.
3. **Sudac u dva koraka i kontrolor u sjeni:** Haiku (~20× jeftiniji) sudi, Sonnet samo granične slučajeve, a dio Haiku presuda Sonnet
   noću ponovo sudi i mjeri slaganje. Kvaliteta se mjeri **stalno u radu**, ne jednom; padne li, sustav se sam vrati na Sonnet.

Cijena, brzina i opterećenje tako su posljedica istih triju poteza, a ne tri zasebna sustava.

## 3. Ciljevi, ne-ciljevi, anti-ciljevi

**Ciljevi**
1. Razred od 40 na satu (krug od 5 min): lik p95 ≤ 10 s, ≥ 98 % presuda od AI-ja (ne neutralnih) prije obračuna, 0 grešaka 5xx.
2. Trošak tjedne igre **≤ 0,25 $** (cilj [PROC], dokazuje test troška R4), razred od 40 ≤ 10 $ po sezoni.
3. Ista kvaliteta presude: jeftiniji put mora se **slagati sa Sonnetom** na mjerilu (`content/hotel/mjerilo/`) u ≥ 90 % kriterija,
   a ocjena 1–5 se ne smije razlikovati za više od 1 ni u jednom slučaju mjerila.
4. Uvijek odgovor, nikad greška: ako AI ne stigne, radi pravilo (današnje rezerve), i to se **vidi** (profesor zna koja je razina radila).
5. Širi se dodavanjem radnika ili instanci, bez prepisivanja (razine 200 / 1 000 / 5 000 iz `temelj-opterecenja-24-10.md`).

**Ne-ciljevi (za pilot)**: više pružatelja odjednom u radu, vlastiti model, ugađanje modela (fine-tuning), Redis, SSE.

**Anti-ciljevi** (neuspjeh čak i ako radi): gužva mijenja ocjenu; jeftiniji model tiho daje drukčije ocjene; student s 20 poruka
zaguši ostale; trošak se ne može objasniti fakultetu po studentu; AI dira jezgru (`sim_hotel*`, `CORE_VERSION`, događaje igre).

## 4. Ograničenja

- **Jezgra se ne dira.** AI izlaz se i dalje upisuje kao događaj; replay ne zove AI. Razina se zapisuje u postojeće polje
  `AiMeta.prompt` (npr. `presuda.v3+trijaza`), pa nema promjene sheme događaja (`crates/game/src/event.rs:199`).
- **AI ključ samo na poslužitelju**; studentov tekst ostaje podatak (`quote_student`).
- **GDPR**: Anthropicov API obrađuje u SAD-u (zadržavanje 30 dana, bez učenja); obrada u EU = Claude preko AWS Bedrocka
  (Frankfurt, EU profil, ~+10 %, bez Batcha) — izbor pružatelja je konfiguracija, ne kod (§6.7). Besplatne razine otpadaju (uče na podacima).
- Promjena sheme baze (`jobs.deadline_at`, `ai_spend.game_id`…) samo uz Leonov OK, zapisana u ADR.
- Najviše 2 Rust toka istodobno; svaka cigla ima čuvara testova prije graditelja.

## 5. Kako izgleda za studenta i profesora

- Student ne primjećuje ništa osim brzine: lik odgovara u nekoliko sekundi i kad je cijeli razred aktivan; presuda stiže dok
  student još radi plan.
- Profesor u pregledu grupe vidi uz svaku presudu **tko je sudio**: „AI", „AI (provjereno)", „pravilo (AI nije stigao)". Kad
  strop troška grupe prijeđe 80 %, vidi upozorenje (§6.6).
- Fakultet dobiva račun po razredu i po studentu (TROSKOVI_I_AI.md), jer svaki poziv zna igru, ulogu i put.

## 6. Dijelovi

### 6.1 Rok na svakom poslu (temelj svega)
- Novi stupci `jobs.deadline_at` i `started_at` — jedna migracija (`0007`; `0006` je grupa iz ADR-066). Rok se računa **po vrsti
  posla pri upisu**, bez znanja o grupi i tempu (AI dio se tako gradi neovisno o grupi):
  lik = +10 s · presuda = +60 s (student čeka presudu u istom krugu, L1-15 iskočni prozor) · obračun = odmah · mentor kruga = +5 min ·
  sezonski mentor i kontrolor u sjeni = +24 h.
- Obračun više ne mjeri rok od svog nastanka: čeka dok postoji presuda `queued` ili `running` (`started_at` + 90 s > sada), uz tvrdi
  strop 10 min od zatvaranja. Odgovor predan u zadnjoj minuti tako dobiva AI presudu, ne pravilo; pravilo radi samo kad AI ne radi.
- Presuda nikad ne ide skupnom obradom (kasni satima, a student je čeka); skupna obrada je samo za poslove s rokom ≥ 2 h.

### 6.2 Raspoređivač po roku (EDF) s trakama
- `claim` uzima posao s **najbližim rokom** (`ORDER BY deadline_at, id`, `FOR UPDATE SKIP LOCKED`) — lik prirodno ide prvi, a
  presuda ne „umire" iza 120 poruka lika.
- Bazen radnika po trakama: LIK 6 · PRESUDA 12 · MENTOR 3 (`HOTELIXSIM_WORKERS_*`); prazna traka smije uzeti tuđi posao, a liku se
  mjesto uvijek čuva. `crates/server/src/worker.rs` (izvučeno iz `main.rs`).
- **Pravednost po studentu**: unutar iste trake i sličnog roka prednost ima student s manje poslova u tijeku (jedan student s 20
  poruka ne zaguši ostale; granica 6 poruka po situaciji već postoji).

### 6.3 Razredni prefiks (predmemorija upita za cijelu grupu)
- Upit se slaže **od zajedničkog prema osobnom**: [uputa suca] [situacija + rubrika + ponuđeni odgovori + sidra iz mjerila]
  ⟵ točka predmemorije ⟶ [razgovor + odgovor studenta]. Danas su situacija i rubrika u osobnom dijelu (`render_assess`).
- Predmemorija se veže uz **sadržaj** prefiksa, ne uz grupu: svi studenti (i iz različitih grupa i igara za vježbu) koji u istih
  5 minuta dobiju **isti predložak situacije** dijele prefiks. Prvi plati upis, ostali čitaju po 10 % cijene. U grupi je prvi krug
  isti za sve (isto sjeme), a dalje se situacije razilaze po stanju hotela (`generator-i-lokal-24-10.md`), ali predložaka je u krugu
  malo, pa se prefiks i dalje dijeli. Zato prefiks smije sadržavati samo tekst predloška, ne brojke ni imena pojedinog hotela. Čitanje iz predmemorije **ne broji se u granicu ulaznih tokena po minuti** (provjereno 24.10.), pa isti
  potez rješava i trošak i najveće usko grlo pri naletu.
- Predmemorija od 5 min obnavlja se svakim čitanjem, pa razred koji radi isti krug drži prefiks „toplim". U tjednom tempu studenti
  rade razvučeno, pa prefiks češće istekne — tada je presuda i dalje Haiku (§6.5), pa je trošak nizak i bez predmemorije.
- Lik dobiva isto: [uputa lika + opis lika i situacije] ⟵ predmemorija ⟶ [tijek razgovora + razgovor]. Tijek (`conversation_stage`)
  se seli iz sustavne upute iza točke predmemorije; to je nova verzija upute lika (`lik.v4`), uz živi test kvalitete.
- Prefiks mora biti dulji od najmanje duljine predmemorije za model (provjera 24.10.: 512 tokena); R4 mjeri stvarne
  `cache_read_input_tokens`, pa se ušteda dokazuje, ne pretpostavlja.

### 6.4 Rok bira put (najjeftiniji koji stigne)
Za svaki posao raspoređivač gleda preostalo vrijeme do roka:

| preostalo do roka | put | za koje poslove |
|---|---|---|
| ≥ 2 h | **skupna obrada** (Batch, −50 %) | sezonski mentor, kontrolor u sjeni |
| sekunde – minute | odmah, prefiks iz predmemorije | lik, presuda, mentor kruga |
| red pun (pritisak) | odmah, bez drugog koraka suca (§6.5), kraći odgovor lika | lik, presuda |
| AI ne radi ili 10 min prošlo | **pravilo** (današnja rezerva), zapisano | sve |

Skupna obrada se šalje u paketima (npr. svakih 10 min ili 50 poslova), rezultat se dočita i upiše kao i danas; ako paket ne stigne do
roka − 1 h, posao ide putem „odmah".

### 6.5 Sudac u dva koraka + kontrolor u sjeni
- **Prvi korak (Haiku)** daje kriterije kao danas (shema `verdict_schema` se ne mijenja), uz `effort: low`.
- **Drugi korak (Sonnet)** samo kad je slučaj granični: ocjena iz kriterija pada točno na prag između dvije ocjene, odgovor je
  dulji od N znakova, ili Haiku i najbliži ponuđeni odgovor znatno odstupaju. Cilj: Sonnet ≤ 20 % presuda [PROC].
- **Kontrolor u sjeni**: nasumičnih 5 % Haiku presuda ponovo sudi Sonnet kroz skupnu obradu (jeftino, bez roka) i bilježi
  slaganje. Kvaliteta se tako **mjeri stalno u radu**, ne samo jednom na mjerilu; ako slaganje padne ispod praga, sustav sam
  prebaci prvi korak na Sonnet i upozori (zapis + `/v1/health/ai`). Ocjena studenta se pritom ne mijenja naknadno.
- Mentor kruga je **najveća stavka troška** (0,16–0,26 $ po igri na Sonnetu, TROSKOVI_I_AI §3); bez promjene mentora cilj ≤ 0,25 $
  po igri ne prolazi. Mentor ostaje Sonnet `effort: low` dok Leonov slijepi pregled parova Sonnet/Haiku (plan, val 3) ne pokaže da je
  Haiku dovoljan; tada ide na Haiku. Sezonski mentor ide skupnom obradom (poslije pilota). Lik ostaje Haiku.

### 6.6 Budžet kao resurs (kanta po grupi)
- Grupa (i svaka igra bez grupe) ima **kantu**: udio mjesečnog stropa raspoređen po krugovima i studentima unaprijed, a ne samo „stani
  kad prođe". Veličina je automatska: mjesečni strop ÷ (broj aktivnih igara × preostali krugovi), s minimumom koji pokriva jednu
  Haiku presudu po situaciji. Današnji stropovi (varijable `HOTELIXSIM_AI_STROP_DAN_USD` i `_MJESEC_USD`, `crates/server/src/config.rs:19-20`)
  ostaju jedina tvrda ograda.
- Prazna kanta **nikad ne spušta presudu na pravilo** (to bi mijenjalo ocjenu): gasi se samo drugi korak suca i skraćuje odgovor lika.
  Pravilo radi tek kad padne tvrda ograda, kao danas. Profesor vidi potrošnju grupe i upozorenje na 80 %.
- `ai_spend` dobiva `game_id`, `role`, `path` (migracija uz OK) → trošak po studentu i po razredu za fakultet.

### 6.7 Pružatelj iza jednog porta
- `AiPort` (`crates/app/src/ports.rs:167`) ostaje jedini ulaz. Pružatelj i model po ulozi su konfiguracija
  (`HOTELIXSIM_AI_PROVIDER=anthropic|bedrock-eu`), cijene u istoj tablici `PRICES` s ključem (pružatelj, model).
- 429/529 postaje `AiError::RateLimited { retry_after }`: zajednička pauza po (pružatelj, model), **ne troši pokušaj**. Pauza i trake
  žive u memoriji procesa: pilot ima **jedan proces** (`serve` s ugrađenim radnicima); za više instanci stanje seli u bazu (poslije pilota).
- Pilot: samo Anthropic. Bedrock EU je spreman kao konfiguracija kad fakultet zatraži obradu u EU (TROSKOVI_I_AI.md §GDPR).

### 6.8 Isti odgovor = ista presuda (poslije pilota)
- Otisak (situacija + verzija upute + normalizirani razgovor i odgovor) → ako postoji presuda s istim otiskom, uzima se ona.
  Štedi na ponovljenim i prepisanim odgovorima; profesoru može pokazati „isti tekst kao kod X" — to traži odluku o privatnosti, pa poslije.

## 7. Što ide kad

| dio | pilot (prije studenata) | poslije pilota |
|---|---|---|
| 6.1 rok na poslu | da | — |
| 6.2 EDF + trake + pravednost | da | globalni strop u bazi za više instanci |
| 6.3 razredni prefiks | da | — |
| 6.4 rok bira put | odmah / pritisak / pravilo **da** | skupna obrada (sezonski mentor, kontrolor) |
| 6.5 sudac u dva koraka | da, **ako mjerilo prođe** (§8); inače Sonnet `effort: medium` | kontrolor u sjeni |
| 6.6 kanta | stupci `ai_spend` + kanta grupe | kanta po fakultetu (`tenant`) |
| 6.7 pružatelj | `RateLimited`, konfiguracija | Bedrock EU u radu |
| 6.8 otisak | — | da |

## 8. Kako se dokazuje (ništa ne ulazi bez mjerenja)

1. **Test opterećenja** (`crates/server/tests/load.rs`, R1): 40/100 igara, lažni AI s kašnjenjima po ulozi i 10 % 429; brzi CI dio
   (vrijeme ×0,01) **mora pasti s današnjim jednim radnikom**, a proći s §6.1–6.2. Realni `#[ignore]` ispisuje p50/p95.
2. **Test troška po igri** (`crates/server/tests/ai_cost.rs`, R4): cijela tjedna i mjesečna sezona s pravim `usage` brojkama iz živih
   prolaza; ispis $ po igri i ×40 po putu; brana: najgori slučaj × 40 ≤ mjesečni strop.
3. **Mjerilo kvalitete** (`ai/tests/live.rs`, `--ignored`, < 1 $): 40 odgovora mjerila kroz Sonnet-high (sadašnje), Sonnet-medium,
   Haiku-low i dvokorak → tablica slaganja po kriteriju i ocjeni. Dok Leon ne ocijeni mjerilo, referenca je Sonnet-high (🟨).
4. **Živi nalet** na pilotu prije prvog sata: 40 presuda odjednom (~0,10–0,50 $), zapisnik.

## 9. Cigle (za plan; sati [PROC])

| # | cigla | datoteke (vlasništvo) | h | Rust |
|---|---|---|---|---|
| AI-1 | rok na poslu + obračun čeka presude u tijeku (migracija `0007`) | `store/migrations/0007_*` (novo), `store/src/jobs.rs`, `app/src/jobs.rs` | 5 | da |
| AI-2 | radnici po trakama + EDF + pravednost | `server/src/worker.rs` (novo), `server/src/main.rs`, `store/src/jobs.rs` (claim) | 7 | da |
| AI-3 | `RateLimited` + pauza po modelu + istek po ulozi | `ai/src/lib.rs`, `ai/src/response.rs`, `app/src/ports.rs`, `app/src/ai_limit.rs` (novo) | 6 | da |
| AI-4 | razredni prefiks (presuda i lik, `lik.v4`) | `ai/src/request.rs`, `prompts/` (nove verzije), `ai/tests` | 5 | da |
| AI-5 | mjerilo kvalitete puteva (živi, < 1 $) | `ai/tests/live.rs`, `ai/src/benchmark.rs` | 3 | da |
| AI-6 | sudac u dva koraka (ako AI-5 prođe) | `app/src/judge.rs` (novo), `app/src/jobs.rs` (run_assess) | 5 | da |
| AI-7 | kanta + `ai_spend` po igri/ulozi/putu (migracija `0008`) | `app/src/spend.rs`, `store/migrations/0008_*`, `store/src/spend.rs` | 5 | da |
| AI-8 | vidljivost: `/v1/health/ai`, „tko je sudio" u pregledu | `api/src/health.rs`, web profesora | 3 | da + web |
| AI-9 | skupna obrada (sezonski mentor, kontrolor) | `ai/src/batch.rs` (novo), `app/src/jobs.rs` | 8 | da |
| AI-10 | kontrolor u sjeni + otisak | `app/src/judge.rs` | 6 | da |

Prije pilota: AI-1 … AI-8 ≈ **39 h**; AI-9 i AI-10 poslije pilota (štede malo dok je presuda Haiku). AI-1/AI-2 zamjenjuju O1/O3 iz
`temelj-opterecenja-24-10.md`, AI-3 zamjenjuje O2.

## 10. Odluke koje su moje (🟨, Leon ih može promijeniti)
1. Rok po vrsti posla pri upisu (lik 10 s, presuda 60 s, mentor 5 min, sezona 24 h); obračun čeka presude u tijeku do 10 min.
2. Prag kvalitete jeftinijeg puta: ≥ 90 % slaganja po kriteriju, nikad razlika ocjene > 1 na mjerilu.
3. Lik ostaje Haiku; mentor kruga Sonnet `effort: low` dok Leonov slijepi pregled ne odobri Haiku; presuda nikad skupnom obradom.
4. Razina se zapisuje u `AiMeta.prompt` (bez promjene događaja).
5. Pilot ide samo preko Anthropica (SAD, 30 dana, bez učenja); Bedrock EU je spreman, ne uključen.
6. Prazna kanta gasi samo drugi korak suca, nikad ne daje pravilo umjesto presude.
7. Migracije: `0006` grupa (ADR-066), `0007` rok na poslu, `0008` `ai_spend` po igri — Leonov OK na plan je OK na ove tri.
