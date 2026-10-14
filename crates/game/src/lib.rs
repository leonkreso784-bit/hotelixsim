//! Igra kao dnevnik događaja (ARHITEKTURA §9.1, ADR-015): događaji, projekcija (fold), runde, roboti i obračun runde.
//! Čisto: bez baze, mreže, sata i vlastite slučajnosti — sve što treba dolazi iz dnevnika i snimka pravila.

pub mod actor;
mod ai_input;
pub mod command;
pub mod costs;
mod debrief_v2;
mod departments;
mod drawn;
pub mod dto;
pub mod dto_v2;
pub mod error;
pub mod event;
pub mod feed;
mod game;
pub mod ids;
pub mod kpi;
mod mentor;
mod mentor_v2;
mod model;
pub mod renewal;
mod resolve;
mod round;
pub mod season;
mod season_mentor;
pub mod talk;
mod verdict;
pub mod view;

pub use actor::{Actor, strategy_from_code};
pub use ai_input::{
    AssessInput, CharacterCard, CharacterInput, DebriefInput, DebriefSituation, DeferredWearFact,
    DepartmentDepartures, OfferedForJudge, RenewalFact, RentCutFact,
};
pub use command::MANDATORY_SITUATIONS;
pub use departments::{
    DepartmentDetail, DepartmentView, DepartmentsView, PlScope, PlView, department_margin_bp,
};
pub use drawn::{DrawnInstance, InstanceRole, SituationCard};
pub use error::GameError;
pub use event::*;
pub use game::{ChatLine, FreeResponse, Game, HotelRecord, RoundStatus, Speaker};
pub use ids::{GameId, GuestId, HotelId};
pub use mentor::{
    ChangeFact, DebriefFacts, DriverFact, DriverSource, Effect, IndexFact, SituationFact,
    debrief_facts, fallback_debrief, wants_ai_mentor, weekly_summary,
};
pub use mentor_v2::{FinanceNote, InvestmentFacts, RenewalText};
pub use model::{CorePlan, CoreState, Model, RoundInfo};
pub use resolve::ResolveStep;
pub use round::{MAX_WEEK, RoundKey};
pub use season::{CurvePoint, SeasonDecisions, SeasonFinances, SeasonHotel, SeasonView};
pub use season_mentor::{
    DecisionFacts, FinanceFacts, OtherHotelFact, RoundFact, SeasonDebriefInput, SeasonFacts,
    SeasonIndexFact, SeasonMentorHotel, SeasonRound, fallback_season_debrief, season_debrief_facts,
    season_debrief_has_parts, wants_season_mentor,
};
pub use sim_hotel_v2::situations::Kind as SituationKind;
pub use talk::{MAX_CHAT_MESSAGES, MAX_TEXT_CHARS, ReplyBody};
pub use verdict::{
    NEUTRAL_DEADLINE, NEUTRAL_FAILED, ValidVerdict, VerdictDraft, VerdictError, quality_from,
};
