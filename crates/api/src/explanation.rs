//! „Zašto se indeksi pomaknuli" za ekran „Mjesec završen": iste činjenice koje dobiva mentor
//! (`game::debrief_facts`), samo s engleskim imenima polja. Tekstovi su već ljudski i prikazuju se kakvi jesu.

use axum::extract::{Path, State};
use axum::response::Json;
use game::{ChangeFact, DebriefFacts, DriverFact, DriverSource, Effect, IndexFact};
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::auth::Token;
use crate::problem::Problem;
use crate::schema::{IndexCode, MonthCode, VarCode, parse_hotel, parse_month};

/// Što pokreće indeks: jedna od varijabli hotela ili `staff_load` (opterećenje osoblja, nije varijabla).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DriverCode {
    Sold,
    Revenue,
    Costs,
    Cash,
    Staff,
    Morale,
    Experience,
    Equipment,
    Satisfaction,
    Rating,
    StaffLoad,
}

impl From<DriverSource> for DriverCode {
    fn from(s: DriverSource) -> Self {
        match s {
            DriverSource::StaffLoad => DriverCode::StaffLoad,
            DriverSource::Var(v) => match VarCode::from(v) {
                VarCode::Sold => DriverCode::Sold,
                VarCode::Revenue => DriverCode::Revenue,
                VarCode::Costs => DriverCode::Costs,
                VarCode::Cash => DriverCode::Cash,
                VarCode::Staff => DriverCode::Staff,
                VarCode::Morale => DriverCode::Morale,
                VarCode::Experience => DriverCode::Experience,
                VarCode::Equipment => DriverCode::Equipment,
                VarCode::Satisfaction => DriverCode::Satisfaction,
                VarCode::Rating => DriverCode::Rating,
            },
        }
    }
}

/// Je li redak gurnuo indeks gore ili dolje (boja na ekranu); razina bez smjera je `neutral`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum EffectCode {
    Up,
    Down,
    Neutral,
}

impl From<Effect> for EffectCode {
    fn from(e: Effect) -> Self {
        match e {
            Effect::Up => EffectCode::Up,
            Effect::Down => EffectCode::Down,
            Effect::Neutral => EffectCode::Neutral,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ExplanationLineBody {
    /// Uzrok riječima („plaće", „situacija „…"").
    pub cause: String,
    /// Iznos s jedinicom, već formatiran („−10 bodova", „12.346 €").
    pub amount: String,
    pub effect: EffectCode,
}

#[derive(Serialize, ToSchema)]
pub struct ExplanationDriverBody {
    pub var: DriverCode,
    pub name: String,
    /// Jedna rečenica: što ova varijabla znači i kako čitati iznose.
    pub meaning: String,
    pub lines: Vec<ExplanationLineBody>,
}

#[derive(Serialize, ToSchema)]
pub struct ExplanationIndexBody {
    pub code: IndexCode,
    pub name: String,
    /// Cijeli bodovi na početku mjeseca.
    pub before: i64,
    /// Cijeli bodovi na kraju mjeseca.
    pub after: i64,
    /// `after − before` (razlika zaokruženih vrijednosti, kao na ekranu).
    pub change: i64,
    pub drivers: Vec<ExplanationDriverBody>,
}

/// Objašnjenje obračunatog mjeseca po indeksima, redom G, R, Z, O, F.
#[derive(Serialize, ToSchema)]
pub struct ExplanationBody {
    pub month: MonthCode,
    /// Napredak od početka igre riječima („−2 boda").
    pub progress_text: String,
    pub indices: Vec<ExplanationIndexBody>,
}

impl From<&ChangeFact> for ExplanationLineBody {
    fn from(c: &ChangeFact) -> Self {
        ExplanationLineBody {
            cause: c.cause.clone(),
            amount: c.amount.clone(),
            effect: c.effect.into(),
        }
    }
}

impl From<&DriverFact> for ExplanationDriverBody {
    fn from(d: &DriverFact) -> Self {
        ExplanationDriverBody {
            var: d.source.into(),
            name: d.what.to_owned(),
            meaning: d.meaning.to_owned(),
            lines: d.changes.iter().map(Into::into).collect(),
        }
    }
}

impl From<&IndexFact> for ExplanationIndexBody {
    fn from(i: &IndexFact) -> Self {
        ExplanationIndexBody {
            code: game::dto::IndexCode::from(i.index).into(),
            name: i.name.to_owned(),
            before: i.before,
            after: i.after,
            change: i.change,
            drivers: i.drivers.iter().map(Into::into).collect(),
        }
    }
}

impl ExplanationBody {
    fn new(month: MonthCode, f: &DebriefFacts) -> Self {
        ExplanationBody {
            month,
            progress_text: f.progress.clone(),
            indices: f.indices.iter().map(Into::into).collect(),
        }
    }
}

/// Zašto su se indeksi obračunatog mjeseca pomaknuli (iste činjenice kao mentor); samo vlasnik.
#[utoipa::path(
    get,
    path = "/v1/hotels/{hotel_id}/months/{month}/explanation",
    params(
        ("hotel_id" = String, Path, description = "Hotel (uuid)"),
        ("month" = MonthCode, Path, description = "Obračunat mjesec")
    ),
    responses(
        (status = 200, description = "Objašnjenje mjeseca po indeksima", body = ExplanationBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Mjesec još nije obračunat", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn explanation(
    State(s): State<AppState>,
    Token(t): Token,
    Path((id, month)): Path<(String, String)>,
) -> Result<Json<ExplanationBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let hotel = parse_hotel(&id)?;
    let month = parse_month(&month)?;
    let f = app::explanation(&s.deps, &caller, hotel, month).await?;
    Ok(Json(ExplanationBody::new(month.into(), &f)))
}
