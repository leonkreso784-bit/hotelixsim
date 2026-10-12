//! Sloj 1 (v2): odjeli hotela i račun dobiti s budžetom (MODEL_V2 §3, §6.5). Samo vlastiti hotel; v1 igra je 409
//! `model_v1`. Tijela su samo brojke (centi, bazni bodovi, bodovi 0–10 000) i imena iz sadržaja.

use axum::extract::{Path, Query, State};
use axum::response::Json;
use game::{DepartmentView, PlScope, PlView, RoundKey};
use serde::{Deserialize, Serialize};
use sim_hotel_v2::department::{DeptOutcome, UndistributedKind, UndistributedOutcome, UsaliClass};
use sim_hotel_v2::usali::{LineKind, Section, UsaliStatement, VarianceLine};
use utoipa::{IntoParams, ToSchema};

use crate::AppState;
use crate::auth::Token;
use crate::problem::Problem;
use crate::schema::{parse_hotel, parse_round};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeptClassCode {
    Rooms,
    FoodBeverage,
    OtherOperated,
    Rentals,
    AdminGeneral,
    SalesMarketing,
    Maintenance,
    Utilities,
}

impl From<UsaliClass> for DeptClassCode {
    fn from(c: UsaliClass) -> Self {
        match c {
            UsaliClass::Rooms => Self::Rooms,
            UsaliClass::FoodBeverage => Self::FoodBeverage,
            UsaliClass::OtherOperated => Self::OtherOperated,
            UsaliClass::Rentals => Self::Rentals,
            UsaliClass::Undistributed(UndistributedKind::AdminGeneral) => Self::AdminGeneral,
            UsaliClass::Undistributed(UndistributedKind::SalesMarketing) => Self::SalesMarketing,
            UsaliClass::Undistributed(UndistributedKind::Maintenance) => Self::Maintenance,
            UsaliClass::Undistributed(UndistributedKind::Utilities) => Self::Utilities,
        }
    }
}

/// Rezultat jednog odjela u jednom obračunatom krugu.
#[derive(Serialize, ToSchema)]
pub struct DeptRoundBody {
    pub round: String,
    pub revenue_cents: i64,
    pub payroll_cents: i64,
    pub cost_of_sales_cents: i64,
    pub other_cents: i64,
    pub profit_cents: i64,
    #[schema(required = true)]
    pub margin_bp: Option<i64>,
    /// Posluženo korištenja (noćenja, obroci…).
    pub served: u32,
    /// Koliko su korištenja gosti htjeli (potražnja); posluženo je najviše kapacitet. 0 za odjel bez gostiju po korištenju.
    pub demand: u32,
    /// Kapacitet u korištenjima za krug.
    pub capacity: u32,
    /// Gostiju po zaposlenom na dan, u stotinkama.
    pub load_hundredths: u16,
    pub usage_bp: u16,
    pub quality_pts: i32,
    pub price_level_pct: u16,
    pub condition_before_pts: i32,
    pub wear_pts: i32,
    pub load_wear_pts: i32,
    pub maintenance_pts: i32,
    pub condition_after_pts: i32,
    pub maintenance_cents: i64,
    pub value_pts: i32,
    pub satisfaction_before_pts: i32,
    pub satisfaction_after_pts: i32,
    /// Zaposleni odjela koji su u krugu otišli (zamijenjeni novima; trošak zamjene je u Upravi).
    pub departures: u16,
}

impl DeptRoundBody {
    fn of(round: RoundKey, o: &DeptOutcome) -> Self {
        DeptRoundBody {
            round: round.code(),
            revenue_cents: o.revenue_cents,
            payroll_cents: o.payroll_cents,
            cost_of_sales_cents: o.cost_of_sales_cents,
            other_cents: o.other_direct_cents,
            profit_cents: o.profit_cents,
            margin_bp: game::department_margin_bp(o),
            served: o.served_uses,
            demand: o.demand_uses,
            capacity: o.capacity_uses,
            load_hundredths: o.load_hundredths,
            usage_bp: o.usage_bp,
            quality_pts: o.quality_pts,
            price_level_pct: o.price_level_pct,
            condition_before_pts: o.condition_before_pts,
            wear_pts: o.wear_pts,
            load_wear_pts: o.load_wear_pts,
            maintenance_pts: o.maintenance_pts,
            condition_after_pts: o.condition_after_pts,
            maintenance_cents: o.maintenance_cents,
            value_pts: o.value_pts,
            satisfaction_before_pts: o.satisfaction_before_pts,
            satisfaction_after_pts: o.satisfaction_after_pts,
            departures: o.departures,
        }
    }
}

/// Neraspoređeni odjel: trošak kruga po stavkama.
#[derive(Serialize, ToSchema)]
pub struct ServiceRoundBody {
    pub round: String,
    pub payroll_cents: i64,
    pub fixed_cents: i64,
    pub per_room_cents: i64,
    pub marketing_cents: i64,
    pub reply_cents: i64,
    pub maintenance_cents: i64,
    pub training_cents: i64,
    pub hiring_cents: i64,
    pub situation_cents: i64,
    pub total_cents: i64,
    /// Zaposleni službe koji su u krugu otišli (zamijenjeni novima; trošak zamjene je u Upravi).
    pub departures: u16,
}

impl ServiceRoundBody {
    fn of(round: RoundKey, u: &UndistributedOutcome) -> Self {
        ServiceRoundBody {
            round: round.code(),
            payroll_cents: u.payroll_cents,
            fixed_cents: u.fixed_cents,
            per_room_cents: u.per_room_cents,
            marketing_cents: u.assigned.marketing_cents,
            reply_cents: u.assigned.reply_cents,
            maintenance_cents: u.assigned.maintenance_cents,
            training_cents: u.assigned.training_cents,
            hiring_cents: u.assigned.hiring_cents,
            situation_cents: u.assigned.situation_cents,
            total_cents: u.total_cents,
            departures: u.departures,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct DepartmentBody {
    pub code: String,
    pub name: String,
    pub class: DeptClassCode,
    pub open: bool,
    pub can_close: bool,
    pub permanent: u16,
    pub seasonal: u16,
    /// Stanje prostora 0–10 000 (na ekranu 0–100).
    pub condition_pts: i32,
    /// Zadovoljstvo gostiju odjelom 0–10 000; `null` za odjele bez gostiju.
    #[schema(required = true)]
    pub guest_satisfaction_pts: Option<i32>,
    /// Rezultat u zadnjem obračunatom krugu; `null` prije prvog obračuna i za neraspoređene odjele.
    #[schema(required = true)]
    pub last: Option<DeptRoundBody>,
    /// Neraspoređeni odjel: trošak zadnjeg kruga po stavkama.
    #[schema(required = true)]
    pub service_last: Option<ServiceRoundBody>,
}

impl From<&DepartmentView> for DepartmentBody {
    fn from(d: &DepartmentView) -> Self {
        DepartmentBody {
            code: d.code.clone(),
            name: d.name.clone(),
            class: d.class.into(),
            open: d.open,
            can_close: d.can_close,
            permanent: d.permanent,
            seasonal: d.seasonal,
            condition_pts: d.condition_pts,
            guest_satisfaction_pts: d.guest_satisfaction_pts,
            last: d.last.as_ref().map(|(r, o)| DeptRoundBody::of(*r, o)),
            service_last: d
                .undistributed_last
                .as_ref()
                .map(|(r, u)| ServiceRoundBody::of(*r, u)),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct DepartmentsBody {
    /// Zadnji obračunati krug; `null` prije prvog obračuna.
    #[schema(required = true)]
    pub round: Option<String>,
    pub departments: Vec<DepartmentBody>,
}

#[derive(Serialize, ToSchema)]
pub struct DepartmentDetailBody {
    pub department: DepartmentBody,
    /// Rezultati po obračunatim krugovima, redom sezone.
    pub history: Vec<DeptRoundBody>,
}

/// Odjeli hotela sa stanjem i zadnjim rezultatom; samo vlasnik, samo v2 igra.
#[utoipa::path(
    get,
    path = "/v1/hotels/{hotel_id}/departments",
    params(("hotel_id" = String, Path, description = "Hotel (uuid)")),
    responses(
        (status = 200, description = "Odjeli hotela", body = DepartmentsBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Igra je na starom modelu (model_v1)", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn departments(
    State(s): State<AppState>,
    Token(t): Token,
    Path(id): Path<String>,
) -> Result<Json<DepartmentsBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let v = app::hotel_departments(&s.deps, &caller, parse_hotel(&id)?).await?;
    Ok(Json(DepartmentsBody {
        round: v.round.map(|r| r.code()),
        departments: v.departments.iter().map(DepartmentBody::from).collect(),
    }))
}

/// Jedan odjel s poviješću rezultata; samo vlasnik, samo v2 igra.
#[utoipa::path(
    get,
    path = "/v1/hotels/{hotel_id}/departments/{code}",
    params(
        ("hotel_id" = String, Path, description = "Hotel (uuid)"),
        ("code" = String, Path, description = "Kod odjela, npr. rooms ili restaurant")
    ),
    responses(
        (status = 200, description = "Odjel s poviješću", body = DepartmentDetailBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Hotel ili odjel ne postoji (unknown_department)", body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Igra je na starom modelu (model_v1)", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn department(
    State(s): State<AppState>,
    Token(t): Token,
    Path((id, code)): Path<(String, String)>,
) -> Result<Json<DepartmentDetailBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let v = app::hotel_department(&s.deps, &caller, parse_hotel(&id)?, &code).await?;
    Ok(Json(DepartmentDetailBody {
        department: DepartmentBody::from(&v.department),
        history: v
            .history
            .iter()
            .map(|(r, o)| DeptRoundBody::of(*r, o))
            .collect(),
    }))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlScopeCode {
    /// Samo traženi krug.
    Round,
    /// Zbroj krugova od prvog do traženog.
    Season,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlSectionCode {
    Revenue,
    DepartmentalExpenses,
    DepartmentalProfit,
    Undistributed,
    Gop,
    FixedCharges,
    Ebitda,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlLineKindCode {
    Revenue,
    Expense,
    Result,
}

/// Jedan redak računa dobiti: ostvareno, budžet i razlika (`variance = actual − budget`). `favorable`: prihod i
/// rezultat su povoljni kad su ≥ budžeta, trošak kad je ≤ budžeta.
#[derive(Serialize, ToSchema)]
pub struct PlLineBody {
    pub code: String,
    pub name: String,
    pub kind: PlLineKindCode,
    pub actual_cents: i64,
    pub budget_cents: i64,
    pub variance_cents: i64,
    pub favorable: bool,
}

#[derive(Serialize, ToSchema)]
pub struct PlSectionBody {
    pub section: PlSectionCode,
    pub lines: Vec<PlLineBody>,
    /// Zbroj sekcije (sekcije s više redaka); GOP i EBITDA su jedan redak u `lines`, pa je `null`.
    #[schema(required = true)]
    pub total: Option<PlLineBody>,
}

/// Ostvareno i budžet jednog omjera ili iznosa; `null` kad je djelitelj nula.
#[derive(Serialize, ToSchema)]
pub struct PlRatioBody {
    #[schema(required = true)]
    pub actual: Option<i64>,
    #[schema(required = true)]
    pub budget: Option<i64>,
}

#[derive(Serialize, ToSchema)]
pub struct PlBody {
    pub round: String,
    pub scope: PlScopeCode,
    pub days: u32,
    pub room_nights: u32,
    /// Obračunati krugovi (za izbornik), redom.
    pub rounds: Vec<String>,
    pub sections: Vec<PlSectionBody>,
    pub gop_margin_bp: PlRatioBody,
    pub goppar_cents: PlRatioBody,
    pub ebitda_margin_bp: PlRatioBody,
}

const SECTIONS: [(Section, PlSectionCode); 7] = [
    (Section::Revenue, PlSectionCode::Revenue),
    (
        Section::DepartmentalExpenses,
        PlSectionCode::DepartmentalExpenses,
    ),
    (
        Section::DepartmentalProfit,
        PlSectionCode::DepartmentalProfit,
    ),
    (Section::Undistributed, PlSectionCode::Undistributed),
    (Section::Gop, PlSectionCode::Gop),
    (Section::FixedCharges, PlSectionCode::FixedCharges),
    (Section::Ebitda, PlSectionCode::Ebitda),
];

/// Ime redaka zbroja i rezultata po sekciji (imena odjela i naknada dolaze iz sadržaja kroz `PlView::names`).
fn fixed_name(section: Section, code: &str) -> Option<&'static str> {
    match (section, code) {
        (Section::Revenue, "total") => Some("Ukupni prihod"),
        (Section::DepartmentalExpenses, "total") => Some("Ukupni troškovi odjela"),
        (Section::DepartmentalProfit, "total") => Some("Ukupna dobit odjela"),
        (Section::Undistributed, "total") => Some("Ukupni neraspoređeni troškovi"),
        (Section::Gop, "gop") => Some("Bruto operativna dobit (GOP)"),
        (Section::FixedCharges, "total") => Some("Ukupne fiksne naknade"),
        (Section::Ebitda, "ebitda") => Some("EBITDA (dobit prije kamata, poreza i amortizacije)"),
        _ => None,
    }
}

fn line_body(pl: &PlView, l: &VarianceLine) -> PlLineBody {
    let name = fixed_name(l.section, &l.code)
        .map(str::to_owned)
        .or_else(|| pl.names.get(&l.code).cloned())
        .unwrap_or_else(|| l.code.clone());
    PlLineBody {
        code: l.code.clone(),
        name,
        kind: match l.kind {
            LineKind::Revenue => PlLineKindCode::Revenue,
            LineKind::Expense => PlLineKindCode::Expense,
            LineKind::Result => PlLineKindCode::Result,
        },
        actual_cents: l.actual_cents,
        budget_cents: l.budget_cents,
        variance_cents: l.variance_cents,
        favorable: l.favorable,
    }
}

fn ratio(actual: Option<i64>, budget: Option<i64>) -> PlRatioBody {
    PlRatioBody { actual, budget }
}

impl PlBody {
    fn of(v: &app::ProfitAndLossView) -> Self {
        let pl = &v.pl;
        let (a, b): (&UsaliStatement, &UsaliStatement) = (&pl.actual, &pl.budget);
        let sections = SECTIONS
            .iter()
            .map(|(section, code)| {
                let of_section = || pl.lines.iter().filter(|l| l.section == *section);
                PlSectionBody {
                    section: *code,
                    lines: of_section()
                        .filter(|l| !l.is_total)
                        .map(|l| line_body(pl, l))
                        .collect(),
                    total: of_section().find(|l| l.is_total).map(|l| line_body(pl, l)),
                }
            })
            .collect();
        PlBody {
            round: pl.round.code(),
            scope: match pl.scope {
                PlScope::Round => PlScopeCode::Round,
                PlScope::Season => PlScopeCode::Season,
            },
            days: a.days,
            room_nights: a.room_nights,
            rounds: v.rounds.iter().map(|r| r.code()).collect(),
            sections,
            gop_margin_bp: ratio(a.gop_margin_bp, b.gop_margin_bp),
            goppar_cents: ratio(a.goppar_cents, b.goppar_cents),
            ebitda_margin_bp: ratio(a.ebitda_margin_bp, b.ebitda_margin_bp),
        }
    }
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PlQuery {
    /// Kod kruga (t01…); bez njega zadnji obračunati krug.
    pub round: Option<String>,
    /// `round` (zadano) ili `season` (zbroj od prvog kruga do traženog).
    pub scope: Option<PlScopeCode>,
}

/// Račun dobiti po USALI-ju s budžetom (ostvareno, budžet i razlika po retku); samo vlasnik, samo v2 igra.
#[utoipa::path(
    get,
    path = "/v1/hotels/{hotel_id}/pl",
    params(("hotel_id" = String, Path, description = "Hotel (uuid)"), PlQuery),
    responses(
        (status = 200, description = "Račun dobiti", body = PlBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, description = "Hotel ili krug ne postoji", body = Problem, content_type = "application/problem+json"),
        (status = 409, description = "Igra je na starom modelu (model_v1), krug nije obračunat ili još nijedan nije (no_resolved_round)", body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn pl(
    State(s): State<AppState>,
    Token(t): Token,
    Path(id): Path<String>,
    Query(q): Query<PlQuery>,
) -> Result<Json<PlBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let hotel = parse_hotel(&id)?;
    let round = q.round.as_deref().map(parse_round).transpose()?;
    let scope = match q.scope.unwrap_or(PlScopeCode::Round) {
        PlScopeCode::Round => PlScope::Round,
        PlScopeCode::Season => PlScope::Season,
    };
    let v = app::hotel_profit_and_loss(&s.deps, &caller, hotel, round, scope).await?;
    Ok(Json(PlBody::of(&v)))
}
