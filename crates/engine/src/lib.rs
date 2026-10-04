//! Generička jezgra: kocka, a kasnije dnevnik i životni ciklus runde. Bez I/O-a, sata i vanjske slučajnosti.
#![deny(clippy::float_arithmetic, clippy::arithmetic_side_effects)]

pub mod rng;

pub use rng::{Rng, label};
