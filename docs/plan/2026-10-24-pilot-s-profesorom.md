# Pilot s profesorom — testovi, AI temelj, grupa igara, L1-15 — plan izgradnje

**Status:** 🟨 **NACRT, čeka Leonov OK** · **Područje:** igra · **Otvoren:** 2026-10-24 · **Grana:** integracijska `model-v2/sloj1`;
svaka cigla na kratkoj grani `pilot/<tok>-<cigla>` u radnom stablu `../hotelixsim-<tok>`, spaja se isti dan.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
> Brief izvođaču = putanja ovog plana + naslov taska; čita samo svoj odjeljak i „Global Constraints".

**Goal:** Profesor na stranici napravi grupu, dobije profesorski kod, studenti uđu kodom za ulaz i igraju svaki svoju igru u
profesorovom tempu; razred od 40 na satu ne mijenja ocjene gužvom, AI je višestruko jeftiniji i to je **izmjereno**, a testovi
pokrivaju ono što bi studenta ili Leona najprije ugrizlo (opterećenje, trošak, pravi tokovi, stanje nakon deploya).

**Architecture:** Jezgra (`sim_hotel*`, `CORE_VERSION` 2.6.0, događaji igre) se **ne dira** — grupa živi u `store`/`app`/`api`/web
([ADR-066](../records/DECISIONS/ADR-066-pilot-s-profesorovim-tempom-grupa-igara.md)), AI temelj u `ai`/`app`/`server`/`store`
([AI_TEMELJ](../architecture/AI_TEMELJ.md)). Iznimka: generator događaja (val 3) mijenja samo postavku nove igre i balans, ne
`CORE_VERSION`. Za trajanje pilota jezgra se zamrzava.

**Tech Stack:** Rust 2024 (1.98) · axum 0.8 + utoipa · sqlx 0.9 + PostgreSQL 16 · tokio · React 19 + Vitest · `node --test` za skripte.

**Spec i podloge:** [ADR-065](../records/DECISIONS/ADR-065-leonove-odluke-24-10-tempo-profesora-lokal-generatori.md) (Leon 24.10.),
[ADR-066](../records/DECISIONS/ADR-066-pilot-s-profesorovim-tempom-grupa-igara.md), [AI_TEMELJ](../architecture/AI_TEMELJ.md),
[TROSKOVI_I_AI](../architecture/TROSKOVI_I_AI.md), [ADR-063](../records/DECISIONS/ADR-063-pilot-jedna-adresa-lozinka-stropovi-ai-tri-igre.md) (pilot).
Podloge u `.superpowers/sdd/2026-10-14-popravak-temelja/`: `revizija-testova-24-10.md` (R1–R17), `opseg-pilota-kod-24-10.md`,
`opseg-pilota-docs-24-10.md`, `generator-i-lokal-24-10.md`, `podloge-plana-24-10.md` (L1-15, pilot, C), `temelj-opterecenja-24-10.md`.
Preuzima otvoreno iz [plana popravka temelja](../archive/2026-10-14-popravak-temelja.md) (L1-15, kartica Pilot dijelovi 8–10, faza C),
koji je 24.10. premješten u arhivu. Ledger: `.superpowers/sdd/2026-10-24-pilot-s-profesorom/progress.md` (novi).

## Pregled u jednoj tablici

| val | što | tokovi (Rust ≤ 2) | rad agenata | zidno [PROC] |
|---|---|---|---|---|
| **0** | **TESTOVI** (ADR-065 t. 11): opterećenje, trošak po igri, lažni AI, web tokovi, dim nakon deploya, vrata | RED-T ∥ AI-T · WEB-T, SKRIPTE | ~40 h | ~14 h |
| **1** | AI temelj I (rok, trake, 429, prefiks) + L1-15 presuda + izlaz iz zastarjele igre | RED ∥ AI · WEB | ~28 h | ~12 h |
| **2** | grupa igara (profesorski kod, kod za ulaz, tempo) + mentor (L1-15) | GRUPA ∥ MENTOR · WEB | ~38 h | ~18 h |
| **3** | AI temelj II (dva koraka, kanta, vidljivost) + generator događaja + celijakija | AI ∥ SADRŽAJ · WEB | ~26 h | ~12 h |
| **4** | dokaz i isporuka: mjerenja, Leon igra, `main`, pilot dio 9–10, deploy | — | ~12 h + Leon ~4 h | ~2 dana |
| poslije | lokal, faza C, razred u jezgri, AI-9/10, brisanje podataka | — | ~90–120 h | — |

**Ukupno do pilota ≈ 145 h rada agenata, ~60 h zidnog vremena** [PROC], uz Leona ~4 h (igra, računi, OK-ovi).
Dio 8 (tekst privatnosti) čeka profesore (ADR-065 t. 5) i **koči samo puštanje studenata**, ne gradnju ni deploy za profesora.

## Pitanja za Leona prije gradnje

1. **Situacije u grupi** (pravilo koje student osjeti): (a) isti početak za cijelu grupu, dalje po stanju svakog hotela — bez promjene
   jezgre (**preporuka**); (b) potpuno isti popis situacija za cijelu grupu — nov način izvlačenja, +4–6 h, `CORE_VERSION` 2.7.0, balans.
2. **Živi AI testovi:** mjerilo puteva suca (AI-5, < 1 $) u valu 3 i živi nalet 40 presuda na pilotu (~0,5 $) — OK?
3. **Playwright** (pravi preglednik protiv pravog API-ja, nova ovisnost, ~0,5 GB preglednika, ~10 h; R6): u valu 4 prije pilota
   (**preporuka**: profesorov tok je nov i ne pokriva ga nijedan test u pregledniku) ili poslije?
4. **Migracije** `0006` grupa, `0007` rok na poslu, `0008` trošak po igri — OK na plan je OK na ove tri (AI_TEMELJ §10 t. 7).
5. **Mjerilo:** dok ne ocijeniš 40 odgovora (`content/hotel/mjerilo/`), referenca kvalitete je Sonnet-high (🟨). Kad stigneš, AI-5 se ponovi.
6. **Lokal** (poslije pilota): 6 brojki u anketi kad budeš htio (`generator-i-lokal-24-10.md` §B).
8. **Mentor kruga na Haikuu?** Mentor je najveća stavka troška (0,16–0,26 $ po igri na Sonnetu). U valu 3 dobiješ 6 parova tekstova
   bez oznake modela; ako ne vidiš razliku, mentor ide na Haiku (trošak po igri pada ispod 0,25 $), inače ostaje Sonnet.
9. Pretpostavke 🟨 [ADR-066](../records/DECISIONS/ADR-066-pilot-s-profesorovim-tempom-grupa-igara.md) t. 4–13 i [AI_TEMELJ §10](../architecture/AI_TEMELJ.md) — pročitaj, „ok" ili ispravak.

## Global Constraints

- **Jezgra se ne dira** (`crates/sim-hotel*`, `CORE_VERSION`, sheme događaja u `crates/game/src/event.rs`), osim gdje task izričito kaže.
  Svaka cigla koja dira `game`, `content` ili `balance` vrti `cargo run -q -p balance --release -- --model v2 --round week --seeds 1000 --check`
  i `--round month`, te v1 `--seeds 100 --check`; v1 se ne mijenja ni u bajtu (`crates/game/tests/v1_frozen.rs`).
- **Čuvar testova prije graditelja** (ADR-038): čuvar piše crvene testove iz „Čuvar pribija", graditelj ih zazeleni, provjeravatelj
  (opus za ulaz, novac i red poslova; inače sonnet) gleda diff prije spajanja. Izlazi u ledger (`<cigla>-tests.md`, `-report.md`, `-review.md`).
- **Cijeli CI lokalno prije predaje** (`.github/workflows/ci.yml`: fmt, clippy, core-deps, testovi, balansi, `check:docs`, `test:docs`,
  `check:kod`, `check:api`, `corepack pnpm lint`, typecheck, test, web build). **Lint prije svakog commita** (B19, sesija 13).
- **Najviše 2 agenta istodobno prevode Rust.** Svako stablo `../hotelixsim-<tok>` ima `CARGO_TARGET_DIR=C:/Users/leonk/Documents/hotelixsim/target/<tok>`
  (B12), svoju bazu ako pokreće poslužitelj (B9), briše se isti dan nakon spajanja. **Ispod 50 GB slobodno ne gradi** (B16). Gradnja samo na punjaču.
- **OpenAPI:** promjena ruta ili shema → `server openapi` + `check:api` + generirani `apps/web/src/api/types.ts` u istoj cigli.
  `crates/api/src/schema.rs` i `crates/api/src/lib.rs` u jednom valu dira **samo jedan** tok.
- **Migracije** redom `0006` (grupa), `0007` (rok), `0008` (trošak); svaka s testom na pravoj bazi (`#[sqlx::test]`) i `sqlx prepare` (offline).
- **Živi AI test, push, spajanje u `main` i deploy** samo uz Leonov izričit OK za tu radnju. Ključ nikad u pregledniku ni u gitu.
- **Pravila i brojke koje student osjeti** idu Leonu kroz anketu prije gradnje (memorija „odluke o igri pitaj Leona").
- **Lažno crveno pod opterećenjem** (vitest timeout, 0xC0000142) se samo ponovi (B15); lokalni API nakon spajanja ponovo izgraditi i `migrate` (B30).

## Tokovi (vlasništvo datoteka)

| tok | stablo | Rust | datoteke (samo njegove u valu) |
|---|---|---|---|
| **RED** | `hotelixsim-red` | da | `crates/server/src/worker.rs` (novo), `server/src/main.rs`, `server/src/lib.rs`, `crates/store/src/jobs.rs`, `crates/app/src/jobs.rs`, `server/tests/load.rs`, `server/tests/worker_shutdown.rs`, migracija `0007` |
| **AI** | `hotelixsim-ai` | da | `crates/ai/**`, `prompts/**`, `crates/app/src/ai_limit.rs` (novo), `app/src/judge.rs` (novo), `app/src/spend.rs`, `app/src/ports.rs` (samo `AiError`), `server/src/config.rs`, `server/tests/ai_cost.rs`, `server/tests/config_guard.rs`, migracija `0008` |
| **VRATA** | `hotelixsim-vrata` | da | `crates/api/tests/gate.rs`, `api/tests/limits.rs` (novo), slabe tvrdnje R17 u `game/tests`, `server/tests/config_ai_cap.rs` |
| **GRUPA** | `hotelixsim-grupa` | da | migracija `0006`, `store/src/classes.rs` (novo), `app/src/classes.rs` (novo), `app/src/commands.rs`, `app/src/demo.rs`, `app/src/error.rs`, `app/src/memory.rs`, `app/src/ports.rs` (port grupe), `api/src/classes.rs` (novo), `api/src/lib.rs`, `api/src/schema.rs`, `api/src/auth.rs`, `api/src/problem.rs`, `openapi.json` |
| **MENTOR** | `hotelixsim-mentor` | da | `crates/game/src/mentor.rs`, `game/src/mentor_v2.rs`, `prompts/mentor*.md` (nove verzije), `game/tests/mentor*.rs` |
| **SADRŽAJ** | `hotelixsim-sadrzaj` | da | `game/src/command.rs` (`GameSettings`), `game/src/model.rs`, `content/hotel/v2/situation_templates.json`, `crates/balance/src/v2_situations.rs`, `game/tests/situations_drawn*.rs`, `v2_sealed.rs` |
| **WEB** | `hotelixsim-web` | ne | `apps/web/src/**`, `packages/i18n/src/hr.ts` |
| **SKRIPTE** | `hotelixsim-skripte` | ne | `scripts/smoke-pilot.mjs`, `scripts/lazni-ai.mjs`, `scripts/opterecenje.mjs` (+ `.test.mjs`), `.github/workflows/ci.yml`, `deploy-pilot.yml` |

Glavna sesija samo orkestrira (nalozi, presude, spajanje) i u svakom valu **odjednom** pokreće sve neovisne tokove u jednoj poruci.

---

## VAL 0 — TESTOVI (~40 h; prije svega ostalog)

Cilj: prije ijedne nove značajke vidjeti gdje sustav puca. Testovi koji opisuju **budući** temelj (R1 s trakama) smiju biti crveni
i stoje `#[ignore = "zelen nakon AI-2"]` dok ih val 1 ne zazeleni — tako su čuvar za val 1.

### Task 0.1: RED-T — test opterećenja reda poslova (R1, R14)
**Tok:** RED · **Rust:** da · **h:** 13 · **Ovisi o:** — · **Provjeravatelj:** opus
- [ ] Izvući `spawn_worker` + `tick` iz `crates/server/src/main.rs:145-172` u `crates/server/src/worker.rs` (lib) s parametrom broja
      radnika (zadano 1, ponašanje isto). Postojeći testovi `main.rs:239-263` sele uz kod.
- [ ] `crates/server/tests/load.rs` (`#[sqlx::test]`): N igara preko `router` (tower `oneshot`), svaka u krugu: vlastiti odgovor na sve
      situacije + 3 poruke liku, pa zatvaranje; lažni AI `Delayed<FakeAi>` s kašnjenjem po ulozi i raspršenjem, 10 % 429.
- [ ] **Brzi CI dio** (kašnjenja ×0,01, rok skaliran): nijedan posao `failed`/`running` na kraju, svaka igra obračunata, bez dvostrukih
      događaja, 0 odgovora 5xx, **0 neutralnih presuda zbog reda**. S N=1 i 40 igara pada → `#[ignore = "zelen nakon AI-2"]`, ispis zapisan u ledger.
- [ ] **Realni** `#[ignore]` (lik 3 s, presuda 15 s ± 10, mentor 20 s): ispis p50/p95 čekanja lika, udio neutralnih, vrijeme do obračuna.
- [ ] R14 `server/tests/worker_shutdown.rs`: `stop` usred spore presude vrati posao u red (`attempts` isti) u ≤ `JOB_GRACE + WORKER_EXIT`, drugi radnik ga dovrši.
**Čuvar pribija:** pragovi lik p95 ≤ 10 s, ≥ 98 % presuda od AI-ja, obračun ≤ 180 s za 40 igara, 0 5xx (AI_TEMELJ §8).

### Task 0.2: AI-T — trošak po igri, lažni AI, 429 (R4, R7, R3)
**Tok:** AI · **Rust:** da · **h:** 10 · **Ovisi o:** —
- [ ] R7: varijabla `HOTELIXSIM_AI_BASE_URL` u `crates/server/src/config.rs`, **odbijena u produkcijskom načinu**; `server/tests/config_guard.rs`.
- [ ] R4 `crates/server/tests/ai_cost.rs`: pravi `ai::Anthropic::with_base_url(lažni HTTP)` (`crates/ai/src/lib.rs:125`) koji po vrsti zahtjeva
      vraća stvarne `usage` brojke (ulaz, izlaz, `cache_read_input_tokens`, `cache_creation_input_tokens`) iz živih prolaza zapisanih u
      `.superpowers/sdd/…/a5-live.txt` i `ai/tests/live.rs`; `Metered` u memorijsku knjigu; cijela tjedna (17 krugova, kao `api/tests/e2e_v2.rs`)
      i mjesečna sezona, scenariji „najgori" (sve vlastiti odgovori, 6 poruka po liku) i „realno".
- [ ] Ispis: $ po igri po ulozi, ×40 po razredu; **brana:** najgori × 40 ≤ mjesečni strop iz varijable. Ispis ide u ledger kao **osnovica troška**
      (TROSKOVI_I_AI.md §3 se zamjenjuje izmjerenim u valu 4).
- [ ] R3 u stilu `ai/tests/usage.rs`: lažni HTTP vraća 429 s `retry-after` k puta pa 200 → posao uspije prije roka; trajni 429 → rezerva i trošak se **ne** računa.
**Čuvar pribija:** zbroj iz knjige = zbroj `usage` × cijena iz `PRICES` (`crates/app/src/spend.rs:71`), i za odrezan odgovor.

### Task 0.3: WEB-T — tokovi studenta u vitestu (R8, R9, R10, R17 web)
**Tok:** WEB · **Rust:** ne · **h:** 7 · **Ovisi o:** —
- [ ] `CloseMonth.test.tsx`: potvrda, „čeka obračun", završetak ispitivanja kad status ≠ `closing` (lažni sat, ne čekanje).
- [ ] `Verdict.test.tsx`: `pending` → gotova, neutralna i zakašnjela oznaka. `Conversation.test.tsx`: „piše…" kroz 3 ispitivanja, prekid pri 5xx.
- [ ] R9: 409 `game_outdated` → hrvatska rečenica i ponuda nove igre (`errors.test.ts`, `App.test.tsx`); ključ u `hr.ts` (**C1-min** web dio — vidi 1.4).
- [ ] R10: 401 bez problem+json tijela ne briše igru ni ne nudi novu (`apps/web/src/api/errors.ts:43-45`).
- [ ] Uski ekran (390 px) za ta tri ekrana.

### Task 0.4: SKRIPTE — lažni AI, opterećenje čitanja, dim nakon deploya (R7 node, R2, R13)
**Tok:** SKRIPTE · **Rust:** ne (provjera u CI-ju) · **h:** 8 · **Ovisi o:** — (R2 mjerenje tek u valu 4)
- [ ] `scripts/lazni-ai.mjs` (+ `.test.mjs`): mali lažni Anthropic, kašnjenje i `usage` podesivi, 429 na zahtjev.
- [ ] `scripts/opterecenje.mjs`: 40 klijenata prema lokalnom `server serve` s lozinkom i lažnim AI-jem; dohvat igre/situacija/razgovora svakih 2 s
      10 min dok drugi zatvaraju krug; p95 po putanji, broj 5xx i isteka bazena. Prag: p95 `GET` ≤ 300 ms, 0 5xx. Varijanta: 40 naredbi na istu igru.
- [ ] `scripts/smoke-pilot.mjs <URL> <lozinka>` (+ `node --test` uz lažni poslužitelj): health 200 bez brojki; `/` bez lozinke 401 + `WWW-Authenticate`;
      s lozinkom 200 HTML; `http://` → https; `POST /v1/demo` → `Set-Cookie` s `HttpOnly; Secure; SameSite=Lax; Max-Age=10368000`; jedan krug
      **samo ponuđenim odgovorima** (bez AI troška) do obračuna; 4. igra 409 `game_limit`; (nakon vala 2) stvori grupu, uđi kodom, profesor otvori krug.
- [ ] Korak u `deploy-pilot.yml` (smoke nakon deploya) i u CI Docker poslu (postgres servis → `migrate` → `serve` → smoke).

### Task 0.5: VRATA — vrata i ograničenja ulaza, slabe tvrdnje (R11, R12, R17)
**Tok:** VRATA · **Rust:** da (kad se oslobodi slot iz 0.1 ili 0.2) · **h:** 4
- [ ] `api/tests/gate.rs` s `web_dir`: `/`, `/index.html`, `/assets/x.js` 401 bez lozinke, 200 s njom; `HEAD`/`OPTIONS` 401; `/v1/health/`, `/v1/health?x`,
      `/V1/health` pribijeni; `Set-Cookie` se nikad ne šalje uz 401.
- [ ] `api/tests/limits.rs`: poruka 2001 znak → 4xx problem+json; tijelo 3 MB → 413; 7. poruka → 409; pisanje bez kolačića → 401/404.
- [ ] 13× `assert!(… .is_ok())` → usporedba vrijednosti (`game/tests/events.rs:221,235`, `server/tests/config_ai_cap.rs:168-218`).

**Kraj vala 0:** ledger dobiva osnovicu troška (R4) i ispis opterećenja s jednim radnikom (R1) → kratka poruka Leonu s brojkama.

---

## VAL 1 — AI temelj I + L1-15 presuda + izlaz iz zastarjele igre (~28 h)

### Task 1.1: RED — rok na poslu i obračun čeka presude u tijeku (AI-1)
**Tok:** RED · **Rust:** da · **h:** 5 · **Ovisi o:** 0.1 · **Provjeravatelj:** opus
- [ ] Migracija `0007`: `jobs.deadline_at`, `jobs.started_at`, indeks `(status, deadline_at, id)`. Rok po vrsti pri upisu (AI_TEMELJ §6.1):
      lik +10 s, presuda +60 s, obračun odmah, mentor +5 min, sezonski mentor +24 h. Stari redovi: `deadline_at = created_at`.
- [ ] `run_resolve` (`crates/app/src/jobs.rs:200-266`): čeka dok postoji presuda `queued`/`running` sa `started_at + 90 s > sada`, tvrdi strop 10 min od
      zatvaranja; `VERDICT_DEADLINE_SECS` (`:25`) se uklanja ili postaje strop. ADR uz cigle: promijenjeno značenje roka.
**Čuvar pribija:** presuda predana 5 s prije zatvaranja dobiva AI presudu (ne neutralnu) i kad je red pun; AI koji ne radi → neutralna nakon ≤ 10 min.

### Task 1.2: RED — radnici po trakama, raspored po roku, pravednost (AI-2)
**Tok:** RED · **Rust:** da · **h:** 7 · **Ovisi o:** 1.1 · **Provjeravatelj:** opus
- [ ] `claim(lease, kinds)` s `kind = ANY($3)`, `ORDER BY deadline_at, id`, `FOR UPDATE SKIP LOCKED` (`crates/store/src/jobs.rs:21-41`).
- [ ] Trake LIK 6 · PRESUDA 12 · MENTOR 3 (`HOTELIXSIM_WORKERS_LIK|PRESUDA|MENTOR`); prazna traka smije uzeti tuđi posao, liku se mjesto čuva.
- [ ] Pravednost: unutar iste trake i roka ±2 s prednost ima igra s manje poslova `running`.
- [ ] Skinuti `#[ignore]` s brzog dijela `load.rs` (0.1) — **mora proći**; realni ispis u ledger.
**Čuvar pribija:** testovi 0.1; plus: student s 20 poruka ne pomiče p95 lika ostalih za više od 20 %.

### Task 1.3: AI — 429 bez potrošenog pokušaja, pauza po modelu, istek po ulozi (AI-3) + presuda 8000 (L14-1a)
**Tok:** AI · **Rust:** da · **h:** 6
- [ ] `AiError::RateLimited { retry_after }` (`crates/ai/src/response.rs:78-80` danas `Unavailable`); `app/src/jobs.rs:156-158` → `defer` bez pokušaja.
- [ ] `crates/app/src/ai_limit.rs` (novo, omot prije `CappedAi`): semafor po ulozi, zajednička pauza po (pružatelj, model) do `Retry-After` ili 2^n + jitter ≤ 60 s.
- [ ] Istek po ulozi: lik 30 s, presuda 90 s, mentor 120 s (danas 150 s za sve, `crates/ai/src/lib.rs:19`).
- [ ] **L14-1a:** presuda `max_tokens` 16000 → 8000 (`crates/ai/src/request.rs:106`).
**Čuvar pribija:** R3 testovi iz 0.2 zeleni; 429 ne smanjuje `attempts`; tijelo presude ima `max_tokens: 8000`.

### Task 1.4: AI — razredni prefiks za presudu i lika (AI-4)
**Tok:** AI · **Rust:** da · **h:** 5 · **Ovisi o:** 1.3
- [ ] `render_assess` (`request.rs`) dijeli na zajednički blok (situacija iz predloška, rubrika, ponuđeni odgovori — **bez** brojki i imena
      pojedinog hotela) s `cache_control` i osobni blok (razgovor, odgovor studenta). Nova verzija upute `presuda.v3` → `presuda.v3.1` samo ako se mijenja tekst.
- [ ] Lik: `character_body` dobiva `cache_control` na [uputa + opis lika i situacije]; `conversation_stage` (`request.rs:137-139`) seli iza
      točke predmemorije → `prompts/lik.v4.md`.
- [ ] `0.2` (R4) se ponovo vrti: ispis prije/poslije u ledger.
**Čuvar pribija:** dva studenta s istim predloškom imaju bajt-isti zajednički blok; studentov tekst nikad nije u zajedničkom bloku; `quote_student` i dalje radi.

### Task 1.5: WEB — iskočni prozor dok AI ocjenjuje (L14-1b) + izlaz iz zastarjele igre (C1-min)
**Tok:** WEB · **Rust:** ne · **h:** 4–5 · **Ovisi o:** 0.3
- [ ] Iskočni prozor (dijalog, a11y: fokus, Esc, `aria-modal`) dok je presuda `pending` (`Verdict.tsx:27-28`, `hr.ts:269,280`), zatvara se sam kad stigne.
- [ ] 409 `game_outdated` (`App.tsx:108-118`): poruka „Igra je napravljena za stariju verziju" + gumb **„Nova igra"** (ključ iz 0.3).
**Čuvar pribija:** vitest za oba; uski ekran.

---

## VAL 2 — grupa igara + mentor (~38 h)

### Task 2.1: GRUPA — baza, port i naredbe (T1–T4 iz `opseg-pilota-kod-24-10.md`)
**Tok:** GRUPA · **Rust:** da · **h:** 12–14 · **Ovisi o:** 1.2 (posao `class_advance` u trakama) · **Provjeravatelj:** opus
- [ ] Migracija `0006`: `classes` (id, `owner_code_hash`, `join_code` UNIQUE 6 zn., model, `round_kind`, `pace` manual|timed, `interval_secs`,
      `entry` auto|next, `released_round`, `next_open_at`, `seed`, `created_at`); `games.class_id NULL`. Isti pojam kao budući razred (ADR-066 t. 4).
- [ ] Profesorski kod 12 znakova bez 0/O/1/l; u bazi samo sažetak (kao kolačić gosta, `crates/api/src/auth.rs`). Kod za ulaz 6 znakova.
- [ ] `start_demo(..., class: Option<&Class>)`: sjeme grupe umjesto `deps.ids.seed()` (`crates/app/src/demo.rs:90`); igra grupe se **ne** broji u 3 igre,
      jedan gost najviše jedna igra po grupi (ADR-066 t. 7).
- [ ] Naredbe: stvori grupu, uđi kodom, otvori krug (gumb), promijeni tempo (od sljedećeg kruga), pregled grupe.
- [ ] `close_round` (`crates/app/src/commands.rs:129`) uz `class_id`: krug r se smije zatvoriti tek kad `released_round > r`, inače 409 `round_locked`.
- [ ] Posao `class_advance:<grupa>:<k>`: entry=auto zatvara krug k−1 svakoj igri grupe (`Actor::System`, zaseban `decide` po igri); pace=timed se
      ponovo upiše s `run_after = next_open_at`; `job_key UNIQUE` čuva od dvostrukog otvaranja.
**Čuvar pribija:** grupa s 3 gosta kroz 2 kruga u sva četiri spoja (manual/timed × auto/next); dvostruki klik „otvori" = jedno otvaranje;
igra bez grupe ponaša se kao danas (postojeći testovi nepromijenjeni); krivi profesorski kod 404 (ne odaje postoji li grupa).

### Task 2.2: GRUPA — API, OpenAPI (T5)
**Tok:** GRUPA · **Rust:** da · **h:** 4 · **Ovisi o:** 2.1
- [ ] `POST /v1/classes` (vraća profesorski kod **jednom** i kod za ulaz), `POST /v1/classes/join`, `GET /v1/classes/{id}` (profesorski kod u
      zaglavlju ili kolačiću profesora), `POST /v1/classes/{id}/rounds/open`, `PATCH /v1/classes/{id}` (tempo). `GameBody` dobiva `next_open_at` i `locked`.
- [ ] `server openapi`, `check:api`, `types.ts`.
**Čuvar pribija:** e2e `api/tests/e2e_classes.rs` (profesor + 3 gosta, oba načina); problem+json za `round_locked`, krivi kod, zatvoren upis.

### Task 2.3: WEB — profesor i student u grupi (W1–W3)
**Tok:** WEB · **Rust:** ne · **h:** 9–11 · **Ovisi o:** 2.2 (`types.ts`)
- [ ] Početni ekran: „Za profesora" → napravi grupu (model, tjedan/mjesec, tempo, ulazak) → ekran s **profesorskim kodom** (veliko, „spremi ga,
      prikazuje se samo jednom") i **kodom za ulaz** za studente. „Imam profesorski kod" → upis koda → pregled grupe.
- [ ] Pregled grupe: studenti (nadimak, krug, status), gumb **„Otvori sljedeći krug"**, postavka tempa, tko kasni.
- [ ] Student: „Imam kod grupe" na početnom ekranu; u grupi stanje „čeka profesora", **odbrojavanje** do otvaranja (pace=timed) i upozorenje minutu
      prije; zadnji spremljeni plan vrijedi (ADR-066 t. 9); u feedu „zatvorio profesor / isteklo vrijeme" (`by: Actor`).
- [ ] Vitest za sve, uski ekran; `poll.ts` ispituje dok je krug zaključan.

### Task 2.4: MENTOR — „niže plaće" i bez obećanja povrata ulaganja (L1-15 V1, V3; ADR-065 t. 3)
**Tok:** MENTOR · **Rust:** da · **h:** 5–7 (+ dijagnoza 1 h) · **Ovisi o:** —
- [ ] V1: mentor ne piše „nižih plaća" kad profil počinje ispod 100 % (`crates/game/src/mentor.rs:390`, `:1160`) — redak morala plaće ≠ 0.
- [ ] V3: kod pada Financija mentor navodi stvarne uzroke (amortizacija obnove, dobavljač, moral) i **nikad ne obećava** da se ulaganje vraća
      (`why_fell` `mentor.rs:1136-1168`, `finance_note` `mentor_v2.rs:265-305`); upute mentora nova verzija (`mentor.v5`, `mentor-sezona.v3`).
- [ ] Pregled teksta: zabranjene fraze („vratit će se", „sljedeći mjesec se vraća") kao test nad svim izlazima pravila mentora.
**Čuvar pribija:** scenariji iz `l114-popis-izmjena.md` (Leonova igra 23.10.); bez promjene brojki igre (balans `cmp` bajt-isti).

---

## VAL 3 — AI temelj II + generator događaja (~26 h)

### Task 3.1: AI — mjerilo puteva suca (AI-5; živi, < 1 $, **uz Leonov OK**)
**Tok:** AI · **Rust:** da · **h:** 3 · **Ovisi o:** 1.4
- [ ] 40 odgovora mjerila (`content/hotel/mjerilo/`) kroz Sonnet-high (današnje), Sonnet-medium, Haiku-low i dvokorak → tablica slaganja po
      kriteriju i ocjeni u ledger (`ai/tests/live.rs`, `ai/src/benchmark.rs`). Prag: ≥ 90 % po kriteriju, nikad razlika ocjene > 1.
- [ ] **Mentor kruga na Haikuu:** 6 krugova iz sezone kroz Sonnet-low (današnje) i Haiku-low → parovi tekstova bez oznake modela u ledger
      za **Leonov slijepi pregled** (pitanje 8). Mentor kruga je najveća stavka troška (TROSKOVI_I_AI §3): bez ovoga cilj ≤ 0,25 $ po igri ne prolazi.

### Task 3.2: AI — sudac u dva koraka (AI-6; **samo ako 3.1 prođe**, inače Sonnet `effort: medium`)
**Tok:** AI · **Rust:** da · **h:** 5 · **Ovisi o:** 3.1
- [ ] `crates/app/src/judge.rs`: Haiku `effort: low` → Sonnet samo za granične (ocjena na pragu, odgovor dulji od N, odstupanje od najbližeg
      ponuđenog); razina u `AiMeta.prompt` (npr. `presuda.v3.1+trijaza`), bez promjene događaja; pritisak u redu (1.2) gasi drugi korak.
**Čuvar pribija:** granični slučajevi idu Sonnetu; jasni ne; R4 ispis: udio Sonneta ≤ 20 % u „realno".

### Task 3.3: AI — kanta po grupi i trošak po igri (AI-7) + vidljivost (AI-8)
**Tok:** AI · **Rust:** da · **h:** 8 · **Ovisi o:** 2.1 (grupa), 3.2
- [ ] Migracija `0008`: `ai_spend.game_id`, `role`, `path` (+ R5 test: zbroj po igri = zbroj redova).
- [ ] Kanta (AI_TEMELJ §6.6): prazna kanta gasi samo drugi korak suca i skraćuje lika; **nikad** pravilo umjesto presude.
- [ ] `GET /v1/health/ai` (dubina reda po traci, najstariji, 429 i pauza, neutralne, rezerve) — **bez** brojki koje odaju igre; profesor u pregledu
      grupe vidi potrošnju grupe i „tko je sudio" (web dio u 3.5).

### Task 3.4: SADRŽAJ — generator događaja i celijakija (ADR-065 t. 4 i 6)
**Tok:** SADRŽAJ · **Rust:** da · **h:** 6–7 · **Ovisi o:** 2.1 (sjeme grupe) · **Brojke:** balans `--check` 1000 tjedno i mjesečno
- [ ] `GameSettings.anchors: bool` (`crates/game/src/command.rs:32-37`), isključeno za nove v2 igre → bez fiksnih sidara S1–S4 (`model.rs:759,785`);
      stare igre i `v2_sealed.rs` nepromijenjeni, novi slučaj u testu; `CORE_VERSION` ostaje 2.6.0.
- [ ] Test: 50 sjemena → prvi krug nema uvijek isti predložak (barem 5 različitih prvih situacija).
- [ ] `balance/src/v2_situations.rs:125-152` bez sidara; brane po potrebi (svaka promjena brane → anketa Leonu s brojkama prije spajanja).
- [ ] Celijakija: jedna rečenica objašnjenja u opisu situacije (`content/hotel/v2/situation_templates.json:1797-1806`); provjera ostalih stručnih pojmova istim pregledom.
- [ ] Ako Leon na pitanje 1 kaže (b): dodatni task „isti popis za grupu" (+4–6 h, `CORE_VERSION` 2.7.0) prije zamrzavanja.

### Task 3.5: WEB — „tko je sudio" i potrošnja u pregledu grupe
**Tok:** WEB · **Rust:** ne · **h:** 2–3 · **Ovisi o:** 3.3

---

## VAL 4 — dokaz i isporuka (~12 h + Leon)

### Task 4.1: Mjerenja i zapisi
- [ ] R2 `opterecenje.mjs` s lažnim AI-jem lokalno (40 klijenata) → ledger; realni `load.rs` ispis; R4 konačni ispis.
- [ ] `TROSKOVI_I_AI.md` §3–4: procjene zamijeniti izmjerenim (R4), uz datum i commit.
- [ ] R16 jednokratno `cargo mutants` na `app/src/jobs.rs`, `app/src/spend.rs`, `app/src/classes.rs`, `api/src/gate.rs`, `api/src/auth.rs` → preživjeli mutanti u testove.
- [ ] (ako Leon kaže da, pitanje 3) R6 Playwright: profesor napravi grupu → 2 studenta uđu kodom → krug → profesor otvori → obračun → mentor; mobilni 390×844.

### Task 4.2: Leon igra i spajanje
- [ ] Lokalno na jednoj adresi: Leon napravi grupu kao profesor, u drugom (privatnom) prozoru uđe kao student; provjera L1-15 (presuda s
      prozorom, mentor, celijakija) i novi početak igre. Nalazi → popravci u istom valu.
- [ ] Završni pregled najjačim modelom (cijeli diff od 9b6f047), kopija grane u `Documents/hotelixsim-kopije/<datum>/`.
- [ ] **Leonov OK** → `model-v2/sloj1` → `main`, push, CI zelen (gledati sam preko `gh`).

### Task 4.3: Pilot dio 9 i 10 (iz arhivirane kartice Pilot)
- [ ] Leon (~1–1,5 h): Railway Hobby + DPA, Anthropic radni prostor (strop 40 $), lozinka ≥ 16 znakova, GitHub okolina `pilot` + `RAILWAY_TOKEN`.
- [ ] Sesija (~2,5–4 h, **uz Leonov OK za deploy**): projekt u Amsterdamu, varijable (nova lozinka baze, stropovi, broj igara, broj radnika po traci),
      prvi deploy, `smoke-pilot.mjs` zelen, UptimeRobot, prvi `pg_dump` i proba vraćanja; probe 401, http→https, kolačić, SIGTERM, strop 0,01 → rezerva, 4. igra 409, mobitel.
- [ ] Živi nalet 40 presuda (uz OK, pitanje 2) → zapisnik.
- [ ] Dio 10: ADR pilota (hosting, stropovi, radnici, grupa), CHANGELOG, dopuna ADR-004.
- [ ] Dio 8 (tekst privatnosti, ADR-065 t. 5): kad profesori daju voditelja obrade, kontakt i rok čuvanja → prije puštanja studenata.

---

## POSLIJE PILOTA (redoslijed, ~90–120 h [PROC])

| redom | što | h | izvor |
|---|---|---|---|
| 1 | **Lokal kao mehanika** (zakupci, cijena po stanju, preuređenje; anketa 6 brojki) — `CORE_VERSION` 2.7.0, balans | 12–16 | ADR-065 t. 2, `generator-i-lokal-24-10.md` §B |
| 2 | **C6** razdvojiti `api/src/schema.rs` (1717 r.) i `game/src/mentor.rs` (1399 r.) | 8–11 | faza C |
| 3 | **C1** puni: stara igra se čita nakon promjene jezgre, „pravila nadograđena od kruga k" | 12–20 | faza C, REVIZIJA #6 |
| 4 | **C2 ostatak:** `LISTEN/NOTIFY`, globalni AI strop u bazi, više instanci, predmemorija igre (C4), ETag/long-poll | 20–30 | `temelj-opterecenja-24-10.md` |
| 5 | **AI-9** skupna obrada (sezonski mentor, kontrolor) · **AI-10** kontrolor u sjeni + otisak | 14 | AI_TEMELJ §6.4–6.8 |
| 6 | **RAZRED U JEZGRI** (ADR-037: jedna igra po razredu, obračun po hotelu, preotimanje) — grupa (`classes`) postaje razred | 45+ | ROADMAP |
| 7 | **C3** `presuda.v4` + mjerilo s Leonovim ocjenama · **C5** pravila iz weba u API | 16–26 | faza C |
| — | R15 brisanje i zadržavanje podataka (uz G-3), Bedrock EU ako fakultet traži obradu u EU, AAI@EduHr | — | TROSKOVI_I_AI §GDPR |

## Prijelaz s plana popravka temelja

Nakon Leonova OK-a: `docs/plan/2026-10-14-popravak-temelja.md` → `docs/archive/` (kartica Pilot dijelovi 1–7 gotovi, 8–10 su ovdje
Task 4.3; faza C je ovdje „Poslije pilota"), poveznice prepisane, `docs/README.md` i ROADMAP upućuju na ovaj plan; novi ledger
`.superpowers/sdd/2026-10-24-pilot-s-profesorom/progress.md` počinje s ovim planom i nalazima sesije 14.
