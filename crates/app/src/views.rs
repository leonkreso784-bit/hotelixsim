//! Čitanje po ulozi (ARHITEKTURA §12): provjere idu redom gost, igra postoji, član, vlasnik.
use game::dto::MonthCode;
use game::feed::FeedItem;
use game::view::{PrivateHotel, PublicHotel};
use game::{Controller, Game, GameId, GuestId, HotelId, RoundStatus};
use sim_hotel::Month;

use crate::demo::load_game;
use crate::{AppError, Caller, Deps, ErrorCode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HotelView {
    pub game_id: GameId,
    pub public: PublicHotel,
    pub private: Option<PrivateHotel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoundView {
    pub month: MonthCode,
    pub status: RoundStatus,
    pub unanswered: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameView {
    pub game_id: GameId,
    pub months: Vec<MonthCode>,
    pub current: Option<RoundView>,
    pub my_hotel: HotelId,
    pub hotels: Vec<PublicHotel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationItem {
    pub code: String,
    pub title: String,
    pub answers: Vec<String>,
    pub chosen: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SituationsView {
    pub month: Option<MonthCode>,
    pub open: bool,
    pub items: Vec<SituationItem>,
}

pub(crate) fn member_hotel(g: &Game, guest: GuestId) -> Result<HotelId, AppError> {
    g.hotels
        .iter()
        .find(|h| matches!(h.setup.controller, Controller::Human { guest_id } if guest_id == guest))
        .map(|h| h.setup.hotel_id)
        .ok_or(AppError::new(ErrorCode::NotInGame))
}

pub(crate) fn require_owner(g: &Game, guest: GuestId, hotel: HotelId) -> Result<(), AppError> {
    let mine = member_hotel(g, guest)?;
    g.hotel(hotel)?;
    if mine == hotel {
        Ok(())
    } else {
        Err(AppError::new(ErrorCode::NotYourHotel))
    }
}

pub(crate) fn round_view(g: &Game, m: Month, mine: HotelId) -> Result<RoundView, AppError> {
    let status = g.status(m)?;
    let unanswered = if status == RoundStatus::Open {
        g.unanswered_count(mine, m)?
    } else {
        0
    };
    Ok(RoundView {
        month: m.into(),
        status,
        unanswered,
    })
}

/// Jedino mjesto s ovim pretraživanjem; koristi ga i `commands` (Task 6).
pub(crate) async fn game_of_hotel(deps: &Deps, hotel: HotelId) -> Result<GameId, AppError> {
    deps.guests
        .game_of_hotel(hotel)
        .await?
        .ok_or(AppError::new(ErrorCode::UnknownHotel))
}

fn hotel_view_of(
    g: &Game,
    game_id: GameId,
    guest: GuestId,
    hotel: HotelId,
) -> Result<HotelView, AppError> {
    let mine = member_hotel(g, guest)?;
    let public = g.public_hotel(hotel)?;
    let private = if mine == hotel {
        Some(g.private_hotel(hotel)?)
    } else {
        None
    };
    Ok(HotelView {
        game_id,
        public,
        private,
    })
}

pub async fn hotel_view(
    deps: &Deps,
    caller: &Caller,
    hotel: HotelId,
) -> Result<HotelView, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let g = load_game(deps, game_id).await?;
    hotel_view_of(&g, game_id, guest, hotel)
}

pub async fn my_hotel(deps: &Deps, caller: &Caller) -> Result<HotelView, AppError> {
    let guest = caller.require()?;
    let (game_id, hotel) = deps
        .guests
        .latest_hotel_of(guest)
        .await?
        .ok_or(AppError::new(ErrorCode::UnknownGame))?;
    let g = load_game(deps, game_id).await?;
    hotel_view_of(&g, game_id, guest, hotel)
}

pub async fn game_view(
    deps: &Deps,
    caller: &Caller,
    game_id: GameId,
) -> Result<GameView, AppError> {
    let guest = caller.require()?;
    let g = load_game(deps, game_id).await?;
    let mine = member_hotel(&g, guest)?;
    let mut setups: Vec<_> = g
        .hotels
        .iter()
        .map(|h| (h.setup.slot, h.setup.hotel_id))
        .collect();
    setups.sort();
    let hotels = setups
        .into_iter()
        .map(|(_, id)| g.public_hotel(id))
        .collect::<Result<Vec<_>, _>>()?;
    let current = g
        .current_month()
        .map(|m| round_view(&g, m, mine))
        .transpose()?;
    Ok(GameView {
        game_id,
        months: g.months.iter().map(|&m| m.into()).collect(),
        current,
        my_hotel: mine,
        hotels,
    })
}

pub async fn situations(
    deps: &Deps,
    caller: &Caller,
    hotel: HotelId,
) -> Result<SituationsView, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let g = load_game(deps, game_id).await?;
    require_owner(&g, guest, hotel)?;
    let Some(month) = g.current_month() else {
        return Ok(SituationsView {
            month: None,
            open: false,
            items: Vec::new(),
        });
    };
    let record = g.hotel(hotel)?;
    let items = g
        .situations_in(month)
        .into_iter()
        .map(|t| SituationItem {
            code: t.code.clone(),
            title: t.title.clone(),
            answers: t.answers.iter().map(|a| a.label.clone()).collect(),
            chosen: record.answers.get(&(month, t.code.clone())).copied(),
        })
        .collect();
    Ok(SituationsView {
        month: Some(month.into()),
        open: g.status(month)? == RoundStatus::Open,
        items,
    })
}

pub async fn feed(deps: &Deps, caller: &Caller, hotel: HotelId) -> Result<Vec<FeedItem>, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let g = load_game(deps, game_id).await?;
    require_owner(&g, guest, hotel)?;
    Ok(g.feed(hotel)?)
}
