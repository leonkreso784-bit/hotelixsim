//! PostgreSQL: dnevnik događaja; kasnije projekcije i red poslova (ARHITEKTURA §9).

use std::time::Duration;

use app::{BoxFut, StatusError, SystemStatus};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

#[derive(Clone)]
pub struct Store {
    pool: PgPool,
}

impl Store {
    /// Ne spaja se odmah: server se pokrene i kad baza nije dostupna, a zdravlje to javi (503).
    pub fn connect_lazy(url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(3))
            .connect_lazy(url)?;
        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await
    }

    pub async fn count_events(&self) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM events"#)
            .fetch_one(&self.pool)
            .await
    }
}

impl SystemStatus for Store {
    fn events_count(&self) -> BoxFut<'_, Result<i64, StatusError>> {
        Box::pin(async move {
            self.count_events().await.map_err(|e| {
                tracing::warn!(error = %e, "provjera zdravlja: upit na bazu nije uspio");
                StatusError(e.to_string())
            })
        })
    }
}
