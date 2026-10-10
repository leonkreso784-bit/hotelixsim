//! Igra i runde: pregled igre, stanje runde i „Zatvori mjesec" (202 sad, 200 već zatvoreno).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;

use crate::AppState;
use crate::auth::Token;
use crate::problem::Problem;
use crate::schema::{GameBody, RoundBody, parse_game, parse_round};

/// Igra u kojoj sudjeluješ: mjeseci, trenutna runda i javni podaci četiriju hotela.
#[utoipa::path(
    get,
    path = "/v1/games/{game_id}",
    params(("game_id" = String, Path, description = "Igra (uuid)")),
    responses(
        (status = 200, description = "Igra", body = GameBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn game(
    State(s): State<AppState>,
    Token(t): Token,
    Path(id): Path<String>,
) -> Result<Json<GameBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let v = app::game_view(&s.deps, &caller, parse_game(&id)?).await?;
    Ok(Json(GameBody::try_from(&v)?))
}

/// Stanje runde; `unanswered` kaže koliko situacija čeka odgovor.
#[utoipa::path(
    get,
    path = "/v1/games/{game_id}/rounds/{month}",
    params(
        ("game_id" = String, Path, description = "Igra (uuid)"),
        ("month" = String, Path, description = "Kod kruga: jun…sep ili t01…t17")
    ),
    responses(
        (status = 200, description = "Stanje runde", body = RoundBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn round(
    State(s): State<AppState>,
    Token(t): Token,
    Path((id, month)): Path<(String, String)>,
) -> Result<Json<RoundBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let r = app::round_status(&s.deps, &caller, parse_game(&id)?, parse_round(&month)?).await?;
    Ok(Json(RoundBody::try_from(&r)?))
}

/// Zatvori mjesec (smije svatko u igri; neodgovorene situacije dobivaju kvalitetu 1).
#[utoipa::path(
    post,
    path = "/v1/games/{game_id}/rounds/{month}/close",
    params(
        ("game_id" = String, Path, description = "Igra (uuid)"),
        ("month" = String, Path, description = "Kod kruga: jun…sep ili t01…t17")
    ),
    responses(
        (status = 202, description = "Zatvoreno sad; obračun čeka radnika", body = RoundBody),
        (status = 200, description = "Već zatvoreno ili obračunato", body = RoundBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Runda još nije otvorena", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn close(
    State(s): State<AppState>,
    Token(t): Token,
    Path((id, month)): Path<(String, String)>,
) -> Result<(StatusCode, Json<RoundBody>), Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let out = app::close_round(&s.deps, &caller, parse_game(&id)?, parse_round(&month)?).await?;
    let status = if out.accepted {
        StatusCode::ACCEPTED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(RoundBody::try_from(&out.round)?)))
}
