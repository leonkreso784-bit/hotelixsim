//! HTTP sloj: Axum rukovatelji i OpenAPI opis (ARHITEKTURA §11). Tanak: logika je u `app` i jezgri.

pub mod auth;
mod demo;
mod departments;
mod explanation;
mod games;
mod gate;
mod health;
mod hotels;
mod kpi;
pub mod problem;
pub mod schema;
mod schema_situations;
mod web;

use std::path::PathBuf;

use axum::Router;
use axum::http::StatusCode;
use problem::{Problem, ProblemCode};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

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
        .routes(routes!(hotels::put_plan))
        .routes(routes!(hotels::situations))
        .routes(routes!(hotels::answer))
        .routes(routes!(hotels::delegate))
        .routes(routes!(hotels::send_message))
        .routes(routes!(hotels::conversation))
        .routes(routes!(hotels::submit_response))
        .routes(routes!(hotels::feed))
        .routes(routes!(explanation::explanation))
        .routes(routes!(kpi::kpi))
        .routes(routes!(departments::departments))
        .routes(routes!(departments::department))
        .routes(routes!(departments::pl))
        .routes(routes!(games::game))
        .routes(routes!(games::season))
        .routes(routes!(games::round))
        .routes(routes!(games::close))
}

async fn unknown_route() -> Problem {
    Problem::new(StatusCode::NOT_FOUND, ProblemCode::UnknownRoute)
}

/// Postavke usmjerivača. Zadano = lokalni način: bez `Secure`, bez vrata, bez weba.
#[derive(Clone, Debug, Default)]
pub struct RouterConfig {
    /// Kolačić gosta dobiva atribut `Secure` (produkcija, HTTPS).
    pub secure_cookie: bool,
    /// Lozinka stranice (Basic Auth, samo lozinka); `None` ili prazno = bez vrata.
    pub site_password: Option<String>,
    /// Mapa izgrađenog weba (`apps/web/dist`); `None` = poslužuje se samo API.
    pub web_dir: Option<PathBuf>,
}

/// Lokalni način: kolačić bez `Secure` (http), bez vrata, bez weba.
pub fn router(state: AppState) -> Router {
    router_with_config(state, RouterConfig::default())
}

/// `secure = true` u produkciji (HTTPS): kolačić gosta dobiva atribut `Secure`.
pub fn router_with_secure_cookie(state: AppState, secure: bool) -> Router {
    router_with_config(
        state,
        RouterConfig {
            secure_cookie: secure,
            ..RouterConfig::default()
        },
    )
}

pub fn router_with_config(state: AppState, cfg: RouterConfig) -> Router {
    let (router, _) = parts().split_for_parts();
    let router = match &cfg.web_dir {
        Some(dir) => router.fallback(web::fallback(dir)),
        None => router.fallback(unknown_route),
    };
    let router = router
        .layer(axum::Extension(auth::CookieMode {
            secure: cfg.secure_cookie,
        }))
        .with_state(state);
    match gate::Gate::new(cfg.site_password.as_deref()) {
        Some(g) => router.layer(axum::middleware::from_fn_with_state(
            g,
            gate::require_password,
        )),
        None => router,
    }
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
