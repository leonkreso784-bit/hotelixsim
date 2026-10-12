//! Greške u obliku application/problem+json (RFC 9457), s kodom koji sučelje prevodi.

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use utoipa::ToSchema;

/// Stabilni kodovi grešaka; ekran ih prevodi (ARHITEKTURA §11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProblemCode {
    GuestRequired,
    NotInGame,
    NotYourHotel,
    UnknownGame,
    UnknownHotel,
    UnknownMonth,
    UnknownSituation,
    UnknownAnswer,
    UnknownProfile,
    UnknownDepartment,
    RoundNotOpen,
    AlreadyAnswered,
    NotFreeText,
    ChatLimit,
    AwaitingReply,
    BadText,
    PriceNotWholeEuro,
    PriceOutOfRange,
    MarketingOutOfRange,
    SeasonalOutOfRange,
    SeasonalStepTooBig,
    WageOutOfRange,
    RenovationTwice,
    GameOutdated,
    InvalidSettings,
    DepartmentPlanInvalid,
    NoResolvedRound,
    SeasonNotOver,
    ModelV1,
    StorageUnavailable,
    Internal,
    /// Samo HTTP sloj: ruta ne postoji.
    UnknownRoute,
    /// Samo HTTP sloj: tijelo zahtjeva nije ispravan JSON traženog oblika.
    InvalidBody,
}

impl From<app::ErrorCode> for ProblemCode {
    fn from(c: app::ErrorCode) -> Self {
        use app::ErrorCode as E;
        match c {
            E::GuestRequired => Self::GuestRequired,
            E::NotInGame => Self::NotInGame,
            E::NotYourHotel => Self::NotYourHotel,
            E::UnknownGame => Self::UnknownGame,
            E::UnknownHotel => Self::UnknownHotel,
            E::UnknownMonth => Self::UnknownMonth,
            E::UnknownSituation => Self::UnknownSituation,
            E::UnknownAnswer => Self::UnknownAnswer,
            E::UnknownProfile => Self::UnknownProfile,
            E::UnknownDepartment => Self::UnknownDepartment,
            E::RoundNotOpen => Self::RoundNotOpen,
            E::AlreadyAnswered => Self::AlreadyAnswered,
            E::NotFreeText => Self::NotFreeText,
            E::ChatLimit => Self::ChatLimit,
            E::AwaitingReply => Self::AwaitingReply,
            E::BadText => Self::BadText,
            E::PriceNotWholeEuro => Self::PriceNotWholeEuro,
            E::PriceOutOfRange => Self::PriceOutOfRange,
            E::MarketingOutOfRange => Self::MarketingOutOfRange,
            E::SeasonalOutOfRange => Self::SeasonalOutOfRange,
            E::SeasonalStepTooBig => Self::SeasonalStepTooBig,
            E::WageOutOfRange => Self::WageOutOfRange,
            E::RenovationTwice => Self::RenovationTwice,
            E::GameOutdated => Self::GameOutdated,
            E::InvalidSettings => Self::InvalidSettings,
            E::DepartmentPlanInvalid => Self::DepartmentPlanInvalid,
            E::InvalidBody => Self::InvalidBody,
            E::NoResolvedRound => Self::NoResolvedRound,
            E::SeasonNotOver => Self::SeasonNotOver,
            E::WrongModel => Self::ModelV1,
            E::StorageUnavailable => Self::StorageUnavailable,
            E::Internal => Self::Internal,
        }
    }
}

impl ProblemCode {
    /// Naslov za čovjeka (hrvatski); sučelje ionako prevodi po kodu.
    pub fn title(self) -> &'static str {
        match self {
            Self::GuestRequired => "Potreban je gost: otvori demo poveznicu",
            Self::NotInGame => "Nisi u ovoj igri",
            Self::NotYourHotel => "To nije tvoj hotel",
            Self::UnknownGame => "Igra ne postoji",
            Self::UnknownHotel => "Hotel ne postoji",
            Self::UnknownMonth => "Taj krug ne postoji u ovoj igri.",
            Self::UnknownSituation => "Situacija ne postoji",
            Self::UnknownAnswer => "Taj odgovor nije ponuđen",
            Self::UnknownProfile => "Nepoznat profil",
            Self::UnknownDepartment => "Taj odjel ne postoji u ovom hotelu.",
            Self::RoundNotOpen => "Runda nije otvorena",
            Self::AlreadyAnswered => "Na situaciju je već odgovoreno",
            Self::NotFreeText => "Situacija nema slobodni odgovor",
            Self::ChatLimit => "Iskorištene su sve poruke u razgovoru",
            Self::AwaitingReply => "Lik još nije odgovorio",
            Self::BadText => "Tekst je prazan ili predug",
            Self::PriceNotWholeEuro => "Cijena mora biti cijeli euro",
            Self::PriceOutOfRange => "Cijena je izvan dopuštenog raspona",
            Self::MarketingOutOfRange => "Marketing je izvan dopuštenog raspona",
            Self::SeasonalOutOfRange => "Broj sezonaca je izvan dopuštenog raspona",
            Self::SeasonalStepTooBig => "Prevelika promjena broja sezonaca u jednom krugu",
            Self::WageOutOfRange => "Plaća je izvan dopuštenog raspona",
            Self::RenovationTwice => "Obnova je moguća samo jednom u sezoni",
            Self::GameOutdated => "Igra je stvorena starijom verzijom igre. Započni novu igru.",
            Self::InvalidSettings => "Te postavke igre nisu dopuštene.",
            Self::DepartmentPlanInvalid => {
                "Postavke odjela nisu dopuštene: provjeri broj sezonaca, razinu cijena i odjele koji se ne smiju zatvoriti."
            }
            Self::NoResolvedRound => "Još nijedan krug nije obračunan, pa nema računa dobiti.",
            Self::SeasonNotOver => {
                "Sezona još traje. Rezultati se vide kad se obračuna zadnji krug."
            }
            Self::ModelV1 => "Ova igra je na starom modelu i nema ovaj prikaz.",
            Self::StorageUnavailable => "Baza nije dostupna",
            Self::Internal => "Greška sustava",
            Self::UnknownRoute => "Nepoznata ruta",
            Self::InvalidBody => "Neispravno tijelo zahtjeva",
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Problem {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub title: &'static str,
    pub status: u16,
    pub code: ProblemCode,
}

impl Problem {
    pub fn new(status: StatusCode, code: ProblemCode) -> Self {
        Problem {
            kind: "about:blank",
            title: code.title(),
            status: status.as_u16(),
            code,
        }
    }
}

impl From<app::AppError> for Problem {
    fn from(e: app::AppError) -> Self {
        use app::ErrorKind as K;
        let status = match e.code.kind() {
            K::Unauthorized => StatusCode::UNAUTHORIZED,
            K::Forbidden => StatusCode::FORBIDDEN,
            K::NotFound => StatusCode::NOT_FOUND,
            K::Conflict => StatusCode::CONFLICT,
            K::Invalid => StatusCode::UNPROCESSABLE_ENTITY,
            K::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            K::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        if status.is_server_error() {
            tracing::error!(code = ?e.code, detail = %e.detail, "greška poslužitelja");
        }
        Problem::new(status, e.code.into())
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let body = serde_json::to_vec(&self).unwrap_or_default();
        (
            status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            body,
        )
            .into_response()
    }
}
