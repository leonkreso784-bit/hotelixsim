use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Json, Response};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::auth::{Token, set_cookie};
use crate::problem::{Problem, ProblemCode};
use crate::schema::{DemoBody, IndexEntry, NewGameBody, ProfileCode, entries};

#[derive(Serialize, ToSchema)]
pub struct IndicesBody {
    pub profile: ProfileCode,
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
pub async fn profile_indices(Path(profile): Path<String>) -> Result<Json<IndicesBody>, Problem> {
    let v = app::profile_indices(&profile)?;
    let profile = ProfileCode::from_code(&profile).ok_or_else(|| {
        Problem::new(
            StatusCode::NOT_FOUND,
            crate::problem::ProblemCode::UnknownProfile,
        )
    })?;
    Ok(Json(IndicesBody {
        profile,
        indices: entries(v),
    }))
}

/// Novi gost (ako nema kolačić) i nova igra s tri robota (DEMO §1.4).
#[utoipa::path(
    post,
    path = "/v1/demo",
    request_body(content = Option<NewGameBody>, description = "Neobavezne postavke igre (model, vrsta kruga)", content_type = "application/json"),
    responses(
        (status = 201, description = "Igra stvorena", body = DemoBody,
         headers(("set-cookie" = String, description = "hotelixsim_guest (HttpOnly), samo za novog gosta"))),
        (status = 422, description = "Postavke nisu dopuštene ili tijelo nije ispravno", body = Problem, content_type = "application/problem+json"),
        (status = 503, description = "Baza nije dostupna", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn start(
    State(s): State<AppState>,
    Token(t): Token,
    body: Bytes,
) -> Result<Response, Problem> {
    let settings = if body.is_empty() {
        app::NewGameSettings::default()
    } else {
        serde_json::from_slice::<NewGameBody>(&body)
            .map_err(|_| Problem::new(StatusCode::UNPROCESSABLE_ENTITY, ProblemCode::InvalidBody))?
            .into()
    };
    let d = app::start_demo(&s.deps, t.as_deref(), settings).await?;
    let body = DemoBody {
        game_id: d.game_id.0.to_string(),
        hotel_id: d.hotel_id.0.to_string(),
    };
    let mut res = (StatusCode::CREATED, Json(body)).into_response();
    if let Some(tok) = d.new_token {
        res.headers_mut()
            .insert(header::SET_COOKIE, set_cookie(&tok));
    }
    Ok(res)
}
