//! Naredbe razgovora, slobodnog odgovora i mentora (DEMO korak 4): čiste, vraćaju događaj za upis.

use sim_hotel::SituationTemplate;

use crate::round::RoundKey;

use crate::error::GameError;
use crate::event::{
    AiMeta, AiSource, CharacterReplied, ChatMessageSent, DebriefRecorded, GameEvent,
    ResponseSubmitted, SeasonDebriefRecorded,
};
use crate::game::{ChatLine, Game, RoundStatus, Speaker, student_count};
use crate::ids::HotelId;

/// Najviše studentovih poruka liku u jednoj situaciji.
pub const MAX_CHAT_MESSAGES: u8 = 6;
/// Najviše znakova (ne bajtova) u poruci ili slobodnom odgovoru.
pub const MAX_TEXT_CHARS: usize = 2000;

/// Tekst koji je lik izgovorio, s podrijetlom (AI ili rezervni).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplyBody {
    pub text: String,
    pub source: AiSource,
    pub meta: Option<AiMeta>,
}

fn clean(text: &str) -> Result<String, GameError> {
    let t = text.trim();
    if t.is_empty() {
        return Err(GameError::EmptyText);
    }
    if t.chars().count() > MAX_TEXT_CHARS {
        return Err(GameError::TextTooLong);
    }
    Ok(t.to_owned())
}

impl Game {
    /// Krug u kojem se situacija igra i njen predložak, tražen po hotelima redom utora
    /// (`UnknownSituation` ako je nema ni u jednom krugu ni kod jednog hotela). Kad je hotel poznat, treba
    /// `situation_round_for`: dva hotela mogu u istom krugu dobiti isti predložak.
    pub fn situation_round(&self, code: &str) -> Result<(RoundKey, &SituationTemplate), GameError> {
        self.hotels
            .iter()
            .find_map(|h| self.situation_round_for(h.setup.hotel_id, code).ok())
            .ok_or(GameError::UnknownSituation)
    }

    /// Zajedničke provjere poruke i slobodnog odgovora; vraća mjesec i trenutni razgovor.
    fn check_talk(
        &self,
        hotel: HotelId,
        code: &str,
    ) -> Result<(RoundKey, Vec<ChatLine>), GameError> {
        let h = self.hotel(hotel)?;
        let (m, _) = self.situation_round_for(hotel, code)?;
        if self.status(m)? != RoundStatus::Open {
            return Err(GameError::RoundNotOpen);
        }
        if !self.ai_entry_for(hotel, code).is_some_and(|a| a.free_text) {
            return Err(GameError::NotFreeText);
        }
        let key = (m, code.to_owned());
        if h.decided(&key) {
            return Err(GameError::AlreadyAnswered);
        }
        Ok((m, h.chats.get(&key).cloned().unwrap_or_default()))
    }

    pub fn send_message(
        &self,
        hotel: HotelId,
        code: &str,
        text: &str,
    ) -> Result<GameEvent, GameError> {
        let (m, lines) = self.check_talk(hotel, code)?;
        let text = clean(text)?;
        if lines.last().is_some_and(|l| l.from == Speaker::Student) {
            return Err(GameError::AwaitingReply);
        }
        if student_count(&lines) >= usize::from(MAX_CHAT_MESSAGES) {
            return Err(GameError::ChatLimit);
        }
        Ok(GameEvent::ChatMessageSent(ChatMessageSent {
            hotel_id: hotel,
            month: m,
            situation: code.to_owned(),
            text,
        }))
    }

    pub fn submit_response(
        &self,
        hotel: HotelId,
        code: &str,
        text: &str,
    ) -> Result<GameEvent, GameError> {
        let (m, _) = self.check_talk(hotel, code)?;
        let text = clean(text)?;
        Ok(GameEvent::ResponseSubmitted(ResponseSubmitted {
            hotel_id: hotel,
            month: m,
            situation: code.to_owned(),
            text,
        }))
    }

    /// `None` = nema što upisati (runda nije otvorena ili je na tu poruku već odgovoreno).
    pub fn record_reply(
        &self,
        hotel: HotelId,
        code: &str,
        reply_to: u8,
        reply: ReplyBody,
    ) -> Result<Option<GameEvent>, GameError> {
        let h = self.hotel(hotel)?;
        let (m, _) = self.situation_round_for(hotel, code)?;
        if self.status(m)? != RoundStatus::Open {
            return Ok(None);
        }
        let empty = Vec::new();
        let lines = h.chats.get(&(m, code.to_owned())).unwrap_or(&empty);
        if usize::from(reply_to) != student_count(lines)
            || lines.last().is_none_or(|l| l.from != Speaker::Student)
        {
            return Ok(None);
        }
        Ok(Some(GameEvent::CharacterReplied(CharacterReplied {
            hotel_id: hotel,
            month: m,
            situation: code.to_owned(),
            reply_to,
            text: reply.text,
            source: reply.source,
            meta: reply.meta,
        })))
    }

    /// `None` = mentor za taj mjesec već postoji.
    pub fn record_debrief(
        &self,
        hotel: HotelId,
        m: impl Into<RoundKey>,
        text: String,
        source: AiSource,
        meta: Option<AiMeta>,
    ) -> Result<Option<GameEvent>, GameError> {
        let m = m.into();
        let h = self.hotel(hotel)?;
        if self.status(m)? != RoundStatus::Resolved {
            return Err(GameError::RoundNotOpen);
        }
        if h.debriefs.contains_key(&m) {
            return Ok(None);
        }
        Ok(Some(GameEvent::DebriefRecorded(DebriefRecorded {
            hotel_id: hotel,
            month: m,
            text,
            source,
            meta,
        })))
    }

    /// Sezonski mentor hotela (v2, nakon zadnjeg kruga); `None` = već postoji.
    pub fn record_season_debrief(
        &self,
        hotel: HotelId,
        text: String,
        source: AiSource,
        meta: Option<AiMeta>,
    ) -> Result<Option<GameEvent>, GameError> {
        self.model().require_v2(&self.content)?;
        let h = self.hotel(hotel)?;
        if self.current_round().is_some() {
            return Err(GameError::SeasonNotOver);
        }
        if h.season_debrief.is_some() {
            return Ok(None);
        }
        Ok(Some(GameEvent::SeasonDebriefRecorded(
            SeasonDebriefRecorded {
                hotel_id: hotel,
                text,
                source,
                meta,
            },
        )))
    }

    /// Razgovor igrača sa likom u situaciji (prazan ako ga nema).
    pub fn chat(&self, hotel: HotelId, code: &str) -> Result<Vec<ChatLine>, GameError> {
        let h = self.hotel(hotel)?;
        let (m, _) = self.situation_round_for(hotel, code)?;
        Ok(h.chats
            .get(&(m, code.to_owned()))
            .cloned()
            .unwrap_or_default())
    }
}
