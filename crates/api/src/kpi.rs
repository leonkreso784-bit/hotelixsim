//! Sloj 0 — hotelski pokazatelji (MODEL_V2 §5): ADR, RevPAR, GOPPAR, točka pokrića… po obračunatom mjesecu.
//! Samo brojke (centi, bazni bodovi, noći); objašnjenja riječima su na webu.

use axum::extract::{Path, State};
use axum::response::Json;
use game::kpi::KpiMonth;
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::auth::Token;
use crate::problem::Problem;
use crate::schema::{MonthCode, calendar_month, parse_hotel};

/// Pokazatelji jednog obračunatog mjeseca. Sve „po sobi" dijeli se sa svim sobama × dani (`room_nights`), pa vrijedi
/// RevPAR = ADR × popunjenost. Iznosi u centima, udjeli u baznim bodovima (10 000 = 100 %); `null` kad je
/// djelitelj nula (npr. ADR u mjesecu bez prodaje).
#[derive(Serialize, ToSchema)]
pub struct KpiMonthBody {
    pub month: MonthCode,
    /// Sve sobe × dani mjeseca.
    pub room_nights: u32,
    pub sold_nights: u32,
    /// Noći soba izvan funkcije (zbog opreme).
    pub lost_nights: u32,
    #[schema(required = true)]
    pub occupancy_bp: Option<i64>,
    /// Najviša moguća popunjenost: sobe u funkciji × dani ÷ sve noći.
    #[schema(required = true)]
    pub sellable_occupancy_bp: Option<i64>,
    #[schema(required = true)]
    pub adr_cents: Option<i64>,
    #[schema(required = true)]
    pub revpar_cents: Option<i64>,
    pub rooms_revenue_cents: i64,
    /// Prihod od soba + ostali prihod.
    pub total_revenue_cents: i64,
    #[schema(required = true)]
    pub trevpar_cents: Option<i64>,
    pub gop_cents: i64,
    #[schema(required = true)]
    pub gop_margin_bp: Option<i64>,
    #[schema(required = true)]
    pub goppar_cents: Option<i64>,
    pub labor_cents: i64,
    #[schema(required = true)]
    pub labor_share_bp: Option<i64>,
    /// Izravni trošak po prodanoj noći iz kategorije hotela (katalog, bez plaća; isti svaki mjesec). Nije USALI
    /// CPOR (svi troškovi odjela soba, i plaće) — taj stiže sa slojem 1; ime polja ostaje zbog kompatibilnosti.
    #[schema(required = true)]
    pub cpor_cents: Option<i64>,
    /// Najniža popunjenost (zaokruženo naviše) pri kojoj GOP nije negativan, uz isti ADR i iste ostale troškove;
    /// može biti iznad `sellable_occupancy_bp` i iznad 10 000.
    #[schema(required = true)]
    pub break_even_occupancy_bp: Option<i64>,
}

impl TryFrom<&KpiMonth> for KpiMonthBody {
    type Error = Problem;

    fn try_from(m: &KpiMonth) -> Result<Self, Problem> {
        let k = &m.kpi;
        Ok(KpiMonthBody {
            month: calendar_month(m.round)?,
            room_nights: k.room_nights,
            sold_nights: k.sold_nights,
            lost_nights: k.lost_nights,
            occupancy_bp: k.occupancy_bp,
            sellable_occupancy_bp: k.sellable_occupancy_bp,
            adr_cents: k.adr_cents,
            revpar_cents: k.revpar_cents,
            rooms_revenue_cents: k.rooms_revenue_cents,
            total_revenue_cents: k.total_revenue_cents,
            trevpar_cents: k.trevpar_cents,
            gop_cents: k.gop_cents,
            gop_margin_bp: k.gop_margin_bp,
            goppar_cents: k.goppar_cents,
            labor_cents: k.labor_cents,
            labor_share_bp: k.labor_share_bp,
            cpor_cents: k.cpor_cents,
            break_even_occupancy_bp: k.break_even_occupancy_bp,
        })
    }
}

/// Obračunati mjeseci redom sezone; prazno prije prvog obračuna.
#[derive(Serialize, ToSchema)]
pub struct KpiBody {
    pub months: Vec<KpiMonthBody>,
}

/// Hotelski pokazatelji po obračunatom mjesecu (računaju se pri čitanju); samo vlasnik.
#[utoipa::path(
    get,
    path = "/v1/hotels/{hotel_id}/kpi",
    params(("hotel_id" = String, Path, description = "Hotel (uuid)")),
    responses(
        (status = 200, description = "Pokazatelji obračunatih mjeseci", body = KpiBody),
        (status = 401, body = Problem, content_type = "application/problem+json"),
        (status = 403, body = Problem, content_type = "application/problem+json"),
        (status = 404, body = Problem, content_type = "application/problem+json")
    )
)]
pub async fn kpi(
    State(s): State<AppState>,
    Token(t): Token,
    Path(id): Path<String>,
) -> Result<Json<KpiBody>, Problem> {
    let caller = app::caller(&s.deps, t.as_deref()).await?;
    let months = app::hotel_kpi(&s.deps, &caller, parse_hotel(&id)?).await?;
    Ok(Json(KpiBody {
        months: months
            .iter()
            .map(KpiMonthBody::try_from)
            .collect::<Result<_, _>>()?,
    }))
}
