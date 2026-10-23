# 2026-10-10-v2-sloj1-kartice — dio 5

Arhiva (zamrznuto). Indeks: [2026-10-10-v2-sloj1-kartice.md](../2026-10-10-v2-sloj1-kartice.md).

### Task 14: Igra — odjeli, račun dobiti, budžet i pokazatelji za v2

**Veličina:** M, 3–4 h · **Ovisi o:** Task 7, Task 8 (pomoćnik `common::v2_weekly_game_resolved_through`), Task 12,
Task 13 · **Paralelno s:** Task 16 · **Model:** sonnet

**Cilj:** `game` sprema račune odjela u obračunu i nudi poglede: odjeli hotela, jedan odjel s poviješću, račun dobiti
kruga ili sezone uz budžet (🟨 P8), pokazatelji sloja 0 s danima razdoblja, i granice plana po odjelu za ekran plana.

**Files:**
- Create: `crates/game/src/departments.rs`, `crates/game/tests/departments.rs`
- Modify: `crates/game/src/{lib,model,dto_v2,kpi,error,game,command}.rs` (`command.rs`: polja `PlanLimits`, niže),
  `crates/game/src/view.rs` samo ako `PlanState` treba prilagodbu zbog uklonjenog `Copy`,
  `crates/game/tests/common/mod.rs` (grana `"none"` u `v2_weekly_game_resolved_through`)
- Ne dirati: jezgra, `crates/api/**` (`From<&PlanLimits> for PlanLimitsBody`, `crates/api/src/schema.rs:202-215`, čita
  polja po imenu pa se prevodi i s novim poljima; tijelo proširuje Task 15), `crates/app/**` (osim mapiranja nove
  greške u `crates/app/src/error.rs`).

**Interfaces:**
- Consumes: Task 12 (`PeriodReport.accounts`), Task 13 (`statement`, `sum_statements`, `budget_variance`).
- Produces:

```rust
// crates/game/src/dto_v2.rs
pub struct DetailDto { pub after: HotelStateV2Dto, #[serde(default, skip_serializing_if = "Option::is_none")] pub accounts: Option<PeriodAccountsDto> }
// PeriodAccountsDto, DeptOutcomeDto, UndistributedOutcomeDto — serde-zrcala polje po polje

// crates/game/src/departments.rs
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepartmentView {
    pub code: String, pub name: String, pub class: UsaliClass, pub open: bool, pub can_close: bool,
    pub permanent: u16, pub seasonal: u16, pub condition_pts: i32,
    /// `None` za odjele bez gostiju (neraspoređeni, najam, parking).
    pub guest_satisfaction_pts: Option<i32>,
    pub last: Option<(RoundKey, DeptOutcome)>,
    pub undistributed_last: Option<(RoundKey, UndistributedOutcome)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepartmentsView { pub round: Option<RoundKey>, pub departments: Vec<DepartmentView> }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepartmentDetail { pub department: DepartmentView, pub history: Vec<(RoundKey, DeptOutcome)> }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlScope { Round, Season }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlView {
    pub round: RoundKey, pub scope: PlScope,
    pub actual: UsaliStatement, pub budget: UsaliStatement, pub lines: Vec<VarianceLine>,
    /// Ime za svaki kod retka (odjeli iz profila, naknade iz kataloga).
    pub names: BTreeMap<String, String>,
}
impl Game {
    pub fn departments(&self, h: HotelId) -> Result<DepartmentsView, GameError>;           // v1 → WrongModel
    pub fn department(&self, h: HotelId, code: &str) -> Result<DepartmentDetail, GameError>; // UnknownDepartment
    /// `round = None` → zadnji obračunati krug; nijedan obračunat → `NothingResolved`.
    pub fn profit_and_loss(&self, h: HotelId, round: Option<RoundKey>, scope: PlScope) -> Result<PlView, GameError>;
}
// crates/game/src/error.rs — dodano: UnknownDepartment, NothingResolved

// crates/game/src/command.rs — `PlanLimits` (command.rs:194) dobiva dva polja; derive postaje
// `Clone, Debug, PartialEq, Eq` (bez `Copy`, zbog mape). v1 `plan_limits` (command.rs:206) puni None / praznu mapu;
// v2 puni `Model::plan_limits` iz `sim_hotel_v2::PlanLimits` (Task 11). Tako granice odjela stižu u
// `view::PlanState.limits` → API `PlanLimitsBody` (Task 15) → ekran plana (Task 21).
pub struct PlanLimits { /* …postojeća polja… */
    /// v2: najveći Σ|cilj − trenutno| sezonaca po odjelima u krugu (`coeffs(kind).seasonal_step`); v1: `None`.
    pub seasonal_step_total: Option<u16>,
    /// v2: granice po kodu odjela (svi odjeli plana); v1: prazno.
    pub departments: BTreeMap<String, DeptLimitsView>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeptLimitsView { pub seasonal_min: u16, pub seasonal_max: u16, pub price_level_min_pct: u16, pub price_level_max_pct: u16, pub can_close: bool }
```

Budžet (🟨 P8): `Model::budget_accounts(content, profile, rounds)` (u `model.rs`) vrti `run_season_with` od početnog
stanja profila, sa `PeriodPlan::default_for` u svakom krugu i bez situacija, po istom kalendaru igre; računa se pri
čitanju (ne sprema se). Za `PlScope::Season` zbrajaju se krugovi od prvog do traženog (`sum_statements`) i za
ostvareno i za budžet. Pokazatelji (`kpi.rs`, sloj 0): `days` iz `round_info`, izravni trošak po noći za v2 iz odjela
Smještaj (`direct_per_use_cents`), kroz `Model::rooms_direct_cost`.

- [ ] **Step 1: Testovi (padaju)** — `crates/game/tests/departments.rs` (koristi `common::v2_weekly_game_resolved_through`
  iz Taska 8):

```rust
mod common;

use game::*;

#[test]
fn departments_of_profile_after_first_week() {
    let g = common::v2_weekly_game_resolved_through("t01");
    let h = common::hotel_ids()[0];
    let v = g.departments(h).unwrap();
    assert_eq!(v.round, Some(RoundKey::Week(1)));
    assert!(v.departments.iter().any(|d| d.code == "rooms" && d.last.is_some()));
    let admin = v.departments.iter().find(|d| d.code == "admin").unwrap();
    assert_eq!(admin.guest_satisfaction_pts, None);
    assert!(admin.undistributed_last.is_some());
}

#[test]
fn department_history_has_one_entry_per_resolved_round() {
    let g = common::v2_weekly_game_resolved_through("t03");
    let d = g.department(common::hotel_ids()[0], "rooms").unwrap();
    assert_eq!(d.history.iter().map(|(r, _)| r.code()).collect::<Vec<_>>(), ["t01", "t02", "t03"]);
    assert!(matches!(g.department(common::hotel_ids()[0], "casino"), Err(GameError::UnknownDepartment)));
}

#[test]
fn pl_lines_align_actual_and_budget() {
    let g = common::v2_weekly_game_resolved_through("t02");
    let pl = g.profit_and_loss(common::hotel_ids()[0], None, PlScope::Round).unwrap();
    assert_eq!(pl.round, RoundKey::Week(2));
    assert_eq!(pl.actual.ebitda_cents, pl.actual.gop_cents - pl.actual.total_fixed_charges_cents);
    for l in &pl.lines {
        assert_eq!(l.variance_cents, l.actual_cents - l.budget_cents);
        assert!(pl.names.contains_key(&l.code) || l.is_total || ["gop", "ebitda"].contains(&l.code.as_str()), "{}", l.code);
    }
    let gop = pl.lines.iter().find(|l| l.code == "gop").unwrap();
    assert_eq!(gop.actual_cents, g.resolved[&RoundKey::Week(2)].hotels[0].report.money.gop_cents);
}

#[test]
fn season_scope_sums_rounds() {
    let g = common::v2_weekly_game_resolved_through("t02");
    let h = common::hotel_ids()[0];
    let one = g.profit_and_loss(h, Some(RoundKey::Week(1)), PlScope::Round).unwrap();
    let two = g.profit_and_loss(h, Some(RoundKey::Week(2)), PlScope::Round).unwrap();
    let season = g.profit_and_loss(h, Some(RoundKey::Week(2)), PlScope::Season).unwrap();
    assert_eq!(season.actual.gop_cents, one.actual.gop_cents + two.actual.gop_cents);
}

#[test]
fn v1_game_has_no_departments_and_no_pl() {
    let (g, h) = common::open_june_game();
    assert_eq!(g.departments(h).unwrap_err(), GameError::WrongModel);
    assert_eq!(g.profit_and_loss(h, None, PlScope::Round).unwrap_err(), GameError::WrongModel);
}

#[test]
fn pl_before_first_resolution_is_nothing_resolved() {
    let g = common::v2_weekly_game_resolved_through("none");
    assert_eq!(g.profit_and_loss(common::hotel_ids()[0], None, PlScope::Round).unwrap_err(), GameError::NothingResolved);
}

#[test]
fn plan_limits_carry_department_limits_only_for_v2() {
    let g = common::v2_weekly_game_resolved_through("none");
    let l = g.private_hotel(common::hotel_ids()[0]).unwrap().plan.unwrap().limits;
    assert!(l.seasonal_step_total.is_some_and(|s| s > 0)); // `periods.json` week.seasonal_step (Task 17 ga smije mijenjati)
    assert!(!l.departments["restaurant"].can_close); // restoran ima svaki profil (Task 10)
    assert!(l.departments.contains_key("rooms"));
    let (v1, h) = common::open_june_game();
    let l1 = v1.private_hotel(h).unwrap().plan.unwrap().limits;
    assert_eq!(l1.seasonal_step_total, None);
    assert!(l1.departments.is_empty());
}

#[test]
fn weekly_kpi_uses_seven_days() {
    let g = common::v2_weekly_game_resolved_through("t01");
    let k = g.hotel_kpi(common::hotel_ids()[0]).unwrap();
    assert_eq!(k[0].kpi.room_nights, 120 * 7);
}
```

(`v2_weekly_game_resolved_through("none")` vraća igru bez ijednog obračuna — dodaj tu granu u pomoćnik.)

- [ ] **Step 2:** `cargo test -p game --test departments` → FAIL.
- [ ] **Step 3: Implementiraj.** Grananje (v1 → `WrongModel`, budžet, izravni trošak) samo u `model.rs`.
- [ ] **Step 4:** `cargo test --workspace; echo EXIT=$?` → zeleno.
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/14: igra — odjeli, račun dobiti s budžetom, pokazatelji po razdoblju".

---

### Task 15: API — odjeli, račun dobiti, plan s odjelima

**Veličina:** M, 3–4 h · **Ovisi o:** Task 14 · **Paralelno s:** Task 17 · **Model:** sonnet

**Cilj:** tri nove rute samo za vlastiti hotel i dodana polja plana; v1 igra dobiva 409 `model_v1`.

**Files:**
- Create: `crates/api/src/departments.rs`, `crates/api/tests/departments.rs`
- Modify: `crates/api/src/{lib,schema,hotels,problem}.rs`, `crates/app/src/{views,commands,error}.rs`, `openapi.json`,
  `packages/api-client/src/schema.d.ts`
- Ne dirati: jezgra, `crates/game/**` (osim sitnog gettera), web, `crates/api/tests/common/mod.rs` (pomoćnici
  `cookie_value` i `run_worker` već postoje od Taska 7).

**Interfaces (Produces):**

```rust
// GET /v1/hotels/{hotel_id}/departments → 200 DepartmentsBody · 401/403/404 · 409 model_v1
#[derive(Serialize, ToSchema)]
pub struct DepartmentsBody { #[schema(required = true)] pub round: Option<String>, pub departments: Vec<DepartmentBody> }
#[derive(Serialize, ToSchema)]
pub struct DepartmentBody {
    pub code: String, pub name: String, pub class: DeptClassCode, pub open: bool, pub can_close: bool,
    pub permanent: u16, pub seasonal: u16,
    /// Stanje prostora 0–10 000 (na ekranu 0–100).
    pub condition_pts: i32,
    /// Zadovoljstvo gostiju odjelom 0–10 000; `null` za odjele bez gostiju.
    #[schema(required = true)] pub guest_satisfaction_pts: Option<i32>,
    #[schema(required = true)] pub last: Option<DeptRoundBody>,
    /// Neraspoređeni odjel: trošak zadnjeg kruga po stavkama.
    #[schema(required = true)] pub service_last: Option<ServiceRoundBody>,
}
#[derive(Serialize, ToSchema)]
pub struct DeptRoundBody {
    pub round: String, pub revenue_cents: i64, pub payroll_cents: i64, pub cost_of_sales_cents: i64, pub other_cents: i64,
    pub profit_cents: i64, #[schema(required = true)] pub margin_bp: Option<i64>,
    pub served: u32, pub capacity: u32, pub load_hundredths: u16, pub usage_bp: u16, pub quality_pts: i32, pub price_level_pct: u16,
    pub condition_before_pts: i32, pub wear_pts: i32, pub load_wear_pts: i32, pub maintenance_pts: i32, pub condition_after_pts: i32,
    pub maintenance_cents: i64, pub value_pts: i32,
    pub satisfaction_before_pts: i32, pub satisfaction_after_pts: i32,
}
#[derive(Serialize, ToSchema)]
pub struct ServiceRoundBody { pub round: String, pub payroll_cents: i64, pub fixed_cents: i64, pub per_room_cents: i64, pub marketing_cents: i64, pub reply_cents: i64, pub maintenance_cents: i64, pub training_cents: i64, pub hiring_cents: i64, pub situation_cents: i64, pub total_cents: i64 }
#[derive(Serialize, ToSchema)] #[serde(rename_all = "snake_case")]
pub enum DeptClassCode { Rooms, FoodBeverage, OtherOperated, Rentals, AdminGeneral, SalesMarketing, Maintenance, Utilities }

// GET /v1/hotels/{hotel_id}/departments/{code} → 200 DepartmentDetailBody · 404 unknown_department · 409 model_v1
#[derive(Serialize, ToSchema)]
pub struct DepartmentDetailBody { pub department: DepartmentBody, pub history: Vec<DeptRoundBody> }

// GET /v1/hotels/{hotel_id}/pl?round=t03&scope=round|season → 200 PlBody · 409 model_v1 · 409 no_resolved_round · 404 unknown_month
// (ime upita `round`, ne `period` iz MODEL_V2 §6.5 — 🟨 P14; bez `round` = zadnji obračunati krug, bez `scope` = round)
#[derive(Serialize, ToSchema)]
pub struct PlBody {
    pub round: String, pub scope: PlScopeCode, pub days: u32, pub room_nights: u32,
    /// Obračunati krugovi (za izbornik), redom.
    pub rounds: Vec<String>,
    pub sections: Vec<PlSectionBody>,
    pub gop_margin_bp: PlRatioBody, pub goppar_cents: PlRatioBody, pub ebitda_margin_bp: PlRatioBody,
}
#[derive(Serialize, ToSchema)] pub struct PlSectionBody { pub section: PlSectionCode, pub lines: Vec<PlLineBody>, #[schema(required = true)] pub total: Option<PlLineBody> }
#[derive(Serialize, ToSchema)] pub struct PlLineBody { pub code: String, pub name: String, pub kind: PlLineKindCode, pub actual_cents: i64, pub budget_cents: i64, pub variance_cents: i64, pub favorable: bool }
#[derive(Serialize, ToSchema)] pub struct PlRatioBody { #[schema(required = true)] pub actual: Option<i64>, #[schema(required = true)] pub budget: Option<i64> }
// PlSectionCode: revenue, departmental_expenses, departmental_profit, undistributed, gop, fixed_charges, ebitda
// PlLineKindCode: revenue, expense, result; PlScopeCode: round, season
// `name` za GOP i EBITDA retke: „Bruto operativna dobit (GOP)" i „EBITDA (dobit prije kamata, poreza i amortizacije)".

// Plan — samo dodano:
// PlanBody { #[serde(default, skip_serializing_if = "Option::is_none")] departments: Option<BTreeMap<String, DeptPlanBody>> }
// DeptPlanBody { open: bool, seasonal_target: u16, price_level_pct: u16, maintenance: MaintenanceCode }
// PlanLimitsBody { seasonal_step_total: Option<u16>, departments: Option<BTreeMap<String, DeptLimitsBody>> }
//   — iz `game::command::PlanLimits.{seasonal_step_total, departments}` (Task 14); prazna mapa → `None`
// DeptLimitsBody { seasonal_min: u16, seasonal_max: u16, price_level_min_pct: u16, price_level_max_pct: u16, can_close: bool }
```

Pravila: tuđi hotel → 403 `not_your_hotel` (MODEL_V2 🟨 13); v2 `PUT plans/{round}` bez `departments` zadržava
postavke odjela iz važećeg plana (to već radi `Game::plan_from_dto` iz Taska 11 — `PlanBody.departments = None` →
prazna mapa u `PlanDto`); v1 `PUT` s `departments` → 422 `invalid_body`; odjelne greške plana → 422
`department_plan_invalid` „Postavke odjela nisu dopuštene: …" (poruka po vrsti greške u `problem.rs`): u
`crates/app/src/error.rs` šest odjelnih varijanti `sim_hotel_v2::PlanError` (`DepartmentUnknown`, `DepartmentMissing`,
`DepartmentCannotClose`, `DepartmentSeasonalOutOfRange`, `ClosedWithSeasonal`, `PriceLevelOutOfRange`) prelaze iz
privremenog `ErrorCode::Internal` (Task 11) u novi `ErrorCode::DepartmentPlanInvalid` (i `ErrorCode::ALL` raste za
jedan); ostale varijante `PlanV2` ostaju na v1 kodovima (Task 6). Redoslijed provjera (Task 11) jamči da plan koji
zatvara restoran dobiva grešku odjela, ne grešku koraka.

- [ ] **Step 1: Testovi (padaju)** — `crates/api/tests/departments.rs`:

```rust
mod common;

use serde_json::json;

async fn v2_after_first_week() -> (axum::Router, String, String, String) {
    let (app, deps, _) = common::app();
    let r = common::call(&app, "POST", "/v1/demo", None, Some(json!({ "model": "v2" }))).await;
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let g = r.body["game_id"].as_str().unwrap().to_owned();
    let h = r.body["hotel_id"].as_str().unwrap().to_owned();
    let close = common::call(&app, "POST", &format!("/v1/games/{g}/rounds/t01/close"), Some(&c), None).await;
    assert_eq!(close.status, 202);
    common::run_worker(&deps).await; // Task 7: premješten iz e2e.rs/games.rs u common/mod.rs
    (app, c, g, h)
}

#[tokio::test]
async fn departments_list_has_rooms_with_last_round() {
    let (app, c, _, h) = v2_after_first_week().await;
    let r = common::call(&app, "GET", &format!("/v1/hotels/{h}/departments"), Some(&c), None).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    assert_eq!(r.body["round"], "t01");
    let rooms = r.body["departments"].as_array().unwrap().iter().find(|d| d["code"] == "rooms").unwrap();
    assert_eq!(rooms["name"], "Smještaj");
    assert_eq!(rooms["class"], "rooms");
    assert!(rooms["last"]["revenue_cents"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn department_detail_and_unknown_department() {
    let (app, c, _, h) = v2_after_first_week().await;
    let r = common::call(&app, "GET", &format!("/v1/hotels/{h}/departments/restaurant"), Some(&c), None).await;
    assert_eq!(r.status, 200);
    assert_eq!(r.body["history"].as_array().unwrap().len(), 1);
    let r = common::call(&app, "GET", &format!("/v1/hotels/{h}/departments/casino"), Some(&c), None).await;
    assert_eq!((r.status.as_u16(), r.body["code"].as_str()), (404, Some("unknown_department")));
}

#[tokio::test]
async fn pl_has_sections_in_usali_order_and_aligned_lines() {
    let (app, c, _, h) = v2_after_first_week().await;
    let r = common::call(&app, "GET", &format!("/v1/hotels/{h}/pl"), Some(&c), None).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    let sections: Vec<&str> = r.body["sections"].as_array().unwrap().iter().map(|s| s["section"].as_str().unwrap()).collect();
    assert_eq!(sections, ["revenue", "departmental_expenses", "departmental_profit", "undistributed", "gop", "fixed_charges", "ebitda"]);
    for s in r.body["sections"].as_array().unwrap() {
        for l in s["lines"].as_array().unwrap() {
            assert_eq!(l["variance_cents"].as_i64().unwrap(), l["actual_cents"].as_i64().unwrap() - l["budget_cents"].as_i64().unwrap());
            assert!(!l["name"].as_str().unwrap().is_empty());
        }
    }
    assert_eq!(r.body["rounds"], json!(["t01"]));
}

#[tokio::test]
async fn pl_before_any_resolution_is_409() {
    let (app, _, _) = common::app();
    let r = common::call(&app, "POST", "/v1/demo", None, Some(json!({ "model": "v2" }))).await;
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let h = r.body["hotel_id"].as_str().unwrap().to_owned();
    let r = common::call(&app, "GET", &format!("/v1/hotels/{h}/pl"), Some(&c), None).await;
    assert_eq!((r.status.as_u16(), r.body["code"].as_str()), (409, Some("no_resolved_round")));
}

#[tokio::test]
async fn v1_game_has_no_departments() {
    let (app, _, _) = common::app();
    let r = common::call(&app, "POST", "/v1/demo", None, None).await;
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let h = r.body["hotel_id"].as_str().unwrap().to_owned();
    let r = common::call(&app, "GET", &format!("/v1/hotels/{h}/departments"), Some(&c), None).await;
    assert_eq!((r.status.as_u16(), r.body["code"].as_str()), (409, Some("model_v1")));
}

#[tokio::test]
async fn other_hotel_departments_are_forbidden() {
    let (app, c, g, _) = v2_after_first_week().await;
    let game = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    let other = game.body["hotels"][1]["hotel_id"].as_str().unwrap().to_owned();
    let r = common::call(&app, "GET", &format!("/v1/hotels/{other}/departments"), Some(&c), None).await;
    assert_eq!(r.status, 403);
}

#[tokio::test]
async fn plan_with_departments_round_trips_and_bad_closing_is_422() {
    let (app, c, _, h) = v2_after_first_week().await;
    let me = common::call(&app, "GET", "/v1/me/hotel", Some(&c), None).await;
    let mut plan = me.body["private"]["plan"]["plan"].clone();
    assert!(plan["departments"]["restaurant"].is_object()); // restoran ima svaki profil i ne smije se zatvoriti
    // Profil gosta ovisi o sjemenu (`game:profile`), pa bar biramo iz granica: svaki profil ima odjel koji se smije zatvoriti.
    let limits = me.body["private"]["plan"]["limits"]["departments"].as_object().unwrap();
    let bar = limits.iter().find(|(_, l)| l["can_close"] == true).map(|(k, _)| k.clone()).unwrap();
    plan["departments"][&bar]["price_level_pct"] = json!(110);
    let r = common::call(&app, "PUT", &format!("/v1/hotels/{h}/plans/t02"), Some(&c), Some(plan.clone())).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    assert_eq!(r.body["departments"][&bar]["price_level_pct"], 110);
    plan["departments"]["restaurant"]["open"] = json!(false);
    plan["departments"]["restaurant"]["seasonal_target"] = json!(0);
    let r = common::call(&app, "PUT", &format!("/v1/hotels/{h}/plans/t02"), Some(&c), Some(plan)).await;
    assert_eq!((r.status.as_u16(), r.body["code"].as_str()), (422, Some("department_plan_invalid")));
}
```

- [ ] **Step 2:** `cargo test -p api --test departments` → FAIL.
- [ ] **Step 3: Implementiraj** (app pogledi i greške → api tijela i rute); `corepack pnpm gen:api`.
- [ ] **Step 4:** `cargo test --workspace; echo EXIT=$?`; `corepack pnpm check:api; echo EXIT=$?`;
  `corepack pnpm typecheck; echo EXIT=$?`.
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/15: API — odjeli, račun dobiti s budžetom, plan po odjelima".

---

### Task 16: Balans v2 — pokretač i nove provjere

**Veličina:** M, 4–5 h · **Ovisi o:** Task 12, Task 13 · **Paralelno s:** Task 14, 15 · **Model:** sonnet

**Cilj:** `balance` vrti v2 za tjedan i mjesec, ispisuje izvještaj i provjerava DEMO §11.1–11.5 i nove kriterije
sloja 1. U ovom tasku provjere **smiju padati** (kalibrira ih Task 17); prihvaćanje je da alat radi i ispisuje.

**Files:**
- Create: `crates/balance/src/v2.rs`, `crates/balance/tests/balance_v2.rs`
- Modify: `crates/balance/src/{lib,main}.rs`, `crates/balance/Cargo.toml` (`sim-hotel-v2.workspace = true`)
- Ne dirati: v1 dio `balance` (izlaz `cargo run -p balance --release -- --seeds 1000 --check` mora ostati bajt-isti),
  sadržaj, jezgra.

**Interfaces (Produces):**

```rust
// crates/balance/src/main.rs: nove zastavice
//   --model v1|v2   (zadano v1; v1 put potpuno nepromijenjen)
//   --round month|week (samo uz v2; zadano week)
//   --content DIR   (za v2 čita DIR i DIR/v2)
// crates/balance/src/v2.rs
pub struct OptionsV2 { pub seeds: u64, pub kind: PeriodKind }
pub struct ReportV2 {
    pub starts: Vec<(Profile, IndexSet)>,
    pub reference_season_gop_margin_bp: i32,
    pub reference_season_ebitda_margin_bp: i32,
    pub reference_dept_margins_bp: Vec<(DeptCode, UsaliClass, Option<i32>)>,  // sezona, referentni profil, zadani plan
    pub reference_labor_share_bp: i32,
    /// Profil C, sezona, zadani plan: Σ prihoda svih odjela osim Smještaja (hrana i piće, ostali operativni, najam,
    /// parking — isto što sažetak zove `other_revenue_cents`) / prihod soba, u bp (MODEL_V2 §8: „C: ostali ≈ 30 %").
    pub reference_other_revenue_share_bp: i32,
    pub medians: Vec<(Profile, Strategy, i32)>, pub profile_medians: Vec<(Profile, i32)>,
    pub probes: Vec<(String, Vec<(Profile, i32)>)>,
    pub week_vs_month: Vec<(Profile, Strategy, i32, i32)>, // medijan napretka tjedno, mjesečno
    pub reproducible: bool,
}
pub fn run_v2(content: &content::Content, opts: &OptionsV2) -> Result<ReportV2, sim_hotel_v2::ResolveError>;
pub fn failures_v2(r: &ReportV2) -> Vec<String>;
pub fn render_v2(r: &ReportV2) -> String;
```

Provjere (`failures_v2`): §11.1 početni indeksi kao v1 (ciljevi `Profile::starting_indices`, tolerancija C ±100, A/B
±200); **GOP marža sezone referentnog hotela 3 500–4 500 bp** (🟨 P4); EBITDA marža ispod GOP marže; marže odjela
(sezona, referentni profil, zadani plan): Smještaj 6 500–8 000, hrana i piće 1 500–3 500 (svaki odjel tog razreda),
wellness/ostali operativni 2 000–4 000 [PROC, MODEL_V2 §8]; udio troška rada u ukupnom prihodu 3 000–4 000 bp;
**ostali prihodi profila C ≈ 30 % prihoda od soba:** `reference_other_revenue_share_bp` u **2 500–3 500 bp** (MODEL_V2
§8 sloj 1, „Kalibracija: za C ostali odjeli ≈ 30 % prihoda soba"). Tolerancija ±500 bp jer: (a) v1 to drži ravno na
3 000 (`catalog.json` `other_revenue_bp`), a indeksi i ljestvica profila C su kalibrirani na tom omjeru prihoda — veće
odstupanje mijenja GOP i financijski indeks C-a toliko da §11.1 (C ±100) postaje upitan; (b) u v2 je omjer izveden
(noćenja × udio korištenja × prosječni račun, s utjecajem razine cijene i zadovoljstva, cjelobrojno zaokruživanje po
krugu), pa točnih 3 000 nije realan cilj; (c) ±5 postotnih bodova je ista širina kao pojas GOP marže (35–45 %) i
ostali [PROC] pojasevi ovog popisa;
§11.3 pravednost (razlika medijana profila ≤ 499, nijedan profil najbolji pod svim strategijama); §11.4 nijedna
proba strogo najbolja za sva tri profila — probe v1 (`Probe::ALL` preneseni na `PeriodPlan`) **plus** odjelne: „zatvori
sve odjele koji se smiju zatvoriti", „razina cijene 120 % u svim odjelima", „razina cijene 90 %", „održavanje odgođeno
svugdje", „održavanje pojačano svugdje", „sezonci samo u Smještaju"; **tjedno ≈ mjesečno:** za svaki profil i
strategiju |medijan tjedno − medijan mjesečno| ≤ 300 bodova (stotinki; 3 boda) [PROC]; §11.5 ponovljivost (sjeme 7,
dvaput isto).

- [ ] **Step 1: Testovi (padaju)** — `crates/balance/tests/balance_v2.rs`:

```rust
use balance::v2::{OptionsV2, failures_v2, render_v2, run_v2};
use content::{Snapshot, repo_dir};
use sim_hotel_v2::PeriodKind;

fn content_v2() -> content::Content {
    Snapshot::read_v2(&repo_dir()).unwrap().build().unwrap()
}

#[test]
fn v2_balance_runs_for_both_kinds_and_is_reproducible() {
    for kind in [PeriodKind::Week, PeriodKind::Month] {
        let r = run_v2(&content_v2(), &OptionsV2 { seeds: 3, kind }).unwrap();
        assert!(r.reproducible);
        assert_eq!(r.starts.len(), 3);
        assert!(render_v2(&r).contains("Marže odjela"));
        assert!(render_v2(&r).contains("Ostali prihodi C / prihod soba"));
        assert!(r.reference_other_revenue_share_bp > 0);
        let _ = failures_v2(&r); // u ovom tasku smiju postojati
    }
}

#[test]
fn degenerate_v2_monthly_balance_matches_v1_numbers() {
    let s = Snapshot::read(&repo_dir()).unwrap();
    let c = s.degenerate_v2().build().unwrap();
    let r2 = run_v2(&c, &OptionsV2 { seeds: 5, kind: PeriodKind::Month }).unwrap();
    let r1 = balance::run(&s.build().unwrap(), &balance::Options { seeds: 5 }).unwrap();
    assert_eq!(r2.starts, r1.starts);
    assert_eq!(r2.profile_medians, r1.profile_medians);
    // Degenerirani katalog: jedan odjel `ostalo` = ShareOfRooms 3 000 bp (v1 `other_revenue_bp`) → omjer 30 % do
    // zaokruživanja po krugu.
    assert!((r2.reference_other_revenue_share_bp - 3_000).abs() <= 1, "{}", r2.reference_other_revenue_share_bp);
}
```

- [ ] **Step 2:** `cargo test -p balance --test balance_v2` → FAIL.
- [ ] **Step 3: Implementiraj** (`run_v2` kao `run` u v1, nad `sim_hotel_v2::run_season`; marže iz
  `usali::statement` + `sum_statements` po sezoni; probe nad `PeriodPlan`). `reference_other_revenue_share_bp` =
  `div_round(Σ revenue_cents odjela s class ≠ Rooms · 10 000, revenue_cents Smještaja)` iz istog sezonskog zbroja
  (profil C, zadani plan, bez situacija); `render_v2` ga ispisuje kao redak „Ostali prihodi C / prihod soba: 29,4 %"
  (ime i jedinica).
- [ ] **Step 4:** `cargo run -q -p balance --release -- --seeds 1000 --check; echo EXIT=$?` → EXIT=0 i izlaz isti kao
  prije taska (prije prve izmjene spremi izlaz te naredbe u datoteku u scratchpadu i na kraju usporedi s `diff`);
  `cargo run -q -p balance --release -- --model v2 --round week --seeds 100; echo EXIT=$?` → ispisuje izvještaj
  (EXIT=0 bez `--check`); zapiši popis provjera koje padaju u izvještaj taska — to je ulaz za Task 17.
- [ ] **Step 5: Sve provjere i commit** „Sloj 1/16: balans v2 — tjedan i mjesec, marže odjela, udio rada, tjedno prema mjesečnom".

---

### Task 17: Kalibracija sadržaja v2

**Veličina:** M–L, 3–6 h (iterativno) · **Ovisi o:** Task 16, Task 10 · **Paralelno s:** Task 15 · **Model:** opus
(prosudba o brojkama)

**Cilj:** `--check` prolazi za v2 tjedno i mjesečno (1000 sjemena), a v1 ostaje kakav je.

**Files:**
- Modify: **samo** `content/hotel/v2/{departments,profiles,economy,periods}.json`, `.github/workflows/ci.yml` (dva
  nova retka balansa s `--seeds 100`)
- Ne dirati: nijedan `.rs` (ako provjera ne može proći bez promjene formule ili praga → stani, zapiši u izvještaj s
  brojkama i prijedlogom; voditelj donosi Ruling).

**Postupak:**
- [ ] **Step 1:** Ulaz: popis padova iz Taska 16. Za svaku brojku koju mijenjaš zapiši u izvještaj: staro → novo,
  koja provjera, zašto je nova vrijednost uvjerljiva za hrvatski hotel 4* na moru (oznaka [PROC]).
- [ ] **Step 2: Redoslijed podešavanja** (svaki krug: `cargo run -q -p balance --release -- --model v2 --round week --seeds 200`):
  (1) početni indeksi profila (stanja odjela, zadovoljstva, raspodjela osoblja); (2) marže odjela (prosječni račun,
  udio korištenja, trošak namirnica, plaće odjela) i **ostali prihodi C 25–35 % prihoda soba** (udio korištenja i
  prosječni račun odjela profila C, najam lokala, parking); (3) GOP marža 35–45 % (stalni dio neraspoređenih, energija po noći,
  fiksne naknade se **ne** diraju da bi se postigla GOP marža — one su ispod GOP-a); (4) udio rada 30–40 %; (5) tjedni
  koeficijenti (`periods.json` `week`) za tjedno ≈ mjesečno; (6) pravednost i dominantne probe.
- [ ] **Step 3: Završna provjera** (svaka zasebno, EXIT=0):
  `cargo run -q -p balance --release -- --seeds 1000 --check` ·
  `cargo run -q -p balance --release -- --model v2 --round week --seeds 1000 --check` ·
  `cargo run -q -p balance --release -- --model v2 --round month --seeds 1000 --check` ·
  `cargo test --workspace` (testovi s brojkama iz sadržaja v2 — npr. `content --test v2_departments` — ne smiju ovisiti
  o vrijednostima koje si promijenio; ako ovise, test je loš: javi).
- [ ] **Step 4:** U `.github/workflows/ci.yml` iza postojećeg retka balansa dodaj
  `- run: cargo run -q -p balance --release -- --model v2 --round week --seeds 100 --check` i isto s `--round month`.
- [ ] **Step 5: Commit** „Sloj 1/17: kalibracija v2 — balans prolazi za tjedan i mjesec" s tablicom promjena u poruci.

---

### Task 18: Web — tjedni krugovi i postavke nove igre

**Veličina:** M, 4–5 h · **Ovisi o:** Task 7 · **Paralelno s:** Task 9–17 · **Model:** sonnet

**Cilj:** v1 igra izgleda točno kao danas; v2 igra radi po tjednima (oznaka kruga, „Zatvori tjedan", ekran
završenog kruga, ljestvica, tijek, pokazatelji po krugu); na početnom ekranu sklopivi dio „Za profesora" stvara v2 igru.

**Files:**
- Modify: `apps/web/src/{api/types,api/queries,format,game,router,App}.ts(x)`, `apps/web/src/ui/Layout.tsx`,
  `apps/web/src/screens/{HotelScreen,CloseMonth,MonthScreen,PlanScreen,FeedScreen,IndicatorScreen,KpiScreen,StartScreen}.tsx`,
  `apps/web/src/screens/month/*.tsx`, `apps/web/src/test/fixtures.ts`, `packages/i18n/src/hr.ts`, odgovarajući `*.test.ts(x)`
- Ne dirati: poslužitelj, `openapi.json`, `packages/api-client` (samo čita tipove iz Taska 7), `src/avatar/**`.

**Interfaces:**
- Consumes: `GameBody.{model, round_kind, rounds}`, `RoundBody.round`, `MonthScoreBody.round`, `round` u tijelima, `NewGameBody`.
- Produces:

```ts
// api/types.ts
export type RoundInfoBody = S["RoundInfoBody"]; export type ModelCode = S["ModelCode"];
export type RoundKindCode = S["RoundKindCode"]; export type NewGameBody = S["NewGameBody"];
// format.ts
/** Mjesec: „lipanj"; tjedan: „29. 6. – 5. 7.". */
export function roundDates(r: RoundInfoBody): string;
/** Mjesec: „Lipanj"; tjedan: „Tjedan 5 (29. 6. – 5. 7.)". */
export function roundTitle(r: RoundInfoBody): string;
// game.ts — ključ je kod kruga (u v1 jednak kodu mjeseca)
export function roundInfo(game: GameBody, code: string): RoundInfoBody | null;
export function roundPosition(game: GameBody): { round: RoundInfoBody; index: number; total: number } | null;
export function isV2(game: GameBody): boolean;
// indicesAt / indicesBefore / progressAt / leaderboard / nextRound primaju kod kruga i čitaju `MonthScoreBody.round`
// router.tsx
//   { name: "round"; code: string }  ← "/krug/:code" i stari "/mjesec/:code" (kod: jun|jul|aug|sep|t01…t17)
// App.tsx: StartScreen šalje NewGameBody samo kad je odabran novi model
```

Tekstovi (`hr.ts`, primjeri ključeva): `round.week` „Tjedan {n}", `chip.week` „TJEDAN {n} · {i}/{total}",
`close.week` „Zatvori tjedan" (za mjesec ostaje postojeći `close.button` „Zatvori mjesec", `hr.ts:88` — ne dodavati
duplikat `close.month`), `round.done.week` „Tjedan {n} završen", `start.pro`
„Za profesora: postavke nove igre", `start.model.v1` „Demo (dva mjeseca, bez odjela)", `start.model.v2` „Novi model:
odjeli i račun dobiti", `start.kind.week` „Po tjednima (preporučeno)", `start.kind.month` „Po mjesecima",
`start.kind.help` „Tjedni krug ima manje odluka, a mjesečna lekcija i mentor dolaze na kraju svakog mjeseca."

- [ ] **Step 1: Testovi (padaju)** — u postojećim datotekama testova:

```ts
// format.test.ts
it("oznaka tjedna i mjeseca", () => {
  const t05 = { code: "t05", kind: "week", index: 4, days: 7, first_day: 29, first_month: "jun", last_day: 5, last_month: "jul", month_ends: true } as const;
  expect(roundDates(t05)).toBe("29. 6. – 5. 7.");
  expect(roundTitle(t05)).toBe("Tjedan 5 (29. 6. – 5. 7.)");
  const jun = { code: "jun", kind: "month", index: 0, days: 30, first_day: 1, first_month: "jun", last_day: 30, last_month: "jun", month_ends: true } as const;
  expect(roundTitle(jun)).toBe("Lipanj");
});
// router.test.tsx
it("krug po kodu i stari put mjeseca", () => {
  expect(matchRoute("/krug/t05")).toEqual({ name: "round", code: "t05" });
  expect(matchRoute("/mjesec/jun")).toEqual({ name: "round", code: "jun" });
  expect(matchRoute("/krug/t18")).toEqual({ name: "notFound" });
});
// HotelScreen.test.tsx
it("tjedna igra: čip TJEDAN 1 · 1/17 i gumb Zatvori tjedan", async () => {
  renderApp(standardApi().on("GET", "/v1/games/:game", weeklyGame()), "/");
  expect(await screen.findByText("TJEDAN 1 · 1/17")).toBeTruthy();
  expect(await screen.findByRole("button", { name: "Zatvori tjedan" })).toBeTruthy();
});
it("v1 igra izgleda kao prije: LIPANJ · 1/2 i Zatvori mjesec", async () => {
  renderApp(standardApi(), "/");
  expect(await screen.findByText("LIPANJ · 1/2")).toBeTruthy();
  expect(await screen.findByRole("button", { name: "Zatvori mjesec" })).toBeTruthy();
});
// MonthScreen.test.tsx
it("završen tjedan na /krug/t01", async () => {
  renderApp(resolvedWeekApi(), "/krug/t01");
  expect(await screen.findByRole("heading", { level: 1, name: "Tjedan 1 završen (1. 6. – 7. 6.)" })).toBeTruthy();
});
// StartScreen (App.test.tsx)
it("Započni igru bez postavki šalje prazno tijelo", async () => {
  const api = startApi();
  renderApp(api, "/");
  fireEvent.click(await screen.findByRole("button", { name: "Započni igru" }));
  await waitFor(() => expect(api.posts("/v1/demo")).toHaveLength(1));
  expect(api.posts("/v1/demo")[0]?.body).toBeUndefined();
});
it("novi model po tjednima šalje jednom tijelo s modelom", async () => {
  const api = startApi();
  renderApp(api, "/");
  fireEvent.click(await screen.findByText("Za profesora: postavke nove igre"));
  fireEvent.click(screen.getByRole("radio", { name: "Novi model: odjeli i račun dobiti" }));
  expect((screen.getByRole("radio", { name: "Po tjednima (preporučeno)" }) as HTMLInputElement).checked).toBe(true);
  const start = screen.getByRole("button", { name: "Započni igru" });
  fireEvent.click(start);
  fireEvent.click(start);
  await waitFor(() => expect(api.posts("/v1/demo")).toHaveLength(1));
  expect(api.posts("/v1/demo")[0]?.body).toEqual({ model: "v2", round_kind: "week" });
});
```

`fixtures.ts` (`apps/web/src/test/fixtures.ts`) dobiva — i to je temelj koji troše Taskovi 19–21:
- `export const HOTEL_ID = "h1";` i `export const GAME_ID = "g1";` — zamijene postojeće literale `"h1"`/`"g1"`
  (`fixtures.ts:59, 95, 120, 123`), da testovi Taskova 19–21 grade putanje kao `` `/v1/hotels/${HOTEL_ID}/plans/t01` ``;
- `startApi()` — gost bez igre: `GET /v1/me/hotel` → 404 `not_in_game` dok igra nije stvorena, zatim `myHotel()`;
  `POST /v1/demo` → 201 `{ game_id: GAME_ID, hotel_id: HOTEL_ID }`; `GET /v1/games/:game` → `game()` (danas takvog
  pomoćnika nema — `App.test.tsx:19-26` ga piše ručno);
- `weeklyGame()` (17 krugova, `model: "v2"`, `round_kind: "week"`, `current.round` = t01) i `resolvedWeekApi()`;
- kostur `export function v2Api(): FakeApi` = `standardApi()` s `GET /v1/games/:game` → `weeklyGame()` i
  `GET /v1/me/hotel` → hotel u krugu t01 (Task 19 dodaje rute odjela, Task 20 `v2PlApi()`, Task 21 plan s odjelima);
- dopunu svih postojećih tvornica poljima `round`/`rounds`/`model` (v1 vrijednosti).

- [ ] **Step 2:** `corepack pnpm --filter @hotelixsim/web test` → FAIL.
- [ ] **Step 3: Implementiraj.** Datumi u `roundDates` iz polja tijela (ekran ne računa kalendar). Čip na `Layout`
  koristi `roundPosition`. Sve što je bilo po `month` čita `round` (za v1 je isto).
- [ ] **Step 4:** `corepack pnpm lint; corepack pnpm typecheck; corepack pnpm test; corepack pnpm --filter @hotelixsim/web build`
  (svaka zasebno, EXIT=0).
- [ ] **Step 5: Commit** „Sloj 1/18: web — tjedni krugovi i postavke nove igre za profesora".

---

### Task 19: Web — Odjeli i detalj odjela

**Veličina:** M, 3–4 h · **Ovisi o:** Task 15, Task 18 · **Paralelno s:** ništa iz weba (val 9; Taskovi 20 i 21
počinju tek kad je ovaj spojen, jer troše njegov `v2Api()`, stavku izbornika i `problem.*` poruke) · **Model:** sonnet

**Cilj (Leon: „stisne wellness → stanje prostora, koliko je zaradio, koliko su gosti bili zadovoljni"):** odjeljak
„Odjeli" s popisom odjela koje hotel ima i ekranom odjela.

**Files:**
- Create: `apps/web/src/screens/DepartmentsScreen.tsx`, `apps/web/src/screens/departments/{DepartmentCard,DepartmentDetail,ServiceCard}.tsx`,
  `apps/web/src/screens/DepartmentsScreen.test.tsx`
- Modify: `apps/web/src/{router,App}.tsx`, `apps/web/src/ui/Layout.tsx` (stavka „Odjeli", ikona `Building2`, samo v2),
  `apps/web/src/api/{types,queries}.ts`, `apps/web/src/test/fixtures.ts`, `packages/i18n/src/hr.ts`,
  `apps/web/src/api/errors.ts` (samo ako treba: `problemMessage` već čita poruku iz ključa `problem.<code>` u
  `hr.ts`, `errors.ts:32-39`, pa je dovoljno dodati ključeve)
- Ne dirati: poslužitelj, ekrani drugih taskova.

**Interfaces:**
- Consumes: `GET /v1/hotels/{id}/departments`, `GET /v1/hotels/{id}/departments/{code}`, `isV2`, `roundTitle`.
- Produces: rute `{ name: "departments" }` (`/odjeli`) i `{ name: "department"; code: string }` (`/odjeli/:code`);
  kuke `useDepartments(hotelId)`, `useDepartment(hotelId, code)`.

Izgled: mobitel — popis kartica (ime, „Stanje prostora 72/100" s trakom, „Zadovoljstvo gostiju 74/100", „Rezultat
zadnjeg tjedna +7.896 €", „Ljudi: 8 stalnih, 10 sezonskih", oznaka „Zatvoren"), dolje „Službe bez prihoda" (Uprava,
Prodaja i marketing, Održavanje, Energija i voda: „Trošak zadnjeg kruga 59.050 €"); klik → puni ekran detalja s
poveznicom „← Odjeli". Od 1024 px: popis lijevo, detalj desno (`lg:grid lg:grid-cols-[22rem_1fr]`). Detalj ima
četiri kartice: **Prostor** (stanje prije → poslije, „trošenje −3", „preopterećenje −1", „održavanje +3", trošak
održavanja), **Gosti** (korištenja, „udio gostiju koji dolaze 45 %", zadovoljstvo prije → poslije, „vrijednost za novac
70/100", razina cijene), **Novac** (prihod, plaće, namirnice i piće / materijal, ostali izravni trošak, rezultat,
marža), **Ljudi** (stalni, sezonski, kapacitet, „opterećenje 1,9 gosta po zaposlenom na dan, norma 1,8"); ispod tablica
po krugovima (laptop) ili popis (mobitel). Bodovi 0–10 000 se prikazuju kao cijeli broj 0–100. v1 igra: ruta pokazuje
rečenicu iz 409 `model_v1`, a izbornik nema „Odjeli".

- [ ] **Step 1: Testovi (padaju)** — `DepartmentsScreen.test.tsx`:

```tsx
describe("Odjeli", () => {
  it("popis: samo odjeli hotela, brojke s imenom i jedinicom", async () => {
    renderApp(v2Api(), "/odjeli");
    expect(await screen.findByRole("heading", { level: 1, name: "Odjeli" })).toBeTruthy();
    const spa = (await screen.findByRole("link", { name: /Mali wellness/ })).closest("article") as HTMLElement;
    expect(within(spa).getByText("Stanje prostora")).toBeTruthy();
    expect(within(spa).getByText("65/100")).toBeTruthy();
    expect(within(spa).getByText("Zadovoljstvo gostiju")).toBeTruthy();
    expect(within(spa).getByText("Ljudi: 2 stalna, 1 sezonski")).toBeTruthy();
    expect(screen.queryByText("Rooftop bar")).toBeNull();
    expect(screen.getByRole("heading", { name: "Službe bez prihoda" })).toBeTruthy();
  });
  it("detalj odjela izravno na putanji: prostor, gosti, novac, ljudi", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    expect(await screen.findByRole("heading", { level: 1, name: "Restoran" })).toBeTruthy();
    for (const h of ["Prostor", "Gosti", "Novac", "Ljudi"]) expect(screen.getByRole("heading", { level: 2, name: h })).toBeTruthy();
    expect(screen.getByText("67.200 €")).toBeTruthy();
    expect(screen.getByText("trošenje −3")).toBeTruthy();
    expect(screen.getByText("udio gostiju koji dolaze 45 %")).toBeTruthy();
  });
  it("računalo: popis i detalj u dva stupca", async () => {
    renderApp(v2Api(), "/odjeli/restaurant");
    const grid = (await screen.findByRole("heading", { level: 1, name: "Restoran" })).closest("[data-layout='list-detail']");
    expect(grid?.className).toMatch(/lg:grid/);
  });
  it("v1 igra: rečenica umjesto odjela i nema stavke izbornika", async () => {
    renderApp(standardApi().on("GET", "/v1/hotels/:hotel/departments", () => problem(409, "model_v1")), "/odjeli");
    expect(await screen.findByText("Ova igra je na starom modelu i nema ovaj prikaz.")).toBeTruthy();
    expect(screen.queryByRole("link", { name: "Odjeli" })).toBeNull();
  });
  it("nepoznat odjel: rečenica, ne prazno", async () => {
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/departments/:code", () => problem(404, "unknown_department")), "/odjeli/casino");
    expect(await screen.findByText("Taj odjel ne postoji u ovom hotelu.")).toBeTruthy();
  });
});
```

`fixtures.ts`: kostur `v2Api()` iz Taska 18 dobiva rute `GET /v1/hotels/:hotel/departments` → `departmentsBody()`
(brojke iz Taska 9: restoran 6 720 000 / 3 780 000 / 2 150 400, stanje 7 000 → 6 900, zadovoljstvo 7 000 → 6 949,
udio 4 500) i `GET /v1/hotels/:hotel/departments/:code` → `departmentDetailBody()`. U `hr.ts` dodaj **sve četiri**
poruke problema sloja 1 odjednom (da ih Taskovi 20 i 21 ne dodaju usporedno): `problem.model_v1` „Ova igra je na
starom modelu i nema ovaj prikaz.", `problem.unknown_department` „Taj odjel ne postoji u ovom hotelu.",
`problem.no_resolved_round` „Račun dobiti postoji tek nakon prvog obračunatog kruga.", `problem.department_plan_invalid`
„Postavke odjela nisu dopuštene. Provjeri sezonce, razinu cijene i koje se odjele smije zatvoriti." —
`problemMessage(error)` ih tada vraća sam.

- [ ] **Step 2:** `corepack pnpm --filter @hotelixsim/web test` → FAIL.
- [ ] **Step 3: Implementiraj** (tokeni, `t()`, `role="status"` za učitavanje, `Link` za kartice, širina trake jedina
  inline vrijednost).
- [ ] **Step 4:** četiri web provjere (EXIT=0) i pregled na 360 px (bez vodoravnog pomicanja) u testu ili pregledniku.
- [ ] **Step 5: Commit** „Sloj 1/19: web — Odjeli i ekran odjela".

---

### Task 20: Web — Račun dobiti i budžet

**Veličina:** M, 3–4 h · **Ovisi o:** Task 15, Task 18, **Task 19** (spojen: `v2Api()` s odjelima, poruka
`problem.no_resolved_round`, izbornik) · **Paralelno s:** Task 21 (val 10; ovaj se spaja prvi) · **Model:** sonnet

**Cilj:** ekran „Račun dobiti" po USALI-ju s budžetom i razlikom, za krug ili sezonu do sada.

**Files:**
- Create: `apps/web/src/screens/PlScreen.tsx`, `apps/web/src/screens/pl/{PlTable,PlCards}.tsx`, `apps/web/src/screens/PlScreen.test.tsx`
- Modify: `apps/web/src/{router,App}.tsx`, `apps/web/src/ui/Layout.tsx` (stavka „Račun dobiti", ikona `Receipt`, samo v2),
  `apps/web/src/api/{types,queries}.ts`, `apps/web/src/test/fixtures.ts` (`v2PlApi()`), `packages/i18n/src/hr.ts`,
  `apps/web/src/test/fakeApi.ts` (zapis upita, niže), `apps/web/src/api/errors.ts` (samo ako treba — poruka
  `problem.no_resolved_round` je već u `hr.ts` iz Taska 19)
- Ne dirati: ekrani Taskova 19 i 21, poslužitelj, `apps/web/src/screens/PlanScreen.tsx` i `screens/plan/**` (Task 21
  radi usporedno).

**Interfaces:**
- Consumes: `GET /v1/hotels/{id}/pl?round=&scope=`.
- Produces: rute `{ name: "pl"; code: string | null }` (`/racun-dobiti` i `/racun-dobiti/:code`); kuka
  `usePl(hotelId, code, scope)`; prekidač „Ovaj krug" / „Sezona do sada" (stanje ekrana, novi upit), izbornik krugova
  iz `PlBody.rounds` (oznaka `roundTitle`).

Izgled: zaglavlje „Račun dobiti — Tjedan 2 (8. 6. – 14. 6.)"; objašnjenje jednom rečenicom: „Ovako hotelski računovođe
slažu rezultat (USALI): prvo odjeli, zatim zajednički troškovi hotela, pa GOP, pa fiksni troškovi zgrade."; „Budžet =
što bi tvoj hotel ostvario uz zadane odluke i bez situacija." Laptop (≥ 1024 px): tablica „Stavka | Ostvareno | Budžet |
Razlika" po sekcijama (naslovi sekcija iz `t()`: „Prihodi odjela", „Troškovi odjela", „Rezultat odjela", „Zajednički
troškovi hotela", „Bruto operativna dobit (GOP)", „Fiksni troškovi zgrade", „EBITDA"); mobitel: kartica po sekciji,
redak = ime, ostvareno, ispod sitno „budžet 12.300 €" i razlika riječima „+1.200 € bolje od budžeta" / „−800 € lošije
od budžeta" (boja tokenom uspjeha ili upozorenja uz riječi, ne samo boja). Ispod: GOP marža, GOPPAR, EBITDA marža,
ostvareno i budžet.

- [ ] **Step 1: Testovi (padaju)** — `PlScreen.test.tsx`:

```tsx
describe("Račun dobiti", () => {
  it("sekcije redom USALI-ja i razlika riječima", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    expect(await screen.findByRole("heading", { level: 1, name: /^Račun dobiti/ })).toBeTruthy();
    const titles = screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent);
    expect(titles).toEqual(["Prihodi odjela", "Troškovi odjela", "Rezultat odjela", "Zajednički troškovi hotela",
      "Bruto operativna dobit (GOP)", "Fiksni troškovi zgrade", "EBITDA"]);
    expect(screen.getAllByText("+1.200 € bolje od budžeta").length).toBeGreaterThan(0);
    expect(screen.getAllByText("−800 € lošije od budžeta").length).toBeGreaterThan(0);
  });
  it("računalo: tablica sa četiri stupca", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    const table = await screen.findByRole("table", { name: "Račun dobiti po USALI-ju" });
    expect(within(table).getAllByRole("columnheader").map((c) => c.textContent)).toEqual(["Stavka", "Ostvareno", "Budžet", "Razlika"]);
    expect(table.closest("section")?.className).toMatch(/hidden .*lg:block/);
  });
  it("sezona do sada šalje jedan novi upit sa scope=season", async () => {
    const api = v2PlApi();
    renderApp(api, "/racun-dobiti");
    const btn = await screen.findByRole("button", { name: "Sezona do sada" });
    const seasonCalls = () =>
      api.calls.filter(
        (c) => c.method === "GET" && c.path === `/v1/hotels/${HOTEL_ID}/pl` && new URLSearchParams(c.search).get("scope") === "season",
      );
    expect(seasonCalls()).toHaveLength(0); // prvi upit ekrana je za krug, ne za sezonu
    fireEvent.click(btn);
    await waitFor(() => expect(seasonCalls()).toHaveLength(1));
    expect(await screen.findByText(/sezona do sada/i)).toBeTruthy();
  });
  it("prije prvog obračuna: rečenica, ne nule", async () => {
    renderApp(v2Api().on("GET", "/v1/hotels/:hotel/pl", () => problem(409, "no_resolved_round")), "/racun-dobiti");
    expect(await screen.findByText("Račun dobiti postoji tek nakon prvog obračunatog kruga.")).toBeTruthy();
    expect(screen.queryByText("0 €")).toBeNull();
  });
  it("GOP i EBITDA imaju objašnjenje riječima", async () => {
    renderApp(v2PlApi(), "/racun-dobiti");
    expect(await screen.findByText(/dobit prije kamata, poreza i amortizacije/)).toBeTruthy();
  });
});
```

`FakeApi` danas u `calls` pamti samo `url.pathname` (`fakeApi.ts:44`), pa bi test „sezona do sada" prošao već na prvom
`GET /pl` i ništa ne bi provjerio. Zato ovaj task u `apps/web/src/test/fakeApi.ts` proširuje zapis:
`calls: { method: string; path: string; search: string; body: unknown }[]` i `this.calls.push({ method: req.method,
path: url.pathname, search: url.search, body })` (rute se i dalje biraju po putanji; `posts()` ostaje isti). U
`v2PlApi()` (= `v2Api()` + `GET /v1/hotels/:hotel/pl`) rukovatelj čita `new URL(req.url).searchParams.get("scope")`
i za `season` vraća drugo tijelo (`scope: "season"`). Test uvozi `HOTEL_ID` iz `test/fixtures.ts` (Task 18).

- [ ] **Step 2–4:** testovi padaju → implementacija → četiri web provjere EXIT=0.
- [ ] **Step 5: Commit** „Sloj 1/20: web — Račun dobiti po USALI-ju s budžetom".

---

### Task 21: Web — Plan po odjelima

**Veličina:** S–M, 2–3 h · **Ovisi o:** Task 15, Task 18, **Task 19** (spojen: `v2Api()`, poruka
`problem.department_plan_invalid`) · **Paralelno s:** Task 20 (val 10) · **Spaja se nakon:** Task 20 · **Model:** sonnet

**Cilj:** u planu kruga (v2) dio „Odjeli": otvoren/zatvoren (ako se smije zatvoriti), sezonci, razina cijene, razina
održavanja; jedan zahtjev „Spremi".

**Files:**
- Create: `apps/web/src/screens/plan/DepartmentPlan.tsx`, `apps/web/src/screens/plan/DepartmentPlan.test.tsx`
- Modify: `apps/web/src/screens/PlanScreen.tsx`, `apps/web/src/test/fixtures.ts` (samo `GET /v1/me/hotel` u
  `v2Api()`: plan kruga t01 s `departments` i granice s `departments` i `seasonal_step_total: 5`),
  `packages/i18n/src/hr.ts` (tekstovi dijela „Odjeli")
- Ne dirati: poslužitelj, ostali ekrani, `router.tsx`, `App.tsx`, `ui/Layout.tsx`, `api/errors.ts`, `test/fakeApi.ts`
  (sve to dira Task 20 usporedno ili Task 19 prije).

**Interfaces:**
- Consumes: `PlanBody.departments`, `PlanLimitsBody.{departments, seasonal_step_total}`, postojeći `useSavePlan`.
- Produces: komponenta `DepartmentPlan({ plan, limits, onChange })` (kontrolirana; ne šalje sama).

Pravila na ekranu: ukupna promjena sezonaca prikazana kao „Ovaj krug se može promijeniti još za 3 sezonca" (iz
`seasonal_step_total` i već napravljenih promjena — računa iz plana i ograničenja koja je dao poslužitelj; ništa novo
se ne izvodi); stepper odjela onemogućen na granici; zatvaranje odjela postavlja sezonce na 0 i onemogućuje stepper;
razina cijene 90–120 % u koracima od 5 („Cijene u odjelu 110 %"); održavanje „odgođeno / redovno / pojačano". Greška 422
`department_plan_invalid` prikazuje se kao rečenica kroz `problemMessage(error)` (ključ
`problem.department_plan_invalid` dodao je Task 19).

- [ ] **Step 1: Testovi (padaju)** — `DepartmentPlan.test.tsx`:

```tsx
describe("Plan po odjelima", () => {
  it("v2: dio Odjeli s retkom za svaki odjel koji se planira", async () => {
    renderApp(v2Api(), "/plan");
    expect(await screen.findByRole("heading", { level: 2, name: "Odjeli" })).toBeTruthy();
    expect(screen.getByRole("group", { name: "Restoran" })).toBeTruthy();
    expect(screen.queryByRole("group", { name: "Uprava" })).toBeNull();
  });
  it("zatvaranje bara stavlja sezonce na 0 i šalje jedan plan", async () => {
    const api = v2Api();
    renderApp(api, "/plan");
    const bar = await screen.findByRole("group", { name: "Lobby bar" });
    fireEvent.click(within(bar).getByRole("checkbox", { name: "Otvoren" }));
    expect((within(bar).getByRole("button", { name: "Više sezonaca" }) as HTMLButtonElement).disabled).toBe(true);
    const save = screen.getByRole("button", { name: "Spremi" });
    fireEvent.click(save);
    fireEvent.click(save);
    await waitFor(() => expect(api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)).toHaveLength(1));
    const body = api.posts(`/v1/hotels/${HOTEL_ID}/plans/t01`)[0]?.body as { departments: Record<string, { open: boolean; seasonal_target: number }> };
    expect(body.departments.lobby_bar).toMatchObject({ open: false, seasonal_target: 0 });
  });
  it("restoran se ne može zatvoriti", async () => {
    renderApp(v2Api(), "/plan");
    const rest = await screen.findByRole("group", { name: "Restoran" });
    expect(within(rest).queryByRole("checkbox", { name: "Otvoren" })).toBeNull();
  });
  it("korak sezonaca potrošen: plus onemogućen i rečenica", async () => {
    renderApp(v2Api(), "/plan");
    const rest = await screen.findByRole("group", { name: "Restoran" });
    const plus = within(rest).getByRole("button", { name: "Više sezonaca" });
    for (let i = 0; i < 5; i++) fireEvent.click(plus);
    expect(screen.getByText("Ovaj krug se sezonci više ne mogu mijenjati.")).toBeTruthy();
    expect((plus as HTMLButtonElement).disabled).toBe(true);
  });
  it("v1 igra nema dio Odjeli", async () => {
    renderApp(standardApi(), "/plan");
    expect(await screen.findByRole("heading", { level: 1 })).toBeTruthy();
    expect(screen.queryByRole("heading", { level: 2, name: "Odjeli" })).toBeNull();
  });
});
```

(`HOTEL_ID` = `"h1"` je konstanta iz `test/fixtures.ts` koju dodaje Task 18 — uvezi je uz `v2Api`, `standardApi`;
`seasonal_step_total` u fiksturi je 5, a `lobby_bar` u fiksturi ima 3 sezonca, pa zatvaranje troši 3 od 5.)

- [ ] **Step 2–4:** testovi padaju → implementacija → četiri web provjere EXIT=0.
- [ ] **Step 5: Commit** „Sloj 1/21: web — plan po odjelima".

---
