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
pub mod department;
pub mod explain;
pub mod indices;
pub mod market;
pub mod plan;
pub mod renewal;
pub mod resolve;
pub mod robots;
pub mod season;
pub mod situations;
pub mod state;
pub mod streams;
pub mod usali;

pub use calendar::{MonthPart, Period, PeriodKind, season_periods};
pub use catalog::{
    BuildingSpec, Catalog, DeptStart, EconomyV2, FinanceBasis, FixedChargeSpec, GroupTerms,
    PeriodCoefficients, ProfileV2, degenerate,
};
pub use department::{BuildingOutcome, StaffingNorm};
pub use indices::{RefTable, period_indices, start_indices};
pub use plan::{PeriodPlan, PlanError, PlanLimits, RenewalLimits};
pub use renewal::{AssetBook, Renewal, RenewalSpec};
pub use resolve::{
    FinanceOutcome, PeriodReport, ResolveError, SituationEffect, resolve_period, situation_effect,
};
pub use robots::robot_plan;
pub use season::{
    Responder, RoundHotel, RoundItem, SeasonResult, SeasonSituations, TemplateDraw, anchor_of,
    collective_for_round, draw_for_round, draw_for_round_with, group_month, run_robot_season,
    run_season, run_season_from, run_season_responding, run_season_with, situations_for,
};
pub use sim_hotel::{
    CatalogError, Category, Cause, Effects, GroupBlock, GuestOutcome, Index, IndexSet, Line,
    Maintenance, MarketOutcome, Mechanic, MechanicError, MoneyOutcome, Month, MonthReport,
    MonthTable, OfferedAnswer, Pending, PeopleOutcome, Profile, ProfileSpec, SituationOutcome,
    SituationTemplate, Strategy, Training, Unit, Var, gain_bp, progress_pts,
};
pub use situations::RoundSituation;
pub use state::{Building, HotelState};

/// Verzija jezgre v2 u `GameCreated` (MODEL_V2 §6.2 t. 3); raste sa svakom promjenom koja igri u tijeku mijenja brojke;
/// čuva je pečat `crates/game/tests/v2_sealed.rs`.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
