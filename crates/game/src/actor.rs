use sim_hotel::Strategy;
use uuid::Uuid;

use crate::ids::GuestId;

/// Tko je izazvao događaj; u bazi kao tekst `guest:<uuid>` · `robot:<kod>` · `system` (CHECK u migraciji 0002).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Actor {
    Guest(GuestId),
    Robot(Strategy),
    System,
}

pub fn strategy_from_code(code: &str) -> Option<Strategy> {
    Strategy::ALL.into_iter().find(|s| s.code() == code)
}

impl Actor {
    pub fn as_db(&self) -> String {
        match self {
            Actor::Guest(g) => format!("guest:{}", g.0),
            Actor::Robot(s) => format!("robot:{}", s.code()),
            Actor::System => "system".to_owned(),
        }
    }

    pub fn parse(s: &str) -> Option<Actor> {
        if s == "system" {
            return Some(Actor::System);
        }
        if let Some(id) = s.strip_prefix("guest:") {
            return Uuid::parse_str(id).ok().map(|u| Actor::Guest(GuestId(u)));
        }
        s.strip_prefix("robot:")
            .and_then(strategy_from_code)
            .map(Actor::Robot)
    }
}
