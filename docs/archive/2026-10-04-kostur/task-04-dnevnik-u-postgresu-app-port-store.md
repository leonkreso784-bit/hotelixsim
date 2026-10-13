### Task 4: Dnevnik u Postgresu (`app` port + `store`)

Preduvjet: Task 0 gotov (`.env` s `DATABASE_URL`, baza `hotelixsim`, uloga s `CREATEDB`).

**Files:**
- Create: `crates/app/Cargo.toml`, `crates/app/src/lib.rs`
- Create: `crates/store/Cargo.toml`, `crates/store/migrations/0001_events.sql`, `crates/store/src/lib.rs`, `crates/store/tests/events.rs`
- Create (generirano): `crates/store/.sqlx/`
- Modify: `Cargo.toml` (members)

**Interfaces:**
- Produces:
  - `app::BoxFut<'a, T>` = `Pin<Box<dyn Future<Output = T> + Send + 'a>>`
  - `app::StatusError(pub String)` (thiserror, poruka „baza nije dostupna: …")
  - `trait app::SystemStatus: Send + Sync { fn events_count(&self) -> BoxFut<'_, Result<i64, StatusError>>; }`
  - `app::Health { pub database: DbState, pub events: Option<i64> }`, `enum app::DbState { Ok, Error }`, `async fn app::check_health(status: &dyn SystemStatus) -> Health`
  - `store::Store` — `Store::connect_lazy(url: &str) -> Result<Store, sqlx::Error>`, `Store::from_pool(PgPool) -> Store`, `store.migrate().await -> Result<(), sqlx::migrate::MigrateError>`, `store.count_events().await -> Result<i64, sqlx::Error>`; `impl app::SystemStatus for Store`.

- [ ] **Step 1: Alat `sqlx-cli` (jednom, traje nekoliko minuta)**

Run: `cargo install sqlx-cli --version 0.9.0 --no-default-features --features postgres,rustls --locked; echo EXIT=$?` → `EXIT=0`
Run: `cargo sqlx --version` → `sqlx-cli 0.9.0`
Ako značajka `rustls` ne postoji u 0.9.0, ponovi bez nje (`--features postgres`); lokalna baza ne traži TLS.

- [ ] **Step 2: Paket `app`**

`crates/app/Cargo.toml`:

```toml
[package]
name = "app"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
thiserror.workspace = true

[dev-dependencies]
tokio.workspace = true

[lints]
workspace = true
```

`crates/app/src/lib.rs`:

```rust
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
        Ok(n) => Health { database: DbState::Ok, events: Some(n) },
        Err(_) => Health { database: DbState::Error, events: None },
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
        assert_eq!(h, Health { database: DbState::Ok, events: Some(3) });
    }

    #[tokio::test]
    async fn broken_database_reports_error_without_count() {
        let h = check_health(&Fake(Err("veza odbijena".into()))).await;
        assert_eq!(h, Health { database: DbState::Error, events: None });
    }
}
```

Dodaj `"crates/app"` i `"crates/store"` u `members` korijenskog `Cargo.toml`.

Run: `cargo test -p app; echo EXIT=$?` → `2 passed`, `EXIT=0`.

- [ ] **Step 3: Migracija `crates/store/migrations/0001_events.sql`** (oblik iz ARHITEKTURA §9.1)

```sql
-- Dnevnik događaja: izvor istine (ARHITEKTURA §9.1). Samo dopisivanje.
-- Zabrana UPDATE/DELETE preko zasebne uloge dolazi u planu „Dnevnik, runde i red poslova".
CREATE TABLE events (
    id           BIGSERIAL PRIMARY KEY,
    game_id      UUID        NOT NULL,
    stream       TEXT        NOT NULL,
    stream_seq   INTEGER     NOT NULL,
    type         TEXT        NOT NULL,
    schema_ver   INTEGER     NOT NULL,
    payload      JSONB       NOT NULL,
    actor        TEXT        NOT NULL,
    recorded_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (stream, stream_seq)
);
```

- [ ] **Step 4: Test koji pada — `crates/store/tests/events.rs`**

```rust
use sqlx::PgPool;
use store::Store;

#[sqlx::test]
async fn fresh_database_has_no_events(pool: PgPool) {
    assert_eq!(Store::from_pool(pool).count_events().await.unwrap(), 0);
}

#[sqlx::test]
async fn counts_inserted_event(pool: PgPool) {
    sqlx::query(
        "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
         VALUES ('00000000-0000-0000-0000-000000000001', 'game:1', 1, 'GameCreated', 1, '{}'::jsonb, 'system')",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(Store::from_pool(pool).count_events().await.unwrap(), 1);
}

#[sqlx::test]
async fn stream_sequence_is_unique(pool: PgPool) {
    let insert = "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
                  VALUES ('00000000-0000-0000-0000-000000000001', 'game:1', 1, 'X', 1, '{}'::jsonb, 'system')";
    sqlx::query(insert).execute(&pool).await.unwrap();
    assert!(sqlx::query(insert).execute(&pool).await.is_err(), "isti (stream, stream_seq) ne smije proći dvaput");
}
```

`#[sqlx::test]` sam primijeni migracije iz `./migrations` paketa u svježu privremenu bazu.

- [ ] **Step 5: Paket `store`**

`crates/store/Cargo.toml`:

```toml
[package]
name = "store"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
app.workspace = true
sqlx.workspace = true

[dev-dependencies]
tokio.workspace = true

[lints]
workspace = true
```

`crates/store/src/lib.rs`:

```rust
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
        let pool = PgPoolOptions::new().acquire_timeout(Duration::from_secs(3)).connect_lazy(url)?;
        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("./migrations").run(&self.pool).await
    }

    pub async fn count_events(&self) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM events"#).fetch_one(&self.pool).await
    }
}

impl SystemStatus for Store {
    fn events_count(&self) -> BoxFut<'_, Result<i64, StatusError>> {
        Box::pin(async move { self.count_events().await.map_err(|e| StatusError(e.to_string())) })
    }
}
```

- [ ] **Step 6: Migriraj razvojnu bazu i pripremi offline podatke**

Run: `cd crates/store && cargo sqlx migrate run; echo EXIT=$?` (čita `DATABASE_URL` iz korijenskog `.env`; ako ga ne nađe, pokreni s `--database-url "postgres://hotelixsim:hotelixsim_lokalno@localhost:5432/hotelixsim"`) → `Applied 1/migrate events`, `EXIT=0`
Run (iz korijena): `cargo sqlx prepare --workspace; echo EXIT=$?` → `query data written to .sqlx`, `EXIT=0`. Zapiši gdje je mapa `.sqlx` nastala (u korijenu workspacea ili u `crates/store`) i commitaj točno tu putanju.

- [ ] **Step 7: Pokreni — mora proći**

Run: `cargo test -p store; echo EXIT=$?` → `3 passed; 0 failed`, `EXIT=0`
Run: `SQLX_OFFLINE=true cargo build -p store; echo EXIT=$?` → `EXIT=0` (dokaz da build ne treba bazu)
Run: `cargo clippy --workspace --all-targets -- -D warnings; echo EXIT=$?` → `EXIT=0`

- [ ] **Step 8: Commit** — `git add Cargo.toml Cargo.lock crates/app crates/store .sqlx` (samo ako je `.sqlx` u korijenu) · poruka: „Kostur 4/8: port SystemStatus i dnevnik u Postgresu"

---

