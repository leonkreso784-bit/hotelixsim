### Task 5: API (Axum + utoipa) i server

**Files:**
- Create: `crates/api/Cargo.toml`, `crates/api/src/lib.rs`, `crates/api/src/health.rs`, `crates/api/src/demo.rs`, `crates/api/src/problem.rs`, `crates/api/tests/http.rs`
- Create: `crates/server/Cargo.toml`, `crates/server/src/main.rs`
- Modify: `Cargo.toml` (members)

**Interfaces:**
- Consumes: `app::{SystemStatus, check_health, Health, DbState}`, `sim_hotel::{Profile, Index}`, `store::Store`
- Produces:
  - `api::AppState { pub status: Arc<dyn app::SystemStatus> }`
  - `api::router(state: AppState) -> axum::Router`
  - `api::openapi() -> utoipa::openapi::OpenApi`, `api::openapi_json() -> String` (lijepo formatiran JSON + `\n` na kraju)
  - HTTP: `GET /v1/health` → 200 `{"database":"ok","events":N}` ili 503 `{"database":"error","events":null}`; `GET /v1/demo/profiles/{profile}/indices` → 200 `{"profile":"C","indices":[{"code":"G","hundredths":5000},…]}` ili 404 `application/problem+json` `{"type":"about:blank","title":"Nepoznat profil","status":404,"code":"unknown_profile"}`
  - Izvršna datoteka `server` s naredbama `serve` (zadano), `migrate`, `openapi`

- [ ] **Step 1: Testovi koji padaju — `crates/api/tests/http.rs`**

```rust
use std::sync::Arc;

use api::{AppState, openapi_json, router};
use app::{BoxFut, StatusError, SystemStatus};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

struct Fake(Result<i64, String>);

impl SystemStatus for Fake {
    fn events_count(&self) -> BoxFut<'_, Result<i64, StatusError>> {
        let r = self.0.clone().map_err(StatusError);
        Box::pin(async move { r })
    }
}

async fn get(status: Fake, uri: &str) -> (StatusCode, Option<String>, Value) {
    let app = router(AppState { status: Arc::new(status) });
    let res = app.oneshot(Request::get(uri).body(Body::empty()).unwrap()).await.unwrap();
    let code = res.status();
    let ctype = res.headers().get("content-type").map(|v| v.to_str().unwrap().to_owned());
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (code, ctype, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn health_ok_reports_event_count() {
    let (code, _, body) = get(Fake(Ok(0)), "/v1/health").await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body, json!({"database": "ok", "events": 0}));
}

#[tokio::test]
async fn health_with_broken_database_is_503_not_panic() {
    let (code, _, body) = get(Fake(Err("veza odbijena".into())), "/v1/health").await;
    assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, json!({"database": "error", "events": null}));
}

#[tokio::test]
async fn profile_c_indices_are_fifty_in_fixed_order() {
    let (code, _, body) = get(Fake(Ok(0)), "/v1/demo/profiles/C/indices").await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        body,
        json!({"profile": "C", "indices": [
            {"code": "G", "hundredths": 5000},
            {"code": "R", "hundredths": 5000},
            {"code": "Z", "hundredths": 5000},
            {"code": "O", "hundredths": 5000},
            {"code": "F", "hundredths": 5000}
        ]})
    );
}

#[tokio::test]
async fn unknown_profile_is_404_problem_json() {
    for uri in ["/v1/demo/profiles/D/indices", "/v1/demo/profiles/a/indices"] {
        let (code, ctype, body) = get(Fake(Ok(0)), uri).await;
        assert_eq!(code, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(ctype.as_deref(), Some("application/problem+json"), "{uri}");
        assert_eq!(body["code"], "unknown_profile", "{uri}");
        assert_eq!(body["status"], 404, "{uri}");
    }
}

#[test]
fn openapi_is_version_3_1_and_lists_paths() {
    let doc: Value = serde_json::from_str(&openapi_json()).unwrap();
    assert_eq!(doc["openapi"], "3.1.0", "TS generator čita samo 3.0 i 3.1");
    assert!(doc["paths"]["/v1/health"]["get"].is_object());
    assert!(doc["paths"]["/v1/demo/profiles/{profile}/indices"]["get"].is_object());
}

#[test]
fn openapi_output_is_stable() {
    assert_eq!(openapi_json(), openapi_json());
    assert!(openapi_json().ends_with('\n'));
}
```

- [ ] **Step 2: Paket `api` — `Cargo.toml` i prazan `lib.rs`, pa pokreni da padne**

`crates/api/Cargo.toml`:

```toml
[package]
name = "api"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
app.workspace = true
sim-hotel.workspace = true
axum.workspace = true
serde.workspace = true
serde_json.workspace = true
utoipa.workspace = true
utoipa-axum.workspace = true

[dev-dependencies]
http-body-util.workspace = true
tokio.workspace = true
tower.workspace = true

[lints]
workspace = true
```

Dodaj `"crates/api"` i `"crates/server"` u `members`. Privremeno stvori `crates/api/src/lib.rs` s jednim retkom `//! API.`

Run: `cargo test -p api; echo EXIT=$?` → greške prevođenja (`router`, `AppState`, `openapi_json` ne postoje), `EXIT=101`.

- [ ] **Step 3: Izvedba**

`crates/api/src/problem.rs`:

```rust
//! Greške u obliku application/problem+json (RFC 9457), s kodom koji sučelje prevodi.

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Problem {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub title: &'static str,
    pub status: u16,
    pub code: &'static str,
}

impl Problem {
    pub fn not_found(title: &'static str, code: &'static str) -> Self {
        Self { kind: "about:blank", title, status: 404, code }
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = serde_json::to_vec(&self).unwrap_or_default();
        (status, [(header::CONTENT_TYPE, "application/problem+json")], body).into_response()
    }
}
```

`crates/api/src/health.rs`:

```rust
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseState {
    Ok,
    Error,
}

#[derive(Serialize, ToSchema)]
pub struct HealthBody {
    pub database: DatabaseState,
    pub events: Option<i64>,
}

/// Zdravlje sustava: je li baza dostupna i koliko je događaja u dnevniku.
#[utoipa::path(
    get,
    path = "/v1/health",
    responses(
        (status = 200, description = "Sve radi", body = HealthBody),
        (status = 503, description = "Baza nije dostupna", body = HealthBody)
    )
)]
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let h = app::check_health(state.status.as_ref()).await;
    let (code, database) = match h.database {
        app::DbState::Ok => (StatusCode::OK, DatabaseState::Ok),
        app::DbState::Error => (StatusCode::SERVICE_UNAVAILABLE, DatabaseState::Error),
    };
    (code, Json(HealthBody { database, events: h.events }))
}
```

`crates/api/src/demo.rs`:

```rust
use axum::extract::Path;
use axum::response::{IntoResponse, Json, Response};
use serde::Serialize;
use sim_hotel::Profile;
use utoipa::ToSchema;

use crate::problem::Problem;

#[derive(Serialize, ToSchema)]
pub struct IndexEntry {
    /// Kod indeksa: G, R, Z, O ili F.
    pub code: String,
    /// Vrijednost u stotinkama: 0–10000 znači 0,00–100,00.
    pub hundredths: u16,
}

#[derive(Serialize, ToSchema)]
pub struct IndicesBody {
    pub profile: String,
    pub indices: Vec<IndexEntry>,
}

/// Početni indeksi profila (DEMO §5).
#[utoipa::path(
    get,
    path = "/v1/demo/profiles/{profile}/indices",
    params(("profile" = String, Path, description = "Profil hotela: A, B ili C")),
    responses(
        (status = 200, description = "Početni indeksi", body = IndicesBody),
        (status = 404, description = "Nepoznat profil", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn profile_indices(Path(profile): Path<String>) -> Response {
    let Some(p) = Profile::from_code(&profile) else {
        return Problem::not_found("Nepoznat profil", "unknown_profile").into_response();
    };
    let indices = p
        .starting_indices()
        .iter()
        .map(|(i, v)| IndexEntry { code: i.code().to_owned(), hundredths: v.hundredths() })
        .collect();
    Json(IndicesBody { profile: p.code().to_owned(), indices }).into_response()
}
```

`crates/api/src/lib.rs`:

```rust
//! HTTP sloj: Axum rukovatelji i OpenAPI opis (ARHITEKTURA §11). Tanak: logika je u `app` i jezgri.

mod demo;
mod health;
mod problem;

use std::sync::Arc;

use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Clone)]
pub struct AppState {
    pub status: Arc<dyn app::SystemStatus>,
}

#[derive(OpenApi)]
#[openapi(info(title = "HotelixSim API", version = "0.1.0"))]
struct ApiDoc;

fn parts() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health::health))
        .routes(routes!(demo::profile_indices))
}

pub fn router(state: AppState) -> Router {
    let (router, _) = parts().split_for_parts();
    router.with_state(state)
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let (_, doc) = parts().split_for_parts();
    doc
}

pub fn openapi_json() -> String {
    let mut s = serde_json::to_string_pretty(&openapi()).expect("OpenAPI se uvijek serijalizira");
    s.push('\n');
    s
}
```

- [ ] **Step 4: Pokreni — mora proći**

Run: `cargo test -p api; echo EXIT=$?` → Expected: `6 passed; 0 failed`, `EXIT=0`.

Ako pada **samo** `openapi_is_version_3_1_and_lists_paths` s `"3.2.0"`: utoipa 6 po zadanom daje 3.2. U `openapi()` prije `doc` postavi verziju 3.1. U utoipa 5 to je `doc.openapi = utoipa::openapi::OpenApiVersion::Version31;`; za točan naziv varijante u 6.0.0 pogledaj `https://docs.rs/utoipa/6.0.0/utoipa/openapi/enum.OpenApiVersion.html`. Ne mijenjaj test.

Ako `routes!` ne prihvaća rukovatelj zbog oblika povratnog tipa, ne mijenjaj ponašanje ni test; prilagodi samo potpis (npr. `-> Response`) i zapiši razlog u izvještaj taska.

- [ ] **Step 5: Server — `crates/server/Cargo.toml` i `src/main.rs`**

```toml
[package]
name = "server"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
api.workspace = true
store.workspace = true
anyhow.workspace = true
axum.workspace = true
tokio.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true

[lints]
workspace = true
```

```rust
//! Kompozicijski korijen: jedino mjesto gdje se sve spaja (ARHITEKTURA §6.2).
//! Naredbe: `serve` (zadano), `migrate`, `openapi`.

use std::sync::Arc;

use anyhow::{Context, bail};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "serve".to_owned());
    match cmd.as_str() {
        "openapi" => {
            print!("{}", api::openapi_json());
            Ok(())
        }
        "migrate" => {
            let store = store::Store::connect_lazy(&database_url()?)?;
            store.migrate().await.context("migracija nije uspjela")?;
            println!("migracije primijenjene");
            Ok(())
        }
        "serve" => serve().await,
        other => bail!("nepoznata naredba: {other} (serve | migrate | openapi)"),
    }
}

fn database_url() -> anyhow::Result<String> {
    std::env::var("DATABASE_URL").context("nedostaje DATABASE_URL (vidi .env.example)")
}

async fn serve() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();
    let addr = std::env::var("HOTELIXSIM_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
    let store = store::Store::connect_lazy(&database_url()?)?;
    let app = api::router(api::AppState { status: Arc::new(store) });
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("ne mogu slušati na {addr} (je li port zauzet?)"))?;
    tracing::info!("HotelixSim API sluša na http://{addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
```

`server` ne čita `.env` sam. Lokalno se varijable učitaju iz ljuske; README (Task 8) to opisuje.

- [ ] **Step 6: Provjera servera**

Run: `cargo run -q -p server -- openapi > <apsolutna putanja scratchpada>/o1.json; echo EXIT=$?` → `EXIT=0`, datoteka počinje s `{` i sadrži `"openapi": "3.1.0"`. (Varijable `$TMP`/`$SCRATCHPAD` na ovom stroju ne postoje; koristi apsolutnu putanju.)
Run: `cargo run -q -p server -- nepostoji; echo EXIT=$?` → poruka `nepoznata naredba: nepostoji`, `EXIT=1`.
Run: `cargo clippy --workspace --all-targets -- -D warnings; echo EXIT=$?` → `EXIT=0`.
Run: `cargo test --workspace; echo EXIT=$?` → svi testovi prolaze, `EXIT=0`. Čitaj završne retke `test result:` za svaki paket, ne međuzbroj.

- [ ] **Step 7: Commit** — `git add Cargo.toml Cargo.lock crates/api crates/server` · poruka: „Kostur 5/8: API (zdravlje, indeksi profila) i server"

---

