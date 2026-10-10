//! Razgovor sa likom i slobodan odgovor (DEMO korak 4): svaka naredba je JEDAN `decide`, a odgovor lika i
//! presudu kasnije piše radnik (`jobs`). AI se ovdje nikad ne zove.
use game::{Actor, ChatLine, Game, HotelId, MAX_CHAT_MESSAGES, RoundStatus, Speaker};

use crate::demo::{fold_raw, load_game, new_event};
use crate::jobs::{ASSESS_KIND, AssessPayload, CHAT_KIND, ChatPayload, assess_key, chat_key};
use crate::views::{game_of_hotel, require_owner};
use crate::{
    AppError, Caller, Decision, Deps, ErrorCode, NewJob, RawEvent, SituationsView, situations,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConversationView {
    pub situation: String,
    pub character_name: String,
    pub role: String,
    pub opening: String,
    pub lines: Vec<ChatLine>,
    /// Koliko poruka student još smije poslati.
    pub remaining: u8,
    pub awaiting_reply: bool,
    /// Runda situacije je otvorena (inače je razgovor samo za čitanje).
    pub open: bool,
}

fn student_count(lines: &[ChatLine]) -> usize {
    lines.iter().filter(|l| l.from == Speaker::Student).count()
}

fn view_of(g: &Game, hotel: HotelId, code: &str) -> Result<ConversationView, AppError> {
    let (m, _) = g.situation_round(code)?;
    let c = g
        .content
        .ai
        .get(code)
        .and_then(|a| a.character.as_ref())
        .ok_or(AppError::new(ErrorCode::NotFreeText))?;
    let lines = g.chat(hotel, code)?;
    let used = u8::try_from(student_count(&lines)).unwrap_or(MAX_CHAT_MESSAGES);
    let open = g.status(m)? == RoundStatus::Open;
    Ok(ConversationView {
        situation: code.to_owned(),
        character_name: c.name.clone(),
        role: c.role.clone(),
        opening: c.opening.clone(),
        awaiting_reply: open && lines.last().is_some_and(|l| l.from == Speaker::Student),
        lines,
        remaining: MAX_CHAT_MESSAGES.saturating_sub(used),
        open,
    })
}

pub async fn conversation(
    deps: &Deps,
    caller: &Caller,
    hotel: HotelId,
    code: &str,
) -> Result<ConversationView, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let g = load_game(deps, game_id).await?;
    require_owner(&g, guest, hotel)?;
    view_of(&g, hotel, code)
}

pub async fn send_message(
    deps: &Deps,
    caller: &Caller,
    hotel: HotelId,
    code: &str,
    text: String,
) -> Result<ConversationView, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let situation = code.to_owned();
    deps.log
        .decide(
            game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                require_owner(&g, guest, hotel)?;
                let ev = g.send_message(hotel, &situation, &text)?;
                // broj studentovih poruka nakon ove = na koju poruku lik odgovara
                let reply_to = u8::try_from(student_count(&g.chat(hotel, &situation)?) + 1)
                    .map_err(|_| AppError::internal("previše poruka"))?;
                let job = NewJob {
                    key: chat_key(game_id, hotel, &situation, reply_to),
                    kind: CHAT_KIND.to_owned(),
                    game_id,
                    payload: serde_json::to_value(ChatPayload {
                        hotel_id: hotel,
                        situation,
                        reply_to,
                    })
                    .map_err(|e| AppError::internal(e.to_string()))?,
                };
                Ok(Decision {
                    events: vec![new_event(&Actor::Guest(guest), &ev)],
                    enqueue: vec![job],
                    finish_job: None,
                })
            }),
        )
        .await?;
    conversation(deps, caller, hotel, code).await
}

pub async fn submit_response(
    deps: &Deps,
    caller: &Caller,
    hotel: HotelId,
    code: &str,
    text: String,
) -> Result<SituationsView, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let situation = code.to_owned();
    deps.log
        .decide(
            game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                require_owner(&g, guest, hotel)?;
                let ev = g.submit_response(hotel, &situation, &text)?;
                let job = NewJob {
                    key: assess_key(game_id, hotel, &situation),
                    kind: ASSESS_KIND.to_owned(),
                    game_id,
                    payload: serde_json::to_value(AssessPayload {
                        hotel_id: hotel,
                        situation,
                    })
                    .map_err(|e| AppError::internal(e.to_string()))?,
                };
                Ok(Decision {
                    events: vec![new_event(&Actor::Guest(guest), &ev)],
                    enqueue: vec![job],
                    finish_job: None,
                })
            }),
        )
        .await?;
    situations(deps, caller, hotel).await
}
