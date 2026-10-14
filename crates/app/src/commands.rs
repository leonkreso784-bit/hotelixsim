//! Naredbe runde: svaka je JEDAN `decide`, pa provjera prava i pravila igre i upis idu pod istim zaključavanjem.
//! Redoslijed provjera: igra postoji (`fold_raw`), pravo (`member_hotel`/`require_owner`), pravilo igre.
use game::RoundKey;
use game::dto::PlanDto;
use game::{Actor, GameEvent, GameId, HotelId};

use crate::demo::{fold_raw, load_game, new_event};
use crate::jobs::{RESOLVE_KIND, ResolvePayload, resolve_key};
use crate::views::{game_of_hotel, member_hotel, require_owner, round_view};
use crate::{
    AppError, Caller, Decision, Deps, ErrorCode, NewJob, RawEvent, RoundView, SituationsView,
    situations,
};

pub async fn set_plan(
    deps: &Deps,
    caller: &Caller,
    hotel: HotelId,
    month: RoundKey,
    plan: PlanDto,
) -> Result<PlanDto, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let m = month;
    // Odgovor je plan kakav je zapisan u `PlanSet` (v2 s odjelima: ukupni sezonci su zbroj odjela; bez odjela:
    // odjeli iz važećeg plana), ne poslano tijelo.
    let mut saved: Option<PlanDto> = None;
    let out = &mut saved;
    deps.log
        .decide(
            game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                require_owner(&g, guest, hotel)?;
                let core = g.plan_from_dto(hotel, m, &plan)?;
                let ev = g.set_plan(hotel, m, core)?;
                if let GameEvent::PlanSet(p) = &ev {
                    *out = Some(p.plan.clone());
                }
                Ok(Decision {
                    events: vec![new_event(&Actor::Guest(guest), &ev)],
                    ..Decision::default()
                })
            }),
        )
        .await?;
    saved.ok_or_else(|| AppError::new(ErrorCode::Internal))
}

pub async fn answer(
    deps: &Deps,
    caller: &Caller,
    hotel: HotelId,
    code: &str,
    choice: u8,
) -> Result<SituationsView, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let code = code.to_owned();
    deps.log
        .decide(
            game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                require_owner(&g, guest, hotel)?;
                let ev = g.choose_answer(hotel, &code, choice)?;
                Ok(Decision {
                    events: vec![new_event(&Actor::Guest(guest), &ev)],
                    ..Decision::default()
                })
            }),
        )
        .await?;
    situations(deps, caller, hotel).await
}

/// „Prepusti voditelju" (Leon P4): zapiše `SituationDelegated` i vrati pogled situacija, kao odgovor na ponuđeni.
pub async fn delegate_situation(
    deps: &Deps,
    caller: &Caller,
    hotel: HotelId,
    code: &str,
) -> Result<SituationsView, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let code = code.to_owned();
    deps.log
        .decide(
            game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                require_owner(&g, guest, hotel)?;
                let ev = g.delegate_situation(hotel, &code)?;
                Ok(Decision {
                    events: vec![new_event(&Actor::Guest(guest), &ev)],
                    ..Decision::default()
                })
            }),
        )
        .await?;
    situations(deps, caller, hotel).await
}

pub async fn round_status(
    deps: &Deps,
    caller: &Caller,
    game_id: GameId,
    month: RoundKey,
) -> Result<RoundView, AppError> {
    let guest = caller.require()?;
    let g = load_game(deps, game_id).await?;
    let mine = member_hotel(&g, guest)?;
    round_view(&g, month, mine)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloseOutcome {
    pub accepted: bool,
    pub round: RoundView,
}

/// „Zatvori mjesec": `RoundClosed` i posao obračuna u istoj transakciji; drugi klik ne piše ništa.
pub async fn close_round(
    deps: &Deps,
    caller: &Caller,
    game_id: GameId,
    month: RoundKey,
) -> Result<CloseOutcome, AppError> {
    let guest = caller.require()?;
    let m = month;
    let decided = deps
        .log
        .decide(
            game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                member_hotel(&g, guest)?;
                match g.close_round(m)? {
                    None => Ok(Decision::default()),
                    Some(ev) => Ok(Decision {
                        events: vec![new_event(&Actor::Guest(guest), &ev)],
                        enqueue: vec![NewJob {
                            key: resolve_key(game_id, m),
                            kind: RESOLVE_KIND.to_owned(),
                            game_id,
                            payload: serde_json::to_value(ResolvePayload { month })
                                .map_err(|e| AppError::internal(e.to_string()))?,
                        }],
                        finish_job: None,
                    }),
                }
            }),
        )
        .await?;
    let g = load_game(deps, game_id).await?;
    let mine = member_hotel(&g, guest)?;
    Ok(CloseOutcome {
        accepted: decided.appended > 0,
        round: round_view(&g, m, mine)?,
    })
}
