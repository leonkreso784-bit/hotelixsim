use game::GameError;
use sim_hotel::PlanError;

use crate::ports::{DecideError, StoreError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorCode {
    GuestRequired,
    NotInGame,
    NotYourHotel,
    UnknownGame,
    UnknownHotel,
    UnknownMonth,
    UnknownSituation,
    UnknownAnswer,
    UnknownProfile,
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
    StorageUnavailable,
    Internal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    Invalid,
    Unavailable,
    Internal,
}

impl ErrorCode {
    pub const ALL: [ErrorCode; 24] = [
        ErrorCode::GuestRequired,
        ErrorCode::NotInGame,
        ErrorCode::NotYourHotel,
        ErrorCode::UnknownGame,
        ErrorCode::UnknownHotel,
        ErrorCode::UnknownMonth,
        ErrorCode::UnknownSituation,
        ErrorCode::UnknownAnswer,
        ErrorCode::UnknownProfile,
        ErrorCode::RoundNotOpen,
        ErrorCode::AlreadyAnswered,
        ErrorCode::NotFreeText,
        ErrorCode::ChatLimit,
        ErrorCode::AwaitingReply,
        ErrorCode::BadText,
        ErrorCode::PriceNotWholeEuro,
        ErrorCode::PriceOutOfRange,
        ErrorCode::MarketingOutOfRange,
        ErrorCode::SeasonalOutOfRange,
        ErrorCode::SeasonalStepTooBig,
        ErrorCode::WageOutOfRange,
        ErrorCode::RenovationTwice,
        ErrorCode::StorageUnavailable,
        ErrorCode::Internal,
    ];

    pub fn kind(self) -> ErrorKind {
        use ErrorCode::*;
        match self {
            GuestRequired => ErrorKind::Unauthorized,
            NotInGame | NotYourHotel => ErrorKind::Forbidden,
            UnknownGame | UnknownHotel | UnknownMonth | UnknownSituation | UnknownProfile => {
                ErrorKind::NotFound
            }
            RoundNotOpen | AlreadyAnswered | ChatLimit | AwaitingReply => ErrorKind::Conflict,
            UnknownAnswer | NotFreeText | BadText | PriceNotWholeEuro | PriceOutOfRange
            | MarketingOutOfRange | SeasonalOutOfRange | SeasonalStepTooBig | WageOutOfRange
            | RenovationTwice => ErrorKind::Invalid,
            StorageUnavailable => ErrorKind::Unavailable,
            Internal => ErrorKind::Internal,
        }
    }

    /// Stabilan kod za API (snake_case); ekran ga prevodi.
    pub fn as_str(self) -> &'static str {
        use ErrorCode::*;
        match self {
            GuestRequired => "guest_required",
            NotInGame => "not_in_game",
            NotYourHotel => "not_your_hotel",
            UnknownGame => "unknown_game",
            UnknownHotel => "unknown_hotel",
            UnknownMonth => "unknown_month",
            UnknownSituation => "unknown_situation",
            UnknownAnswer => "unknown_answer",
            UnknownProfile => "unknown_profile",
            RoundNotOpen => "round_not_open",
            AlreadyAnswered => "already_answered",
            NotFreeText => "not_free_text",
            ChatLimit => "chat_limit",
            AwaitingReply => "awaiting_reply",
            BadText => "bad_text",
            PriceNotWholeEuro => "price_not_whole_euro",
            PriceOutOfRange => "price_out_of_range",
            MarketingOutOfRange => "marketing_out_of_range",
            SeasonalOutOfRange => "seasonal_out_of_range",
            SeasonalStepTooBig => "seasonal_step_too_big",
            WageOutOfRange => "wage_out_of_range",
            RenovationTwice => "renovation_twice",
            StorageUnavailable => "storage_unavailable",
            Internal => "internal",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{code:?}: {detail}")]
pub struct AppError {
    pub code: ErrorCode,
    /// Za dnevnik poslužitelja; ne šalje se klijentu.
    pub detail: String,
}

impl AppError {
    pub fn new(code: ErrorCode) -> Self {
        AppError {
            code,
            detail: String::new(),
        }
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        AppError {
            code: ErrorCode::Internal,
            detail: detail.into(),
        }
    }
}

impl From<GameError> for AppError {
    fn from(e: GameError) -> Self {
        use ErrorCode as C;
        let code = match &e {
            GameError::UnknownHotel => C::UnknownHotel,
            GameError::UnknownMonth => C::UnknownMonth,
            GameError::UnknownSituation => C::UnknownSituation,
            GameError::UnknownAnswer => C::UnknownAnswer,
            GameError::RoundNotOpen => C::RoundNotOpen,
            GameError::AlreadyAnswered => C::AlreadyAnswered,
            GameError::NotFreeText => C::NotFreeText,
            GameError::ChatLimit => C::ChatLimit,
            GameError::AwaitingReply => C::AwaitingReply,
            GameError::EmptyText | GameError::TextTooLong => C::BadText,
            GameError::Plan(p) => match p {
                PlanError::PriceNotWholeEuro => C::PriceNotWholeEuro,
                PlanError::PriceOutOfRange => C::PriceOutOfRange,
                PlanError::MarketingOutOfRange => C::MarketingOutOfRange,
                PlanError::SeasonalOutOfRange => C::SeasonalOutOfRange,
                PlanError::SeasonalStepTooBig => C::SeasonalStepTooBig,
                PlanError::WageOutOfRange => C::WageOutOfRange,
                PlanError::RenovationTwice => C::RenovationTwice,
                PlanError::Calc => C::Internal,
            },
            GameError::Corrupt(_) | GameError::CoreMismatch(_) => C::Internal,
        };
        AppError {
            code,
            detail: format!("{e:?}"),
        }
    }
}

impl From<StoreError> for AppError {
    fn from(e: StoreError) -> Self {
        AppError {
            code: ErrorCode::StorageUnavailable,
            detail: e.0,
        }
    }
}

impl From<DecideError> for AppError {
    fn from(e: DecideError) -> Self {
        match e {
            DecideError::Rejected(a) => a,
            DecideError::Store(s) => s.into(),
        }
    }
}
