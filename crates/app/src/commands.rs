//! Naredbe runde: svaka je JEDAN `decide`, pa provjera prava i pravila igre i upis idu pod istim zaključavanjem.
//! Redoslijed provjera: igra postoji (`fold_raw`), pravo (`member_hotel`/`require_owner`), pravilo igre.
use game::dto::{MonthCode, PlanDto};
use game::{Actor, GameId, HotelId};
use sim_hotel::{Month, MonthPlan};

use crate::demo::{fold_raw, load_game, new_event};
use crate::jobs::{RESOLVE_KIND, ResolvePayload, resolve_key};
use crate::views::{game_of_hotel, member_hotel, require_owner, round_view};
use crate::{
    AppError, Caller, Decision, Deps, NewJob, RawEvent, RoundView, SituationsView, situations,
};

pub async fn set_plan(
    deps: &Deps,
    caller: &Caller,
    hotel: HotelId,
    month: MonthCode,
    plan: PlanDto,
) -> Result<PlanDto, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let m: Month = month.into();
    let core = MonthPlan::from(&plan);
    deps.log
        .decide(
            game_id,
            Box::new(move |raw: &[RawEvent]| {
                let g = fold_raw(raw)?;
                require_owner(&g, guest, hotel)?;
                let ev = g.set_plan(hotel, m, core)?;
                Ok(Decision {
                    events: vec![new_event(&Actor::Guest(guest), &ev)],
                    ..Decision::default()
                })
            }),
        )
        .await?;
    Ok(plan)
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

pub async fn round_status(
    deps: &Deps,
    caller: &Caller,
    game_id: GameId,
    month: MonthCode,
) -> Result<RoundView, AppError> {
    let guest = caller.require()?;
    let g = load_game(deps, game_id).await?;
    let mine = member_hotel(&g, guest)?;
    round_view(&g, month.into(), mine)
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
    month: MonthCode,
) -> Result<CloseOutcome, AppError> {
    let guest = caller.require()?;
    let m: Month = month.into();
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
