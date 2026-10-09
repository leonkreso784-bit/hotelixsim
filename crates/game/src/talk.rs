//! Naredbe razgovora, slobodnog odgovora i mentora (DEMO korak 4): čiste, vraćaju događaj za upis.

use sim_hotel::Month;

use crate::error::GameError;
use crate::event::{
    AiMeta, AiSource, CharacterReplied, ChatMessageSent, DebriefRecorded, GameEvent,
    ResponseSubmitted,
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
    /// Mjesec u kojem se situacija igra (`UnknownSituation` ako nije u mjesecima ove igre).
    pub fn situation_month(&self, code: &str) -> Result<Month, GameError> {
        self.content
            .situations
            .iter()
            .find(|t| t.code == code && self.months.contains(&t.month))
            .map(|t| t.month)
            .ok_or(GameError::UnknownSituation)
    }

    /// Zajedničke provjere poruke i slobodnog odgovora; vraća mjesec i trenutni razgovor.
    fn check_talk(&self, hotel: HotelId, code: &str) -> Result<(Month, Vec<ChatLine>), GameError> {
        let h = self.hotel(hotel)?;
        let m = self.situation_month(code)?;
        if self.status(m)? != RoundStatus::Open {
            return Err(GameError::RoundNotOpen);
        }
        if !self.content.ai.get(code).is_some_and(|a| a.free_text) {
            return Err(GameError::NotFreeText);
        }
        let key = (m, code.to_owned());
        if h.answers.contains_key(&key) || h.responses.contains_key(&key) {
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
            month: m.into(),
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
            month: m.into(),
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
        let m = self.situation_month(code)?;
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
            month: m.into(),
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
        m: Month,
        text: String,
        source: AiSource,
        meta: Option<AiMeta>,
    ) -> Result<Option<GameEvent>, GameError> {
        let h = self.hotel(hotel)?;
        if self.status(m)? != RoundStatus::Resolved {
            return Err(GameError::RoundNotOpen);
        }
        if h.debriefs.contains_key(&m) {
            return Ok(None);
        }
        Ok(Some(GameEvent::DebriefRecorded(DebriefRecorded {
            hotel_id: hotel,
            month: m.into(),
            text,
            source,
            meta,
        })))
    }

    /// Razgovor igrača sa likom u situaciji (prazan ako ga nema).
    pub fn chat(&self, hotel: HotelId, code: &str) -> Result<Vec<ChatLine>, GameError> {
        let h = self.hotel(hotel)?;
        let m = self.situation_month(code)?;
        Ok(h.chats
            .get(&(m, code.to_owned()))
            .cloned()
            .unwrap_or_default())
    }
}
