### Task 5: `app` — portovi, greške, lažna pohrana, gost i pogledi po ulozi

**Files:**
- Modify: `crates/app/Cargo.toml`, `crates/app/src/lib.rs` (postojeći kod zdravlja ostaje; dodaj module i `Deps`)
- Create: `crates/app/src/{ports,error,memory,demo,views}.rs`
- Test: `crates/app/tests/views.rs`

**Interfaces:**
- Consumes: Task 2–4 (`game::*`, `game::game::{Game, RoundStatus}`, `game::command::new_game`, `game::view::*`,
  `game::feed::FeedItem`), `content::Snapshot`.
- Produces (re-export iz `app` korijena; `pub use game::{GameId, HotelId, GuestId}` da `store` i `api` ne ovise o `game`):
  - `ports`: `RawEvent { seq: i64, kind: String, schema_ver: i32, payload: serde_json::Value, actor: String }`,
    `NewEvent { kind: String, schema_ver: i32, payload: serde_json::Value, actor: String }`,
    `NewJob { key: String, kind: String, game_id: GameId, payload: serde_json::Value }`,
    `Job { id: i64, key: String, kind: String, game_id: GameId, payload: serde_json::Value, attempts: i32 }`,
    `NewHotel { hotel_id: HotelId, slot: i16, guest_id: Option<GuestId> }`,
    `NewGame { game_id: GameId, created_by: GuestId, hotels: Vec<NewHotel>, events: Vec<NewEvent> }`,
    `#[derive(Default)] Decision { events: Vec<NewEvent>, enqueue: Option<NewJob>, finish_job: Option<i64> }`,
    `Decided { last_seq: i64, appended: usize }`, `StoreError(pub String)`,
    `enum DecideError { Rejected(AppError), Store(StoreError) }`,
    `type Decide<'a> = Box<dyn FnOnce(&[RawEvent]) -> Result<Decision, AppError> + Send + 'a>`,
    traitovi `EventLog`, `Guests`, `JobQueue`, `Ids` (potpisi u Step 3), `RandomIds`, `pub const MAX_ATTEMPTS: i32 = 5`.
  - `error`: `ErrorCode` (popis u Step 3), `ErrorKind { Unauthorized, Forbidden, NotFound, Conflict, Invalid, Unavailable, Internal }`,
    `ErrorCode::{kind, as_str}`, `AppError { code: ErrorCode, detail: String }` s `AppError::{new, internal}`,
    `From<GameError>`, `From<StoreError>`, `From<DecideError>`.
  - `Deps { status: Arc<dyn SystemStatus>, log: Arc<dyn EventLog>, guests: Arc<dyn Guests>, jobs: Arc<dyn JobQueue>,
    ids: Arc<dyn Ids>, snapshot: Arc<content::Snapshot> }` (`Clone`).
  - `demo`: `Caller { guest: Option<GuestId> }` s `Caller::require(&self) -> Result<GuestId, AppError>`;
    `token_hash(&str) -> String`; `caller(&Deps, Option<&str>) -> Result<Caller, AppError>`;
    `DemoStarted { game_id, hotel_id, new_token: Option<String> }`; `start_demo(&Deps, Option<&str>) -> Result<DemoStarted, AppError>`;
    `profile_indices(&str) -> Result<[u16; 5], AppError>`; `pub(crate) fold_raw(&[RawEvent]) -> Result<Game, AppError>`;
    `pub(crate) new_event(&Actor, &GameEvent) -> NewEvent`; `pub(crate) load_game(&Deps, GameId) -> Result<Game, AppError>`.
  - `views`: `HotelView { game_id: GameId, public: PublicHotel, private: Option<PrivateHotel> }`,
    `RoundView { month: MonthCode, status: RoundStatus, unanswered: u8 }`,
    `GameView { game_id: GameId, months: Vec<MonthCode>, current: Option<RoundView>, my_hotel: HotelId, hotels: Vec<PublicHotel> }`,
    `SituationItem { code: String, title: String, answers: Vec<String>, chosen: Option<u8> }`,
    `SituationsView { month: Option<MonthCode>, open: bool, items: Vec<SituationItem> }`;
    `my_hotel`, `game_view`, `hotel_view`, `situations`, `feed` (potpisi u Step 4);
    `pub(crate) member_hotel(&Game, GuestId) -> Result<HotelId, AppError>`,
    `pub(crate) require_owner(&Game, GuestId, HotelId) -> Result<(), AppError>`,
    `pub(crate) round_view(&Game, Month, HotelId) -> Result<RoundView, AppError>`.
  - `memory`: `Memory` (izvodi sva četiri porta + `SystemStatus`), `MemJob`, `JobStatus`, `SeqIds`,
    `deps(content::Snapshot) -> (Deps, Arc<Memory>)`; pomoćnici `Memory::{jobs, events, expire_leases, push_job}`.

- [ ] **Step 1: Ovisnosti.** `crates/app/Cargo.toml` `[dependencies]`: `game.workspace = true`,
  `content.workspace = true`, `sim-hotel.workspace = true`, `serde.workspace = true`, `serde_json.workspace = true`,
  `sha2.workspace = true`, `thiserror.workspace = true`, `uuid = { workspace = true, features = ["v4"] }`.

- [ ] **Step 2: Testovi (crveni).** `crates/app/tests/views.rs`:

```rust
use app::memory::{Memory, deps};
use app::*;
use content::{Snapshot, repo_dir};
use std::sync::Arc;

fn setup() -> (Deps, Arc<Memory>) {
    deps(Snapshot::read(&repo_dir()).unwrap())
}

async fn guest(d: &Deps) -> (Caller, DemoStarted) {
    let s = start_demo(d, None).await.unwrap();
    let c = caller(d, s.new_token.as_deref()).await.unwrap();
    (c, s)
}

fn robot_hotel(mem: &Memory, game: GameId) -> HotelId {
    let created = &mem.events(game)[0];
    HotelId(created.payload["hotels"][1]["hotel_id"].as_str().unwrap().parse().unwrap())
}

#[tokio::test]
async fn start_demo_creates_guest_then_reuses_it() {
    let (d, _) = setup();
    let first = start_demo(&d, None).await.unwrap();
    let tok = first.new_token.clone().expect("novi gost dobiva token");
    assert_eq!(tok.len(), 32);
    let second = start_demo(&d, Some(&tok)).await.unwrap();
    assert_eq!(second.new_token, None, "postojeći gost ne dobiva novi kolačić");
    assert_ne!(second.game_id, first.game_id);
    let c = caller(&d, Some(&tok)).await.unwrap();
    assert_eq!(my_hotel(&d, &c).await.unwrap().game_id, second.game_id, "moj hotel = zadnja igra");
    let unknown = start_demo(&d, Some("0123456789abcdef0123456789abcdef")).await.unwrap();
    assert!(unknown.new_token.is_some(), "nepoznat token = novi gost");
}

#[tokio::test]
async fn visibility_by_role() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let (other, _) = guest(&d).await;
    let anon = Caller { guest: None };
    let own = hotel_view(&d, &me, s.hotel_id).await.unwrap();
    assert!(own.private.is_some(), "vlasnik vidi stvarne brojke");
    let robot = hotel_view(&d, &me, robot_hotel(&mem, s.game_id)).await.unwrap();
    assert!(robot.private.is_none(), "tuđi hotel: samo ocjena i indeksi (ARHITEKTURA §12)");
    assert_eq!(hotel_view(&d, &other, s.hotel_id).await.unwrap_err().code, ErrorCode::NotInGame);
    assert_eq!(hotel_view(&d, &anon, s.hotel_id).await.unwrap_err().code, ErrorCode::GuestRequired);
    assert_eq!(game_view(&d, &other, s.game_id).await.unwrap_err().code, ErrorCode::NotInGame);
    assert_eq!(my_hotel(&d, &anon).await.unwrap_err().code, ErrorCode::GuestRequired);
}

#[tokio::test]
async fn game_view_lists_four_hotels_with_open_june() {
    let (d, _) = setup();
    let (me, s) = guest(&d).await;
    let g = game_view(&d, &me, s.game_id).await.unwrap();
    assert_eq!(g.hotels.len(), 4);
    assert_eq!(g.my_hotel, s.hotel_id);
    let r = g.current.unwrap();
    assert_eq!((r.month, r.status, r.unanswered), (game::dto::MonthCode::Jun, game::game::RoundStatus::Open, 2));
}

#[tokio::test]
async fn situations_and_feed_are_owner_only() {
    let (d, mem) = setup();
    let (me, s) = guest(&d).await;
    let v = situations(&d, &me, s.hotel_id).await.unwrap();
    assert!(v.open);
    assert_eq!(v.items.iter().map(|i| i.code.as_str()).collect::<Vec<_>>(), ["S1", "S3"]);
    assert!(v.items.iter().all(|i| i.answers.len() == 4 && i.chosen.is_none()));
    let robot = robot_hotel(&mem, s.game_id);
    assert_eq!(situations(&d, &me, robot).await.unwrap_err().code, ErrorCode::NotYourHotel);
    assert_eq!(feed(&d, &me, robot).await.unwrap_err().code, ErrorCode::NotYourHotel);
    assert!(!feed(&d, &me, s.hotel_id).await.unwrap().is_empty());
}

#[tokio::test]
async fn unknown_ids_and_profiles() {
    let (d, _) = setup();
    let (me, _) = guest(&d).await;
    let nobody = uuid::Uuid::from_u128(7);
    assert_eq!(hotel_view(&d, &me, HotelId(nobody)).await.unwrap_err().code, ErrorCode::UnknownHotel);
    assert_eq!(game_view(&d, &me, GameId(nobody)).await.unwrap_err().code, ErrorCode::UnknownGame);
    assert_eq!(profile_indices("C").unwrap(), [5000; 5]);
    assert_eq!(profile_indices("D").unwrap_err().code, ErrorCode::UnknownProfile);
}
```
  (`app` dev-ovisnosti: `tokio.workspace = true` postoji; dodaj `uuid.workspace = true` i `game.workspace = true`
  ako ih testovi ne vide kroz re-export.)

- [ ] **Step 3: Pokreni** `cargo test -p app --test views; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 4: `ports.rs` i `error.rs`.**

```rust
// ports.rs — sučelja prema vanjskom svijetu (ARHITEKTURA §6.2); izvodi ih `store`, u testovima `memory`.
use crate::{AppError, BoxFut};
use game::{GameId, GuestId, HotelId};

// RawEvent, NewEvent, NewJob, Job, NewHotel, NewGame, Decision, Decided — polja točno kao u Interfaces,
// derive `Clone, Debug, PartialEq` (Decision još i `Default`).

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
    fn decide<'a>(&'a self, game: GameId, decide: Decide<'a>) -> BoxFut<'a, Result<Decided, DecideError>>;
}

pub trait Guests: Send + Sync {
    fn create_guest(&self, id: GuestId, token_hash: String) -> BoxFut<'_, Result<(), StoreError>>;
    fn guest_by_token_hash(&self, token_hash: String) -> BoxFut<'_, Result<Option<GuestId>, StoreError>>;
    fn game_of_hotel(&self, hotel: HotelId) -> BoxFut<'_, Result<Option<GameId>, StoreError>>;
    /// Hotel gosta u njegovoj zadnjoj igri.
    fn latest_hotel_of(&self, guest: GuestId) -> BoxFut<'_, Result<Option<(GameId, HotelId)>, StoreError>>;
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
```

```rust
// error.rs
use game::GameError;
use sim_hotel::PlanError;

use crate::ports::{DecideError, StoreError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorCode {
    GuestRequired,
    NotInGame,
    NotYourHotel,
    UnknownGame,
    UnknownHotel,
    UnknownMonth,
    UnknownSituation,
    UnknownAnswer,
    UnknownProfile,
    RoundNotOpen,
    AlreadyAnswered,
    PriceNotWholeEuro,
    PriceOutOfRange,
    MarketingOutOfRange,
    SeasonalOutOfRange,
    SeasonalStepTooBig,
    WageOutOfRange,
    RenovationTwice,
    StorageUnavailable,
    Internal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind { Unauthorized, Forbidden, NotFound, Conflict, Invalid, Unavailable, Internal }

impl ErrorCode {
    pub const ALL: [ErrorCode; 20] = [/* svih 20 varijanti redom kako su navedene */];

    pub fn kind(self) -> ErrorKind {
        use ErrorCode::*;
        match self {
            GuestRequired => ErrorKind::Unauthorized,
            NotInGame | NotYourHotel => ErrorKind::Forbidden,
            UnknownGame | UnknownHotel | UnknownMonth | UnknownSituation | UnknownProfile => ErrorKind::NotFound,
            RoundNotOpen | AlreadyAnswered => ErrorKind::Conflict,
            UnknownAnswer | PriceNotWholeEuro | PriceOutOfRange | MarketingOutOfRange | SeasonalOutOfRange
            | SeasonalStepTooBig | WageOutOfRange | RenovationTwice => ErrorKind::Invalid,
            StorageUnavailable => ErrorKind::Unavailable,
            Internal => ErrorKind::Internal,
        }
    }

    /// Stabilan kod za API (snake_case); ekran ga prevodi.
    pub fn as_str(self) -> &'static str { /* iscrpni match: "guest_required", "not_in_game", … */ }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{code:?}: {detail}")]
pub struct AppError {
    pub code: ErrorCode,
    /// Za dnevnik poslužitelja; ne šalje se klijentu.
    pub detail: String,
}

impl AppError {
    pub fn new(code: ErrorCode) -> Self { AppError { code, detail: String::new() } }
    pub fn internal(detail: impl Into<String>) -> Self { AppError { code: ErrorCode::Internal, detail: detail.into() } }
}

impl From<GameError> for AppError {
    fn from(e: GameError) -> Self {
        use ErrorCode as C;
        let code = match &e {
            GameError::UnknownHotel => C::UnknownHotel,
            GameError::UnknownMonth => C::UnknownMonth,
            GameError::UnknownSituation => C::UnknownSituation,
            GameError::UnknownAnswer => C::UnknownAnswer,
            GameError::RoundNotOpen => C::RoundNotOpen,
            GameError::AlreadyAnswered => C::AlreadyAnswered,
            GameError::Plan(p) => match p {
                PlanError::PriceNotWholeEuro => C::PriceNotWholeEuro,
                PlanError::PriceOutOfRange => C::PriceOutOfRange,
                PlanError::MarketingOutOfRange => C::MarketingOutOfRange,
                PlanError::SeasonalOutOfRange => C::SeasonalOutOfRange,
                PlanError::SeasonalStepTooBig => C::SeasonalStepTooBig,
                PlanError::WageOutOfRange => C::WageOutOfRange,
                PlanError::RenovationTwice => C::RenovationTwice,
                PlanError::Calc => C::Internal,
            },
            GameError::Corrupt(_) => C::Internal,
        };
        AppError { code, detail: format!("{e:?}") }
    }
}

impl From<StoreError> for AppError {
    fn from(e: StoreError) -> Self { AppError { code: ErrorCode::StorageUnavailable, detail: e.0 } }
}

impl From<DecideError> for AppError {
    fn from(e: DecideError) -> Self {
        match e { DecideError::Rejected(a) => a, DecideError::Store(s) => s.into() }
    }
}
```

- [ ] **Step 5: `lib.rs`.** Iza postojećeg koda zdravlja (`BoxFut`, `StatusError`, `SystemStatus`, `check_health`
  ostaju gdje jesu):

```rust
pub mod demo;
pub mod error;
pub mod memory;
pub mod ports;
pub mod views;

pub use demo::*;
pub use error::*;
pub use game::{GameId, GuestId, HotelId};
pub use ports::*;
pub use views::*;

use std::sync::Arc;

/// Sve što slučajevi upotrebe trebaju; sastavlja ga `server` (Postgres) ili `memory::deps` (testovi).
#[derive(Clone)]
pub struct Deps {
    pub status: Arc<dyn SystemStatus>,
    pub log: Arc<dyn EventLog>,
    pub guests: Arc<dyn Guests>,
    pub jobs: Arc<dyn JobQueue>,
    pub ids: Arc<dyn Ids>,
    pub snapshot: Arc<content::Snapshot>,
}
```

- [ ] **Step 6: `demo.rs`.**

```rust
use game::command::new_game;
use game::game::Game;
use game::{Actor, GameEvent, GuestId, HotelId, GameId, Recorded, SCHEMA_VER};
use sha2::{Digest, Sha256};
use sim_hotel::{Index, Profile};

use crate::{AppError, Deps, ErrorCode, NewEvent, NewGame, NewHotel, RawEvent};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Caller { pub guest: Option<GuestId> }

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
    let Some(t) = token else { return Ok(Caller { guest: None }) };
    Ok(Caller { guest: deps.guests.guest_by_token_hash(token_hash(t)).await? })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DemoStarted { pub game_id: GameId, pub hotel_id: HotelId, pub new_token: Option<String> }

/// `POST /v1/demo` (DEMO §1.4): gost (postojeći ili novi) i nova igra s tri robota u jednoj transakciji.
pub async fn start_demo(deps: &Deps, token: Option<&str>) -> Result<DemoStarted, AppError> {
    let (guest, new_token) = match caller(deps, token).await?.guest {
        Some(g) => (g, None),
        None => {
            let id = GuestId(deps.ids.uuid());
            let tok = deps.ids.uuid().simple().to_string();
            deps.guests.create_guest(id, token_hash(&tok)).await?;
            (id, Some(tok))
        }
    };
    let game_id = GameId(deps.ids.uuid());
    let hotel_ids = [(); 4].map(|_| HotelId(deps.ids.uuid()));
    let events = new_game(game_id, deps.ids.seed(), &deps.snapshot, guest, hotel_ids)?;
    let hotels = hotel_ids
        .iter()
        .zip(1_i16..)
        .map(|(&hotel_id, slot)| NewHotel { hotel_id, slot, guest_id: (slot == 1).then_some(guest) })
        .collect();
    deps.log
        .create_game(NewGame {
            game_id,
            created_by: guest,
            hotels,
            events: events.iter().map(|(a, e)| new_event(a, e)).collect(),
        })
        .await?;
    Ok(DemoStarted { game_id, hotel_id: hotel_ids[0], new_token })
}

/// Početni indeksi profila (DEMO §5) — prije u `api/src/demo.rs` mimo `app` (BACKLOG [dnevnik]).
pub fn profile_indices(code: &str) -> Result<[u16; 5], AppError> {
    let p = Profile::from_code(code).ok_or(AppError::new(ErrorCode::UnknownProfile))?;
    let s = p.starting_indices();
    Ok(Index::ALL.map(|i| s.get(i).hundredths()))
}

pub(crate) fn new_event(actor: &Actor, ev: &GameEvent) -> NewEvent {
    NewEvent { kind: ev.kind().to_owned(), schema_ver: SCHEMA_VER, payload: ev.to_payload(), actor: actor.as_db() }
}

pub(crate) fn fold_raw(raw: &[RawEvent]) -> Result<Game, AppError> {
    if raw.is_empty() {
        return Err(AppError::new(ErrorCode::UnknownGame));
    }
    let records = raw
        .iter()
        .map(|r| {
            let actor = Actor::parse(&r.actor).ok_or_else(|| AppError::internal(format!("actor {}", r.actor)))?;
            let event = GameEvent::from_record(&r.kind, r.schema_ver, r.payload.clone())
                .map_err(|e| AppError::internal(format!("{e:?}")))?;
            Ok(Recorded { seq: r.seq, actor, event })
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    Ok(Game::fold(&records)?)
}

pub(crate) async fn load_game(deps: &Deps, id: GameId) -> Result<Game, AppError> {
    fold_raw(&deps.log.load(id).await?)
}
```

- [ ] **Step 7: `views.rs`.** Strukture iz Interfaces (derive `Clone, Debug, PartialEq, Eq`), pomoćnici i slučajevi:

```rust
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
    if mine == hotel { Ok(()) } else { Err(AppError::new(ErrorCode::NotYourHotel)) }
}

pub(crate) fn round_view(g: &Game, m: Month, mine: HotelId) -> Result<RoundView, AppError> {
    let status = g.status(m)?;
    let unanswered = if status == RoundStatus::Open { g.unanswered_count(mine, m)? } else { 0 };
    Ok(RoundView { month: m.into(), status, unanswered })
}

async fn game_of_hotel(deps: &Deps, hotel: HotelId) -> Result<GameId, AppError> {
    deps.guests.game_of_hotel(hotel).await?.ok_or(AppError::new(ErrorCode::UnknownHotel))
}

pub async fn hotel_view(deps: &Deps, caller: &Caller, hotel: HotelId) -> Result<HotelView, AppError> {
    let guest = caller.require()?;
    let game_id = game_of_hotel(deps, hotel).await?;
    let g = load_game(deps, game_id).await?;
    let mine = member_hotel(&g, guest)?;
    let public = g.public_hotel(hotel)?;
    let private = if mine == hotel { Some(g.private_hotel(hotel)?) } else { None };
    Ok(HotelView { game_id, public, private })
}
```
  Ostali slučajevi, istim redom provjera (**gost → igra postoji → član → vlasnik**):
  - `my_hotel(deps, caller) -> Result<HotelView, AppError>`: `latest_hotel_of(guest)`; nema igre → `UnknownGame`;
    zatim kao `hotel_view`.
  - `game_view(deps, caller, game_id) -> Result<GameView, AppError>`: `load_game` (prazno → `UnknownGame`),
    `member_hotel`, `hotels` = `public_hotel` za svaki hotel po slotu, `current` = `round_view(current_month)` ili `None`.
  - `situations(deps, caller, hotel) -> Result<SituationsView, AppError>`: `require_owner`; `month = current_month()`;
    `open = status == Open`; stavke za `situations_in(month)`: `answers` su samo `label`-i (indeks = položaj), `chosen`
    iz `answers` hotela. **Kvaliteta i mehanike se ne kopiraju.** Igra gotova → `month: None`, prazno.
  - `feed(deps, caller, hotel) -> Result<Vec<FeedItem>, AppError>`: `require_owner`, pa `g.feed(hotel)`.

- [ ] **Step 8: `memory.rs`.** Ista pravila kao `store`, u `Mutex<Mem>`; zaključavanje se nikad ne drži preko `await`
  (svaka metoda izračuna rezultat sinkrono pa vrati `Box::pin(async move { r })`).

```rust
//! Pohrana u memoriji za testove `app` i `api`: ista pravila kao `store`, bez baze i bez sata.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobStatus { Queued, Running, Done, Failed }

#[derive(Clone, Debug, PartialEq)]
pub struct MemJob { pub id: i64, pub key: String, pub kind: String, pub game_id: GameId,
                    pub payload: serde_json::Value, pub status: JobStatus, pub attempts: i32 }

#[derive(Default)]
struct Mem {
    events: BTreeMap<GameId, Vec<RawEvent>>,
    guests: BTreeMap<String, GuestId>,
    hotels: BTreeMap<HotelId, (GameId, Option<GuestId>)>,
    games: Vec<(GameId, GuestId)>, // redom stvaranja
    jobs: Vec<MemJob>,
}

#[derive(Default)]
pub struct Memory { inner: Mutex<Mem> }

/// Predvidljivi identifikatori: uuid 1, 2, 3 …; sjeme uvijek 42.
#[derive(Default)]
pub struct SeqIds(AtomicU64);
impl Ids for SeqIds {
    fn uuid(&self) -> uuid::Uuid { uuid::Uuid::from_u128(u128::from(self.0.fetch_add(1, Ordering::SeqCst) + 1)) }
    fn seed(&self) -> u64 { 42 }
}

pub fn deps(snapshot: content::Snapshot) -> (Deps, Arc<Memory>) {
    let m = Arc::new(Memory::default());
    let d = Deps { status: m.clone(), log: m.clone(), guests: m.clone(), jobs: m.clone(),
                   ids: Arc::new(SeqIds::default()), snapshot: Arc::new(snapshot) };
    (d, m)
}
```
  Pravila izvedbe (moraju odgovarati `store`-u, testovi Taska 6 i 9–10 ovise o njima):
  - `create_game`: upiši događaje s `seq` 1..=n, hotele u `hotels`, `(game, created_by)` na kraj `games`.
  - `decide`: kloniraj tok, pozovi `decide`; `Err` → `DecideError::Rejected` bez promjene; `Ok` → dodaj događaje s
    `seq = last + 1 …`, `enqueue` dodaj samo ako posao s istim `key` ne postoji (status `Queued`, `attempts` 0, `id` =
    broj poslova + 1), `finish_job` → `Done`. Vrati `Decided { last_seq, appended }`.
  - `claim`: prvi posao po `id` sa statusom `Queued` i `attempts < MAX_ATTEMPTS` → `Running`, `attempts += 1`, vrati
    ga (zakup u memoriji nikad ne istječe sam — vidi `expire_leases`).
  - `fail`: `attempts >= MAX_ATTEMPTS` → `Failed`, inače `Queued`.
  - `latest_hotel_of`: zadnja igra gosta u `games`, pa hotel s tim gostom u toj igri.
  - `SystemStatus::events_count` = ukupan broj događaja.
  - Pomoćnici za testove: `jobs() -> Vec<MemJob>`, `events(GameId) -> Vec<RawEvent>`, `expire_leases()` (sve
    `Running` → `Queued`: kao da je radnik pao i zakup istekao), `push_job(NewJob)` (doda posao **bez** provjere ključa:
    simulira dvostruku isporuku).

- [ ] **Step 9: Pokreni** `cargo test -p app; echo EXIT=$?` → `EXIT=0` (i stari testovi zdravlja).

- [ ] **Step 10: Provjere i commit** — `crates/app`, `Cargo.lock`, poruka
  „Dnevnik 5/12: app — portovi, greške, lažna pohrana, gost i pogledi po ulozi".

---

