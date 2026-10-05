//! Pohrana u memoriji za testove `app` i `api`: ista pravila kao `store`, bez baze i bez sata.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::{
    BoxFut, Decide, DecideError, Decided, Deps, EventLog, GameId, Guests, HotelId, Ids, Job,
    JobQueue, MAX_ATTEMPTS, NewGame, NewJob, RawEvent, StatusError, StoreError, SystemStatus,
};
use game::GuestId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Running,
    Done,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MemJob {
    pub id: i64,
    pub key: String,
    pub kind: String,
    pub game_id: GameId,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub attempts: i32,
}

#[derive(Default)]
struct Mem {
    events: BTreeMap<GameId, Vec<RawEvent>>,
    guests: BTreeMap<String, GuestId>,
    hotels: BTreeMap<HotelId, (GameId, Option<GuestId>)>,
    /// Redom stvaranja.
    games: Vec<(GameId, GuestId)>,
    jobs: Vec<MemJob>,
}

#[derive(Default)]
pub struct Memory {
    inner: Mutex<Mem>,
}

/// Predvidljivi identifikatori: uuid 1, 2, 3 …; sjeme uvijek 42.
#[derive(Default)]
pub struct SeqIds(AtomicU64);

impl Ids for SeqIds {
    fn uuid(&self) -> uuid::Uuid {
        uuid::Uuid::from_u128(u128::from(self.0.fetch_add(1, Ordering::SeqCst) + 1))
    }
    fn seed(&self) -> u64 {
        42
    }
}

pub fn deps(snapshot: content::Snapshot) -> (Deps, Arc<Memory>) {
    let m = Arc::new(Memory::default());
    let d = Deps {
        status: m.clone(),
        log: m.clone(),
        guests: m.clone(),
        jobs: m.clone(),
        ids: Arc::new(SeqIds::default()),
        snapshot: Arc::new(snapshot),
    };
    (d, m)
}

fn ready<'a, T: Send + 'a>(v: T) -> BoxFut<'a, T> {
    Box::pin(async move { v })
}

fn push_job_to(mem: &mut Mem, j: NewJob) {
    let id = i64::try_from(mem.jobs.len()).unwrap_or(i64::MAX - 1) + 1;
    mem.jobs.push(MemJob {
        id,
        key: j.key,
        kind: j.kind,
        game_id: j.game_id,
        payload: j.payload,
        status: JobStatus::Queued,
        attempts: 0,
    });
}

impl Memory {
    fn lock(&self) -> MutexGuard<'_, Mem> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn jobs(&self) -> Vec<MemJob> {
        self.lock().jobs.clone()
    }

    pub fn events(&self, game: GameId) -> Vec<RawEvent> {
        self.lock().events.get(&game).cloned().unwrap_or_default()
    }

    /// Kao da je radnik pao i zakup istekao: svi poslovi `Running` vraćaju se u red.
    pub fn expire_leases(&self) {
        for j in &mut self.lock().jobs {
            if j.status == JobStatus::Running {
                j.status = JobStatus::Queued;
            }
        }
    }

    /// Doda posao BEZ provjere ključa: simulira dvostruku isporuku.
    pub fn push_job(&self, j: NewJob) {
        push_job_to(&mut self.lock(), j);
    }
}

impl SystemStatus for Memory {
    fn events_count(&self) -> BoxFut<'_, Result<i64, StatusError>> {
        let n: usize = self.lock().events.values().map(Vec::len).sum();
        ready(Ok(i64::try_from(n).unwrap_or(i64::MAX)))
    }
}

impl EventLog for Memory {
    fn create_game(&self, game: NewGame) -> BoxFut<'_, Result<(), StoreError>> {
        let mut m = self.lock();
        let raw = game
            .events
            .into_iter()
            .zip(1_i64..)
            .map(|(e, seq)| RawEvent {
                seq,
                kind: e.kind,
                schema_ver: e.schema_ver,
                payload: e.payload,
                actor: e.actor,
            })
            .collect();
        m.events.insert(game.game_id, raw);
        for h in game.hotels {
            m.hotels.insert(h.hotel_id, (game.game_id, h.guest_id));
        }
        m.games.push((game.game_id, game.created_by));
        ready(Ok(()))
    }

    fn load(&self, game: GameId) -> BoxFut<'_, Result<Vec<RawEvent>, StoreError>> {
        ready(Ok(self.events(game)))
    }

    fn decide<'a>(
        &'a self,
        game: GameId,
        decide: Decide<'a>,
    ) -> BoxFut<'a, Result<Decided, DecideError>> {
        let mut m = self.lock();
        let stream = m.events.get(&game).cloned().unwrap_or_default();
        let decision = match decide(&stream) {
            Ok(d) => d,
            Err(e) => return ready(Err(DecideError::Rejected(e))),
        };
        let mut last = stream.last().map_or(0, |e| e.seq);
        let appended = decision.events.len();
        let tail = m.events.entry(game).or_default();
        for e in decision.events {
            last += 1;
            tail.push(RawEvent {
                seq: last,
                kind: e.kind,
                schema_ver: e.schema_ver,
                payload: e.payload,
                actor: e.actor,
            });
        }
        if let Some(j) = decision.enqueue
            && !m.jobs.iter().any(|x| x.key == j.key)
        {
            push_job_to(&mut m, j);
        }
        if let Some(id) = decision.finish_job
            && let Some(j) = m.jobs.iter_mut().find(|j| j.id == id)
        {
            j.status = JobStatus::Done;
        }
        ready(Ok(Decided {
            last_seq: last,
            appended,
        }))
    }
}

impl Guests for Memory {
    fn create_guest(&self, id: GuestId, token_hash: String) -> BoxFut<'_, Result<(), StoreError>> {
        self.lock().guests.insert(token_hash, id);
        ready(Ok(()))
    }

    fn guest_by_token_hash(
        &self,
        token_hash: String,
    ) -> BoxFut<'_, Result<Option<GuestId>, StoreError>> {
        ready(Ok(self.lock().guests.get(&token_hash).copied()))
    }

    fn game_of_hotel(&self, hotel: HotelId) -> BoxFut<'_, Result<Option<GameId>, StoreError>> {
        ready(Ok(self.lock().hotels.get(&hotel).map(|(g, _)| *g)))
    }

    fn latest_hotel_of(
        &self,
        guest: GuestId,
    ) -> BoxFut<'_, Result<Option<(GameId, HotelId)>, StoreError>> {
        let m = self.lock();
        let found = m
            .games
            .iter()
            .rev()
            .find(|(_, by)| *by == guest)
            .and_then(|(game, _)| {
                m.hotels
                    .iter()
                    .find(|(_, (g, owner))| g == game && *owner == Some(guest))
                    .map(|(h, _)| (*game, *h))
            });
        ready(Ok(found))
    }
}

impl JobQueue for Memory {
    fn claim(&self, _lease_secs: i32) -> BoxFut<'_, Result<Option<Job>, StoreError>> {
        let mut m = self.lock();
        let job = m
            .jobs
            .iter_mut()
            .find(|j| j.status == JobStatus::Queued && j.attempts < MAX_ATTEMPTS)
            .map(|j| {
                j.status = JobStatus::Running;
                j.attempts += 1;
                Job {
                    id: j.id,
                    key: j.key.clone(),
                    kind: j.kind.clone(),
                    game_id: j.game_id,
                    payload: j.payload.clone(),
                    attempts: j.attempts,
                }
            });
        ready(Ok(job))
    }

    fn fail(&self, id: i64, _error: String) -> BoxFut<'_, Result<(), StoreError>> {
        if let Some(j) = self.lock().jobs.iter_mut().find(|j| j.id == id) {
            j.status = if j.attempts >= MAX_ATTEMPTS {
                JobStatus::Failed
            } else {
                JobStatus::Queued
            };
        }
        ready(Ok(()))
    }
}
