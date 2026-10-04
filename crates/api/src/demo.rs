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
        .map(|(i, v)| IndexEntry {
            code: i.code().to_owned(),
            hundredths: v.hundredths(),
        })
        .collect();
    Json(IndicesBody {
        profile: p.code().to_owned(),
        indices,
    })
    .into_response()
}
