//! Hotelska domena: indeksi, profili, a kasnije stvarni sloj, mehanike i obračun (DEMO.md).
#![forbid(
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::default_numeric_fallback
)]

pub mod catalog;
pub mod index;
pub mod market;
pub mod month;
pub mod plan;
pub mod profile;
pub mod state;

pub use catalog::{
    ActionSpec, Catalog, CatalogError, Category, CategorySpec, Economy, LevelEffect, MechanicSpec,
    ProfileSpec, TrainingEffect,
};
pub use index::{Index, IndexSet, IndexValue, NamedIndices};
pub use market::{MarketOutcome, market};
pub use month::{Month, MonthTable};
pub use plan::{Maintenance, MonthPlan, PlanError, Training};
pub use profile::Profile;
pub use state::{GroupBlock, HotelState, Pending, turnover_bp};
