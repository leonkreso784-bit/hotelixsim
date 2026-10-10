//! Hotelska domena: indeksi, profili, a kasnije stvarni sloj, mehanike i obračun (DEMO.md).
#![forbid(
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::default_numeric_fallback
)]

pub mod catalog;
pub mod explain;
pub mod index;
pub mod indices;
pub mod market;
pub mod mechanic;
pub mod month;
pub mod plan;
pub mod profile;
pub mod resolve;
pub mod robots;
pub mod score;
pub mod season;
pub mod state;

pub use catalog::{
    ActionSpec, Catalog, CatalogError, Category, CategorySpec, Economy, LevelEffect, MechanicSpec,
    ProfileSpec, TrainingEffect,
};
pub use explain::{Cause, Line, Unit, Var};
pub use index::{Index, IndexSet, IndexValue, NamedIndices};
pub use indices::{
    Driver, LOAD_LIMIT_HUNDREDTHS, RawIndices, RefPoint, RefTable, base_labor_cents, centred,
    index_drivers, load_penalty, month_indices, raw_indices, retention, start_indices,
};
pub use market::{MarketOutcome, market};
pub use mechanic::{
    Effects, Mechanic, MechanicError, OfferedAnswer, SituationOutcome, SituationTemplate, effects,
};
pub use month::{Month, MonthTable};
pub use plan::{Maintenance, MonthPlan, PlanError, Training};
pub use profile::Profile;
pub use resolve::{
    GuestOutcome, MoneyOutcome, MonthReport, PeopleOutcome, ResolveError, resolve_month,
};
pub use robots::{Strategy, euro_bounds, seasonal_range};
pub use score::{gain_bp, progress_pts};
pub use season::{SeasonResult, run_season, run_season_with};
pub use state::{GroupBlock, HotelState, Pending, turnover_bp};

/// Verzija jezgre koja ulazi u GameCreated (ARHITEKTURA §7.2).
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
