# 2026-10-10-v2-sloj1-kartice — dio 3

Arhiva (zamrznuto). Indeks: [2026-10-10-v2-sloj1-kartice.md](../2026-10-10-v2-sloj1-kartice.md).

### Task 7: App i API — nova igra s postavkama, kodovi kruga, polja `round`

**Veličina:** M, 3–4 h · **Ovisi o:** Task 6 · **Paralelno s:** Task 9 (Task 8 čeka ovaj task) · **Model:** sonnet

**Cilj:** `POST /v1/demo` prima neobavezne postavke (`model`, `round_kind`); sve putanje s `{month}` primaju i kod
tjedna; tijela dobivaju `round` (kod i kalendar kruga), a postojeće polje `month` ostaje (za tjedan: mjesec u kojem
krug počinje).

**Files:**
- Modify: `crates/app/src/{lib,demo,views,commands,memory}.rs`, `crates/server/src/main.rs`,
  `crates/api/src/{demo,games,hotels,schema,kpi,explanation,problem}.rs`, `openapi.json`,
  `packages/api-client/src/schema.d.ts`, `crates/app/Cargo.toml` i `crates/api/Cargo.toml`
  (`sim-hotel-v2.workspace = true` — za `PeriodKind` u `NewGameSettings`/`RoundInfoBody`), `Cargo.lock`
- Modify: `crates/api/tests/common/mod.rs` — dodaj `pub fn cookie_value(set_cookie: &str) -> String` (vrijednost
  `hotelixsim_guest=…` do `;`, isto što radi `token` u `crates/api/tests/e2e.rs:28-36`) i premjesti
  `pub async fn run_worker(d: &app::Deps)` (`while app::run_next_job(d).await.unwrap() {}`) iz
  `crates/api/tests/e2e.rs:25-27` i `crates/api/tests/games.rs:5-7` (tamo obriši lokalne kopije; `e2e.rs` umjesto
  `token(..)` zove `common::cookie_value(..)`). Taskovi 15 i 22 ih koriste.
- Mehaničke izmjene literala (Global Constraints; očekivanja se ne mijenjaju): `Deps { …, snapshot_v2 }` u
  `crates/api/tests/e2e.rs:14` i `:338` (`Arc::new(Snapshot::read_v2(&repo_dir()).unwrap())`); novi argument
  `NewGameSettings::default()` u pozivima `start_demo` u `crates/app/tests/rounds.rs:13`, `crates/app/tests/talk.rs:14`,
  `crates/app/tests/views.rs:11, 30, 33, 45`.
- Create: `crates/api/tests/rounds_v2.rs`
- Ne dirati: `crates/game/src/**` osim ako fali javni getter (tada najmanji), `apps/web/**` (Task 18 troši tipove).

**Interfaces:**
- Consumes: Task 6 (`new_game_with`, `GameSettings`, `Game::{model, round_info, rounds_info}`, `RoundInfo`,
  `GameError::{Outdated, InvalidSettings}`).
- Produces:

```rust
// crates/app/src/lib.rs — Deps
pub struct Deps { /* …postojeće… */ pub snapshot_v2: Arc<content::Snapshot> }
// crates/app/src/demo.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NewGameSettings { pub model: content::Model, pub kind: Option<sim_hotel_v2::PeriodKind> }
impl Default for NewGameSettings { /* v1, None */ }
pub async fn start_demo(deps: &Deps, token: Option<&str>, settings: NewGameSettings) -> Result<DemoStarted, AppError>;
// v1 + None|Month → v1 mjesečna (lipanj–srpanj); v1 + Week → InvalidSettings; v2 + None → tjedan (🟨 P2); v2 + kind → kind.
// crates/app/src/memory.rs: `deps(snapshot)` puni `snapshot_v2` iz `Snapshot::read_v2(&content::repo_dir())`.
// crates/app/src/views.rs: GameView { …, model: game::Model, rounds: Vec<RoundInfo> }, RoundView { round: RoundInfo, … }.
```

```rust
// crates/api/src/schema.rs — samo dodano
#[derive(Serialize, Deserialize, ToSchema, Clone, Copy)] #[serde(rename_all = "lowercase")]
pub enum ModelCode { V1, V2 }
/// Iz `game::Model::code()` ("v1" | "v2") — bez `match` na `game::Model` u api (Global Constraints).
impl ModelCode { pub fn from_code(code: &str) -> ModelCode; } // "v2" → V2, inače V1
#[derive(Serialize, Deserialize, ToSchema, Clone, Copy)] #[serde(rename_all = "lowercase")]
pub enum RoundKindCode { Month, Week }
#[derive(Deserialize, ToSchema)] #[serde(deny_unknown_fields)]
pub struct NewGameBody { pub model: Option<ModelCode>, pub round_kind: Option<RoundKindCode> }
/// Krug igre i njegov kalendar; `index` od 0; datumi su dan u mjesecu (od 1) i mjesec.
#[derive(Serialize, ToSchema, Clone)]
pub struct RoundInfoBody {
    pub code: String, pub kind: RoundKindCode, pub index: u8, pub days: u8,
    pub first_day: u8, pub first_month: MonthCode, pub last_day: u8, pub last_month: MonthCode,
    /// Završava li u ovom krugu kalendarski mjesec (tada dolazi mjesečna lekcija).
    pub month_ends: bool,
}
// Dodana polja (postojeća ostaju):
// GameBody { model: ModelCode, round_kind: RoundKindCode, rounds: Vec<RoundInfoBody> }
// RoundBody { round: RoundInfoBody }
// MonthScoreBody { round: String }        LastMonthBody { round: RoundInfoBody }
// plan u PrivateHotelBody { round: String } SituationsBody { round: Option<String> }
// svaka varijanta FeedItemBody { round: String }   KpiMonthBody { round: String, days: u8 }
// tijelo objašnjenja mjeseca { round: String }
pub fn parse_round(s: &str) -> Result<game::RoundKey, Problem>; // zamjenjuje parse_month; 404 unknown_month
```

`POST /v1/demo` tijelo je neobavezno: handler prima `axum::body::Bytes`; prazno → zadane postavke; inače
`serde_json::from_slice::<NewGameBody>` (greška → 422 `invalid_body`). Opis parametra `{month}` u `utoipa::path` postaje
`("month" = String, Path, description = "Kod kruga: jun…sep ili t01…t17")`. Naslov problema `unknown_month` postaje
„Taj krug ne postoji u ovoj igri."

- [ ] **Step 1: Testovi (padaju)** — `crates/api/tests/rounds_v2.rs` (pomoćnici `common::{app, call}` kao u
  `crates/api/tests/games.rs`; kolačić iz `set_cookie` prvog odgovora):

```rust
mod common;

use serde_json::json;

async fn new_game(body: Option<serde_json::Value>) -> (axum::Router, String, String, String) {
    let (app, _, _) = common::app();
    let r = common::call(&app, "POST", "/v1/demo", None, body).await;
    assert_eq!(r.status, 201, "{:?}", r.body);
    let cookie = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let game = r.body["game_id"].as_str().unwrap().to_owned();
    let hotel = r.body["hotel_id"].as_str().unwrap().to_owned();
    (app, cookie, game, hotel)
}

#[tokio::test]
async fn post_demo_without_body_still_creates_two_month_v1_game() {
    let (app, c, g, _) = new_game(None).await;
    let r = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    assert_eq!(r.body["model"], "v1");
    assert_eq!(r.body["round_kind"], "month");
    assert_eq!(r.body["months"], json!(["jun", "jul"]));
    let codes: Vec<&str> = r.body["rounds"].as_array().unwrap().iter().map(|x| x["code"].as_str().unwrap()).collect();
    assert_eq!(codes, ["jun", "jul"]);
    assert_eq!(r.body["current"]["round"]["code"], "jun");
}

#[tokio::test]
async fn post_demo_v2_defaults_to_weekly_full_season() {
    let (app, c, g, _) = new_game(Some(json!({ "model": "v2" }))).await;
    let r = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    assert_eq!(r.body["model"], "v2");
    assert_eq!(r.body["round_kind"], "week");
    assert_eq!(r.body["rounds"].as_array().unwrap().len(), 17);
    assert_eq!(
        r.body["rounds"][4],
        json!({ "code": "t05", "kind": "week", "index": 4, "days": 7, "first_day": 29, "first_month": "jun",
                "last_day": 5, "last_month": "jul", "month_ends": true })
    );
    assert_eq!(r.body["current"]["round"]["code"], "t01");
    assert_eq!(r.body["current"]["month"], "jun");
}

#[tokio::test]
async fn post_demo_v2_monthly_has_four_rounds() {
    let (app, c, g, _) = new_game(Some(json!({ "model": "v2", "round_kind": "month" }))).await;
    let r = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    assert_eq!(r.body["rounds"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn post_demo_v1_with_weeks_is_422_invalid_settings() {
    let (app, _, _) = common::app();
    let r = common::call(&app, "POST", "/v1/demo", None, Some(json!({ "model": "v1", "round_kind": "week" }))).await;
    assert_eq!(r.status, 422);
    assert_eq!(r.body["code"], "invalid_settings");
}

#[tokio::test]
async fn post_demo_with_unknown_field_is_422_invalid_body() {
    let (app, _, _) = common::app();
    let r = common::call(&app, "POST", "/v1/demo", None, Some(json!({ "modell": "v2" }))).await;
    assert_eq!(r.status, 422);
    assert_eq!(r.body["code"], "invalid_body");
}

#[tokio::test]
async fn round_paths_accept_week_codes() {
    let (app, c, g, h) = new_game(Some(json!({ "model": "v2" }))).await;
    let r = common::call(&app, "GET", &format!("/v1/games/{g}/rounds/t01"), Some(&c), None).await;
    assert_eq!(r.status, 200);
    assert_eq!(r.body["round"]["code"], "t01");
    let plan = common::call(&app, "GET", "/v1/me/hotel", Some(&c), None).await.body["private"]["plan"]["plan"].clone();
    let r = common::call(&app, "PUT", &format!("/v1/hotels/{h}/plans/t01"), Some(&c), Some(plan)).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    let r = common::call(&app, "POST", &format!("/v1/games/{g}/rounds/t01/close"), Some(&c), None).await;
    assert_eq!(r.status, 202);
}

#[tokio::test]
async fn week_code_in_monthly_game_is_404() {
    let (app, c, g, _) = new_game(None).await;
    let r = common::call(&app, "GET", &format!("/v1/games/{g}/rounds/t03"), Some(&c), None).await;
    assert_eq!(r.status, 404);
    assert_eq!(r.body["code"], "unknown_month");
}

#[tokio::test]
async fn outdated_v2_game_is_409_game_outdated() {
    let (app, deps, mem) = common::app();
    let r = common::call(&app, "POST", "/v1/demo", None, Some(json!({ "model": "v2" }))).await;
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let g = r.body["game_id"].as_str().unwrap().to_owned();
    mem.patch_first_event(&g, |p| p["core_version"] = json!("1.9.9"));
    let r = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    assert_eq!(r.status, 409);
    assert_eq!(r.body["code"], "game_outdated");
    assert_eq!(r.body["title"], "Igra je stvorena starijom verzijom igre. Započni novu igru.");
    drop(deps);
}
```

`common::cookie_value` i `common::run_worker` su u `crates/api/tests/common/mod.rs` (Files gore; danas ne postoje —
`common::call` sam dodaje `hotelixsim_guest=`, pa mu se predaje samo vrijednost kolačića).
`Memory::patch_first_event(&self, game_id: &str, f: impl FnOnce(&mut serde_json::Value))` dodaj u
`crates/app/src/memory.rs` s komentarom „samo za testove: mijenja spremljeni GameCreated". `deps_with_ai` u istoj
datoteci puni `snapshot_v2` iz `Snapshot::read_v2(&content::repo_dir())`.

- [ ] **Step 2:** `cargo test -p api --test rounds_v2` → FAIL.
- [ ] **Step 3: Implementiraj** (app → api), `RoundInfoBody::from(&RoundInfo)` iz `period.first_date()`,
  `last_date()`, `month_ending_in().is_some()`; `month` polja iz `round.period.start_month()`;
  `GameBody.model = ModelCode::from_code(view.model.code())` (nikad `match` na `game::Model` u app/api).
- [ ] **Step 4:** `corepack pnpm gen:api; echo EXIT=$?`, zatim `cargo test --workspace; echo EXIT=$?` i
  `corepack pnpm typecheck; echo EXIT=$?` (web se mora prevesti s dodanim poljima — samo dodana, pa prolazi).
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/7: nova igra s postavkama modela i kruga; kodovi tjedna u putanjama".

**Prihvaćanje:** svi stari API testovi zeleni bez promjene očekivanja (osim novih polja); `openapi.json` u diffu
samo dodaje (provjeri `git diff openapi.json | grep '^-' | grep -v '^---'` — samo opis parametra `{month}` i naslov).

---

### Task 8: Mentor u tjednoj igri

**Veličina:** S–M, 2–3 h · **Ovisi o:** Task 6, **Task 7** (test u `app` stvara v2 tjednu igru kroz
`NewGameSettings` i `Deps.snapshot_v2`) · **Paralelno s:** Task 10, Task 13, Task 18 (val 4) · **Model:** sonnet

**Cilj (🟨 P3, Leon ADR-028 t. 5 „mentor tjedno kratko bez AI-ja"):** u tjednoj igri AI mentor se zove samo u krugu u
kojem završava kalendarski mjesec (t05, t09, t14, t17) — bez ključa tada dolazi puni rezervni mentor u četiri dijela,
kao danas. **Ostali tjedni dobivaju samo kratki sažetak „tri uzroka" bez AI-ja** (ne puni rezervni mentor). Činjenice
mentora govore o tjednu riječima („u tjednu od 29. lipnja do 5. srpnja") i uspoređuju s prethodnim krugom.

**Files:**
- Modify: `crates/game/src/{mentor,ai_input,lib}.rs` (`lib.rs`: izvoz `wants_ai_mentor` i `weekly_summary` uz
  postojeće `debrief_facts, fallback_debrief`, `lib.rs:30`), `crates/app/src/jobs.rs`,
  `crates/game/tests/common/mod.rs` (novi pomoćnik `v2_weekly_game_resolved_through`)
- Create: `crates/game/tests/mentor_weekly.rs`; dodaj test u `crates/app/tests/rounds.rs`
- Mehaničke izmjene literala (Global Constraints; očekivanja se ne mijenjaju) — tri nova polja `DebriefInput` s
  v1 vrijednostima srpnja (`round_kind: "month".into(), first_date: (1, MonthCode::Jul), last_date: (31,
  MonthCode::Jul)`) u `crates/game/tests/mentor.rs:49` (`sample()`) i `crates/ai/tests/request.rs:133`
  (`sample_debrief()`); ništa drugo u `crates/ai/**`.
- Ne dirati: `prompts/*.md`, `crates/ai/src/**`, `crates/app/src/memory.rs` (`FakeAi` već broji pozive:
  `FakeAi::calls()` vraća `(presuda, lik, mentor)`, `memory.rs:94-100`), mjesečne tekstove mentora (v1 igre moraju
  dati isti tekst — očekivanja postojećih testova u `crates/game/tests/mentor.rs` se ne mijenjaju).

**Interfaces:**
- Consumes: `Game::round_info`, `Period::{first_date, last_date, month_ending_in}`, Task 5 `round_locative`, Task 7
  `NewGameSettings`, `start_demo(deps, token, settings)`.
- Produces:

```rust
// crates/game/src/ai_input.rs — DebriefInput dobiva (serde default):
pub round_kind: String,           // "month" | "week"
pub first_date: (u8, MonthCode), pub last_date: (u8, MonthCode),
// crates/game/src/mentor.rs
/// Treba li za ovaj krug zvati AI mentora (🟨 P3): v1 i mjesečna v2 uvijek; tjedna samo kad u krugu završava mjesec.
pub fn wants_ai_mentor(g: &Game, r: RoundKey) -> Result<bool, GameError>;
/// Kratki tjedni sažetak bez AI-ja („tri uzroka", Leon ADR-028 t. 5) za tjedan u kojem ne završava mjesec.
pub fn weekly_summary(input: &DebriefInput) -> String;
```

Mjesto u rečenici: mjesec → `round_locative` iz Taska 5 („u lipnju", tekst isti kao danas); tjedan → „u tjednu od
{d}. {genitiv mjeseca} do {d}. {genitiv mjeseca}" (genitivi: lipnja, srpnja, kolovoza, rujna); „sljedeći put" za
tjedan je „sljedeći tjedan". `DebriefFacts.month` i `DebriefFacts.next_month` postaju `String` / `Option<String>`
(tekst tjedna se slaže iz datuma); postojeći testovi se prilagođavaju samo tipom (`.as_deref()`), ne tekstom.

`weekly_summary` (iz `debrief_facts(input)`, samo postojeće rečenice i pomoćnici iz `mentor.rs`):
1. `"{Mjesto velikim slovom} je napredak tvog hotela od početka igre {napredak}."` (ista prva rečenica kao
   `fallback_debrief`), prazan redak;
2. naslov `Tri uzroka`, pa najviše tri rečenice za tri indeksa s najvećom apsolutnom promjenom (`change ≠ 0`; jednaki →
   redoslijed `INDEX_ORDER`): pao → `why_fell(i)` („Indeks … pao je ponajviše zbog …"), porastao → `"Indeks {} porastao
   je s {} na {}."`; nijedan se nije pomaknuo → `"Ovaj tjedan se nijedan indeks nije pomaknuo."`;
3. zadnji redak `"Puni pregled s mentorom stiže na kraju mjeseca."`.
Bez dijelova „Što je išlo dobro / Što nije valjalo / Zašto / Što probati" i **bez** rečenice „AI objašnjenje danas nije
bilo dostupno…" (AI se ovdje namjerno ne zove).

- [ ] **Step 1: Testovi (padaju)** — `crates/game/tests/mentor_weekly.rs`:

```rust
mod common;

use game::*;

#[test]
fn week_locative_names_both_dates() {
    let g = common::v2_weekly_game_resolved_through("t05");
    let h = common::hotel_ids()[0];
    let input = g.debrief_input(h, RoundKey::Week(5)).unwrap();
    let facts = debrief_facts(&input);
    assert_eq!(facts.month, "u tjednu od 29. lipnja do 5. srpnja");
    assert_eq!(facts.next_month.as_deref(), Some("sljedeći tjedan"));
}

#[test]
fn first_week_compares_with_start_of_game() {
    let g = common::v2_weekly_game_resolved_through("t01");
    let input = g.debrief_input(common::hotel_ids()[0], RoundKey::Week(1)).unwrap();
    assert!(input.prev_round.is_none());
}

#[test]
fn ai_mentor_only_when_a_month_ends() {
    let g = common::v2_weekly_game_resolved_through("t09");
    let wants: Vec<u8> = (1..=9).filter(|&n| wants_ai_mentor(&g, RoundKey::Week(n)).unwrap()).collect();
    assert_eq!(wants, [5, 9]);
    let (v1, _) = common::open_june_game();
    assert!(wants_ai_mentor(&v1, sim_hotel::Month::Jun.into()).unwrap());
}

#[test]
fn weekly_summary_is_three_causes_without_ai_note() {
    let g = common::v2_weekly_game_resolved_through("t02");
    let input = g.debrief_input(common::hotel_ids()[0], RoundKey::Week(2)).unwrap();
    let text = weekly_summary(&input);
    assert!(text.starts_with("U tjednu od 8. lipnja do 14. lipnja je napredak tvog hotela od početka igre "), "{text}");
    assert!(text.contains("\nTri uzroka\n"), "{text}");
    assert!(text.lines().filter(|l| l.starts_with("Indeks ")).count() <= 3, "{text}");
    assert!(text.ends_with("Puni pregled s mentorom stiže na kraju mjeseca."), "{text}");
    for long in ["Što je išlo dobro", "Što nije valjalo", "Zašto\n", "Što probati", "AI objašnjenje"] {
        assert!(!text.contains(long), "{long:?} u {text}");
    }
}
```

U `crates/game/tests/common/mod.rs` dodaj `pub fn v2_weekly_game_resolved_through(code: &str) -> Game` (stvara v2
tjednu igru kao `model_v2.rs` — `new_game_with(.., &Snapshot::read_v2(&repo_dir()).unwrap(), .., GameSettings { kind:
PeriodKind::Week })` — pa zatvara i obračunava krugove redom do uključivo `code`).

`crates/app/tests/rounds.rs` — novi test (nakon Taska 7 postoji sve što treba):

```rust
#[tokio::test]
async fn weekly_debrief_calls_ai_only_at_month_end() {
    let ai = Arc::new(app::memory::FakeAi::good());
    let (d, mem) = app::memory::deps_with_ai(Snapshot::read(&repo_dir()).unwrap(), ai.clone());
    let settings = NewGameSettings { model: content::Model::V2, kind: Some(sim_hotel_v2::PeriodKind::Week) };
    let s = start_demo(&d, None, settings).await.unwrap();
    let me = caller(&d, s.new_token.as_deref()).await.unwrap();
    for n in 1..=5 {
        assert!(close_round(&d, &me, s.game_id, game::RoundKey::Week(n)).await.unwrap().accepted, "t0{n}");
        while run_next_job(&d).await.unwrap() {}
    }
    let debriefs: Vec<_> = mem.events(s.game_id).into_iter().filter(|e| e.kind == "DebriefRecorded").collect();
    assert_eq!(debriefs.len(), 5);
    assert_eq!(ai.calls().2, 1); // samo t05 (završava lipanj)
    for e in &debriefs[..4] {
        assert_eq!(e.payload["source"], "fallback");
        assert!(e.payload["text"].as_str().unwrap().contains("Tri uzroka"));
        assert!(!e.payload["text"].as_str().unwrap().contains("Što je išlo dobro"));
    }
    assert_eq!(debriefs[4].payload["month"], "t05");
    assert_eq!(debriefs[4].payload["source"], "ai");
}
```

(`close_round` prima `RoundKey` od Taska 5; `CloseOutcome.accepted` postoji danas, `rounds.rs:83`.)

- [ ] **Step 2:** `cargo test -p game --test mentor_weekly; cargo test -p app --test rounds` → FAIL.
- [ ] **Step 3: Implementiraj.** U `run_debrief` (`crates/app/src/jobs.rs:311-341`), prije poziva AI-ja: ako
  `!wants_ai_mentor(&g, r)?` → odmah `(weekly_summary(&input), AiSource::Fallback, None)`, bez poziva
  `deps.ai.debrief`. Inače kao danas: AI, a kad AI ne radi (nema ključa, ispad) → `fallback_debrief(&input)` (puni,
  četiri dijela).
- [ ] **Step 4:** `cargo test --workspace; echo EXIT=$?` → zeleno (očekivanja postojećih mentor testova nepromijenjena).
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/8: mentor u tjednoj igri — tjedno tri uzroka, AI na kraju mjeseca".

---

### Task 9: Odjeli u jezgri v2 — tipovi i formule odjela (čisto)

**Veličina:** M, 3–4 h · **Ovisi o:** Task 6 · **Paralelno s:** Task 7 · **Model:** sonnet

**Cilj:** čiste funkcije jednog odjela po USALI razredu, s ručno izračunatim testovima; još nisu spojene u obračun
(to je Task 11–12), pa se ovaj task može pregledati sam za sebe.

**Files:**
- Create: `crates/sim-hotel-v2/src/department.rs`, `crates/sim-hotel-v2/tests/department.rs`
- Modify: `crates/sim-hotel-v2/src/{lib,catalog}.rs`
- Ne dirati: `resolve.rs`, `state.rs`, `plan.rs` (Task 11), `crates/game/**`, `crates/content/**`. `crates/content`
  se ne lomi: od Taska 4 gradi v2 katalog kao `degenerate(&base)` pa postavi `periods`, pa nova polja `Catalog`
  (`departments`, `profiles`, `economy`) do Taska 10 ostaju degenerirana (`cargo test --workspace` mora biti zelen).

**Interfaces (Produces):**

```rust
// crates/sim-hotel-v2/src/department.rs
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeptCode(pub String);
impl DeptCode { pub fn rooms() -> DeptCode; pub fn as_str(&self) -> &str; }

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UndistributedKind { AdminGeneral, SalesMarketing, Maintenance, Utilities }
/// USALI razred odjela (MODEL_V2 §3.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UsaliClass { Rooms, FoodBeverage, OtherOperated, Rentals, Undistributed(UndistributedKind) }
impl UsaliClass {
    /// rooms, food_beverage, other_operated, rentals, admin_general, sales_marketing, maintenance, utilities
    pub fn code(self) -> &'static str;
    pub fn from_code(s: &str) -> Option<UsaliClass>;
    pub fn is_undistributed(self) -> bool;
    /// Redoslijed u računu dobiti: Smještaj, hrana i piće, ostali operativni, ostali prihodi, A&G, S&M, POM, energija.
    pub fn rank(self) -> u8;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RevenueModel {
    /// Prihod soba (samo Smještaj): računa ga tržište.
    Rooms,
    /// Noćenja gostiju × udio koji koristi odjel × prosječni račun.
    PerGuest { usage_bp: MonthTable<u16>, check_cents: i64, price_elasticity_bp: i32, satisfaction_elasticity_bp: i32 },
    /// Stalni iznos mjesečno + iznos po noćenju (parking, najam lokala).
    Fixed { monthly_cents: i64, per_guest_night_cents: i64 },
    /// Udio prihoda soba — samo degenerirani katalog (v1 `other_revenue_bp`).
    ShareOfRooms { bp: u16 },
    /// Neraspoređeni odjeli.
    None,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptSpec {
    pub name: String,
    pub class: UsaliClass,
    pub revenue: RevenueModel,
    pub cost_of_sales_bp: u16,
    pub direct_per_use_cents: i64,
    pub wage_cents: i64,
    /// Korištenja po zaposlenom na dan, u desetinkama (Smještaj 24 = 2,4 prodane noći kao u v1).
    pub capacity_tenths_per_staff_day: u16,
    /// Trošenje stanja prostora mjesečno (pozitivan broj, oduzima se).
    pub wear_pts_per_month: i32,
    /// Razine održavanja: [odgođeno, redovno, pojačano] — mjesečni trošak i bodovi stanja.
    pub maintenance: [LevelEffect; 3],
    pub fixed_monthly_cents: i64,
    pub per_occupied_room_cents: i64,
    pub seasonal_max: u16,
    pub seasonal_plan: MonthTable<u16>,
    pub can_close: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
/// `class` se prepisuje iz `DeptSpec` pri stvaranju stanja i ne mijenja se; služi da `HotelState::summary(&self)`
/// (Task 11) bez kataloga zna koji su odjeli operativni.
pub struct DeptState { pub class: UsaliClass, pub open: bool, pub permanent: u16, pub seasonal: u16, pub condition_pts: i32, pub satisfaction_pts: i32 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptPlan { pub open: bool, pub seasonal_target: u16, pub price_level_pct: u16, pub maintenance: Maintenance }

pub struct DeptInput<'a> {
    pub period: &'a Period,
    pub spec: &'a DeptSpec,
    pub state: &'a DeptState,
    pub plan: &'a DeptPlan,
    pub guest_nights: u32,
    pub rooms_revenue_cents: i64,
    pub experience_pts: i32,
    pub morale_pts: i32,
    pub wage_pct: u16,
    pub satisfaction_step_bp: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptOutcome {
    pub code: DeptCode, pub class: UsaliClass, pub open: bool, pub staff: u16,
    pub capacity_uses: u32, pub demand_uses: u32, pub served_uses: u32, pub usage_bp: u16,
    pub load_hundredths: u16, pub load_penalty_pts: i32, pub quality_pts: i32, pub price_level_pct: u16,
    pub revenue_cents: i64, pub payroll_cents: i64, pub cost_of_sales_cents: i64, pub other_direct_cents: i64, pub profit_cents: i64,
    pub condition_before_pts: i32, pub wear_pts: i32, pub load_wear_pts: i32, pub maintenance_pts: i32, pub condition_after_pts: i32,
    /// Trošak održavanja ovog odjela — knjiži se u neraspoređeni odjel „Održavanje" (🟨 P9), ne u rezultat odjela.
    pub maintenance_cents: i64,
    pub value_pts: i32, pub satisfaction_before_pts: i32, pub satisfaction_target_pts: i32, pub satisfaction_after_pts: i32,
    /// Težina u kvaliteti i zadovoljstvu hotela: korištenja (Smještaj: noćenja gostiju); 0 = ne dotiče goste.
    pub guest_weight: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssignedCosts { pub marketing_cents: i64, pub reply_cents: i64, pub maintenance_cents: i64, pub training_cents: i64, pub hiring_cents: i64, pub situation_cents: i64 }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndistributedOutcome {
    pub code: DeptCode, pub kind: UndistributedKind, pub staff: u16,
    pub payroll_cents: i64, pub fixed_cents: i64, pub per_room_cents: i64, pub assigned: AssignedCosts, pub total_cents: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeriodAccounts {
    pub days: u8, pub rooms: u16,
    pub departments: BTreeMap<DeptCode, DeptOutcome>,            // Smještaj + operativni + ostali prihodi
    pub undistributed: BTreeMap<DeptCode, UndistributedOutcome>,
    pub fixed_charges: BTreeMap<String, i64>,                    // osiguranje, najam, porez — razmjerno razdoblju
    pub interest_cents: i64, pub capex_cents: i64,
}

/// Jedan operativni odjel ili odjel ostalih prihoda (ne Smještaj, ne neraspoređeni) u razdoblju.
pub fn dept_step(i: &DeptInput) -> Calc<DeptOutcome>;
/// Neraspoređeni odjel: plaće + stalni dio + po prodanoj noći + troškovi odluka koje mu pripadaju.
pub fn undistributed_step(code: &DeptCode, spec: &DeptSpec, state: &DeptState, fixed_monthly_cents: i64, p: &Period, sold: u32, wage_pct: u16, assigned: AssignedCosts) -> Calc<UndistributedOutcome>;
/// Raspodjela ukupnog broja sezonaca po odjelima (vidi formulu niže).
pub fn allocate_seasonal(current: &BTreeMap<DeptCode, u16>, plan: &BTreeMap<DeptCode, u16>, max: &BTreeMap<DeptCode, u16>, total_target: u16) -> BTreeMap<DeptCode, u16>;

// crates/sim-hotel-v2/src/catalog.rs — dodano
pub struct EconomyV2 { pub guests_per_room_hundredths: u16, pub price_level_min_pct: u16, pub price_level_max_pct: u16, pub fixed_charges: BTreeMap<String, FixedChargeSpec> }
pub struct FixedChargeSpec { pub name: String }
pub struct DeptStart { pub name: Option<String>, pub open: bool, pub permanent: u16, pub seasonal: u16, pub condition_pts: i32, pub satisfaction_pts: i32, pub fixed_monthly_cents: Option<i64> }
pub struct ProfileV2 { pub departments: BTreeMap<DeptCode, DeptStart>, pub fixed_charges_cents: BTreeMap<String, i64> }
pub struct Catalog { pub base, pub periods, pub departments: BTreeMap<DeptCode, DeptSpec>, pub profiles: BTreeMap<Profile, ProfileV2>, pub economy: EconomyV2 }
impl Catalog { pub fn dept(&self, c: &DeptCode) -> Result<&DeptSpec, CatalogError>; pub fn profile_v2(&self, p: Profile) -> Result<&ProfileV2, CatalogError>; pub fn dept_name(&self, p: Profile, c: &DeptCode) -> Result<String, CatalogError>; }
```

**Formule odjela `dept_step` (MODEL_V2 §3.2; sve kroz `engine::calc`, oznake: `p` razdoblje):**

| korak | formula |
|---|---|
| 1 osnovni udio | `usage_base = avg_u16(usage_bp, p)` (PerGuest) |
| 2 učinak cijene | `f_price = clamp(10 000 + div_round(price_elasticity_bp·(level − 100), 100), 0, 20 000)` |
| 3 učinak ugleda odjela | `f_sat = clamp(10 000 + div_round(satisfaction_elasticity_bp·(sat_prije − 7 000), 10 000), 0, 20 000)` (zadovoljstvo **s početka** kruga — nema kruženja) |
| 4 udio | `usage = clamp(div_round(usage_base·f_price·f_sat, 10⁸), 0, 10 000)` |
| 5 potražnja | `demand = div_round(guest_nights·usage, 10 000)`; zatvoren odjel → 0 |
| 6 ljudi i kapacitet | `staff = permanent + (otvoren ? seasonal_target : 0)`; `capacity = div_floor(staff·p.days·cap_tenths, 10)` |
| 7 usluženo | `served = min(demand, capacity)` |
| 8 opterećenje | `staff = 0` → `served > 0 ? 300 : 0`; inače `clamp(div_round(served·2 400, staff·p.days·cap_tenths), 0, 300)` (pri punom kapacitetu 240, kao v1) |
| 9 kazna i kvaliteta | `penalty = sim_hotel::load_penalty(load)`; `quality = div_round(4 000·exp + 3 000·morale + 3 000·(10 000 − penalty), 10 000)` (v1) |
| 10 prihod | PerGuest: `served·div_round(check·level, 100)`; Fixed: `prorate(monthly, p) + guest_nights·per_guest_night`; ShareOfRooms: `div_round(rooms_revenue·bp, 10 000)` |
| 11 troškovi | `cost_of_sales = div_round(revenue·cost_of_sales_bp, 10 000)`; `other_direct = served·direct_per_use`; `payroll = prorate(staff_svi·⌊wage/100⌋·wage_pct, p)` gdje `staff_svi = permanent + seasonal_target` (zatvoren: samo `permanent`) |
| 12 rezultat | `profit = revenue − payroll − cost_of_sales − other_direct` |
| 13 stanje prostora | `wear = prorate(−wear_pts_per_month, p)`; `load_wear = load > 180 ? prorate(−100, p) : 0`; `maint = prorate(level.equipment_pts, p)`; `maint_cost = prorate(level.cost_cents, p)`; `after = clamp(before + wear + load_wear + maint, 0, 10 000)` |
| 14 vrijednost | `value = clamp(7 000 − 100·(level − 100), 0, 10 000)` |
| 15 zadovoljstvo odjelom | `target = div_round(4 500·quality + 3 000·after + 2 500·value, 10 000)`; `sat_after = clamp(sat + div_round(step·(target − sat), 10 000), 0, 10 000)`; zatvoren ili Rentals/ShareOfRooms: ostaje `sat` |
| 16 težina | PerGuest otvoren: `served`; ostalo 0 |

`undistributed_step`: `payroll = prorate(staff·⌊wage/100⌋·wage_pct, p)`; `fixed = prorate(fixed_monthly, p)`;
`per_room = sold·per_occupied_room_cents`; `total = payroll + fixed + per_room + Σ assigned`. Koje odluke pripadaju
kojem odjelu (raspodjelu radi Task 12): Prodaja i marketing ← marketing + odgovaranje na recenzije; Održavanje ←
održavanje svih odjela; Uprava ← obuka + zapošljavanje (najam i odlasci) + troškovi situacija; Energija ← ništa.

`allocate_seasonal`: kreni od `current` (zatvoreni → 0); dok je zbroj < `total_target` i postoji odjel ispod
`max`: +1 odjelu s najvećim `plan − trenutno` (jednaki: manji kod); dok je zbroj > `total_target`: −1 odjelu s
najvećim `trenutno − plan` među onima s > 0 (jednaki: manji kod). Odjeli kojih nema u `max` imaju najviše 0.

`Catalog::validate` dodaje: postoji `rooms` s `UsaliClass::Rooms` i `RevenueModel::Rooms`; za svaku
`UndistributedKind` točno jedan odjel; svaki profil ima `rooms` i sva četiri neraspoređena; odjeli profila postoje u
katalogu; Σ `permanent` i Σ `seasonal` po profilu jednaki su v1 profilu (`base.profiles`), pa je v1 sažetak osoblja
isti; stanja i zadovoljstva u `0..=10 000`; `capacity_tenths_per_staff_day > 0` za odjele sa `seasonal_max > 0` ili
osobljem **osim neraspoređenih** (`!class.is_undistributed()` — Uprava, Prodaja, Održavanje imaju stalne ljude i
kapacitet 0, Task 10); fiksne naknade profila postoje u `economy.fixed_charges`; `90 ≤ price_level_min_pct < 100 <
price_level_max_pct ≤ 150`. `degenerate(base)` dobiva: `rooms` (wage = plaća kategorije reference, kapacitet 24,
trošenje 400, razine održavanja iz `base.actions`, `seasonal_max = base.actions.seasonal_max`, plan sezonaca iz
kategorije, izravni trošak = `cpor_cents`), `ostalo` (`Rentals`, `ShareOfRooms { bp: base.economy.other_revenue_bp }`,
bez osoblja), četiri neraspoređena bez osoblja i troškova; profili: `rooms` s v1 osobljem, opremom (`condition`) i
zadovoljstvom, `admin.fixed_monthly_cents = base fixed_cents`, bez fiksnih naknada; `guests_per_room_hundredths = 100`,
razine cijene 90–120.

- [ ] **Step 1: Testovi s ručno izračunatim brojkama (padaju)** — `tests/department.rs`:

```rust
use std::collections::BTreeMap;

use sim_hotel_v2::calendar::{PeriodKind, month_period, period_by_code};
use sim_hotel_v2::department::*;
use sim_hotel_v2::{Maintenance, Month, MonthTable};
use sim_hotel::LevelEffect;

fn restaurant() -> DeptSpec {
    DeptSpec {
        name: "Restoran".into(),
        class: UsaliClass::FoodBeverage,
        revenue: RevenueModel::PerGuest {
            usage_bp: MonthTable([4_500, 4_800, 5_000, 4_500]),
            check_cents: 3_200,
            price_elasticity_bp: -15_000,
            satisfaction_elasticity_bp: 5_000,
        },
        cost_of_sales_bp: 3_200,
        direct_per_use_cents: 0,
        wage_cents: 210_000,
        capacity_tenths_per_staff_day: 50,
        wear_pts_per_month: 300,
        maintenance: [
            LevelEffect { cost_cents: 50_000, equipment_pts: 0 },
            LevelEffect { cost_cents: 500_000, equipment_pts: 300 },
            LevelEffect { cost_cents: 1_000_000, equipment_pts: 600 },
        ],
        fixed_monthly_cents: 0,
        per_occupied_room_cents: 0,
        seasonal_max: 20,
        seasonal_plan: MonthTable([10, 12, 15, 12]),
        can_close: false,
    }
}

fn state() -> DeptState {
    DeptState { class: UsaliClass::FoodBeverage, open: true, permanent: 8, seasonal: 10, condition_pts: 7_000, satisfaction_pts: 7_000 }
}

fn plan(seasonal: u16, level: u16) -> DeptPlan {
    DeptPlan { open: true, seasonal_target: seasonal, price_level_pct: level, maintenance: Maintenance::Regular }
}

fn june(spec: &DeptSpec, st: &DeptState, pl: &DeptPlan) -> DeptOutcome {
    let p = month_period(Month::Jun);
    dept_step(&DeptInput {
        period: &p, spec, state: st, plan: pl, guest_nights: 4_666, rooms_revenue_cents: 35_000_000,
        experience_pts: 6_000, morale_pts: 5_500, wage_pct: 100, satisfaction_step_bp: 5_000,
    })
    .unwrap()
}

#[test]
fn restaurant_in_june_by_hand() {
    let o = june(&restaurant(), &state(), &plan(10, 100));
    assert_eq!((o.usage_bp, o.demand_uses, o.capacity_uses, o.served_uses), (4_500, 2_100, 2_700, 2_100));
    assert_eq!((o.load_hundredths, o.load_penalty_pts, o.quality_pts), (187, 700, 6_840));
    assert_eq!(o.revenue_cents, 6_720_000);
    assert_eq!(o.cost_of_sales_cents, 2_150_400);
    assert_eq!(o.payroll_cents, 3_780_000);
    assert_eq!(o.profit_cents, 789_600);
    assert_eq!((o.wear_pts, o.load_wear_pts, o.maintenance_pts, o.condition_after_pts), (-300, -100, 300, 6_900));
    assert_eq!(o.maintenance_cents, 500_000);
    assert_eq!((o.value_pts, o.satisfaction_target_pts, o.satisfaction_after_pts), (7_000, 6_898, 6_949));
    assert_eq!(o.guest_weight, 2_100);
}

#[test]
fn higher_price_level_raises_check_lowers_usage_and_value() {
    let o = june(&restaurant(), &state(), &plan(10, 110));
    assert_eq!((o.usage_bp, o.demand_uses), (3_825, 1_785));
    assert_eq!(o.revenue_cents, 1_785 * 3_520);
    assert_eq!(o.value_pts, 6_000);
}

#[test]
fn understaffed_restaurant_serves_capacity_and_loses_quality() {
    let o = june(&restaurant(), &state(), &plan(0, 100));
    assert_eq!((o.staff, o.capacity_uses, o.served_uses), (8, 1_200, 1_200));
    assert_eq!((o.load_hundredths, o.load_penalty_pts, o.quality_pts), (240, 6_000, 5_250));
}

#[test]
fn closed_department_has_no_revenue_but_pays_permanent_staff() {
    let mut spec = restaurant();
    spec.can_close = true;
    let mut pl = plan(0, 100);
    pl.open = false;
    let o = june(&spec, &state(), &pl);
    assert_eq!((o.served_uses, o.revenue_cents, o.guest_weight), (0, 0, 0));
    assert_eq!(o.payroll_cents, 8 * 2_100 * 100);
    assert_eq!(o.satisfaction_after_pts, 7_000);
    assert_eq!(o.condition_after_pts, 7_000 - 300 + 300);
}

#[test]
fn week_two_pays_seven_thirtieths_of_june_payroll() {
    let p = period_by_code(PeriodKind::Week, "t02").unwrap();
    let o = dept_step(&DeptInput {
        period: &p, spec: &restaurant(), state: &state(), plan: &plan(10, 100), guest_nights: 1_089,
        rooms_revenue_cents: 0, experience_pts: 6_000, morale_pts: 5_500, wage_pct: 100, satisfaction_step_bp: 1_500,
    })
    .unwrap();
    assert_eq!(o.payroll_cents, 3_780_000 * 14 / 30 - 3_780_000 * 7 / 30);
}

#[test]
fn share_of_rooms_and_fixed_revenue_models() {
    let mut s = restaurant();
    s.revenue = RevenueModel::ShareOfRooms { bp: 3_000 };
    s.class = UsaliClass::Rentals;
    let st = DeptState { permanent: 0, seasonal: 0, ..state() };
    assert_eq!(june(&s, &st, &plan(0, 100)).revenue_cents, 10_500_000);
    s.revenue = RevenueModel::Fixed { monthly_cents: 600_000, per_guest_night_cents: 150 };
    let o = june(&s, &st, &plan(0, 100));
    assert_eq!(o.revenue_cents, 600_000 + 4_666 * 150);
    assert_eq!(o.guest_weight, 0);
}

#[test]
fn undistributed_admin_by_hand() {
    let mut spec = restaurant();
    spec.class = UsaliClass::Undistributed(UndistributedKind::AdminGeneral);
    spec.revenue = RevenueModel::None;
    spec.wage_cents = 300_000;
    let st = DeptState { class: spec.class, open: true, permanent: 3, seasonal: 0, condition_pts: 10_000, satisfaction_pts: 0 };
    let assigned = AssignedCosts { training_cents: 500_000, hiring_cents: 1_500_000, situation_cents: 5_000, ..Default::default() };
    let o = undistributed_step(&DeptCode("admin".into()), &spec, &st, 3_000_000, &month_period(Month::Jun), 2_592, 100, assigned).unwrap();
    assert_eq!((o.payroll_cents, o.fixed_cents, o.per_room_cents), (900_000, 3_000_000, 0));
    assert_eq!(o.total_cents, 900_000 + 3_000_000 + 500_000 + 1_500_000 + 5_000);
}

fn map(xs: &[(&str, u16)]) -> BTreeMap<DeptCode, u16> {
    xs.iter().map(|(k, v)| (DeptCode((*k).into()), *v)).collect()
}

#[test]
fn allocate_seasonal_adds_to_largest_gap_then_smaller_code() {
    let cur = map(&[("rooms", 12), ("restaurant", 10), ("lobby_bar", 3), ("spa", 1)]);
    let plan = map(&[("rooms", 16), ("restaurant", 12), ("lobby_bar", 4), ("spa", 2)]);
    let max = map(&[("rooms", 30), ("restaurant", 20), ("lobby_bar", 8), ("spa", 6)]);
    assert_eq!(allocate_seasonal(&cur, &plan, &max, 30), map(&[("rooms", 15), ("restaurant", 11), ("lobby_bar", 3), ("spa", 1)]));
}

#[test]
fn allocate_seasonal_removes_from_largest_surplus() {
    let cur = map(&[("rooms", 12), ("restaurant", 10), ("lobby_bar", 3), ("spa", 1)]);
    let plan = map(&[("rooms", 10), ("restaurant", 10), ("lobby_bar", 3), ("spa", 1)]);
    let max = map(&[("rooms", 30), ("restaurant", 20), ("lobby_bar", 8), ("spa", 6)]);
    assert_eq!(allocate_seasonal(&cur, &plan, &max, 23), map(&[("rooms", 10), ("restaurant", 10), ("lobby_bar", 2), ("spa", 1)]));
}

#[test]
fn allocate_seasonal_never_exceeds_department_max() {
    let cur = map(&[("rooms", 0), ("spa", 0)]);
    let plan = map(&[("rooms", 50), ("spa", 50)]);
    let max = map(&[("rooms", 3), ("spa", 2)]);
    assert_eq!(allocate_seasonal(&cur, &plan, &max, 40), map(&[("rooms", 3), ("spa", 2)]));
}

#[test]
fn degenerate_catalog_validates_and_has_rooms_and_four_services() {
    let c = sim_hotel_v2::degenerate(&common_base());
    c.validate().unwrap();
    let kinds: Vec<UsaliClass> = c.departments.values().map(|d| d.class).collect();
    assert!(kinds.contains(&UsaliClass::Rooms));
    for k in [UndistributedKind::AdminGeneral, UndistributedKind::SalesMarketing, UndistributedKind::Maintenance, UndistributedKind::Utilities] {
        assert_eq!(kinds.iter().filter(|c| **c == UsaliClass::Undistributed(k)).count(), 1);
    }
}

#[path = "common/mod.rs"]
mod common;
fn common_base() -> sim_hotel::Catalog {
    common::test_catalog()
}
```

- [ ] **Step 2:** `cargo test -p sim-hotel-v2 --test department` → FAIL.
- [ ] **Step 3: Implementiraj** `department.rs` i dopune `catalog.rs` (uključujući `validate` i `degenerate`).
  Postojeći testovi paketa (ekvivalencija, razdoblja) ostaju zeleni — `degenerate` se proširuje, obračun ga još ne čita.
- [ ] **Step 4:** `cargo test -p sim-hotel-v2; echo EXIT=$?` → zeleno; `node scripts/probe-mutacija.mjs; echo EXIT=$?`.
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/9: odjeli po USALI-ju u jezgri v2 — tipovi i formule odjela".

---
