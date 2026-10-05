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
    /// Broj događaja u dnevniku; `null` kad baza nije dostupna.
    #[schema(required = true)]
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
    let h = app::check_health(state.deps.status.as_ref()).await;
    let (code, database) = match h.database {
        app::DbState::Ok => (StatusCode::OK, DatabaseState::Ok),
        app::DbState::Error => (StatusCode::SERVICE_UNAVAILABLE, DatabaseState::Error),
    };
    (
        code,
        Json(HealthBody {
            database,
            events: h.events,
        }),
    )
}
