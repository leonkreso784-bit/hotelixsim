# Dnevnik, runde i red poslova — plan izgradnje

**Status:** ✅ ISPUNJEN 2026-10-05 · **Otvoren:** 2026-10-05 · **Grana:** `gradnja/dnevnik-runde` · **Rok demoa:** ~14.10. ([ADR-012](../records/DECISIONS.md#adr-012))

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Igra (igrač + tri robota) živi kao dnevnik događaja u Postgresu: gost preko poveznice dobije hotel, mijenja
plan mjeseca i bira ponuđene odgovore dok je runda otvorena, „Zatvori mjesec" upiše zatvaranje i posao u red, a radnik
obračuna sve hotele jezgrom — za lipanj i srpanj, s ljestvicom i kronološkim feedom hotela preko API-ja (bez ekrana).

**Architecture:** Novi čisti crate `game` (iznad jezgre, bez baze i mreže) drži događaje, serde DTO-e, projekciju
(fold), pravila runde, poteze robota i obračun runde iz **snimka pravila** zapisanog u `GameCreated`. `app` dobiva
portove (`EventLog`, `Guests`, `JobQueue`, `Ids`) i slučajeve upotrebe s pravom pristupa po ulozi; svaka naredba je
jedna transakcija „zaključaj igru → učitaj tok → odluči (čista funkcija) → upiši". `store` izvodi portove u Postgresu
(okidač nepromjenjivosti, tablice `guests/games/hotels/jobs`, `FOR UPDATE SKIP LOCKED` sa zakupom), `api` samo zove
`app`, a `server` učitava sadržaj pri startu i vrti radnika kao tokio zadatak.

**Tech Stack:** Rust 1.98.1 (edition 2024) · Axum 0.8 + utoipa 6 · sqlx 0.9 (Postgres, `uuid`, `json`, offline `.sqlx/`) ·
`uuid` 1 · `sha2` 0.10 · serde/serde_json · Postgres 16 (lokalno 5433) · Node 24 skripte brana · openapi-typescript klijent.

**Spec:** [DEMO.md](../product/DEMO.md) §1 (demo način, gost, roboti, 15 minuta), §6.1, §7 (raspored), §10 ·
[ARHITEKTURA.md](../architecture/ARHITEKTURA.md) §6.2, §7.2, §8.1–8.4, §9.1–9.4, §11, §12 · pretpostavke
[ADR-015](../records/DECISIONS.md#adr-015) · stavke `[dnevnik]` u [BACKLOG.md](../records/BACKLOG.md) ·
oblik plana po uzoru na [plan jezgre](../archive/2026-10-04-jezgra.md) · tok rada [AGENTI.md](../workflow/AGENTI.md).

**Leonove proizvodne odluke (05.10.)** — vrijede kao spec: (1) gost preko poveznice sad samo kao pozadina
(`POST /v1/demo`, kolačić `httpOnly`, nova igra s tri robota); limit 3 igre po IP-u i brisanje nakon 7 dana idu u
drugi val. (2) „Zatvori mjesec" s neodgovorenom situacijom je dopušten; neodgovorena dobiva ocjenu 1, a API kaže koliko
ih je neodgovoreno. (3) Mjesec smije zatvoriti svatko tko je u igri. (4) „Kao BitLife": „Zatvori mjesec" je „Age +";
plan se mijenja bilo kad dok je runda otvorena (svaka promjena je događaj, zadnja vrijedi, nedirnuto = zadano iz
DEMO §6.1); API daje kronološki feed hotela; svaka promjena nosi uzrok. Opseg: samo lipanj (S1, S3) i srpanj (S2);
samo ponuđeni odgovori (AI, slobodan tekst, likovi i mentor su korak 4 — **ne izmišljaj** `VerdictRecorded`).

## Global Constraints

- **Jezgra** (`engine`, `sim-hotel`) i dalje ima **nula** vanjskih ovisnosti i sve brane ADR-009; jedine promjene jezgre
  su u Tasku 1 (polje `unanswered`, `SituationOutcome::unanswered`, `CORE_VERSION`). Jezgra nema serde.
- **`game`** smije ovisiti samo o `engine`, `sim-hotel`, `content`, `serde`, `serde_json`, `uuid` (bez značajke `v4`).
  Nikad `tokio`, `sqlx*`, `axum`, `hyper`, `reqwest`, `mio`, `socket2` — čuva `scripts/check-core-deps.mjs` (Task 2).
  Bez `f32`/`f64`, bez sata, bez `HashMap`; slučajnost samo `engine::Rng::derive(seed, …)`.
- **Obračun koristi snimak iz `GameCreated`, nikad trenutne datoteke** (`content::Snapshot`); otisak pravila
  `ruleset_id = "sha256:<hex>"` kanonskog JSON-a snimka.
- Događaji: `GameCreated`, `PlanSet`, `AnswerChosen`, `RoundClosed`, `RoundResolved`; `schema_ver = 1`; stupac `type`
  nosi ime, `payload` samo tijelo. Tok: `game:<uuid>`. `actor`: `guest:<uuid>` · `robot:<kod strategije>` · `system`.
- Jedinice kao u jezgri: novac `i64` `_cents` · bodovi `_pts` (stotinke, 0..=10 000) · `_bp` · indeksi `u16` stotinke ·
  napredak `i32` stotinke boda (`progress_hundredths`, 10 000 = +100).
- **Vidljivost (ARHITEKTURA §12):** tuđi hotel = samo ukupna ocjena i pet indeksa; vlasnik vidi stvarne brojke, plan,
  situacije i feed. Kvaliteta i mehanike ponuđenih odgovora **nikad** ne izlaze kroz API.
- Greške API-ja: `application/problem+json`, `code` je **enum** u OpenAPI-ju (`ProblemCode`); nikad prazan odgovor.
- SQL: novi `sqlx::query!` → `cargo sqlx prepare --workspace` i commit `.sqlx/`; u **testovima** samo funkcije
  `sqlx::query(...)`/`query_scalar(...)` (ne makroi), da offline cache ne ovisi o testovima. Testovi baze: `#[sqlx::test]`
  (baza mora raditi: `bash scripts/baza.sh start`).
- Hrvatski u komentarima, porukama i naslovima grešaka; nazivi u kodu engleski.
- Commit poruke na hrvatskom, `git commit -F <datoteka u scratchpadu>`, izričite putanje, oblik „Dnevnik N/12: …";
  **push samo uz Leonov OK**.
- Provjere prije commita svakog taska (svaka zasebno, s `; echo EXIT=$?`, bez `| tail`):
  `cargo fmt --all -- --check` · `cargo clippy --workspace --all-targets -- -D warnings` · `node scripts/check-core-deps.mjs` ·
  `cargo test --workspace` · `cargo run -q -p balance --release -- --seeds 100 --check`. Taskovi koji mijenjaju API
  dodaju `corepack pnpm gen:api`, `corepack pnpm typecheck` i `corepack pnpm test`.

## Review Focus

1. **Dvostruki klik ili dvije kartice na „Zatvori mjesec"**: točno jedan `RoundClosed`, jedan posao i jedan
   `RoundResolved`; drugi klik vraća trenutno stanje (Task 6 `double_close_is_idempotent`, Task 7
   `decide_enqueues_job_once_per_key`, Task 10 `close_twice_is_202_then_200`).
2. **Zastario ekran**: plan ili odgovor za mjesec koji je upravo zatvoren mora dati 409 `round_not_open`, nikad tihi upis
   u srpanj (Task 3 `stale_commands_are_rejected`, Task 6 `plan_for_closed_month_rejected`).
3. **Radnik padne usred posla ili isti posao stigne dvaput**: istekao zakup vraća posao u red, a ponovni obračun ne
   upisuje drugi `RoundResolved` (Task 4 `resolve_job_twice_is_noop`, Task 6 `crashed_worker_job_is_reclaimed_and_duplicate_job_is_noop`, Task 8
   `expired_lease_is_reclaimed`).
4. **Tuđi hotel, tuđa igra, bez kolačića ili s pokvarenim kolačićem**: nikad privatne brojke; 401/403 kao problem+json
   (Task 5 `visibility_by_role`, Task 9 `foreign_and_anonymous_requests`).
5. **Sadržaj se promijeni nakon stvaranja igre ili JSONB promijeni redoslijed ključeva**: igra računa po svom snimku,
   ponovni obračun je bajt-isti nakon tipiziranog čitanja (Task 1 `fingerprint_ignores_key_order`, Task 4
   `resolution_uses_snapshot_not_files`, Task 12 `replay_from_database_matches`).

Dodatno (ne-5, ali pokriveno): neispravno JSON tijelo ili nepoznata vrijednost enuma u planu → 422 `invalid_body`
(Task 10 `bad_plan_body_is_422_problem`).

---

## Pregled datoteka

```text
Cargo.toml                                   + member game; deps uuid, sha2; sqlx +uuid +json; tokio +time
crates/sim-hotel/src/mechanic.rs             SituationTemplate.unanswered, SituationOutcome::unanswered
crates/sim-hotel/src/lib.rs                  CORE_VERSION
crates/content/src/lib.rs                    DTO-i Serialize; Snapshot (read/build/fingerprint); polje unanswered
content/hotel/situations.json                + "unanswered" za S1–S4 (nacrt, Leonova provjera)
crates/game/                                 NOVO  ids, actor, dto, event, game (fold), command, resolve, feed, view
scripts/check-core-deps.mjs                  + pravilo čistoće za `game`
crates/app/src/{lib,ports,error,demo,views,commands,jobs,memory}.rs   portovi, slučajevi upotrebe, lažna pohrana
crates/store/migrations/0002_events_append_only.sql   NOVO  indeks, okidač, CHECK na actor
crates/store/migrations/0003_platform.sql    NOVO  guests, games, hotels, jobs
crates/store/src/{lib,log,guests,jobs}.rs    izvedbe portova
crates/api/src/{lib,problem,auth,schema,health,demo,hotels,games}.rs  rute, problem kodovi, kolačić
crates/server/src/main.rs                    Deps, sadržaj pri startu, radnik
scripts/check-clean.mjs                      NOVO  check:api pada i na nepraćenu generiranu datoteku
openapi.json, packages/api-client/src/schema.d.ts   GENERIRANO
.sqlx/                                       GENERIRANO
```

**Paralelizacija:** taskovi su redom ovisni o sučeljima. Jedini sigurni par za paralelne izvođače je **Task 6 ‖ Task 7**
(nakon Taska 5; Task 6 piše samo `crates/app/src/{commands,jobs}.rs` i `crates/app/tests/rounds.rs`, Task 7 samo
`crates/store/**` i `.sqlx/`). Sve ostalo ide redom.

---

## Taskovi

| N | Task |
|---|---|
| 1 | [Task 1: Snimak pravila u `content` i ishod neodgovorene situacije](2026-10-05-dnevnik-runde/task-01-snimak-pravila-u-content-i-ishod.md) |
| 2 | [Task 2: Crate `game` — identifikatori, actor, događaji i DTO-i](2026-10-05-dnevnik-runde/task-02-crate-game-identifikatori-actor-dogadaji-i.md) |
| 3 | [Task 3: `game` — projekcija (fold), stanje runde, nova igra, potezi robota i naredbe](2026-10-05-dnevnik-runde/task-03-game-projekcija-fold-stanje-runde-nova.md) |
| 4 | [Task 4: `game` — obračun runde, ponovljivost, feed i pogledi](2026-10-05-dnevnik-runde/task-04-game-obracun-runde-ponovljivost-feed-i.md) |
| 5 | [Task 5: `app` — portovi, greške, lažna pohrana, gost i pogledi po ulozi](2026-10-05-dnevnik-runde/task-05-app-portovi-greske-lazna-pohrana-gost.md) |
| 6 | [Task 6: `app` — naredbe runde, zatvaranje i radnik obračuna](2026-10-05-dnevnik-runde/task-06-app-naredbe-runde-zatvaranje-i-radnik.md) |
| 7 | [Task 7: `store` — migracije, dnevnik i gosti u Postgresu](2026-10-05-dnevnik-runde/task-07-store-migracije-dnevnik-i-gosti-u.md) |
| 8 | [Task 8: `store` — red poslova (`FOR UPDATE SKIP LOCKED`, zakup, pokušaji)](2026-10-05-dnevnik-runde/task-08-store-red-poslova-for-update-skip.md) |
| 9 | [Task 9: `api` — temelj (Deps, problem kodovi, kolačić gosta, zamjenska ruta) i rute gosta i hotela](2026-10-05-dnevnik-runde/task-09-api-temelj-deps-problem-kodovi-kolacic.md) |
| 10 | [Task 10: `api` — rute igre: plan, situacije, runda, zatvaranje i feed](2026-10-05-dnevnik-runde/task-10-api-rute-igre-plan-situacije-runda.md) |
| 11 | [Task 11: `server` — radnik u procesu, brana nepraćenih generiranih datoteka, upute](2026-10-05-dnevnik-runde/task-11-server-radnik-u-procesu-brana-nepracenih.md) |
| 12 | [Task 12: Kraj do kraja na pravoj bazi, regeneracija i sve provjere CI-ja](2026-10-05-dnevnik-runde/task-12-kraj-do-kraja-na-pravoj-bazi.md) |

## Završni pregled (opus)

Nakon Taska 12 orkestrator pokreće **završni pregled cijele grane** `gradnja/dnevnik-runde` na najjačem dostupnom modelu
(opus), po [AGENTI.md](../workflow/AGENTI.md) §1 t. 5. Pregled posebno gleda: (1) Review Focus gore, točku po točku;
(2) svaku naredbu u `app` — je li cijela provjera unutar `decide`; (3) vidljivost — postoji li ijedan put do tuđih
stvarnih brojki, kvalitete ili mehanika; (4) jesu li događaji samodostatni za ponovni obračun bez datoteka;
(5) jezgra i `game` bez novih ovisnosti (`check-core-deps`). Nalazi koji se ne popravljaju idu u
[BACKLOG](../records/BACKLOG.md); zatim `/kraj` (ledger u `docs/archive/`, CHANGELOG, plan u arhivu s pečatom).
**Push i spajanje u `main` samo uz Leonov izričit OK.**

## Što ovaj plan namjerno ne radi

AI presuda, slobodan tekst, likovi i mentor (korak 4; nema `VerdictRecorded`) · ekrani (korak 5) · profesorov pogled i
`?uloga=profesor` (korak 6) · kolovoz i rujan, splet okolnosti i tragovi (drugi val) · limit po IP-u i brisanje gostiju
(drugi val) · projekcijske tablice (fold pri čitanju je dovoljan za 4 hotela × 2 mjeseca).
