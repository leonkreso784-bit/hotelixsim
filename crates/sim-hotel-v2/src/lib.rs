//! Model v2 (MODEL_V2.md): hotel od stvari, razdoblje od N dana (tjedan ili mjesec). v1 (`sim_hotel`) je zamrznut;
//! odande se ponovno koriste katalog (`Catalog::base`), mehanike situacija, indeksi, ocjena napretka i izlazne
//! strukture izvješća, a svako razdoblje daje i v1 sažetak (`PeriodReport::summary`).
#![forbid(
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::default_numeric_fallback
)]

pub mod calendar;
pub mod catalog;
pub mod explain;
pub mod indices;
pub mod market;
pub mod plan;
pub mod resolve;
pub mod robots;
pub mod season;
pub mod state;
pub mod streams;

pub use calendar::{MonthPart, Period, PeriodKind, season_periods};
pub use catalog::{Catalog, PeriodCoefficients, degenerate};
pub use indices::{RefTable, period_indices, start_indices};
pub use plan::{PeriodPlan, PlanError};
pub use resolve::{PeriodReport, ResolveError};
pub use season::{SeasonResult, run_season, run_season_with};
pub use sim_hotel::{
    CatalogError, Category, Cause, Effects, GroupBlock, GuestOutcome, Index, IndexSet, Line,
    Maintenance, MarketOutcome, Mechanic, MechanicError, MoneyOutcome, Month, MonthReport,
    MonthTable, OfferedAnswer, Pending, PeopleOutcome, Profile, ProfileSpec, SituationOutcome,
    SituationTemplate, Strategy, Training, Unit, Var, gain_bp, progress_pts,
};
pub use state::HotelState;

/// Verzija jezgre v2 u `GameCreated` (MODEL_V2 §6.2 t. 3); raste po sloju, ne unutar sloja 1.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
