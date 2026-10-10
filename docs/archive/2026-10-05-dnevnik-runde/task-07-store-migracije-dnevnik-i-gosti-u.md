### Task 7: `store` — migracije, dnevnik i gosti u Postgresu

**Files:**
- Create: `crates/store/migrations/0002_events_append_only.sql`, `crates/store/migrations/0003_platform.sql`
- Modify: `crates/store/migrations/0001_events.sql` — **ne dirati** (primijenjene migracije se ne mijenjaju; sqlx
  provjerava kontrolni zbroj)
- Modify: `crates/store/src/lib.rs`; Create: `crates/store/src/log.rs`, `crates/store/src/guests.rs`
- Modify: `crates/store/Cargo.toml`, `Cargo.toml` (sqlx značajke `uuid`, `json`)
- Modify: `crates/store/tests/events.rs`; Create: `crates/store/tests/log.rs`
- Modify: `.sqlx/` (regenerirano)

**Interfaces:**
- Consumes: Task 5 (`app::{EventLog, Guests, RawEvent, NewEvent, NewGame, NewHotel, Decision, Decided, DecideError,
  Decide, StoreError, GameId, GuestId, HotelId}`).
- Produces: `impl EventLog for Store`, `impl Guests for Store`; tablice `guests`, `games` (`seq BIGSERIAL`), `hotels`,
  `jobs` (stupac `job_key`, ne `key`), indeks `events_game_idx`, okidači `events_no_update_delete` i
  `events_no_truncate` (SQLSTATE `42501`), CHECK `events_actor_format` (SQLSTATE `23514`).

- [ ] **Step 1: Ovisnosti.** Workspace `sqlx` značajke: `["runtime-tokio", "postgres", "macros", "migrate", "uuid", "json"]`.
  `crates/store/Cargo.toml`: `serde_json.workspace = true`, `uuid.workspace = true`; dev: `serde_json`, `uuid`, `tokio`.

- [ ] **Step 2: Migracije.** `0002_events_append_only.sql`:

```sql
-- Dnevnik je samo za dopisivanje (ARHITEKTURA §9.1, ADR-015 t. 5): baza odbija izmjenu, brisanje i pražnjenje.
-- Uloga bez prava UPDATE/DELETE dolazi prije Railwaya (BACKLOG); okidač radi već sad, i u sqlx::test bazama.
CREATE INDEX events_game_idx ON events (game_id, id);

ALTER TABLE events ADD CONSTRAINT events_actor_format
    CHECK (actor ~ '^(guest:[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}|robot:[a-z_]+|system)$');

CREATE FUNCTION events_append_only() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'tablica events je samo za dopisivanje (% odbijen)', TG_OP
        USING ERRCODE = 'insufficient_privilege';
END
$$;

CREATE TRIGGER events_no_update_delete BEFORE UPDATE OR DELETE ON events
    FOR EACH ROW EXECUTE FUNCTION events_append_only();
CREATE TRIGGER events_no_truncate BEFORE TRUNCATE ON events
    FOR EACH STATEMENT EXECUTE FUNCTION events_append_only();
```
  `0003_platform.sql`:

```sql
-- Platformske tablice (ARHITEKTURA §9.3): obične tablice za pretragu; istina o igri je u events.
CREATE TABLE guests (
    id          UUID PRIMARY KEY,
    token_hash  TEXT NOT NULL UNIQUE,          -- "sha256:<hex>" tokena iz kolačića; sam token se ne sprema
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE games (
    id          UUID PRIMARY KEY,
    seq         BIGSERIAL UNIQUE,              -- redoslijed stvaranja („zadnja igra gosta"), neovisno o satu
    created_by  UUID NOT NULL REFERENCES guests (id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE hotels (
    id        UUID PRIMARY KEY,
    game_id   UUID NOT NULL REFERENCES games (id),
    slot      SMALLINT NOT NULL CHECK (slot BETWEEN 1 AND 4),
    guest_id  UUID REFERENCES guests (id),     -- NULL = robot
    UNIQUE (game_id, slot)
);
CREATE INDEX hotels_guest_idx ON hotels (guest_id);

-- Red poslova (ARHITEKTURA §9.4, ADR-015 t. 8).
CREATE TABLE jobs (
    id            BIGSERIAL PRIMARY KEY,
    job_key       TEXT NOT NULL UNIQUE,        -- npr. resolve:<game>:<mjesec>; drugi upis istog ključa se preskače
    kind          TEXT NOT NULL,
    game_id       UUID NOT NULL,
    payload       JSONB NOT NULL,
    status        TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'done', 'failed')),
    attempts      INTEGER NOT NULL DEFAULT 0,
    locked_until  TIMESTAMPTZ,
    last_error    TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX jobs_ready_idx ON jobs (id) WHERE status IN ('queued', 'running');
```

- [ ] **Step 3: Testovi (crveni).** U `tests/events.rs` zamijeni `stream_sequence_is_unique`:

```rust
fn db_code(e: sqlx::Error) -> String {
    e.as_database_error().and_then(|d| d.code()).map(|c| c.into_owned()).unwrap_or_default()
}

#[sqlx::test]
async fn stream_sequence_is_unique_per_stream(pool: PgPool) {
    let insert = |stream: &'static str| {
        sqlx::query(
            "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
             VALUES ('00000000-0000-0000-0000-000000000001', $1, 1, 'X', 1, '{}'::jsonb, 'system')",
        )
        .bind(stream)
    };
    insert("game:1").execute(&pool).await.unwrap();
    let err = insert("game:1").execute(&pool).await.unwrap_err();
    assert_eq!(db_code(err), "23505", "isti (stream, stream_seq) ne smije proći dvaput");
    insert("game:2").execute(&pool).await.expect("isti redni broj u drugom toku je dopušten");
}

#[sqlx::test]
async fn events_are_append_only(pool: PgPool) {
    sqlx::query(
        "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
         VALUES ('00000000-0000-0000-0000-000000000001', 'game:1', 1, 'X', 1, '{}'::jsonb, 'system')",
    )
    .execute(&pool)
    .await
    .unwrap();
    for sql in ["UPDATE events SET type = 'Y'", "DELETE FROM events", "TRUNCATE events"] {
        let err = sqlx::query(sql).execute(&pool).await.unwrap_err();
        assert_eq!(db_code(err), "42501", "{sql}");
    }
    assert_eq!(Store::from_pool(pool).count_events().await.unwrap(), 1);
}

#[sqlx::test]
async fn actor_must_follow_convention(pool: PgPool) {
    let err = sqlx::query(
        "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
         VALUES ('00000000-0000-0000-0000-000000000001', 'game:1', 1, 'X', 1, '{}'::jsonb, 'leon')",
    )
    .execute(&pool)
    .await
    .unwrap_err();
    assert_eq!(db_code(err), "23514");
}
```
  `tests/log.rs`:

```rust
use app::{Decision, DecideError, EventLog, GameId, Guests, GuestId, HotelId, NewEvent, NewGame, NewHotel, NewJob, RawEvent};
use serde_json::json;
use sqlx::PgPool;
use store::Store;
use uuid::Uuid;

fn ev(kind: &str) -> NewEvent {
    NewEvent { kind: kind.into(), schema_ver: 1, payload: json!({ "k": kind }), actor: "system".into() }
}

async fn game(store: &Store, n: u128) -> (GameId, GuestId, HotelId) {
    let (g, guest, h) = (GameId(Uuid::from_u128(n)), GuestId(Uuid::from_u128(n + 1000)), HotelId(Uuid::from_u128(n + 2000)));
    store.create_guest(guest, format!("sha256:{n}")).await.unwrap();
    store
        .create_game(NewGame { game_id: g, created_by: guest,
            hotels: vec![NewHotel { hotel_id: h, slot: 1, guest_id: Some(guest) }],
            events: vec![ev("GameCreated")] })
        .await
        .unwrap();
    (g, guest, h)
}

fn append(kind: &'static str) -> app::Decide<'static> {
    Box::new(move |_: &[RawEvent]| Ok(Decision { events: vec![ev(kind)], ..Decision::default() }))
}

#[sqlx::test]
async fn create_then_decide_continues_the_stream(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    let d = s.decide(g, append("PlanSet")).await.unwrap();
    assert_eq!((d.last_seq, d.appended), (2, 1));
    let raw = s.load(g).await.unwrap();
    assert_eq!(raw.iter().map(|r| (r.seq, r.kind.as_str())).collect::<Vec<_>>(), [(1, "GameCreated"), (2, "PlanSet")]);
    assert_eq!(raw[1].payload, json!({ "k": "PlanSet" }));
    assert!(s.load(GameId(Uuid::from_u128(77))).await.unwrap().is_empty());
}

#[sqlx::test]
async fn decide_rejection_writes_nothing(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    let r = s
        .decide(g, Box::new(|_: &[RawEvent]| Err(app::AppError::new(app::ErrorCode::RoundNotOpen))))
        .await;
    assert!(matches!(r, Err(DecideError::Rejected(e)) if e.code == app::ErrorCode::RoundNotOpen));
    assert_eq!(s.load(g).await.unwrap().len(), 1);
}

#[sqlx::test]
async fn concurrent_decides_are_serialized(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    let tasks: Vec<_> = (0..20)
        .map(|_| {
            let s = s.clone();
            tokio::spawn(async move { s.decide(g, append("X")).await.map(|d| d.last_seq).map_err(|e| format!("{e:?}")) })
        })
        .collect();
    for t in tasks {
        t.await.unwrap().expect("zaključavanje igre: nijedan sukob rednog broja");
    }
    let seqs: Vec<i64> = s.load(g).await.unwrap().iter().map(|r| r.seq).collect();
    assert_eq!(seqs, (1..=21).collect::<Vec<i64>>());
}

#[sqlx::test]
async fn decide_enqueues_job_once_per_key(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    for _ in 0..2 {
        s.decide(g, Box::new(move |_: &[RawEvent]| Ok(Decision {
            events: vec![],
            enqueue: Some(NewJob { key: "resolve:x:jun".into(), kind: "resolve_round".into(), game_id: g, payload: json!({"month": "jun"}) }),
            finish_job: None,
        })))
        .await
        .unwrap();
    }
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs").fetch_one(&pool).await.unwrap();
    assert_eq!(n, 1);
}

#[sqlx::test]
async fn guests_and_hotels_lookup(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g1, guest, h1) = game(&s, 1).await;
    assert_eq!(s.guest_by_token_hash("sha256:1".into()).await.unwrap(), Some(guest));
    assert_eq!(s.guest_by_token_hash("sha256:nema".into()).await.unwrap(), None);
    assert_eq!(s.game_of_hotel(h1).await.unwrap(), Some(g1));
    assert_eq!(s.latest_hotel_of(guest).await.unwrap(), Some((g1, h1)));
    let (g2, h2) = (GameId(Uuid::from_u128(5)), HotelId(Uuid::from_u128(6)));
    s.create_game(NewGame { game_id: g2, created_by: guest,
        hotels: vec![NewHotel { hotel_id: h2, slot: 1, guest_id: Some(guest) }], events: vec![ev("GameCreated")] })
        .await
        .unwrap();
    assert_eq!(s.latest_hotel_of(guest).await.unwrap(), Some((g2, h2)), "zadnja igra po games.seq");
}
```

- [ ] **Step 4: Pokreni** `bash scripts/baza.sh status` (ako ne radi: `bash scripts/baza.sh start`), pa
  `set -a; source .env; set +a; cargo sqlx migrate run --source crates/store/migrations; echo EXIT=$?` → `EXIT=0`;
  `cargo test -p store; echo EXIT=$?` → ne prevodi se (`EventLog` nije izveden), `EXIT=101`.

- [ ] **Step 5: `log.rs`.**

```rust
use app::{BoxFut, Decide, DecideError, Decided, EventLog, GameId, NewEvent, NewGame, RawEvent, StoreError};
use sqlx::PgConnection;

use crate::Store;

pub(crate) fn store_err(e: sqlx::Error) -> StoreError {
    StoreError(e.to_string())
}

async fn load_in(conn: &mut PgConnection, stream: &str) -> Result<Vec<RawEvent>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT stream_seq, type AS "kind", schema_ver, payload, actor
           FROM events WHERE stream = $1 ORDER BY stream_seq"#,
        stream
    )
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| RawEvent { seq: i64::from(r.stream_seq), kind: r.kind, schema_ver: r.schema_ver, payload: r.payload, actor: r.actor })
        .collect())
}

async fn insert_events(conn: &mut PgConnection, game: GameId, stream: &str, last: i64, events: &[NewEvent]) -> Result<(), sqlx::Error> {
    for (e, seq) in events.iter().zip(last + 1..) {
        let seq = i32::try_from(seq).map_err(|_| sqlx::Error::Protocol("redni broj izvan INTEGER".into()))?;
        sqlx::query!(
            "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            game.0, stream, seq, e.kind, e.schema_ver, e.payload, e.actor
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

impl EventLog for Store {
    fn create_game(&self, g: NewGame) -> BoxFut<'_, Result<(), StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(store_err)?;
            sqlx::query!("INSERT INTO games (id, created_by) VALUES ($1, $2)", g.game_id.0, g.created_by.0)
                .execute(&mut *tx).await.map_err(store_err)?;
            for h in &g.hotels {
                sqlx::query!("INSERT INTO hotels (id, game_id, slot, guest_id) VALUES ($1, $2, $3, $4)",
                    h.hotel_id.0, g.game_id.0, h.slot, h.guest_id.map(|x| x.0))
                    .execute(&mut *tx).await.map_err(store_err)?;
            }
            insert_events(&mut tx, g.game_id, &g.game_id.stream(), 0, &g.events).await.map_err(store_err)?;
            tx.commit().await.map_err(store_err)
        })
    }

    fn load(&self, game: GameId) -> BoxFut<'_, Result<Vec<RawEvent>, StoreError>> {
        Box::pin(async move {
            let mut conn = self.pool.acquire().await.map_err(store_err)?;
            load_in(&mut conn, &game.stream()).await.map_err(store_err)
        })
    }

    fn decide<'a>(&'a self, game: GameId, decide: Decide<'a>) -> BoxFut<'a, Result<Decided, DecideError>> {
        Box::pin(async move {
            let st = |e: sqlx::Error| DecideError::Store(store_err(e));
            let stream = game.stream();
            let mut tx = self.pool.begin().await.map_err(st)?;
            // Jedna naredba po igri u isto vrijeme (ADR-015 t. 4); UNIQUE(stream, stream_seq) je zadnja brana.
            sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
                .bind(&stream)
                .execute(&mut *tx)
                .await
                .map_err(st)?;
            let raw = load_in(&mut tx, &stream).await.map_err(st)?;
            let last = raw.last().map_or(0, |r| r.seq);
            let d = decide(&raw).map_err(DecideError::Rejected)?; // `tx` se odbacuje = ROLLBACK
            insert_events(&mut tx, game, &stream, last, &d.events).await.map_err(st)?;
            if let Some(j) = &d.enqueue {
                sqlx::query!(
                    "INSERT INTO jobs (job_key, kind, game_id, payload) VALUES ($1, $2, $3, $4) \
                     ON CONFLICT (job_key) DO NOTHING",
                    j.key, j.kind, j.game_id.0, j.payload
                )
                .execute(&mut *tx).await.map_err(st)?;
            }
            if let Some(id) = d.finish_job {
                sqlx::query!("UPDATE jobs SET status = 'done', locked_until = NULL, updated_at = now() WHERE id = $1", id)
                    .execute(&mut *tx).await.map_err(st)?;
            }
            tx.commit().await.map_err(st)?;
            let appended = d.events.len();
            let added = i64::try_from(appended).map_err(|_| DecideError::Store(StoreError("previše događaja".into())))?;
            Ok(Decided { last_seq: last + added, appended })
        })
    }
}
```
  (`pg_advisory_xact_lock` vraća `void`, zato je to funkcija `sqlx::query`, ne makro — makro ne zna tip `void`.)

- [ ] **Step 6: `guests.rs`** — `impl Guests for Store` s `query!` upitima:
  `INSERT INTO guests (id, token_hash) VALUES ($1, $2)` · `SELECT id FROM guests WHERE token_hash = $1` ·
  `SELECT game_id FROM hotels WHERE id = $1` ·
  `SELECT h.game_id, h.id FROM hotels h JOIN games g ON g.id = h.game_id WHERE h.guest_id = $1 ORDER BY g.seq DESC LIMIT 1`
  (`fetch_optional`, rezultat u `GuestId`/`GameId`/`HotelId`). U `lib.rs`: `mod guests; mod log;`, polje `pool`
  ostaje privatno ali vidljivo modulima (`pub(crate)` ako treba), komentar zaglavlja „dnevnik, gosti, red poslova".

- [ ] **Step 7: Offline cache.** `set -a; source .env; set +a; SQLX_OFFLINE=false cargo sqlx prepare --workspace; echo EXIT=$?`
  → `EXIT=0`, nove datoteke u `.sqlx/`.

- [ ] **Step 8: Pokreni** `cargo test -p store; echo EXIT=$?` → `EXIT=0`; zatim provjere iz Global Constraints i
  `cargo sqlx prepare --workspace --check; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 9: Commit** — `Cargo.toml Cargo.lock crates/store .sqlx`, poruka
  „Dnevnik 7/12: store — nepromjenjiv dnevnik, platformske tablice, decide pod zaključavanjem igre".

---

