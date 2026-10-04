//! Hotelska domena: indeksi, profili, a kasnije stvarni sloj, mehanike i obračun (DEMO.md).
#![deny(clippy::float_arithmetic, clippy::arithmetic_side_effects)]

pub mod index;
pub mod profile;

pub use index::{Index, IndexSet, IndexValue};
pub use profile::Profile;
