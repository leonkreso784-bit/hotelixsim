//! Sučelja prema vanjskom svijetu (ARHITEKTURA §6.2); izvodi ih `store`, u testovima `memory`.
use crate::{AppError, BoxFut};
use game::{GameId, GuestId, HotelId};

/// Događaj kako je pročitan iz baze; `seq` dodjeljuje pohrana.
#[derive(Clone, Debug, PartialEq)]
pub struct RawEvent {
    pub seq: i64,
    pub kind: String,
    pub schema_ver: i32,
    pub payload: serde_json::Value,
    pub actor: String,
}

/// Događaj za upis; redni broj dodjeljuje pohrana.
#[derive(Clone, Debug, PartialEq)]
pub struct NewEvent {
    pub kind: String,
    pub schema_ver: i32,
    pub payload: serde_json::Value,
    pub actor: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewJob {
    pub key: String,
    pub kind: String,
    pub game_id: GameId,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    pub id: i64,
    pub key: String,
    pub kind: String,
    pub game_id: GameId,
    pub payload: serde_json::Value,
    pub attempts: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewHotel {
    pub hotel_id: HotelId,
    pub slot: i16,
    pub guest_id: Option<GuestId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewGame {
    pub game_id: GameId,
    pub created_by: GuestId,
    pub hotels: Vec<NewHotel>,
    pub events: Vec<NewEvent>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Decision {
    pub events: Vec<NewEvent>,
    pub enqueue: Option<NewJob>,
    pub finish_job: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Decided {
    pub last_seq: i64,
    pub appended: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("pohrana: {0}")]
pub struct StoreError(pub String);

#[derive(Debug)]
pub enum DecideError {
    /// Pravilo igre ili pravo pristupa odbilo je naredbu; ništa nije upisano.
    Rejected(AppError),
    Store(StoreError),
}

pub type Decide<'a> = Box<dyn FnOnce(&[RawEvent]) -> Result<Decision, AppError> + Send + 'a>;

/// Najviše pokušaja jednog posla prije statusa `failed` (ADR-015 t. 8).
pub const MAX_ATTEMPTS: i32 = 5;

pub trait EventLog: Send + Sync {
    /// `games`, `hotels` i prvi događaji igre (redni brojevi od 1) u JEDNOJ transakciji.
    fn create_game(&self, game: NewGame) -> BoxFut<'_, Result<(), StoreError>>;
    /// Cijeli tok `game:<id>` redom; prazan popis = igra ne postoji.
    fn load(&self, game: GameId) -> BoxFut<'_, Result<Vec<RawEvent>, StoreError>>;
    /// U JEDNOJ transakciji pod zaključavanjem igre: učitaj tok, pozovi `decide`, upiši događaje (redni brojevi
    /// nastavljaju tok), upiši posao (`ON CONFLICT` po ključu: ništa) i označi `finish_job` gotovim.
    /// `decide` koji vrati grešku ne upisuje ništa.
    fn decide<'a>(
        &'a self,
        game: GameId,
        decide: Decide<'a>,
    ) -> BoxFut<'a, Result<Decided, DecideError>>;
}

pub trait Guests: Send + Sync {
    fn create_guest(&self, id: GuestId, token_hash: String) -> BoxFut<'_, Result<(), StoreError>>;
    fn guest_by_token_hash(
        &self,
        token_hash: String,
    ) -> BoxFut<'_, Result<Option<GuestId>, StoreError>>;
    fn game_of_hotel(&self, hotel: HotelId) -> BoxFut<'_, Result<Option<GameId>, StoreError>>;
    /// Hotel gosta u njegovoj zadnjoj igri.
    fn latest_hotel_of(
        &self,
        guest: GuestId,
    ) -> BoxFut<'_, Result<Option<(GameId, HotelId)>, StoreError>>;
}

pub trait JobQueue: Send + Sync {
    /// Preuzmi najstariji posao u redu ili posao s isteklim zakupom; zakup traje `lease_secs` (ARHITEKTURA §9.4).
    fn claim(&self, lease_secs: i32) -> BoxFut<'_, Result<Option<Job>, StoreError>>;
    /// Vrati posao u red, ili ga označi `failed` kad je potrošio `MAX_ATTEMPTS`.
    fn fail(&self, id: i64, error: String) -> BoxFut<'_, Result<(), StoreError>>;
}

/// Izvor identifikatora i sjemena; u testovima predvidljiv (`memory::SeqIds`).
pub trait Ids: Send + Sync {
    fn uuid(&self) -> uuid::Uuid;
    fn seed(&self) -> u64;
}

pub struct RandomIds;

impl Ids for RandomIds {
    fn uuid(&self) -> uuid::Uuid {
        uuid::Uuid::new_v4()
    }
    fn seed(&self) -> u64 {
        let (hi, lo) = uuid::Uuid::new_v4().as_u64_pair();
        hi ^ lo.rotate_left(32)
    }
}
