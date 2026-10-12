//! Igra kao dnevnik događaja (ARHITEKTURA §9.1, ADR-015): događaji, projekcija (fold), runde, roboti i obračun runde.
//! Čisto: bez baze, mreže, sata i vlastite slučajnosti — sve što treba dolazi iz dnevnika i snimka pravila.

pub mod actor;
mod ai_input;
pub mod command;
pub mod costs;
mod departments;
pub mod dto;
pub mod dto_v2;
pub mod error;
pub mod event;
pub mod feed;
mod game;
pub mod ids;
pub mod kpi;
mod mentor;
mod model;
mod resolve;
mod round;
pub mod talk;
mod verdict;
pub mod view;

pub use actor::{Actor, strategy_from_code};
pub use ai_input::{
    AssessInput, CharacterCard, CharacterInput, DebriefInput, DebriefSituation,
    DepartmentDepartures, OfferedForJudge,
};
pub use departments::{
    DepartmentDetail, DepartmentView, DepartmentsView, PlScope, PlView, department_margin_bp,
};
pub use error::GameError;
pub use event::*;
pub use game::{ChatLine, FreeResponse, Game, HotelRecord, RoundStatus, Speaker};
pub use ids::{GameId, GuestId, HotelId};
pub use mentor::{
    ChangeFact, DebriefFacts, DriverFact, DriverSource, Effect, IndexFact, SituationFact,
    debrief_facts, fallback_debrief, wants_ai_mentor, weekly_summary,
};
pub use model::{CorePlan, CoreState, Model, RoundInfo};
pub use resolve::ResolveStep;
pub use round::{MAX_WEEK, RoundKey};
pub use talk::{MAX_CHAT_MESSAGES, MAX_TEXT_CHARS, ReplyBody};
pub use verdict::{
    NEUTRAL_DEADLINE, NEUTRAL_FAILED, ValidVerdict, VerdictDraft, VerdictError, quality_from,
};
