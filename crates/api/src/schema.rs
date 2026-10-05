//! Oblici JSON-a koje API vidi (OpenAPI). Zrcale `game::view` i `game::dto` polje za polje;
//! jezgra nema serde ni utoipa pa se pretvara ovdje. Polje `Option` u odgovoru je uvijek `required`
//! (TS tip `T | null`, ne `T?`).

use axum::http::StatusCode;
use game::command::PlanLimits;
use game::dto::{INDEX_ORDER, IndexCode as CoreIndex, PlanDto};
use game::view::{LastMonth, MonthScore, PlanState, PrivateHotel, PublicHotel, RealNumbers, Seat};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::problem::{Problem, ProblemCode};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MonthCode {
    Jun,
    Jul,
    Aug,
    Sep,
}

impl From<game::dto::MonthCode> for MonthCode {
    fn from(m: game::dto::MonthCode) -> Self {
        use game::dto::MonthCode as M;
        match m {
            M::Jun => MonthCode::Jun,
            M::Jul => MonthCode::Jul,
            M::Aug => MonthCode::Aug,
            M::Sep => MonthCode::Sep,
        }
    }
}

impl From<MonthCode> for game::dto::MonthCode {
    fn from(m: MonthCode) -> Self {
        match m {
            MonthCode::Jun => Self::Jun,
            MonthCode::Jul => Self::Jul,
            MonthCode::Aug => Self::Aug,
            MonthCode::Sep => Self::Sep,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum IndexCode {
    G,
    R,
    Z,
    O,
    F,
}

impl From<CoreIndex> for IndexCode {
    fn from(c: CoreIndex) -> Self {
        match c {
            CoreIndex::G => IndexCode::G,
            CoreIndex::R => IndexCode::R,
            CoreIndex::Z => IndexCode::Z,
            CoreIndex::O => IndexCode::O,
            CoreIndex::F => IndexCode::F,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct IndexEntry {
    /// Kod indeksa: G, R, Z, O ili F.
    pub code: IndexCode,
    /// Vrijednost u stotinkama: 0–10000 znači 0,00–100,00.
    pub hundredths: u16,
}

/// Pet indeksa u stalnom redoslijedu G, R, Z, O, F.
pub fn entries(v: [u16; 5]) -> Vec<IndexEntry> {
    INDEX_ORDER
        .iter()
        .zip(v)
        .map(|(&c, hundredths)| IndexEntry {
            code: c.into(),
            hundredths,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrainingCode {
    None,
    Basic,
    Full,
}

impl From<game::dto::TrainingCode> for TrainingCode {
    fn from(t: game::dto::TrainingCode) -> Self {
        use game::dto::TrainingCode as T;
        match t {
            T::None => TrainingCode::None,
            T::Basic => TrainingCode::Basic,
            T::Full => TrainingCode::Full,
        }
    }
}

impl From<TrainingCode> for game::dto::TrainingCode {
    fn from(t: TrainingCode) -> Self {
        match t {
            TrainingCode::None => Self::None,
            TrainingCode::Basic => Self::Basic,
            TrainingCode::Full => Self::Full,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceCode {
    Deferred,
    Regular,
    Intensive,
}

impl From<game::dto::MaintenanceCode> for MaintenanceCode {
    fn from(m: game::dto::MaintenanceCode) -> Self {
        use game::dto::MaintenanceCode as M;
        match m {
            M::Deferred => MaintenanceCode::Deferred,
            M::Regular => MaintenanceCode::Regular,
            M::Intensive => MaintenanceCode::Intensive,
        }
    }
}

impl From<MaintenanceCode> for game::dto::MaintenanceCode {
    fn from(m: MaintenanceCode) -> Self {
        match m {
            MaintenanceCode::Deferred => Self::Deferred,
            MaintenanceCode::Regular => Self::Regular,
            MaintenanceCode::Intensive => Self::Intensive,
        }
    }
}

/// Plan mjeseca: tijelo zahtjeva i dio odgovora.
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanBody {
    pub price_cents: i64,
    pub marketing_cents: i64,
    pub seasonal_target: u16,
    pub wage_pct: u16,
    pub training: TrainingCode,
    pub maintenance: MaintenanceCode,
    pub renovation: bool,
    pub reply_reviews: bool,
}

impl From<&PlanDto> for PlanBody {
    fn from(p: &PlanDto) -> Self {
        PlanBody {
            price_cents: p.price_cents,
            marketing_cents: p.marketing_cents,
            seasonal_target: p.seasonal_target,
            wage_pct: p.wage_pct,
            training: p.training.into(),
            maintenance: p.maintenance.into(),
            renovation: p.renovation,
            reply_reviews: p.reply_reviews,
        }
    }
}

impl From<&PlanBody> for PlanDto {
    fn from(p: &PlanBody) -> Self {
        PlanDto {
            price_cents: p.price_cents,
            marketing_cents: p.marketing_cents,
            seasonal_target: p.seasonal_target,
            wage_pct: p.wage_pct,
            training: p.training.into(),
            maintenance: p.maintenance.into(),
            renovation: p.renovation,
            reply_reviews: p.reply_reviews,
        }
    }
}

/// Granice izbornika radnji za ekran.
#[derive(Serialize, ToSchema)]
pub struct PlanLimitsBody {
    pub price_min_cents: i64,
    pub price_max_cents: i64,
    pub marketing_max_cents: i64,
    pub seasonal_min: u16,
    pub seasonal_max: u16,
    pub wage_min_pct: u16,
    pub wage_max_pct: u16,
    pub renovation_available: bool,
}

impl From<&PlanLimits> for PlanLimitsBody {
    fn from(l: &PlanLimits) -> Self {
        PlanLimitsBody {
            price_min_cents: l.price_min_cents,
            price_max_cents: l.price_max_cents,
            marketing_max_cents: l.marketing_max_cents,
            seasonal_min: l.seasonal_min,
            seasonal_max: l.seasonal_max,
            wage_min_pct: l.wage_min_pct,
            wage_max_pct: l.wage_max_pct,
            renovation_available: l.renovation_available,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SeatKind {
    Human,
    Robot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum StrategyCode {
    Stedisa,
    Brizni,
    Lovac,
    Nasumicni,
}

#[derive(Serialize, ToSchema)]
pub struct SeatBody {
    pub kind: SeatKind,
    /// Samo za robota.
    #[schema(required = true)]
    pub strategy: Option<StrategyCode>,
}

/// Pokvaren dnevnik (nepoznat kod profila ili strategije) nije greška klijenta: 500 uz zapis u log.
fn corrupt(what: &str, value: &str) -> Problem {
    tracing::error!(%value, "{what} u dnevniku nije prepoznat");
    Problem::new(StatusCode::INTERNAL_SERVER_ERROR, ProblemCode::Internal)
}

impl TryFrom<&Seat> for SeatBody {
    type Error = Problem;

    fn try_from(s: &Seat) -> Result<Self, Problem> {
        Ok(match s {
            Seat::Human => SeatBody {
                kind: SeatKind::Human,
                strategy: None,
            },
            Seat::Robot { strategy } => SeatBody {
                kind: SeatKind::Robot,
                strategy: Some(match strategy.as_str() {
                    "stedisa" => StrategyCode::Stedisa,
                    "brizni" => StrategyCode::Brizni,
                    "lovac" => StrategyCode::Lovac,
                    "nasumicni" => StrategyCode::Nasumicni,
                    other => return Err(corrupt("strategija robota", other)),
                }),
            },
        })
    }
}

#[derive(Serialize, ToSchema)]
pub struct MonthScoreBody {
    pub month: MonthCode,
    pub indices: Vec<IndexEntry>,
    /// Napredak u odnosu na početak, u stotinkama boda (10000 = +100).
    pub progress_hundredths: i32,
}

impl From<&MonthScore> for MonthScoreBody {
    fn from(m: &MonthScore) -> Self {
        MonthScoreBody {
            month: m.month.into(),
            indices: entries(m.indices),
            progress_hundredths: m.progress_hundredths,
        }
    }
}

/// Ono što vidi svatko u igri: ocjena i pet indeksa, nikad stvarne brojke.
#[derive(Serialize, ToSchema)]
pub struct PublicHotelBody {
    pub hotel_id: String,
    pub slot: u8,
    pub name: String,
    pub seat: SeatBody,
    pub start: Vec<IndexEntry>,
    pub months: Vec<MonthScoreBody>,
}

impl TryFrom<&PublicHotel> for PublicHotelBody {
    type Error = Problem;

    fn try_from(h: &PublicHotel) -> Result<Self, Problem> {
        Ok(PublicHotelBody {
            hotel_id: h.hotel_id.0.to_string(),
            slot: h.slot,
            name: h.name.clone(),
            seat: (&h.seat).try_into()?,
            start: entries(h.start),
            months: h.months.iter().map(MonthScoreBody::from).collect(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
pub enum ProfileCode {
    A,
    B,
    C,
}

impl ProfileCode {
    /// Profil iz koda jezgre (A, B ili C); jezgra ne daje druge.
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "A" => Some(ProfileCode::A),
            "B" => Some(ProfileCode::B),
            "C" => Some(ProfileCode::C),
            _ => None,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct RealNumbersBody {
    pub rooms: u16,
    pub permanent: u16,
    pub seasonal: u16,
    pub wage_pct: u16,
    pub morale_pts: i32,
    pub experience_pts: i32,
    pub equipment_pts: i32,
    pub satisfaction_pts: i32,
    pub rating: u16,
    pub cash_cents: i64,
    pub load_hundredths: u16,
}

impl From<&RealNumbers> for RealNumbersBody {
    fn from(n: &RealNumbers) -> Self {
        RealNumbersBody {
            rooms: n.rooms,
            permanent: n.permanent,
            seasonal: n.seasonal,
            wage_pct: n.wage_pct,
            morale_pts: n.morale_pts,
            experience_pts: n.experience_pts,
            equipment_pts: n.equipment_pts,
            satisfaction_pts: n.satisfaction_pts,
            rating: n.rating,
            cash_cents: n.cash_cents,
            load_hundredths: n.load_hundredths,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct LastMonthBody {
    pub month: MonthCode,
    pub sold: u32,
    pub available: u32,
    pub rooms_revenue_cents: i64,
    pub gop_cents: i64,
    pub goppar_cents: i64,
}

impl From<&LastMonth> for LastMonthBody {
    fn from(m: &LastMonth) -> Self {
        LastMonthBody {
            month: m.month.into(),
            sold: m.sold,
            available: m.available,
            rooms_revenue_cents: m.rooms_revenue_cents,
            gop_cents: m.gop_cents,
            goppar_cents: m.goppar_cents,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct PlanStateBody {
    pub month: MonthCode,
    pub plan: PlanBody,
    pub limits: PlanLimitsBody,
}

impl From<&PlanState> for PlanStateBody {
    fn from(p: &PlanState) -> Self {
        PlanStateBody {
            month: p.month.into(),
            plan: (&p.plan).into(),
            limits: (&p.limits).into(),
        }
    }
}

/// Samo za vlasnika hotela.
#[derive(Serialize, ToSchema)]
pub struct PrivateHotelBody {
    pub profile: ProfileCode,
    pub now: RealNumbersBody,
    #[schema(required = true)]
    pub last_month: Option<LastMonthBody>,
    /// Dok je trenutni mjesec otvoren.
    #[schema(required = true)]
    pub plan: Option<PlanStateBody>,
}

impl TryFrom<&PrivateHotel> for PrivateHotelBody {
    type Error = Problem;

    fn try_from(p: &PrivateHotel) -> Result<Self, Problem> {
        Ok(PrivateHotelBody {
            profile: ProfileCode::from_code(&p.profile)
                .ok_or_else(|| corrupt("profil hotela", &p.profile))?,
            now: (&p.now).into(),
            last_month: p.last_month.as_ref().map(LastMonthBody::from),
            plan: p.plan.as_ref().map(PlanStateBody::from),
        })
    }
}

#[derive(Serialize, ToSchema)]
pub struct HotelBody {
    pub game_id: String,
    pub hotel: PublicHotelBody,
    /// Prazno za tuđi hotel.
    #[schema(required = true)]
    pub private: Option<PrivateHotelBody>,
}

impl TryFrom<&app::HotelView> for HotelBody {
    type Error = Problem;

    fn try_from(v: &app::HotelView) -> Result<Self, Problem> {
        Ok(HotelBody {
            game_id: v.game_id.0.to_string(),
            hotel: (&v.public).try_into()?,
            private: v
                .private
                .as_ref()
                .map(PrivateHotelBody::try_from)
                .transpose()?,
        })
    }
}

#[derive(Serialize, ToSchema)]
pub struct DemoBody {
    pub game_id: String,
    pub hotel_id: String,
}

pub fn parse_game(s: &str) -> Result<app::GameId, Problem> {
    s.parse()
        .map(app::GameId)
        .map_err(|_| Problem::new(StatusCode::NOT_FOUND, ProblemCode::UnknownGame))
}

pub fn parse_hotel(s: &str) -> Result<app::HotelId, Problem> {
    s.parse()
        .map(app::HotelId)
        .map_err(|_| Problem::new(StatusCode::NOT_FOUND, ProblemCode::UnknownHotel))
}

pub fn parse_month(s: &str) -> Result<game::dto::MonthCode, Problem> {
    serde_json::from_value::<MonthCode>(serde_json::Value::String(s.to_owned()))
        .map(Into::into)
        .map_err(|_| Problem::new(StatusCode::NOT_FOUND, ProblemCode::UnknownMonth))
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::HotelId;

    #[test]
    fn unknown_robot_strategy_is_internal_problem() {
        let bad = Seat::Robot {
            strategy: "nepoznata".into(),
        };
        let e = SeatBody::try_from(&bad).err().expect("greška");
        assert_eq!((e.status, e.code), (500, ProblemCode::Internal));
        let ok = SeatBody::try_from(&Seat::Robot {
            strategy: "lovac".into(),
        })
        .unwrap();
        assert_eq!(ok.strategy, Some(StrategyCode::Lovac));
    }

    #[test]
    fn unknown_profile_is_internal_problem() {
        let p = PrivateHotel {
            profile: "X".into(),
            now: RealNumbers {
                rooms: 0,
                permanent: 0,
                seasonal: 0,
                wage_pct: 0,
                morale_pts: 0,
                experience_pts: 0,
                equipment_pts: 0,
                satisfaction_pts: 0,
                rating: 0,
                cash_cents: 0,
                load_hundredths: 0,
            },
            last_month: None,
            plan: None,
        };
        let e = PrivateHotelBody::try_from(&p).err().expect("greška");
        assert_eq!((e.status, e.code), (500, ProblemCode::Internal));
    }

    #[test]
    fn bad_robot_in_public_hotel_fails_whole_body() {
        let h = PublicHotel {
            hotel_id: HotelId(uuid::Uuid::nil()),
            slot: 2,
            name: "H".into(),
            seat: Seat::Robot {
                strategy: "x".into(),
            },
            start: [0; 5],
            months: Vec::new(),
        };
        assert!(PublicHotelBody::try_from(&h).is_err());
    }

    #[test]
    fn parse_month_accepts_only_lowercase_codes() {
        assert_eq!(parse_month("jul").unwrap(), game::dto::MonthCode::Jul);
        assert!(parse_month("Jul").is_err());
        assert!(parse_month("x").is_err());
    }
}
