use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::actor::Actor;
use crate::dto::{MonthReportDto, PlanDto};
use crate::dto_v2::DetailDto;
use crate::ids::{GameId, GuestId, HotelId};
use crate::round::RoundKey;

/// Verzija oblika svih događaja; promjena oblika = nova verzija i „podizanje" starih pri čitanju (ARHITEKTURA §9.1).
pub const SCHEMA_VER: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameCreated {
    pub game_id: GameId,
    pub seed: u64,
    /// Otisak snimka (`content::fingerprint_value` nad `snapshot`; stare igre: `legacy_fingerprint`); fold ga provjerava.
    pub ruleset_id: String,
    pub core_version: String,
    /// Sirovi snimak kakav je spremljen; otisak se računa nad njim. U kutiji jer je velik u odnosu na ostale događaje
    /// (`clippy::large_enum_variant`).
    pub snapshot: Box<serde_json::Value>,
    pub months: Vec<RoundKey>,
    pub hotels: Vec<HotelSetup>,
    /// v2: kalendar krugova u trenutku stvaranja (igra ga nikad ne računa ponovno). v1: prazno.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rounds: Vec<RoundDefDto>,
}

/// Jedan krug v2 igre kakav je zapisan pri stvaranju (`sim_hotel_v2::Period` uz ključ kruga).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundDefDto {
    pub code: RoundKey,
    /// `PeriodKind::code()`: "month" | "week".
    pub kind: String,
    pub index: u8,
    pub start_day: u16,
    pub days: u8,
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
    pub month: RoundKey,
    pub plan: PlanDto,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnswerChosen {
    pub hotel_id: HotelId,
    pub month: RoundKey,
    pub situation: String,
    pub answer: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundClosed {
    pub month: RoundKey,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundResolved {
    pub month: RoundKey,
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
    /// v2: stanje v2 nakon kruga (izvor stanja idućeg kruga); v1 ga nema, pa je njen JSON isti.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<DetailDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SituationResult {
    pub code: String,
    /// `None` = nitko nije odgovorio do zatvaranja (kvaliteta 1, DEMO §1.2).
    pub answer: Option<u8>,
    pub quality: u8,
    /// Slobodan odgovor (kvaliteta iz presude, `answer` je tada `None`); stari događaji ga nemaju.
    #[serde(default, skip_serializing_if = "is_false")]
    pub free_text: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Tko je napisao tekst: AI ili napisani rezervni (kad AI ne radi, ARHITEKTURA §8.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiSource {
    Ai,
    Fallback,
}

/// Podaci o AI pozivu uz događaj (za nadzor i trošak).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiMeta {
    pub model: String,
    pub prompt: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriterionVerdict {
    pub n: u8,
    pub met: bool,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatMessageSent {
    pub hotel_id: HotelId,
    pub month: RoundKey,
    pub situation: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterReplied {
    pub hotel_id: HotelId,
    pub month: RoundKey,
    pub situation: String,
    /// Redni broj studentove poruke na koju lik odgovara (1..=6).
    pub reply_to: u8,
    pub text: String,
    pub source: AiSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<AiMeta>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseSubmitted {
    pub hotel_id: HotelId,
    pub month: RoundKey,
    pub situation: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerdictRecorded {
    pub hotel_id: HotelId,
    pub month: RoundKey,
    pub situation: String,
    /// Pet ocjena kriterija (prazno kod neutralne presude).
    pub criteria: Vec<CriterionVerdict>,
    /// Ponuđeni odgovor čije se mehanike primjenjuju; `None` = neutralna presuda (mehanike `unanswered`).
    pub nearest_answer: Option<u8>,
    pub quality: u8,
    pub rationale: String,
    pub source: AiSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<AiMeta>,
    /// Na pregled profesoru (neutralna presuda ili AI ispad, ARHITEKTURA §8.3 korak 5).
    pub review: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebriefRecorded {
    pub hotel_id: HotelId,
    pub month: RoundKey,
    pub text: String,
    pub source: AiSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<AiMeta>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameEvent {
    GameCreated(GameCreated),
    PlanSet(PlanSet),
    AnswerChosen(AnswerChosen),
    RoundClosed(RoundClosed),
    RoundResolved(RoundResolved),
    ChatMessageSent(ChatMessageSent),
    CharacterReplied(CharacterReplied),
    ResponseSubmitted(ResponseSubmitted),
    VerdictRecorded(VerdictRecorded),
    /// Presuda koja je stigla nakon neutralne presude ili obračuna: zapisana, ne mijenja obračun (ARHITEKTURA §9.4).
    VerdictLate(VerdictRecorded),
    DebriefRecorded(DebriefRecorded),
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
            GameEvent::ChatMessageSent(_) => "ChatMessageSent",
            GameEvent::CharacterReplied(_) => "CharacterReplied",
            GameEvent::ResponseSubmitted(_) => "ResponseSubmitted",
            GameEvent::VerdictRecorded(_) => "VerdictRecorded",
            GameEvent::VerdictLate(_) => "VerdictLate",
            GameEvent::DebriefRecorded(_) => "DebriefRecorded",
        }
    }

    pub fn to_payload(&self) -> serde_json::Value {
        let v = match self {
            GameEvent::GameCreated(e) => serde_json::to_value(e),
            GameEvent::PlanSet(e) => serde_json::to_value(e),
            GameEvent::AnswerChosen(e) => serde_json::to_value(e),
            GameEvent::RoundClosed(e) => serde_json::to_value(e),
            GameEvent::RoundResolved(e) => serde_json::to_value(e),
            GameEvent::ChatMessageSent(e) => serde_json::to_value(e),
            GameEvent::CharacterReplied(e) => serde_json::to_value(e),
            GameEvent::ResponseSubmitted(e) => serde_json::to_value(e),
            GameEvent::VerdictRecorded(e) | GameEvent::VerdictLate(e) => serde_json::to_value(e),
            GameEvent::DebriefRecorded(e) => serde_json::to_value(e),
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
            "ChatMessageSent" => GameEvent::ChatMessageSent(body(kind, payload)?),
            "CharacterReplied" => GameEvent::CharacterReplied(body(kind, payload)?),
            "ResponseSubmitted" => GameEvent::ResponseSubmitted(body(kind, payload)?),
            "VerdictRecorded" => GameEvent::VerdictRecorded(body(kind, payload)?),
            "VerdictLate" => GameEvent::VerdictLate(body(kind, payload)?),
            "DebriefRecorded" => GameEvent::DebriefRecorded(body(kind, payload)?),
            other => return Err(EventError::UnknownKind(other.to_owned())),
        })
    }
}
