use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::dto::{MonthCode, MonthReportDto, PlanDto};
use crate::ids::{GameId, GuestId, HotelId};

/// Verzija oblika svih događaja; promjena oblika = nova verzija i „podizanje" starih pri čitanju (ARHITEKTURA §9.1).
pub const SCHEMA_VER: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameCreated {
    pub game_id: GameId,
    pub seed: u64,
    /// `snapshot.fingerprint()`; fold ga provjerava.
    pub ruleset_id: String,
    pub core_version: String,
    /// U kutiji jer je snimak velik u odnosu na ostale događaje (`clippy::large_enum_variant`).
    pub snapshot: Box<content::Snapshot>,
    pub months: Vec<MonthCode>,
    pub hotels: Vec<HotelSetup>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotelSetup {
    pub hotel_id: HotelId,
    /// 1..=4; određuje redoslijed obračuna i tokove kocke (nikad uuid ni vrijeme dolaska, ARHITEKTURA §7.2).
    pub slot: u8,
    pub name: String,
    /// "A" | "B" | "C"
    pub profile: String,
    pub controller: Controller,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Controller {
    Human {
        guest_id: GuestId,
    },
    /// `strategy` = `Strategy::code()`
    Robot {
        strategy: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanSet {
    pub hotel_id: HotelId,
    pub month: MonthCode,
    pub plan: PlanDto,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnswerChosen {
    pub hotel_id: HotelId,
    pub month: MonthCode,
    pub situation: String,
    pub answer: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundClosed {
    pub month: MonthCode,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundResolved {
    pub month: MonthCode,
    pub hotels: Vec<HotelResolution>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotelResolution {
    pub hotel_id: HotelId,
    /// Plan koji je stvarno obračunat (zadnji `PlanSet` ili zadani iz DEMO §6.1).
    pub plan: PlanDto,
    pub situations: Vec<SituationResult>,
    pub report: MonthReportDto,
    /// Pet indeksa nakon mjeseca, redoslijedom `INDEX_ORDER`.
    pub indices: [u16; 5],
    pub progress_hundredths: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SituationResult {
    pub code: String,
    /// `None` = nitko nije odgovorio do zatvaranja (kvaliteta 1, DEMO §1.2).
    pub answer: Option<u8>,
    pub quality: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameEvent {
    GameCreated(GameCreated),
    PlanSet(PlanSet),
    AnswerChosen(AnswerChosen),
    RoundClosed(RoundClosed),
    RoundResolved(RoundResolved),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventError {
    UnknownKind(String),
    UnsupportedVersion { kind: String, version: i32 },
    Payload { kind: String, message: String },
}

/// Zapisan događaj: redni broj u toku igre i tko ga je izazvao.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorded {
    pub seq: i64,
    pub actor: Actor,
    pub event: GameEvent,
}

fn body<T: DeserializeOwned>(kind: &str, v: serde_json::Value) -> Result<T, EventError> {
    serde_json::from_value(v).map_err(|e| EventError::Payload {
        kind: kind.to_owned(),
        message: e.to_string(),
    })
}

impl GameEvent {
    pub fn kind(&self) -> &'static str {
        match self {
            GameEvent::GameCreated(_) => "GameCreated",
            GameEvent::PlanSet(_) => "PlanSet",
            GameEvent::AnswerChosen(_) => "AnswerChosen",
            GameEvent::RoundClosed(_) => "RoundClosed",
            GameEvent::RoundResolved(_) => "RoundResolved",
        }
    }

    pub fn to_payload(&self) -> serde_json::Value {
        let v = match self {
            GameEvent::GameCreated(e) => serde_json::to_value(e),
            GameEvent::PlanSet(e) => serde_json::to_value(e),
            GameEvent::AnswerChosen(e) => serde_json::to_value(e),
            GameEvent::RoundClosed(e) => serde_json::to_value(e),
            GameEvent::RoundResolved(e) => serde_json::to_value(e),
        };
        v.expect("događaj nema mapu s ne-tekstualnim ključem pa se uvijek serijalizira")
    }

    pub fn from_record(
        kind: &str,
        schema_ver: i32,
        payload: serde_json::Value,
    ) -> Result<GameEvent, EventError> {
        if schema_ver != SCHEMA_VER {
            return Err(EventError::UnsupportedVersion {
                kind: kind.to_owned(),
                version: schema_ver,
            });
        }
        Ok(match kind {
            "GameCreated" => GameEvent::GameCreated(body(kind, payload)?),
            "PlanSet" => GameEvent::PlanSet(body(kind, payload)?),
            "AnswerChosen" => GameEvent::AnswerChosen(body(kind, payload)?),
            "RoundClosed" => GameEvent::RoundClosed(body(kind, payload)?),
            "RoundResolved" => GameEvent::RoundResolved(body(kind, payload)?),
            other => return Err(EventError::UnknownKind(other.to_owned())),
        })
    }
}
