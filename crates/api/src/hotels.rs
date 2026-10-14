//! Čitanje hotela po ulozi: vlastiti (sa stvarnim brojkama) i tuđi (ocjena i pet indeksa).

use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;

use crate::AppState;
use crate::auth::Token;
use crate::problem::{Problem, ProblemCode};
use crate::schema::{
    AnswerBody, ConversationBody, FeedBody, FeedItemBody, HotelBody, PlanBody, SituationsBody,
    TextBody, parse_hotel, parse_round,
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
        ("month" = String, Path, description = "Kod kruga: jun…sep ili t01…t17")
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
    let month = parse_round(&month)?;
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

/// Prepusti situaciju voditelju odjela (ocjena 2, bez AI-ja); prve tri situacije kruga su obvezne.
#[utoipa::path(
    post,
    path = "/v1/hotels/{hotel_id}/situations/{code}/delegate",
    params(
        ("hotel_id" = String, Path, description = "Hotel (uuid)"),
        ("code" = String, Path, description = "Šifra situacije (primjerak ima točku, npr. `P-KLIMA.t03`)")
    ),
    responses(
        (status = 200, description = "Situacije nakon prepuštanja", body = SituationsBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Obvezna situacija, već odgovoreno ili runda nije otvorena", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn delegate(
    State(s): State<AppState>,
    Token(t): Token,
    Path((id, code)): Path<(String, String)>,
) -> Result<Json<SituationsBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let v = app::delegate_situation(&s.deps, &caller, parse_hotel(&id)?, &code).await?;
    Ok(Json(SituationsBody::try_from(&v)?))
}

/// Poruka liku u razgovoru; odgovor lika stiže kasnije (radnik), pa je odgovor 202.
#[utoipa::path(
    post,
    path = "/v1/hotels/{hotel_id}/situations/{code}/messages",
    params(
        ("hotel_id" = String, Path, description = "Hotel (uuid)"),
        ("code" = String, Path, description = "Šifra situacije")
    ),
    request_body = TextBody,
    responses(
        (status = 202, description = "Poruka je primljena; lik odgovara kasnije", body = ConversationBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Runda nije otvorena, sve poruke iskorištene ili lik još nije odgovorio", body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "Situacija nema lika, tekst prazan ili predug, ili neispravno tijelo", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn send_message(
    State(s): State<AppState>,
    Token(t): Token,
    Path((id, code)): Path<(String, String)>,
    body: Result<Json<TextBody>, JsonRejection>,
) -> Result<(StatusCode, Json<ConversationBody>), Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let hotel = parse_hotel(&id)?;
    let b = body_or_problem(&s, &caller, hotel, body).await?;
    let v = app::send_message(&s.deps, &caller, hotel, &code, b.text).await?;
    Ok((StatusCode::ACCEPTED, Json(ConversationBody::from(&v))))
}

/// Razgovor s likom situacije (otvaranje, poruke, preostali broj poruka).
#[utoipa::path(
    get,
    path = "/v1/hotels/{hotel_id}/situations/{code}/conversation",
    params(
        ("hotel_id" = String, Path, description = "Hotel (uuid)"),
        ("code" = String, Path, description = "Šifra situacije")
    ),
    responses(
        (status = 200, description = "Razgovor", body = ConversationBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "Situacija nema lika", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn conversation(
    State(s): State<AppState>,
    Token(t): Token,
    Path((id, code)): Path<(String, String)>,
) -> Result<Json<ConversationBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let v = app::conversation(&s.deps, &caller, parse_hotel(&id)?, &code).await?;
    Ok(Json(ConversationBody::from(&v)))
}

/// Slobodan odgovor na situaciju; presuda stiže kasnije (radnik), pa je odgovor 202.
#[utoipa::path(
    post,
    path = "/v1/hotels/{hotel_id}/situations/{code}/response",
    params(
        ("hotel_id" = String, Path, description = "Hotel (uuid)"),
        ("code" = String, Path, description = "Šifra situacije")
    ),
    request_body = TextBody,
    responses(
        (status = 202, description = "Odgovor je primljen; presuda stiže kasnije", body = SituationsBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Već odgovoreno ili runda nije otvorena", body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "Situacija nije za slobodan odgovor, tekst prazan ili predug, ili neispravno tijelo", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn submit_response(
    State(s): State<AppState>,
    Token(t): Token,
    Path((id, code)): Path<(String, String)>,
    body: Result<Json<TextBody>, JsonRejection>,
) -> Result<(StatusCode, Json<SituationsBody>), Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let hotel = parse_hotel(&id)?;
    let b = body_or_problem(&s, &caller, hotel, body).await?;
    let v = app::submit_response(&s.deps, &caller, hotel, &code, b.text).await?;
    Ok((StatusCode::ACCEPTED, Json(SituationsBody::try_from(&v)?)))
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
