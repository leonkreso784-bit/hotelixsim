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
