//! Hotelska domena: indeksi, profili, a kasnije stvarni sloj, mehanike i obračun (DEMO.md).
#![forbid(
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::default_numeric_fallback
)]

pub mod index;
pub mod profile;

pub use index::{Index, IndexSet, IndexValue};
pub use profile::Profile;
