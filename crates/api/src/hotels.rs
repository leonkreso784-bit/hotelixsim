//! Čitanje hotela po ulozi: vlastiti (sa stvarnim brojkama) i tuđi (ocjena i pet indeksa).

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;

use crate::AppState;
use crate::auth::Token;
use crate::problem::{Problem, ProblemCode};
use crate::schema::{
    AnswerBody, FeedBody, FeedItemBody, HotelBody, MonthCode, PlanBody, SituationsBody,
    parse_hotel, parse_month,
};

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

/// Tijelo se ocjenjuje tek nakon provjere gosta, hotela i vlasništva: anonimac, tuđi ili nepoznat hotel
/// dobivaju 401/403/404 i uz loše tijelo, a 422 tek vlasnik hotela. Razlog odbijanja ide u log (bez tijela).
async fn body_or_problem<T>(
    s: &AppState,
    caller: &app::Caller,
    hotel: app::HotelId,
    body: Result<Json<T>, JsonRejection>,
) -> Result<T, Problem> {
    match body {
        Ok(Json(v)) => Ok(v),
        Err(rejection) => {
            let v = app::hotel_view(&s.deps, caller, hotel).await?;
            if v.private.is_none() {
                return Err(app::AppError::new(app::ErrorCode::NotYourHotel).into());
            }
            tracing::debug!(reason = %rejection, "tijelo zahtjeva odbijeno");
            Err(Problem::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                ProblemCode::InvalidBody,
            ))
        }
    }
}

/// Plan za mjesec runde; zadnji zapisani plan vrijedi.
#[utoipa::path(
    put,
    path = "/v1/hotels/{hotel_id}/plans/{month}",
    params(
        ("hotel_id" = String, Path, description = "Hotel (uuid)"),
        ("month" = MonthCode, Path, description = "Mjesec runde")
    ),
    request_body = PlanBody,
    responses(
        (status = 200, description = "Plan je zapisan; zadnji vrijedi", body = PlanBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Runda nije otvorena", body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "Plan izvan granica ili neispravno tijelo", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn put_plan(
    State(s): State<AppState>,
    Token(t): Token,
    Path((id, month)): Path<(String, String)>,
    body: Result<Json<PlanBody>, JsonRejection>,
) -> Result<Json<PlanBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let hotel = parse_hotel(&id)?;
    let month = parse_month(&month)?;
    let plan = body_or_problem(&s, &caller, hotel, body).await?;
    let saved = app::set_plan(&s.deps, &caller, hotel, month, (&plan).into()).await?;
    Ok(Json(PlanBody::from(&saved)))
}

/// Situacije trenutnog mjeseca; odgovori su samo oznake, nikad kvaliteta.
#[utoipa::path(
    get,
    path = "/v1/hotels/{hotel_id}/situations",
    params(("hotel_id" = String, Path, description = "Hotel (uuid)")),
    responses(
        (status = 200, description = "Situacije", body = SituationsBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn situations(
    State(s): State<AppState>,
    Token(t): Token,
    Path(id): Path<String>,
) -> Result<Json<SituationsBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let v = app::situations(&s.deps, &caller, parse_hotel(&id)?).await?;
    Ok(Json(SituationsBody::try_from(&v)?))
}

/// Odgovor na situaciju; jednom po situaciji.
#[utoipa::path(
    post,
    path = "/v1/hotels/{hotel_id}/situations/{code}/answer",
    params(
        ("hotel_id" = String, Path, description = "Hotel (uuid)"),
        ("code" = String, Path, description = "Šifra situacije")
    ),
    request_body = AnswerBody,
    responses(
        (status = 200, description = "Situacije nakon odgovora", body = SituationsBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Već odgovoreno ili runda nije otvorena", body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "Odgovor nije ponuđen ili neispravno tijelo", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn answer(
    State(s): State<AppState>,
    Token(t): Token,
    Path((id, code)): Path<(String, String)>,
    body: Result<Json<AnswerBody>, JsonRejection>,
) -> Result<Json<SituationsBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let hotel = parse_hotel(&id)?;
    let b = body_or_problem(&s, &caller, hotel, body).await?;
    let v = app::answer(&s.deps, &caller, hotel, &code, b.answer).await?;
    Ok(Json(SituationsBody::try_from(&v)?))
}

/// Dnevnik hotela, od najstarijeg: osnova vremenske crte; samo vlasnik.
#[utoipa::path(
    get,
    path = "/v1/hotels/{hotel_id}/feed",
    params(("hotel_id" = String, Path, description = "Hotel (uuid)")),
    responses(
        (status = 200, description = "Dnevnik", body = FeedBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn feed(
    State(s): State<AppState>,
    Token(t): Token,
    Path(id): Path<String>,
) -> Result<Json<FeedBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let items = app::feed(&s.deps, &caller, parse_hotel(&id)?).await?;
    Ok(Json(FeedBody {
        items: items
            .iter()
            .map(FeedItemBody::try_from)
            .collect::<Result<_, _>>()?,
    }))
}
