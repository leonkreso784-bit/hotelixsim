//! Generička jezgra: kocka, a kasnije dnevnik i životni ciklus runde. Bez I/O-a, sata i vanjske slučajnosti.
#![forbid(
    clippy::float_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::default_numeric_fallback
)]

pub mod rng;

pub use rng::{Rng, label};
