//! HTTP sloj: Axum rukovatelji i OpenAPI opis (ARHITEKTURA §11). Tanak: logika je u `app` i jezgri.

pub mod auth;
mod demo;
mod games;
mod health;
mod hotels;
pub mod problem;
pub mod schema;

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
        .routes(routes!(hotels::send_message))
        .routes(routes!(hotels::conversation))
        .routes(routes!(hotels::submit_response))
        .routes(routes!(hotels::feed))
        .routes(routes!(games::game))
        .routes(routes!(games::round))
        .routes(routes!(games::close))
}

async fn unknown_route() -> Problem {
    Problem::new(StatusCode::NOT_FOUND, ProblemCode::UnknownRoute)
}

pub fn router(state: AppState) -> Router {
    let (router, _) = parts().split_for_parts();
    router.fallback(unknown_route).with_state(state)
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
