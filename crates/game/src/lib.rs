//! Igra kao dnevnik događaja (ARHITEKTURA §9.1, ADR-015): događaji, projekcija (fold), runde, roboti i obračun runde.
//! Čisto: bez baze, mreže, sata i vlastite slučajnosti — sve što treba dolazi iz dnevnika i snimka pravila.

pub mod actor;
pub mod dto;
pub mod error;
pub mod event;
pub mod ids;

pub use actor::{Actor, strategy_from_code};
pub use error::GameError;
pub use event::*;
pub use ids::{GameId, GuestId, HotelId};
