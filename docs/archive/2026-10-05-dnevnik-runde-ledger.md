# SDD ledger — plan: 2026-10-05-dnevnik-runde.md

> **✅ ISPUNJEN 2026-10-05** — radni ledger plana [2026-10-05-dnevnik-runde.md](2026-10-05-dnevnik-runde.md); kopija iz `.superpowers/sdd/` (gitignorirano). Odgođene sitnice su prenesene u [BACKLOG](../records/BACKLOG.md).


Spec: nema zasebnog speca; autoritet = ARHITEKTURA.md, DEMO.md, ADR-015 (🟨) + Leonove odluke 05.10. (gost pozadina sad; zatvaranje uz upozorenje, neodgovoreno=1; zatvara svatko u igri; različiti profili; BitLife osjećaj). Rulings provisional.
Grana: gradnja/dnevnik-runde, start 7db601d

## Pre-flight sken

Sken 05.10. (samo čitanje plana i koda na 7db601d). Tri nalaza provjerena pokusom u scratchpadu (rustc/clippy 1.98):
E0659 dvosmislenost `game`, `clippy::large_enum_variant`, nekorišten `use` traita za poziv na `dyn`.

### 1. Parovi taskova (dijeljena datoteka ili sučelje)

| # | Taskovi | Proizvodi → troši | Nalaz |
| --- | --- | --- | --- |
| 1 | T1→T2 | `content::Snapshot` (Clone/Debug/Eq/Serialize/Deserialize), `fingerprint()`, `sim_hotel::CORE_VERSION` → `GameCreated.snapshot/ruleset_id/core_version`, test `events.rs` | OK |
| 2 | T1→T3 | `Snapshot::build() -> Result<Content, ContentError>`, `Content: PartialEq` → `Game::created`, `fold_rejects_broken_logs` (`economy.hire_cents` postoji) | OK |
| 3 | T1→T4 | `SituationOutcome::unanswered(&SituationTemplate)`, `SituationTemplate.unanswered` → `resolve_round`; S1 neodgovoren = ServiceRecovery q1 → −400 | OK |
| 4 | T2→T3 (`crates/game/src/lib.rs`, `tests/events.rs`) | T3 dodaje `pub mod game;` → `use game::*;` u `tests/events.rs` (T2) uvozi modul `game` uz `use game::dto::*` | **NESKLAD-1** E0659 „`game` is ambiguous" (glob vs vanjski crate) — events.rs se prestaje prevoditi od T3 |
| 5 | T2→T3 | `GameEvent`, `Recorded`, `Actor::{as_db,parse}`, `strategy_from_code`, `Controller`, `HotelSetup`, `corrupt`, `From<&MonthPlan> for PlanDto`, `From<Month> for MonthCode`, `TryFrom<&HotelStateDto>` | OK |
| 6 | T2→T4 | `MonthReportDto::from(&MonthReport)`, `indices_dto`, `HotelResolution`, `SituationResult`, `LineDto`, `CauseDto::Situation` | OK |
| 7 | T3→T4 (`tests/round.rs` → `tests/common/mod.rs`) | pomoćnici `hotel_ids/GUEST/created/push` premješteni | **NESKLAD-2** u round.rs ostaju nekorišteni `use content::{Snapshot, repo_dir}` i `use uuid::Uuid` → `-D warnings`; i round.rs/resolve.rs/common imaju `use game::*` + `use game::…` (NESKLAD-1) |
| 8 | T3→T4 | `Game{hotels,history,content,last_seq}`, `state_at`, `effective_plan`, `situations_in`, `refs`, `robot_moves`, `latest_state`, `plan_limits`, `apply` | OK |
| 9 | T2/T3/T4→T5 | `new_game(GameId,u64,&Snapshot,GuestId,[HotelId;4])` (poziv s `&Arc<Snapshot>` → deref OK), `SCHEMA_VER`, `GameEvent::{kind,to_payload,from_record}`, `public_hotel/private_hotel/feed/unanswered_count/status/current_month` | OK |
| 10 | T4→T5/T6 | `PrivateHotel.plan: Option<PlanState{plan: PlanDto}>` → testovi `my_hotel(..).private.unwrap().plan.unwrap().plan` → `set_plan(.., PlanDto)` | OK |
| 11 | T5→T6 (`crates/app/src/lib.rs`) | `fold_raw/new_event/load_game` (pub(crate) kroz `pub use demo::*`), `member_hotel/require_owner/round_view`, `Decision{events,enqueue,finish_job}`, `NewJob`, `MAX_ATTEMPTS`, `Memory::{jobs,events,expire_leases,push_job}`, `JobStatus::{Done,Failed}` | OK (sitno: `commands.rs` treba i `use crate::situations` za `answer`) |
| 12 | T5↔T6 | `game_of_hotel` privatna async fn u `views.rs` i doslovno ista u `commands.rs` | **NESKLAD-3** (pregled: dupliciran blok) |
| 13 | T5→T7 | `EventLog::{create_game,load,decide}`, `Guests::{create_guest,guest_by_token_hash,game_of_hotel,latest_hotel_of}`, `RawEvent{seq i64,kind,schema_ver i32,payload,actor}` ↔ `events.stream_seq INTEGER` (`i64::from`), `schema_ver INTEGER`, `NewHotel.slot i16` ↔ `SMALLINT`, `Decided{last_seq,appended: usize}`, `DecideError::{Rejected,Store}` | OK |
| 14 | T5→T8 | `JobQueue::{claim(i32), fail(i64,String)}`, `Job{id i64,key,kind,game_id,payload,attempts i32}` ↔ `jobs.id BIGSERIAL, attempts INTEGER, job_key` | OK |
| 15 | T7→T8 (`crates/store/src/lib.rs`, `tests/`) | tablica `jobs` (status `queued/running/done/failed`, `locked_until`, `last_error`), `log::store_err` pub(crate), `decide` + `finish_job` → `'done'` | OK; **NESKLAD-4** „kopiraj uvoze iz tests/log.rs" donosi nekorišten `DecideError` u `tests/jobs.rs` |
| 16 | T6‖T7 (paralelno) | datoteke disjunktne; T6 Files ipak navodi `crates/app/src/lib.rs` (tekst „Paralelizacija" kaže da ne) | **NESKLAD-5** (rizik): `cargo test -p store` i `cargo sqlx prepare --workspace` prevode `app` — napola napisan T6 ruši T7 ako dijele radno stablo |
| 17 | T6 memory ↔ T7/T8 store | pravila: ključ posla jedinstven, `claim` samo `attempts < MAX`, `fail` → `failed` na MAX, `finish_job` → done | OK (Memory ne obara istekao zadnji pokušaj — test to ne traži) |
| 18 | T5→T9 | `app::{Deps,caller,start_demo,my_hotel,hotel_view,profile_indices,AppError,ErrorCode(20),ErrorKind,HotelView,memory::deps}` → `ProblemCode` 20+2=22, `From<AppError> for Problem`; token `uuid.simple()` = 32 hex ↔ `Token` filtar 32 hex | OK |
| 19 | T9 (`crates/api/Cargo.toml`) | `schema.rs` koristi `game::view`/`game::dto` (Interfaces: „api dodaje `game.workspace = true`") | **NESKLAD-6** Files/Step 1 T9 ne navode `game` u `[dependencies]` |
| 20 | T7/T8→T9 (`crates/server/src/main.rs`) | `Store: SystemStatus + EventLog + Guests + JobQueue` → `deps(store, snapshot)` | OK |
| 21 | T6→T10 | `set_plan(&Deps,&Caller,HotelId,MonthCode,PlanDto)->PlanDto`, `answer(..,&str,u8)->SituationsView`, `round_status`, `close_round->CloseOutcome{accepted,round}`, `RoundView{month,status,unanswered}` → `RoundBody`, `feed->Vec<FeedItem>` → `FeedItemBody` | OK |
| 22 | T9→T10 (`schema.rs`, `hotels.rs`, `lib.rs`, `tests/http.rs`) | `parse_month→game::dto::MonthCode`, `Problem::new`, `ProblemCode::InvalidBody`, `PlanBody↔PlanDto`, `parts()` | OK |
| 23 | T9/T10→T11 | `deps()`, `content_snapshot()`, `app::run_next_job`; `check:api` nakon commita T10; BACKLOG različite stavke | OK |
| 24 | T9→T12 (`crates/api/Cargo.toml`) | T9 MIČE `sim-hotel` iz api; e2e koristi `sim_hotel::Month::Jun` | **NESKLAD-7** T12 Files dodaje samo `sqlx`, `store` (dev); `sim-hotel` je u bilješci „ako ih test ne vidi" |
| 25 | T9→T12 | `tests/common/mod.rs::{call, assert_problem}` (`#![allow(dead_code)]`); fn `app()` je u prostoru vrijednosti pa ne smeta putanji `app::…` (provjereno) | OK |
| 26 | T7/T8→T12 | `Store::from_pool`, `migrations = "../store/migrations"`, `jobs.status <> 'done'`, `events.type` | OK |
| 27 | T5→T12 | `d.log.load(game)` na `Arc<dyn EventLog>` + `use app::EventLog;` | **NESKLAD-8** uvoz je nekorišten (metoda `dyn` traita ne treba trait u dosegu) → `-D warnings` |
| 28 | T2/T3/T4→T12 | `game::Recorded`, `Actor::parse`, `GameEvent::from_record`, `game::game::Game::fold`, `resolve_round` (bez `use game::*` → bez E0659) | OK |
| 29 | T1→T9/T11 (server) | `Snapshot::read/build/fingerprint`, `content::repo_dir()` | OK |
| 30 | T2→T12 (`check-core-deps.mjs`) | pravilo `PURE.game` + funkcija `tree(pkg)` | OK |

### 2. Svaki task sam sa sobom

| Task | Nalaz |
| --- | --- |
| 1 | OK. `robots.rs:133` jedini literal `SituationTemplate` uz `content` (grep); `test_catalog` ima `recovery_pts_per_q: 200` → −400; `effects` pušta q=1; balans ne gradi predloške. Slabo: `snapshot_builds_same_content_as_load` je tautologija jer `load` = `Snapshot::read()?.build()` (vidi §3). |
| 2 | **NESKLAD-9** `GameEvent::GameCreated(GameCreated)` nosi cijeli `content::Snapshot` (~300+ B) naspram `PlanSet` (~50 B) → `clippy::large_enum_variant` (perf ⊂ all = deny) pada; provjereno pokusom. Ostalo OK: `Strategy::ALL/code`, `Category::code/from_code`, `Index::ALL`, `IndexValue::hundredths`, sva polja `MarketOutcome/PeopleOutcome/MoneyOutcome/GuestOutcome/Line/Var/Cause/Unit` postoje točno tako; testovi koriste samo normalne ovisnosti. |
| 3 | NESKLAD-1 (lib.rs `pub mod game`). Potpisi jezgre OK: `Strategy::plan(self,&HotelState,Month,&Catalog,&mut Rng)->Result<MonthPlan,ResolveError>`, `choose(self,&[OfferedAnswer],&Catalog,Category,&mut Rng)->Result<usize,_>`, `RefTable::compute`, `start_indices`, `MonthPlan::{default_for→CatalogError, validate→PlanError}`, `Month::position()->usize`, `Rng::derive/range`, `calc::{div_floor,add,mul,narrow}`. `plan_limits` = točno `validate`. Neiskorišteni `MonthCode, PlanDto` u command.rs plan sam rješava. |
| 4 | NESKLAD-1 i NESKLAD-2. `month_indices(&MonthReport,&Catalog,&RefTable)`, `progress_pts(&IndexSet,&IndexSet)->Calc<i32>`, `SituationOutcome::from_answer(&str,&OfferedAnswer)` OK. `fixed_cents` ulazi u `costs`→`gop` (resolve.rs `money`) pa `assert_ne!(gop)` vrijedi. S1 odgovor 0 = kvaliteta 5 (situations.json) OK. Test `game::view::Seat` u izrazu se uz `use game::*` razrješava u `game::game::view` → E0659/E0433. |
| 5 | OK. app Cargo dobiva `game, content, sim-hotel, serde, serde_json, sha2, thiserror, uuid(v4)`; `BoxFut` i `SystemStatus` postoje u `app/src/lib.rs`; `Profile::starting_indices()` C = [5000;5]; `ErrorCode::ALL` 20 = popis. NESKLAD-3 (dupli `game_of_hotel`, nastaje u T6). |
| 6 | OK uz sitnicu: `commands.rs` uvozi ne navode `situations`. `Month::code()` postoji (`resolve_key`). Memory-semantika daje točno testove `double_close`, `crashed_worker…`, `failing_job…` (5 pokušaja → Failed). |
| 7 | OK. `0001_events.sql`: `stream_seq/schema_ver INTEGER`, `game_id UUID`, `payload JSONB`, `actor TEXT NOT NULL` → tipovi u `load_in/insert_events` slažu se (`i64::from(i32)` nije `useless_conversion`). `Store` je `Clone`, `pool` privatan ali vidljiv podmodulima. `ORDER BY … FOR UPDATE SKIP LOCKED LIMIT 1` je valjan PG redoslijed (T8). |
| 8 | NESKLAD-4. Inače OK: `make_interval(secs => $1)` traži `f64` (`f64::from(i32)`), dopušteno izvan jezgre (check-core-deps skenira floatove samo u engine/sim-hotel). |
| 9 | NESKLAD-6. Postojeći `Problem::not_found`/`code: &'static str` i `AppState{status}` zamjenjuju se u istom tasku (demo.rs, health.rs, http.rs, server) — dosljedno. `IndexEntry` iz demo.rs mora se maknuti u korist `schema::IndexEntry` (inače dva istoimena schema-tipa u OpenAPI-ju). |
| 10 | OK. Redoslijed u `put_plan`: tijelo (422) prije gosta (401) — testovi to ne proturječe. `plans/aug` → 404 `unknown_month` iz `Game::status`, `plans/xyz` iz `parse_month`. |
| 11 | OK. tokio `time` dodan; `check-clean.mjs` samostalan. |
| 12 | NESKLAD-7 i NESKLAD-8. Inače OK (`RandomIds` → nasumičan profil, testovi ne ovise o njemu). |

### 3. Proturječja s Global Constraints i ono što bi pregled označio

- **GC „`game` bez f32/f64, sata, HashMap"** nije ničim čuvan: `check-core-deps` (T2) gleda samo ovisnosti, float-sken samo engine/sim-hotel, `game` nema `clippy.toml`. Popravak (T2): kopiraj `crates/sim-hotel/clippy.toml` u `crates/game/clippy.toml` i dodaj `"crates/game"` u petlju `FLOAT_SUFFIX` u `check-core-deps.mjs`.
- **GC „uuid bez značajke v4"**: `app` (T5) uključuje `v4`, a ujedinjavanje značajki u workspaceu daje je i `game`-u u svakom zajedničkom buildu. Bezopasno (PURE je popis zabrana), ali rečenica u GC-u nije doslovno istinita — preformulirati u „game ne traži v4".
- **T5 „re-export da `store` i `api` ne ovise o `game`"** naspram T9 „api dodaje `game`" — plan sam priznaje; uskladiti rečenicu u T5.
- **Dupliciran blok:** `game_of_hotel` (T5 views.rs = T6 commands.rs) — popravak: u T5 `pub(crate) async fn game_of_hotel`, u T6 `use crate::views::game_of_hotel` umjesto kopije.
- **Dupliciran blok (svjesno):** `plan_limits` (T3) prepisuje `euro_bounds` i `seasonal_range` iz `sim-hotel/src/robots.rs`; izvući ih bi značilo dirati jezgru izvan T1 (GC). Ostaviti, uz komentar „zrcalo `robots::euro_bounds`; test veže na `validate`".
- **Plan traži kopiranje pomoćnika testova** (T6 rounds.rs iz views.rs; T8 jobs.rs iz log.rs) — pregled će to vidjeti kao dupli kod; jeftinije: `crates/app/tests/common/mod.rs` i `crates/store/tests/common/mod.rs` kao u `game`/`api`.
- **Test bez prave tvrdnje:** T1 `snapshot_builds_same_content_as_load` (load je doslovno `Snapshot::read()?.build()`). Popravak: zamijeniti tvrdnjom da `build()` iz snimka pročitanog iz JSON-a (`to_value`→`from_value`) daje isti `Content`, ili test izbaciti.
- Slabe tvrdnje (ne greška): `assert!(!feed(..).is_empty())` (T5); u T10/T12 pozivi `answer`/`close` bez provjere statusa prije glavne tvrdnje.

### 4. Različiti profili (ADR-015 t. 13) — testovi koji pretpostavljaju isti profil

Nijedan. Provjereno: T3 `new_game_writes_created…` tvrdi upravo A/B/C među robotima; T4 `unanswered_s1_costs_satisfaction` (−400 ne ovisi o profilu: `recovery_pts_per_q·(q−3)`, granica 800), `resolution_uses_snapshot_not_files` (mijenja `fixed_cents` sva tri profila), T3 `plan_limits…` (`renovation_available`, sezonci — svi profili imaju `seasonal: 28`, kategoriju 4, nisu obnovljeni), T6/T10/T12 marketing 500 000 ≤ 4 000 000 za svaki profil, T4 `replay…` (isto sjeme). T2 `samples()` ima dva hotela „C", ali se ne folda (samo serde) — OK. Jedina skrivena pretpostavka: svi profili su kategorija „4" (`profiles.json`); robusno i ako se to promijeni.

### 5. clippy `-D warnings` (all=deny) i check-core-deps

- NESKLAD-9 `large_enum_variant` (T2) — **pada**.
- NESKLAD-1 E0659 (T3/T4, i T2-test od T3) — ne prevodi se.
- NESKLAD-2, -4, -8 nekorišteni uvozi — padaju pod `-D warnings`.
- T3 `command.rs` `MonthCode, PlanDto` — plan već kaže „ukloni ako clippy prijavi".
- `Sha256::digest(&bytes)` (T1) — `needless_borrows_for_generic_args` NE pali (provjereno).
- check-core-deps: bez prekršaja (`f64` samo u `store`; `game` bez zabranjenih paketa).

### NESKLADI i najmanji popravci

1. **T3 (+T2, T4 testovi)** `pub mod game;` + `use game::*;` → E0659. Popravak u T3: `mod game;` (privatno) + `pub use game::{Game, HotelRecord, RoundStatus};` i sve `game::game::X` u T4/T5/T6/T12 → `game::X`; ILI preimenovati modul (`pub mod play;`). Minimalno bez preimenovanja: u `tests/{events,round,resolve}.rs` i `tests/common/mod.rs` zamijeniti `use game::*;` izričitim popisom (`use game::{Actor, GameError, GameEvent, GameId, GuestId, HotelId, Recorded, …}`).
2. **T4** nakon premještanja pomoćnika iz round.rs ukloniti `use content::{Snapshot, repo_dir};` i `use uuid::Uuid;` (dodati rečenicu u Step 1).
3. **T5/T6** `game_of_hotel` u views.rs `pub(crate)`; T6 ga uvozi, ne kopira.
4. **T8** Step 1: „kopiraj uvoze bez `DecideError`" (ili zajednički `tests/common/mod.rs`).
5. **T6‖T7** izvoditi u odvojenim radnim stablima (`isolation: worktree`) ili redom; i T6 Files uskladiti s „Paralelizacija" (lib.rs dira T6).
6. **T9** Files/Step 1: `crates/api/Cargo.toml` `[dependencies]` + `game.workspace = true`.
7. **T12** Files: api dev `sim-hotel.workspace = true` (ili u e2e `game::dto::MonthCode::Jun.into()` umjesto `sim_hotel::Month::Jun`).
8. **T12** e2e: obrisati `use app::EventLog;` u `replay_from_database_matches`.
9. **T2** `GameCreated.snapshot: Box<content::Snapshot>` (T2 test `snapshot: Box::new(snap)`, T3 `new_game` `Box::new(snapshot.clone())`; serde/`fingerprint()`/`build()` rade kroz Box) ILI `#[allow(clippy::large_enum_variant)]` na `GameEvent` s komentarom „GameCreated jednom po igri".


## Pre-flight rulings (nose se u dispatch navedenog taska)
- PF1 [T3,T4,T5,T6,T12] Ruling: u `crates/game/src/lib.rs` modul igre je privatan `mod game;` + `pub use game::{Game, HotelRecord, RoundStatus};`; svaki `game::game::X` u planu čitaj kao `game::X` — izbjegava E0659 dvosmislenost s imenom cratea — cijena ako krivo: preimenovanje putanja.
- PF2 [T2] Ruling: `GameCreated.snapshot: Box<content::Snapshot>` (serde je proziran za Box) — clippy large_enum_variant pod deny — cijena: nikakva.
- PF3 [T4] Ruling: obrisati nekorištene uvoze (`content::{Snapshot, repo_dir}`, `uuid::Uuid`) u `tests/round.rs` nakon premještanja u common — -D warnings — cijena: nikakva.
- PF4 [T5,T6] Ruling: `game_of_hotel` jednom, `pub(crate)` u `app/src/views.rs`; T6 ga uvozi, ne kopira — DRY — cijena: nikakva.
- PF5 [T8] Ruling: uvozi u `store/tests/jobs.rs` bez `DecideError` (samo što se koristi) — -D warnings — cijena: nikakva.
- PF6 [T6,T7] Ruling: taskovi strogo redom, nikad paralelno; odjeljak „Paralelizacija" plana se ignorira — T7 prevodi app — cijena: sporije, ništa drugo.
- PF7 [T12] Ruling: obrisati nekorišteni `use app::EventLog;` — cijena: nikakva.
- PF8 [T9] Ruling: dodati `game.workspace = true` u `[dependencies]` cratea api — schema.rs ga treba — cijena: nikakva.
- PF9 [T12] Ruling: e2e koristi `game::dto::MonthCode::Jun` (+ `.into()` ako treba); ako konverzija ne postoji, `sim-hotel` kao dev-ovisnost apija — api ostaje odvojen od jezgre u produkcijskom kodu — cijena: nikakva.
- PF10 [T2] Ruling: dodati `crates/game/clippy.toml` (kopija sim-hotelovog: zabrana floatova/sata/HashMapa) i `crates/game` u provjeru floatova u `scripts/check-core-deps.mjs` — Global Constraint inače nije provjeren — cijena: malo više posla u T2.
- PF11 [T1] Ruling: tautološki test `snapshot_builds_same_content_as_load` zamijeniti testom koji ima smisla: snimak → serde JSON → natrag → `build()` daje isti katalog/situacije kao `content::load`, i `fingerprint()` je stabilan kroz taj krug — test koji ništa ne tvrdi je defekt — cijena: nikakva.
- PF12 [T1,T3] Ruling: T1 (jedini task koji smije dirati jezgru) izlaže `euro_bounds` i `seasonal_range` iz `sim-hotel/src/robots.rs` kao javne (pub, re-export u lib.rs); T3 `plan_limits` ih zove umjesto kopiranja — DRY bez kršenja „jezgra samo u T1" — cijena: dva pub fn više u javnom sučelju jezgre.
- PF13 [T6,T8] Ruling: kopirani testni pomoćnici među crateovima prihvaćeni (testovi različitih crateova ne dijele kod) — cijena: malo dupliciranja u testovima.
Task 1: dispatched (base 7db601d, implementer sonnet)
Task 1: minor (deferred): robots.rs:685-686 dupli doc-komentar na euro_bounds
Task 1: minor (deferred): snapshot.rs dva preklapajuća roundtrip testa (string i Value put)
Task 1: minor (deferred): Snapshot::build klonira cijeli snimak
Task 1: complete (commits 7db601d..76ae988, review clean)
Task 2: ⚠️ resolved: clippy.toml identičan sim-hotelovom (cmp); actor CHECK provjerava T7
Task 2: minor (deferred): nema testa da INDEX_ORDER/indices_dto prate Index::ALL
Task 2: minor (deferred): JSON oblik Controller i GameId::stream() nisu prikovani testom
Task 2: minor (deferred): HotelSetup.profile i Controller::Robot.strategy su String — T3 mora validirati pri stvaranju/foldu
Task 2: complete (commits 76ae988..3b68d86, review clean)
Task 3: dispatched (base 3b68d86, implementer sonnet)
Task 3: minor (deferred): game.rs state_at/latest_state dupliciraju blok čitanja RoundResolved (helper resolved_after)
Task 3: minor (deferred): apply ne provjerava poznatu situaciju/indeks odgovora/mjesec ni jedinstvenost hotela u created
Task 3: minor (deferred): apply kod duplog odgovora prepiše vrijednost prije greške (Entry::Vacant)
Task 3: minor (deferred): testovi — A/B/C samo za seed 42, nema testa determinizma, seasonal_min ispod granice, renovation_available false
Task 3: minor (deferred): robot_moves ne provjerava da je mjesec Open
Task 3: complete (commits 3b68d86..60201e0, review clean)
Task 4: dispatched (base 60201e0, implementer sonnet)
Task 4: ⚠️ resolved: nema HashMap/HashSet u crates/game/src (grep); polja Game pokrivena zelenim testovima T3/T4
Task 4: minor (deferred): feed test "only mine" slab — dodati simetrični slučaj (robotov feed bez mog odgovora)
Task 4: minor (deferred): lookup HotelResolution ponovljen 3x (feed.rs, view.rs x2) + situation lookup u feed.rs — helper RoundResolved::for_hotel
Task 4: minor (deferred): tests/resolve.rs resolution_uses_snapshot_not_files ručno gradi created (created_from u common)
Task 4: minor (deferred): srpanj provjeren samo brojem događaja; private_hotel.now nakon kraja igre; S3 linija neodgovorenog
Task 4: complete (commits 60201e0..92e86d6, review clean)
Task 5: dispatched (base 92e86d6, implementer sonnet)
Task 5: note: SeqIds daje mala uuid (gosti 1-2, igra 3, hoteli 4-7) — 'nepoznati' id u testovima T6+ mora biti velik (npr. 9999)
Task 5: ⚠️ resolved: PublicHotel sadržaj potvrđen u T4 pregledu; check-core-deps prošao (uuid v4 samo u app)
Task 5: carry→T8: Postgres `fail` samo WHERE status='running' (ne uskrsavati Done); istekao zakup s iscrpljenim pokušajima → `failed`, ne Queued. Memory paritet po mogućnosti isto.
Task 5: minor (deferred): memory create_game tiho prepisuje postojeću igru (Postgres bi pao na PK)
Task 5: minor (deferred): demo.rs:148 komentar „u jednoj transakciji" netočan (gost i igra zasebni pozivi; siroče gost moguć)
Task 5: minor (deferred): testovi — semantika memory porta, situations za zatvorenu/završenu igru, token_hash, ErrorCode::ALL potpunost
Task 5: complete (commits 92e86d6..3a410f3, review clean)
Task 6: dispatched (base 3a410f3, implementer sonnet)
Task 6: review: Approved ali 2 Important u portu JobQueue (stale fail bez identiteta zahtjeva; pad na zadnjem pokušaju ostavlja Running zauvijek) → fix round 1
Task 6: Ruling: port JobQueue::fail nosi identitet zahtjeva (attempts iz claim) i djeluje samo ako se poklapa; istekao zakup s iscrpljenim pokušajima → Failed — popravak sad u app jer T8 izvodi isti port u Postgresu — cijena ako krivo: jedna promjena potpisa porta
Task 6: minor (deferred): Failed resolve posao ostavlja rundu u Closing bez oporavka (BACKLOG)
Task 6: minor (deferred): run_next_job gubi izvornu grešku ako fail sam padne
Task 6: minor (deferred): set_plan/answer zovu game_of_hotel prije provjere prava (UnknownHotel vs NotYourHotel otkriva postojanje)
Task 6: minor (deferred): nema izričitog testa zatvaranja s neodgovorenom situacijom ni da moj odgovor preživi zatvaranje
Task 6: fix round 1/5 (3 addressed, 0 open — identitet zahtjeva u fail/finish, istek na zadnjem pokušaju→Failed, izvorna greška; commits 3e1adcc..86e50d9)
Task 6: carry→T8: Postgres fail I finish (Decision.finish_job) traže poklapanje id+attempts+status running; pravilo „istekao zakup na zadnjem pokušaju → failed" primijeniti u claim (pri preuzimanju isteklih)
Task 6: minor (deferred): nema testa grane dvostrukog pada (fail sam padne) i Failed kroz run_next_job end-to-end
Task 6: complete (commits 3a410f3..86e50d9, review clean after fix round 1)
Task 7: dispatched (base 86e50d9, implementer sonnet)
Task 7: minor (deferred): Memory paritet — create_game/create_guest tiho prepisuju duplikate (Postgres daje grešku)
Task 7: Ruling (deferred minor): latest_hotel_of — vrijedi Postgres semantika (najnovija igra u kojoj gost ima hotel); Memory uskladiti u završnom valu — cijena ako krivo: nikakva za demo (jedan igrač)
Task 7: minor (deferred): nema testa da je advisory lock otpušten nakon odbijanja (još jedan decide nakon rejection)
Task 7: minor (deferred): seq overflow kao sqlx::Error::Protocol; dvostruke dev-ovisnosti serde_json/uuid u store; test duple igre samo is_err
Task 7: complete (commits 86e50d9..5b13405, review clean)
Task 8: dispatched (base 5b13405, implementer sonnet)
Task 8: ⚠️ resolved: .sqlx upiti provjereni okom od recenzenta; CI `sqlx prepare --check` će potvrditi u T12
Task 8: minor (deferred): COALESCE(last_error,…) kod isteka na zadnjem pokušaju čuva stariju grešku (zbunjujuće)
Task 8: minor (deferred): teoretski deadlock u expiry UPDATE bez SKIP LOCKED/reda
Task 8: minor (deferred): test konkurentnosti s jednim poslom ne dokazuje preskakanje (2+ posla, N preuzimača)
Task 8: minor (deferred): kopirani pomoćnici testova (plan-mandated, PF13)
Task 8: complete (commits 5b13405..69d9d51, review clean)
Task 9: dispatched (base 69d9d51, implementer sonnet)
Task 9: review: Approved ali 2 Important (tihi pad profila na C; tihi strategy null za nepoznat kod robota) → fix round 1
Task 9: Ruling: pretvorbe u api postaju fallible → ProblemCode Internal (500) + tracing::error!; ne zatvaramo enum u game (fold u T3 već odbija nepoznate kodove, pa je stanje nedostižno osim kod pokvarenog dnevnika) — cijena ako krivo: kasnije zatvoriti tipove u game
Task 9: minor (deferred): testovi kolačića (31/33 znaka, ne-hex, nepoznat token), me/hotel bez igre, allowlist ključeva tuđeg hotela
Task 9: minor (deferred): provjeriti da AppError.detail ne nosi unos gosta; Problem body unwrap_or_default
Task 9: fix round 1/5 (3 addressed, 0 open — profil i strategija fallible→500, parse_month; commits c8b988b..b2c80fe)
Task 9: minor (deferred): corrupt() log bez id-a hotela; mapiranje strategija string-match paralelno s game
Task 9: complete (commits 69d9d51..b2c80fe, review clean after fix round 1)
Task 10: dispatched (base b2c80fe, implementer sonnet)
Task 10: review: Needs fixes — 3 Important (tijelo prije autentikacije; answer bez mjeseca; tanki testovi pristupa) → fix round 1
Task 10: Ruling: I1 popraviti (caller + putanja prije tijela; JsonRejection log) iako plan-mandated — redoslijed 401→404→403 je Review Focus — cijena: nikakva
Task 10: Ruling: I2 bez mjeseca u putanji — šifre situacija jedinstvene po igri (S1,S3 lipanj; S2 srpanj); dodati test da odgovor na lipanjsku šifru nakon prelaska u srpanj daje 409/404; ako ne, popraviti app::answer da provjerava pripadnost otvorenom mjesecu — cijena ako krivo: kad sadržaj ponovi šifru u više mjeseci treba mjesec u putanji (BACKLOG)
Task 10: Ruling: I3 dodati tablične testove 401/404/403 za sve nove rute + PUT plan za srpanj dok je lipanj otvoren → 409
Task 10: minor (deferred): unanswered izveden na dva mjesta (app RoundView i schema SituationsBody)
Task 10: minor (deferred): testovi loših tijela (422), feed by/month/redoslijed, close na resolved → 200; FeedItemBody bez discriminatora
Task 10: fix round 1/5 (3 addressed, 0 open — pristup prije tijela, zastarjeli odgovor 409 dokazan testom, tablični testovi pristupa; commits b0f2a35..a20cc0b). Re-review: testovi bi pali na starom kodu.
Task 10: minor (deferred): `let _ = other_tok;` supresori u testovima; tuđi gost dobiva not_in_game umjesto maskiranog 404 (dosljedno s ostalim rutama)
Task 10: complete (commits b2c80fe..a20cc0b, review clean after fix round 1)
Task 11: dispatched (base a20cc0b, implementer sonnet)
Task 11: minor (deferred): panic u run_next_job tiho ubije radnika (server radi, poslovi stoje) — BACKLOG ili spawn po iteraciji
Task 11: minor (deferred): `?` nakon axum::serve preskače worker.abort(); check-clean bez try/catch; check:api sad širi (staged=dirty)
Task 11: complete (commits a20cc0b..cb15937, review clean)
Task 12: dispatched (base cb15937, implementer sonnet)
Task 12: review: Needs fixes — 2 Important plan-mandated (e2e ne provjerava ishode po hotelu/mjesecu ni robote; drugo zatvaranje bez provjere statusa) → fix round 1
Task 12: Ruling: popraviti iako plan-mandated — e2e je jedini dokaz da partija stvarno radi — cijena: nikakva; usput docs sitnice (§9.4 retry bez odgode, §9.1 primjer toka, §9.3 tablice, §6.2 strelica game→engine)
Task 12: minor (deferred): replay testira samo lipanj
Task 12: fix round 1/5 (3 addressed, 0 open — e2e ishodi/roboti/neodgovoreno, 202 srpanj, docs; commits cfe7015..d42a091)
Task 12: minor (deferred): e2e lookup neodgovorenog pretpostavlja jednu po mjesecu; robot provjere samo postojanje događaja
Task 12: complete (commits cb15937..d42a091, review clean after fix round 1)
Final review: Ready with fixes — I1 panika ubija radnika; I2 promjena oblika content DTO ruši postojeće igre (otisak iz tipiziranog snimka)
Final: Ruling: I2 sad samo dokumentirati (ADR-015 t.3 + BACKLOG korak 4: otisak nad kanonskim Value spremljenog snimka, nova polja uz #[serde(default)]) — promjena semantike otiska na kraju grane je rizičnija od pravila — cijena ako krivo: korak 4 mora to riješiti prije prve promjene DTO-a
Final: Ruling: T7 latest_hotel_of — uskladiti Memory s Postgresom u fix valu (5 redaka)
Final: fix wave dispatched (base d42a091)
Final: re-review — I1,I2,M1,M2,M4 addressed; M3 nedostaje 1 BACKLOG redak (T4 resolution_uses_snapshot_not_files ručno gradi created)
Final: parked — M3 jedan redak — Ruling: sitnica bez utjecaja na kod; dodaje se pri /kraj uz kopiju ledgera u docs/archive — cijena: nikakva. Clippy sumnja na `if true { panic!() }` opovrgnuta izvješćem (clippy EXIT=0).
Final: clean (commits d42a091..5894cf1)

## Završni pregled grane (sažetak)

**Pregledao:** opus, 05.10., grana `gradnja/dnevnik-runde` (5efd7b8 → d42a091, 18 commitova). **Presuda:** 🟨 spremno uz popravke (0 kritičnih, 2 važna, 4 obavezne sitnice). Nakon popravaka (5894cf1) ponovni pregled čist.

### Važno

**I1. Radnik tiho umre na panici, a obračuni stanu do restarta** — `crates/server/src/main.rs:59-71`.
`spawn_worker` vrti `run_next_job` izravno u jednom tokio zadatku. Panika bilo gdje na putu (npr. `expect` u
`GameEvent::to_payload`/`Snapshot::fingerprint`, ili buduća greška u jezgri) završi zadatak; `JoinHandle` se nikad ne
čita, server i dalje odgovara, a svaka iduća „Zatvori mjesec" ostaje u `closing` bez traga u logu osim jedne linije
panike. Na demu pred profesorom to izgleda kao zaglavljena igra. Popravak (jeftin): svaku iteraciju pokreni kao
`tokio::spawn(async move { app::run_next_job(&d).await })` s kloniranim `Deps` i čekaj `JoinHandle`; na
`JoinError::is_panic()` → `tracing::error!` i nastavi petlju. Posao koji je bio u radu ostaje `running`, zakup istekne
i posao se preuzme ponovno; deterministička panika nakon 5 pokušaja postaje `failed` (postojeće pravilo).
(Ledger: Task 11 minor — dižem na Važno.)

**I2. Snimak pravila je zamrznut oblikom `content` DTO-a, a fold to provjerava pri svakom čitanju** —
`crates/game/src/game.rs:70`, `crates/content/src/lib.rs` (`Snapshot`, `SituationDto` s `deny_unknown_fields` i
obaveznim `unanswered`).
`Game::created` ponovno serijalizira TIPIZIRANI snimak i uspoređuje SHA s `ruleset_id`. Posljedica: svaka promjena
oblika content DTO-a (novo polje — korak 4 će dodati rubrike/likove u `situations.json` —, preimenovanje, uklanjanje)
čini SVAKU postojeću igru trajno nečitljivom (500 na svim rutama): bez `#[serde(default)]` pada deserijalizacija, s
njim se otisak više ne poklapa → `Corrupt`. Za svježu lokalnu bazu to nije problem, ali ADR-015 t. 3 i ARHITEKTURA
§9.1 („stari se događaji pri čitanju podignu") to obećavaju, a izgrađeno to ne podržava i nigdje ne piše.
Popravak prije spajanja (samo dokument, 10 min): u ADR-015 t. 3 i BACKLOG `[korak 4]` zapisati pravilo „promjena oblika
content DTO-a = nova verzija `GameCreated` s podizanjem; do tada nova polja samo s `#[serde(default)]`". Popravak u
koraku 4 (kod): otisak računati nad kanonskim `serde_json::Value` spremljenog snimka (serde_json je bez
`preserve_order` pa su ključevi sortirani) umjesto nad tipiziranim ponovnim zapisom — tada `default` polja ne kvare otisak.

**Obavezno prije spajanja** (popravljeno u 5894cf1, ponovni pregled čist): radnik preživi paniku (I1), pravilo otiska snimka zapisano (I2), odgođene sitnice upisane u BACKLOG.
