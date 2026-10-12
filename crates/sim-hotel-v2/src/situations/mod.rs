//! Situacije iz predložaka (plan situacija §3–§4): koliko ih krug ima i koje se izvlače. Jezgra ne zna za tekstove
//! predložaka (oni su u sadržaju); ovdje su samo pravila pojave i broj.

mod count;

pub use count::{expected_occupancy_bp, period_count, situation_count, weekly_count};
