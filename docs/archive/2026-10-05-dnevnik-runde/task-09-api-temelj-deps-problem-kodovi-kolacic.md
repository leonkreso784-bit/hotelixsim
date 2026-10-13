### Task 9: `api` — temelj (Deps, problem kodovi, kolačić gosta, zamjenska ruta) i rute gosta i hotela

**Files:**
- Modify: `crates/api/Cargo.toml` (makni `sim-hotel`; dodaj `uuid.workspace = true`, `tracing.workspace = true`;
  dev: `content.workspace = true`)
- Modify: `crates/api/src/lib.rs`, `crates/api/src/problem.rs`, `crates/api/src/health.rs`, `crates/api/src/demo.rs`
- Create: `crates/api/src/auth.rs`, `crates/api/src/schema.rs`, `crates/api/src/hotels.rs`
- Modify: `crates/server/Cargo.toml` (`app`, `content`), `crates/server/src/main.rs` (sastavljanje `Deps`, sadržaj pri startu)
- Modify: `crates/api/tests/http.rs`; Create: `crates/api/tests/common/mod.rs`, `crates/api/tests/hotels.rs`
- Modify: `openapi.json`, `packages/api-client/src/schema.d.ts` (generirano)
- Modify: `docs/records/BACKLOG.md` (brišu se riješene stavke `[dnevnik]`: HealthBody `required`, OpenAPI `info`,
  fallback ruta, test OpenAPI-ja za 503/404, `api` zove `sim-hotel` mimo `app`, enum kodovi u OpenAPI-ju)

**Interfaces:**
- Consumes: Task 5 (`app::{Deps, caller, start_demo, my_hotel, hotel_view, profile_indices, AppError, ErrorCode,
  ErrorKind, HotelView, memory}`), `game::view::*` i `game::dto::*` **samo kroz** tipove koje `app` vraća (api dodaje
  ovisnost `game.workspace = true` za te tipove — dopušteno: `game` je čist).
- Produces:
  - `AppState { pub deps: app::Deps }` (`Clone`); `router(AppState) -> Router` sa zamjenskom rutom `unknown_route`.
  - `problem::ProblemCode` — enum u OpenAPI-ju: svih 20 `app::ErrorCode` + `UnknownRoute` + `InvalidBody`
    (snake_case); `Problem { type, title, status, code: ProblemCode }`; `impl From<AppError> for Problem`;
    `Problem::new(StatusCode, ProblemCode)`.
  - `auth::{COOKIE = "hotelixsim_guest", Token(pub Option<String>), set_cookie(&str) -> HeaderValue}`; `Token` je
    `FromRequestParts` (nikad ne odbija; token prihvaća samo ako je točno 32 heksadecimalna znaka).
  - `schema`: `MonthCode`, `IndexCode`, `IndexEntry { code: IndexCode, hundredths: u16 }`, `entries([u16; 5]) -> Vec<IndexEntry>`,
    `TrainingCode`, `MaintenanceCode`, `PlanBody`, `PlanLimitsBody`, `SeatKind`, `StrategyCode`, `SeatBody`,
    `MonthScoreBody`, `PublicHotelBody`, `ProfileCode`, `RealNumbersBody`, `LastMonthBody`, `PlanStateBody`,
    `PrivateHotelBody`, `HotelBody`, `DemoBody`; `parse_game(&str)`, `parse_hotel(&str)`, `parse_month(&str)` →
    `Result<_, Problem>` (neispravno = 404 `unknown_game` / `unknown_hotel` / `unknown_month`).
  - Rute: `POST /v1/demo` (201 + `Set-Cookie` kad je gost nov) · `GET /v1/me/hotel` · `GET /v1/hotels/{hotel_id}` ·
    `GET /v1/demo/profiles/{profile}/indices` (sad preko `app::profile_indices`) · `GET /v1/health`.

- [ ] **Step 1: Pomoćnik testova.** `crates/api/tests/common/mod.rs`:

```rust
#![allow(dead_code)]
use std::sync::Arc;

use api::{AppState, router};
use app::memory::{Memory, deps};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use content::{Snapshot, repo_dir};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

pub struct Resp {
    pub status: StatusCode,
    pub ctype: Option<String>,
    pub set_cookie: Option<String>,
    pub body: Value,
}

pub fn app() -> (Router, app::Deps, Arc<Memory>) {
    let (d, mem) = deps(Snapshot::read(&repo_dir()).unwrap());
    (router(AppState { deps: d.clone() }), d, mem)
}

pub async fn call(app: &Router, method: &str, uri: &str, cookie: Option<&str>, body: Option<Value>) -> Resp {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(t) = cookie {
        req = req.header("cookie", format!("hotelixsim_guest={t}"));
    }
    let req = match body {
        Some(b) => req.header("content-type", "application/json").body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let header = |n: &str| res.headers().get(n).map(|v| v.to_str().unwrap().to_owned());
    let (status, ctype, set_cookie) = (res.status(), header("content-type"), header("set-cookie"));
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap_or(Value::Null) };
    Resp { status, ctype, set_cookie, body }
}

/// Novi gost s igrom: (token, game_id, hotel_id).
pub async fn demo(app: &Router) -> (String, String, String) {
    let r = call(app, "POST", "/v1/demo", None, None).await;
    assert_eq!(r.status, StatusCode::CREATED);
    let c = r.set_cookie.expect("novi gost dobiva kolačić");
    let tok = c.split(';').next().unwrap().strip_prefix("hotelixsim_guest=").unwrap().to_owned();
    (tok, r.body["game_id"].as_str().unwrap().into(), r.body["hotel_id"].as_str().unwrap().into())
}

pub fn robot_hotel(mem: &Memory, game: &str) -> String {
    let g = app::GameId(game.parse().unwrap());
    mem.events(g)[0].payload["hotels"][1]["hotel_id"].as_str().unwrap().to_owned()
}

pub fn assert_problem(r: &Resp, status: u16, code: &str) {
    assert_eq!(r.status.as_u16(), status, "{:?}", r.body);
    assert_eq!(r.ctype.as_deref(), Some("application/problem+json"));
    assert_eq!(r.body["code"], code);
    assert_eq!(r.body["status"], status);
}
```

- [ ] **Step 2: Testovi (crveni).** `crates/api/tests/hotels.rs`:

```rust
mod common;
use common::*;

#[tokio::test]
async fn demo_sets_httponly_cookie_once() {
    let (app, _, _) = app();
    let r = call(&app, "POST", "/v1/demo", None, None).await;
    let c = r.set_cookie.clone().unwrap();
    for part in ["hotelixsim_guest=", "HttpOnly", "SameSite=Lax", "Path=/", "Max-Age=604800"] {
        assert!(c.contains(part), "{c}");
    }
    let tok = c.split(';').next().unwrap().strip_prefix("hotelixsim_guest=").unwrap();
    let again = call(&app, "POST", "/v1/demo", Some(tok), None).await;
    assert_eq!(again.status.as_u16(), 201);
    assert_eq!(again.set_cookie, None, "postojeći gost ne dobiva novi kolačić");
    assert_ne!(again.body["game_id"], r.body["game_id"]);
}

#[tokio::test]
async fn me_hotel_shows_private_numbers_and_june_plan() {
    let (app, _, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    let r = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    assert_eq!(r.status.as_u16(), 200);
    assert_eq!(r.body["game_id"], game.as_str());
    assert_eq!(r.body["hotel"]["hotel_id"], hotel.as_str());
    assert_eq!(r.body["hotel"]["seat"]["kind"], "human");
    assert!(r.body["private"]["now"]["cash_cents"].is_i64());
    assert_eq!(r.body["private"]["plan"]["month"], "jun");
    assert_eq!(r.body["hotel"]["start"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn foreign_and_anonymous_requests() {
    let (app, _, mem) = app();
    let (tok, game, hotel) = demo(&app).await;
    let (other, _, _) = demo(&app).await;
    assert_problem(&call(&app, "GET", "/v1/me/hotel", None, None).await, 401, "guest_required");
    assert_problem(&call(&app, "GET", "/v1/me/hotel", Some("nije-token"), None).await, 401, "guest_required");
    let uri = format!("/v1/hotels/{hotel}");
    assert_problem(&call(&app, "GET", &uri, Some(&other), None).await, 403, "not_in_game");
    let robot = call(&app, "GET", &format!("/v1/hotels/{}", robot_hotel(&mem, &game)), Some(&tok), None).await;
    assert_eq!(robot.status.as_u16(), 200);
    assert!(robot.body["private"].is_null(), "tuđi hotel bez stvarnih brojki");
    assert!(robot.body["hotel"]["start"].is_array());
}

#[tokio::test]
async fn unknown_ids_and_routes_are_problem_404() {
    let (app, _, _) = app();
    let (tok, _, _) = demo(&app).await;
    let nobody = "00000000-0000-0000-0000-00000000abcd";
    assert_problem(&call(&app, "GET", &format!("/v1/hotels/{nobody}"), Some(&tok), None).await, 404, "unknown_hotel");
    assert_problem(&call(&app, "GET", "/v1/hotels/nije-uuid", Some(&tok), None).await, 404, "unknown_hotel");
    assert_problem(&call(&app, "GET", "/v1/nema-me", None, None).await, 404, "unknown_route");
}
```
  `tests/http.rs`: zamijeni lažni `Fake` sustavom `common::app()` uz zamjenu zdravlja
  (`let (_, mut d, _) = app(); d.status = Arc::new(Fake(Err(..))); router(AppState { deps: d })`); postojeći testovi
  zdravlja i profila ostaju istog sadržaja. Test OpenAPI-ja proširi:

```rust
#[test]
fn openapi_documents_contract() {
    let doc: Value = serde_json::from_str(&openapi_json()).unwrap();
    assert_eq!(doc["openapi"], "3.1.0", "TS generator čita samo 3.0 i 3.1");
    assert!(doc["info"]["description"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(doc["info"]["license"]["name"].is_string());
    for (path, method) in [("/v1/health", "get"), ("/v1/demo", "post"), ("/v1/me/hotel", "get"),
                           ("/v1/hotels/{hotel_id}", "get"), ("/v1/demo/profiles/{profile}/indices", "get")] {
        assert!(doc["paths"][path][method].is_object(), "{method} {path}");
    }
    assert!(doc["paths"]["/v1/health"]["get"]["responses"]["503"].is_object());
    let not_found = &doc["paths"]["/v1/hotels/{hotel_id}"]["get"]["responses"]["404"]["content"];
    assert!(not_found["application/problem+json"].is_object());
    let required = doc["components"]["schemas"]["HealthBody"]["required"].as_array().unwrap();
    assert!(required.contains(&json!("events")), "events je obavezan (može biti null)");
    let codes = doc["components"]["schemas"]["ProblemCode"]["enum"].as_array().unwrap();
    for c in ["unknown_route", "invalid_body", "guest_required", "round_not_open", "price_not_whole_euro"] {
        assert!(codes.contains(&json!(c)), "{c}");
    }
    assert_eq!(codes.len(), 22);
    assert!(doc["components"]["schemas"]["IndexCode"]["enum"].is_array());
}
```

- [ ] **Step 3: Pokreni** `cargo test -p api; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 4: `problem.rs`.**

```rust
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

/// Stabilni kodovi grešaka; ekran ih prevodi (ARHITEKTURA §11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProblemCode {
    GuestRequired, NotInGame, NotYourHotel, UnknownGame, UnknownHotel, UnknownMonth, UnknownSituation,
    UnknownAnswer, UnknownProfile, RoundNotOpen, AlreadyAnswered, PriceNotWholeEuro, PriceOutOfRange,
    MarketingOutOfRange, SeasonalOutOfRange, SeasonalStepTooBig, WageOutOfRange, RenovationTwice,
    StorageUnavailable, Internal,
    /// Samo HTTP sloj: ruta ne postoji.
    UnknownRoute,
    /// Samo HTTP sloj: tijelo zahtjeva nije ispravan JSON traženog oblika.
    InvalidBody,
}

impl From<app::ErrorCode> for ProblemCode {
    fn from(c: app::ErrorCode) -> Self {
        use app::ErrorCode as E;
        match c { E::GuestRequired => Self::GuestRequired, /* … iscrpno, svih 20, bez `_` … */ E::Internal => Self::Internal }
    }
}

impl ProblemCode {
    /// Naslov za čovjeka (hrvatski); sučelje ionako prevodi po kodu.
    pub fn title(self) -> &'static str {
        match self {
            Self::GuestRequired => "Potreban je gost: otvori demo poveznicu",
            Self::NotInGame => "Nisi u ovoj igri",
            Self::NotYourHotel => "To nije tvoj hotel",
            Self::UnknownGame => "Igra ne postoji",
            Self::UnknownHotel => "Hotel ne postoji",
            Self::UnknownMonth => "Mjesec nije u ovoj igri",
            Self::UnknownSituation => "Situacija ne postoji",
            Self::UnknownAnswer => "Taj odgovor nije ponuđen",
            Self::UnknownProfile => "Nepoznat profil",
            Self::RoundNotOpen => "Runda nije otvorena",
            Self::AlreadyAnswered => "Na situaciju je već odgovoreno",
            Self::PriceNotWholeEuro => "Cijena mora biti cijeli euro",
            Self::PriceOutOfRange => "Cijena je izvan dopuštenog raspona",
            Self::MarketingOutOfRange => "Marketing je izvan dopuštenog raspona",
            Self::SeasonalOutOfRange => "Broj sezonaca je izvan dopuštenog raspona",
            Self::SeasonalStepTooBig => "Prevelika promjena broja sezonaca u jednom mjesecu",
            Self::WageOutOfRange => "Plaća je izvan dopuštenog raspona",
            Self::RenovationTwice => "Obnova je moguća samo jednom u sezoni",
            Self::StorageUnavailable => "Baza nije dostupna",
            Self::Internal => "Greška sustava",
            Self::UnknownRoute => "Nepoznata ruta",
            Self::InvalidBody => "Neispravno tijelo zahtjeva",
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct Problem {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub title: &'static str,
    pub status: u16,
    pub code: ProblemCode,
}

impl Problem {
    pub fn new(status: StatusCode, code: ProblemCode) -> Self {
        Problem { kind: "about:blank", title: code.title(), status: status.as_u16(), code }
    }
}

impl From<app::AppError> for Problem {
    fn from(e: app::AppError) -> Self {
        use app::ErrorKind as K;
        let status = match e.code.kind() {
            K::Unauthorized => StatusCode::UNAUTHORIZED,
            K::Forbidden => StatusCode::FORBIDDEN,
            K::NotFound => StatusCode::NOT_FOUND,
            K::Conflict => StatusCode::CONFLICT,
            K::Invalid => StatusCode::UNPROCESSABLE_ENTITY,
            K::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            K::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if status.is_server_error() {
            tracing::error!(code = ?e.code, detail = %e.detail, "greška poslužitelja");
        }
        Problem::new(status, e.code.into())
    }
}
```
  `IntoResponse for Problem` ostaje kakav jest (`application/problem+json`).

- [ ] **Step 5: `auth.rs`.**

```rust
use std::convert::Infallible;

use axum::extract::FromRequestParts;
use axum::http::{HeaderMap, HeaderValue, header, request::Parts};

pub const COOKIE: &str = "hotelixsim_guest";

/// Token gosta iz kolačića; pokvaren ili nepostojeći = `None` (anoniman pozivatelj, odluku donosi `app`).
pub struct Token(pub Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for Token {
    type Rejection = Infallible;
    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        Ok(Token(read(&parts.headers)))
    }
}

fn read(h: &HeaderMap) -> Option<String> {
    h.get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(k, _)| *k == COOKIE)
        .map(|(_, v)| v.to_owned())
        .filter(|v| v.len() == 32 && v.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// `Secure` dolazi s HTTPS-om prije Railwaya (BACKLOG); lokalno je http.
pub fn set_cookie(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!("{COOKIE}={token}; HttpOnly; SameSite=Lax; Path=/; Max-Age=604800"))
        .expect("token je heksadecimalan")
}
```

- [ ] **Step 6: `schema.rs`.** Svi tipovi `#[derive(Serialize, ToSchema)]` (+ `Deserialize` za `PlanBody` i enume koje
  tijelo zahtjeva nosi; `PlanBody` uz to `#[serde(deny_unknown_fields)]`). **Svako `Option` polje odgovora nosi
  `#[schema(required = true)]`** (TS tip `T | null`, ne `T?`). Enumi: `MonthCode` (`lowercase`: jun, jul, aug, sep),
  `IndexCode` (G, R, Z, O, F), `TrainingCode` i `MaintenanceCode` (`snake_case`), `SeatKind` (human, robot),
  `StrategyCode` (`lowercase`: stedisa, brizni, lovac, nasumicni), `ProfileCode` (A, B, C). Strukture zrcale
  `game::view` polje za polje, s ovim razlikama: identifikatori su `String` (uuid u tekstu),
  indeksi su `Vec<IndexEntry>` (`entries` po `game::dto::INDEX_ORDER`), `PublicHotelBody.seat: SeatBody { kind,
  strategy: Option<StrategyCode> }`. `HotelBody { game_id: String, hotel: PublicHotelBody, private: Option<PrivateHotelBody> }`,
  `DemoBody { game_id: String, hotel_id: String }`. Pretvorbe `From<&game::view::X>`/`From<&app::HotelView>` i
  `From<&PlanBody> for game::dto::PlanDto` žive ovdje. `parse_*`:

```rust
pub fn parse_hotel(s: &str) -> Result<app::HotelId, Problem> {
    s.parse().map(app::HotelId).map_err(|_| Problem::new(StatusCode::NOT_FOUND, ProblemCode::UnknownHotel))
}
// parse_game isto s UnknownGame; parse_month: "jun"|"jul"|"aug"|"sep" → game::dto::MonthCode, inače UnknownMonth.
```

- [ ] **Step 7: Rute.** `health.rs`: `#[schema(required = true)] pub events: Option<i64>` i `state.deps.status`.
  `demo.rs`: `profile_indices` zove `app::profile_indices` i vraća `IndicesBody { profile: String, indices: Vec<IndexEntry> }`
  (`IndexEntry.code` je sad `IndexCode`; JSON isti, test profila C ostaje nepromijenjen); nova ruta:

```rust
/// Novi gost (ako nema kolačić) i nova igra s tri robota (DEMO §1.4).
#[utoipa::path(
    post,
    path = "/v1/demo",
    responses(
        (status = 201, description = "Igra stvorena", body = DemoBody,
         headers(("set-cookie" = String, description = "hotelixsim_guest (HttpOnly) — samo za novog gosta"))),
        (status = 503, description = "Baza nije dostupna", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn start(State(s): State<AppState>, Token(t): Token) -> Result<Response, Problem> {
    let d = app::start_demo(&s.deps, t.as_deref()).await?;
    let body = DemoBody { game_id: d.game_id.0.to_string(), hotel_id: d.hotel_id.0.to_string() };
    let mut res = (StatusCode::CREATED, Json(body)).into_response();
    if let Some(tok) = d.new_token {
        res.headers_mut().insert(header::SET_COOKIE, set_cookie(&tok));
    }
    Ok(res)
}
```
  `hotels.rs`: `GET /v1/me/hotel` (`app::my_hotel`) i `GET /v1/hotels/{hotel_id}` (`app::hotel_view`), oba vraćaju
  `HotelBody`; u `responses` navedi 200, 401, 403, 404 (problem+json). Obrazac rukovatelja:

```rust
pub async fn hotel(State(s): State<AppState>, Token(t): Token, Path(id): Path<String>) -> Result<Json<HotelBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let v = app::hotel_view(&s.deps, &caller, parse_hotel(&id)?).await?;
    Ok(Json(HotelBody::from(&v)))
}
```
  `lib.rs`:

```rust
#[derive(Clone)]
pub struct AppState {
    pub deps: app::Deps,
}

#[derive(OpenApi)]
#[openapi(info(
    title = "HotelixSim API",
    version = "0.1.0",
    description = "Simulacija vođenja hotela za učenje (FMTU Opatija). Igra je dnevnik događaja; ovaj API ga čita i piše.",
    license(name = "Vlasnički — sva prava pridržana (Leon Kreso)")
))]
struct ApiDoc;

fn parts() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health::health))
        .routes(routes!(demo::profile_indices))
        .routes(routes!(demo::start))
        .routes(routes!(hotels::me_hotel))
        .routes(routes!(hotels::hotel))
}

async fn unknown_route() -> Problem {
    Problem::new(StatusCode::NOT_FOUND, ProblemCode::UnknownRoute)
}

pub fn router(state: AppState) -> Router {
    let (router, _) = parts().split_for_parts();
    router.fallback(unknown_route).with_state(state)
}
```
  Ako `ProblemCode`/`IndexCode` ne uđu u `components.schemas` sami (utoipa ih skuplja kroz `Problem`/`IndexEntry`),
  dodaj `components(schemas(ProblemCode, IndexCode))` u `#[openapi(...)]`.

- [ ] **Step 8: Server sastavlja `Deps`.** `crates/server/Cargo.toml`: `app.workspace = true`, `content.workspace = true`.
  U `main.rs`:

```rust
/// Sadržaj se učitava i provjerava pri startu: server s pokvarenim katalogom ne smije ni krenuti (ADR-015 t. 12).
fn content_snapshot() -> anyhow::Result<content::Snapshot> {
    let dir = std::env::var("HOTELIXSIM_CONTENT").map(std::path::PathBuf::from).unwrap_or_else(|_| content::repo_dir());
    let snap = content::Snapshot::read(&dir).map_err(|e| anyhow::anyhow!("sadržaj u {}: {e:?}", dir.display()))?;
    snap.build().map_err(|e| anyhow::anyhow!("sadržaj u {} nije ispravan: {e:?}", dir.display()))?;
    Ok(snap)
}

fn deps(store: store::Store, snapshot: content::Snapshot) -> app::Deps {
    let s = Arc::new(store);
    app::Deps {
        status: s.clone(),
        log: s.clone(),
        guests: s.clone(),
        jobs: s,
        ids: Arc::new(app::RandomIds),
        snapshot: Arc::new(snapshot),
    }
}
```
  `serve()`: `let snapshot = content_snapshot()?;` **prije** `connect_lazy`, zatim
  `api::router(api::AppState { deps: deps(store, snapshot) })`; u log upiši `tracing::info!(ruleset = %snapshot.fingerprint(), …)`
  (izračunaj prije premještanja). Migracije se pri startu **ne** pokreću (naredba `migrate` ostaje zasebna — ADR-015 t. 12).

- [ ] **Step 9: Pokreni** `cargo test -p api; echo EXIT=$?` → `EXIT=0`; `corepack pnpm gen:api; echo EXIT=$?` → `EXIT=0`;
  `corepack pnpm typecheck; echo EXIT=$?` i `corepack pnpm test; echo EXIT=$?` → `EXIT=0` (sučelje čita `code` kao
  string, unija literala mu je dodjeljiva); provjere iz Global Constraints.

- [ ] **Step 10: BACKLOG.** Obriši šest riješenih stavki navedenih u **Files** (povijest je u gitu).

- [ ] **Step 11: Commit** — `crates/api crates/server Cargo.lock openapi.json packages/api-client/src/schema.d.ts
  docs/records/BACKLOG.md`, poruka „Dnevnik 9/12: api — Deps, problem kodovi kao enum, kolačić gosta, rute gosta i hotela".

---

