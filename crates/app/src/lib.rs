//! Slučajevi upotrebe i portovi (ARHITEKTURA §6.2). Ne zna za Postgres ni HTTP.

use std::future::Future;
use std::pin::Pin;

pub type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, thiserror::Error)]
#[error("baza nije dostupna: {0}")]
pub struct StatusError(pub String);

/// Port prema pohrani za provjeru zdravlja sustava.
pub trait SystemStatus: Send + Sync {
    fn events_count(&self) -> BoxFut<'_, Result<i64, StatusError>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DbState {
    Ok,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Health {
    pub database: DbState,
    pub events: Option<i64>,
}

pub async fn check_health(status: &dyn SystemStatus) -> Health {
    match status.events_count().await {
        Ok(n) => Health {
            database: DbState::Ok,
            events: Some(n),
        },
        Err(_) => Health {
            database: DbState::Error,
            events: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake(Result<i64, String>);

    impl SystemStatus for Fake {
        fn events_count(&self) -> BoxFut<'_, Result<i64, StatusError>> {
            let r = self.0.clone().map_err(StatusError);
            Box::pin(async move { r })
        }
    }

    #[tokio::test]
    async fn healthy_database_reports_count() {
        let h = check_health(&Fake(Ok(3))).await;
        assert_eq!(
            h,
            Health {
                database: DbState::Ok,
                events: Some(3)
            }
        );
    }

    #[tokio::test]
    async fn broken_database_reports_error_without_count() {
        let h = check_health(&Fake(Err("veza odbijena".into()))).await;
        assert_eq!(
            h,
            Health {
                database: DbState::Error,
                events: None
            }
        );
    }
}
