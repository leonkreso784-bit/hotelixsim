//! Igra kao dnevnik događaja (ARHITEKTURA §9.1, ADR-015): događaji, projekcija (fold), runde, roboti i obračun runde.
//! Čisto: bez baze, mreže, sata i vlastite slučajnosti — sve što treba dolazi iz dnevnika i snimka pravila.

pub mod actor;
mod ai_input;
pub mod command;
pub mod dto;
pub mod error;
pub mod event;
pub mod feed;
mod game;
pub mod ids;
mod mentor;
mod resolve;
pub mod talk;
mod verdict;
pub mod view;

pub use actor::{Actor, strategy_from_code};
pub use ai_input::{
    AssessInput, CharacterCard, CharacterInput, DebriefInput, DebriefSituation, OfferedForJudge,
};
pub use error::GameError;
pub use event::*;
pub use game::{ChatLine, FreeResponse, Game, HotelRecord, RoundStatus, Speaker};
pub use ids::{GameId, GuestId, HotelId};
pub use mentor::{
    ChangeFact, DebriefFacts, DriverFact, IndexFact, SituationFact, debrief_facts, fallback_debrief,
};
pub use resolve::ResolveStep;
pub use talk::{MAX_CHAT_MESSAGES, MAX_TEXT_CHARS, ReplyBody};
pub use verdict::{
    NEUTRAL_DEADLINE, NEUTRAL_FAILED, ValidVerdict, VerdictDraft, VerdictError, quality_from,
};
