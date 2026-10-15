use game::command::{GameSettings, new_game_with};
use game::{Actor, Game, GameEvent, GameId, GuestId, HotelId, Recorded, SCHEMA_VER};
use sha2::{Digest, Sha256};
use sim_hotel::{Index, Profile};
use sim_hotel_v2::PeriodKind;

use crate::{AppError, Deps, ErrorCode, NewEvent, NewGame, NewHotel, RawEvent};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Caller {
    pub guest: Option<GuestId>,
}

impl Caller {
    pub fn require(&self) -> Result<GuestId, AppError> {
        self.guest.ok_or(AppError::new(ErrorCode::GuestRequired))
    }
}

/// U bazi je samo otisak tokena; sam token živi u kolačiću `httpOnly`.
pub fn token_hash(token: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(token.as_bytes()))
}

/// Gost iz kolačića; nepoznat ili pokvaren token = anoniman pozivatelj (nikad greška).
pub async fn caller(deps: &Deps, token: Option<&str>) -> Result<Caller, AppError> {
    let Some(t) = token else {
        return Ok(Caller { guest: None });
    };
    Ok(Caller {
        guest: deps.guests.guest_by_token_hash(token_hash(t)).await?,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoStarted {
    pub game_id: GameId,
    pub hotel_id: HotelId,
    pub new_token: Option<String>,
}

/// Postavke nove igre: model pravila i (neobavezno) vrsta kruga.
/// v1 + `None`|`Month` → v1 mjesečna (lipanj–srpanj); v1 + `Week` → `InvalidSettings`;
/// v2 + `None` → tjedna cijela sezona (🟨 P2); v2 + vrsta → ta vrsta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NewGameSettings {
    pub model: content::Model,
    pub kind: Option<PeriodKind>,
}

impl Default for NewGameSettings {
    fn default() -> Self {
        NewGameSettings {
            model: content::Model::V1,
            kind: None,
        }
    }
}

/// `POST /v1/demo` (DEMO §1.4): gost (postojeći ili novi) i nova igra s tri robota. Igra se stvara u jednoj
/// transakciji, ali gost se upisuje prije nje (dvije zasebne radnje; siroče gost je opisan u BACKLOG-u).
pub async fn start_demo(
    deps: &Deps,
    token: Option<&str>,
    settings: NewGameSettings,
) -> Result<DemoStarted, AppError> {
    let (guest, new_guest) = match caller(deps, token).await?.guest {
        Some(g) => (g, None),
        None => (
            GuestId(deps.ids.uuid()),
            Some(deps.ids.uuid().simple().to_string()),
        ),
    };
    let game_id = GameId(deps.ids.uuid());
    let hotel_ids = [(); 4].map(|_| HotelId(deps.ids.uuid()));
    let (snapshot, default_kind) =
        game::Model::new_game_snapshot(settings.model, &deps.snapshot, &deps.snapshot_v2);
    let kind = settings.kind.unwrap_or(default_kind);
    // Postavke se provjere prije upisa gosta: odbijena igra ne ostavlja siroče gosta.
    let events = new_game_with(
        game_id,
        deps.ids.seed(),
        snapshot,
        guest,
        hotel_ids,
        GameSettings {
            kind,
            situations: None,
        },
    )?;
    if let Some(tok) = &new_guest {
        deps.guests.create_guest(guest, token_hash(tok)).await?;
    }
    let hotels = hotel_ids
        .iter()
        .zip(1_i16..)
        .map(|(&hotel_id, slot)| NewHotel {
            hotel_id,
            slot,
            guest_id: (slot == 1).then_some(guest),
        })
        .collect();
    deps.log
        .create_game(NewGame {
            game_id,
            created_by: guest,
            hotels,
            events: events.iter().map(|(a, e)| new_event(a, e)).collect(),
        })
        .await?;
    Ok(DemoStarted {
        game_id,
        hotel_id: hotel_ids[0],
        new_token: new_guest,
    })
}

/// Početni indeksi profila (DEMO §5) — prije u `api/src/demo.rs` mimo `app` (BACKLOG [dnevnik]).
pub fn profile_indices(code: &str) -> Result<[u16; 5], AppError> {
    let p = Profile::from_code(code).ok_or(AppError::new(ErrorCode::UnknownProfile))?;
    let s = p.starting_indices();
    Ok(Index::ALL.map(|i| s.get(i).hundredths()))
}

pub(crate) fn new_event(actor: &Actor, ev: &GameEvent) -> NewEvent {
    NewEvent {
        kind: ev.kind().to_owned(),
        schema_ver: SCHEMA_VER,
        payload: ev.to_payload(),
        actor: actor.as_db(),
    }
}

pub(crate) fn fold_raw(raw: &[RawEvent]) -> Result<Game, AppError> {
    if raw.is_empty() {
        return Err(AppError::new(ErrorCode::UnknownGame));
    }
    let records = raw
        .iter()
        .map(|r| {
            let actor = Actor::parse(&r.actor)
                .ok_or_else(|| AppError::internal(format!("actor {}", r.actor)))?;
            let event = GameEvent::from_record(&r.kind, r.schema_ver, r.payload.clone())
                .map_err(|e| AppError::internal(format!("{e:?}")))?;
            Ok(Recorded {
                seq: r.seq,
                actor,
                event,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    Ok(Game::fold(&records)?)
}

pub(crate) async fn load_game(deps: &Deps, id: GameId) -> Result<Game, AppError> {
    fold_raw(&deps.log.load(id).await?)
}
