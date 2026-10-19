//! Slučajevi upotrebe i portovi (ARHITEKTURA §6.2). Ne zna za Postgres ni HTTP.

pub mod commands;
pub mod demo;
pub mod error;
pub mod jobs;
pub mod look;
pub mod memory;
pub mod ports;
pub mod spend;
pub mod talk;
pub mod views;

pub use commands::*;
pub use demo::*;
pub use error::*;
pub use game::{GameId, GuestId, HotelId};
pub use jobs::*;
pub use look::*;
pub use ports::*;
pub use talk::*;
pub use views::*;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Sve što slučajevi upotrebe trebaju; sastavlja ga `server` (Postgres) ili `memory::deps` (testovi).
#[derive(Clone)]
pub struct Deps {
    pub status: Arc<dyn SystemStatus>,
    pub log: Arc<dyn EventLog>,
    pub guests: Arc<dyn Guests>,
    pub jobs: Arc<dyn JobQueue>,
    pub ai: Arc<dyn AiPort>,
    pub ids: Arc<dyn Ids>,
    pub snapshot: Arc<content::Snapshot>,
    /// Pravila modela v2 (`Snapshot::read_v2`); nova igra iz `NewGameSettings { model: V2, .. }` ide iz njih.
    pub snapshot_v2: Arc<content::Snapshot>,
}

pub type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, thiserror::Error)]
#[error("baza nije dostupna: {0}")]
pub struct StatusError(pub String);

/// Port prema pohrani za provjeru zdravlja sustava.
pub trait SystemStatus: Send + Sync {
    /// Je li pohrana živa (`SELECT 1`); ne broji ništa i ne dira tablice.
    fn ping(&self) -> BoxFut<'_, Result<(), StatusError>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DbState {
    Ok,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Health {
    pub database: DbState,
}

pub async fn check_health(status: &dyn SystemStatus) -> Health {
    match status.ping().await {
        Ok(()) => Health {
            database: DbState::Ok,
        },
        Err(_) => Health {
            database: DbState::Error,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake(Result<(), String>);

    impl SystemStatus for Fake {
        fn ping(&self) -> BoxFut<'_, Result<(), StatusError>> {
            let r = self.0.clone().map_err(StatusError);
            Box::pin(async move { r })
        }
    }

    #[tokio::test]
    async fn healthy_database_reports_ok() {
        let h = check_health(&Fake(Ok(()))).await;
        assert_eq!(
            h,
            Health {
                database: DbState::Ok
            }
        );
    }

    #[tokio::test]
    async fn broken_database_reports_error() {
        let h = check_health(&Fake(Err("veza odbijena".into()))).await;
        assert_eq!(
            h,
            Health {
                database: DbState::Error
            }
        );
    }
}
