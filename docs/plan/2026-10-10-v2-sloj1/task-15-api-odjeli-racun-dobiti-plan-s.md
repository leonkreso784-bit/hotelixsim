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

