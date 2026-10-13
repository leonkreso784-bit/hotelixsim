### Task 7: App i API — nova igra s postavkama, kodovi kruga, polja `round`

**Veličina:** M, 3–4 h · **Ovisi o:** Task 6 · **Paralelno s:** Task 8, Task 9 · **Model:** sonnet

**Cilj:** `POST /v1/demo` prima neobavezne postavke (`model`, `round_kind`); sve putanje s `{month}` primaju i kod
tjedna; tijela dobivaju `round` (kod i kalendar kruga), a postojeće polje `month` ostaje (za tjedan: mjesec u kojem
krug počinje).

**Files:**
- Modify: `crates/app/src/{lib,demo,views,commands,memory}.rs`, `crates/server/src/main.rs`,
  `crates/api/src/{demo,games,hotels,schema,kpi,explanation,problem}.rs`, `openapi.json`,
  `packages/api-client/src/schema.d.ts`
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

`common::cookie_value` dodaj u `crates/api/tests/common/mod.rs` ako ne postoji (izvuče vrijednost
`hotelixsim_guest=…` do `;`). `Memory::patch_first_event(&self, game_id: &str, f: impl FnOnce(&mut serde_json::Value))`
dodaj u `crates/app/src/memory.rs` s komentarom „samo za testove: mijenja spremljeni GameCreated".

- [ ] **Step 2:** `cargo test -p api --test rounds_v2` → FAIL.
- [ ] **Step 3: Implementiraj** (app → api), `RoundInfoBody::from(&RoundInfo)` iz `period.first_date()`,
  `last_date()`, `month_ending_in().is_some()`; `month` polja iz `round.period.start_month()`.
- [ ] **Step 4:** `corepack pnpm gen:api; echo EXIT=$?`, zatim `cargo test --workspace; echo EXIT=$?` i
  `corepack pnpm typecheck; echo EXIT=$?` (web se mora prevesti s dodanim poljima — samo dodana, pa prolazi).
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/7: nova igra s postavkama modela i kruga; kodovi tjedna u putanjama".

**Prihvaćanje:** svi stari API testovi zeleni bez promjene očekivanja (osim novih polja); `openapi.json` u diffu
samo dodaje (provjeri `git diff openapi.json | grep '^-' | grep -v '^---'` — samo opis parametra `{month}` i naslov).

---

