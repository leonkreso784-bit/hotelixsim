//! Čitanje hotela po ulozi: vlastiti (sa stvarnim brojkama) i tuđi (ocjena i pet indeksa).

use axum::extract::{Path, State};
use axum::response::Json;

use crate::AppState;
use crate::auth::Token;
use crate::problem::Problem;
use crate::schema::{HotelBody, parse_hotel};

/// Moj hotel u mojoj zadnjoj igri, sa stvarnim brojkama i planom trenutnog mjeseca.
#[utoipa::path(
    get,
    path = "/v1/me/hotel",
    responses(
        (status = 200, description = "Moj hotel", body = HotelBody),
        (status = 401, description = "Potreban je gost", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Nisi u ovoj igri", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Gost još nema igru", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn me_hotel(
    State(s): State<AppState>,
    Token(t): Token,
) -> Result<Json<HotelBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let v = app::my_hotel(&s.deps, &caller).await?;
    Ok(Json(HotelBody::try_from(&v)?))
}

/// Hotel u igri u kojoj sudjeluješ; tuđi hotel bez stvarnih brojki.
#[utoipa::path(
    get,
    path = "/v1/hotels/{hotel_id}",
    params(("hotel_id" = String, Path, description = "Identifikator hotela (uuid)")),
    responses(
        (status = 200, description = "Hotel", body = HotelBody),
        (status = 401, description = "Potreban je gost", body = Problem, content_type = "application/problem+json"),
        (status = 403, description = "Nisi u ovoj igri", body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Hotel ne postoji", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn hotel(
    State(s): State<AppState>,
    Token(t): Token,
    Path(id): Path<String>,
) -> Result<Json<HotelBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let v = app::hotel_view(&s.deps, &caller, parse_hotel(&id)?).await?;
    Ok(Json(HotelBody::try_from(&v)?))
}
