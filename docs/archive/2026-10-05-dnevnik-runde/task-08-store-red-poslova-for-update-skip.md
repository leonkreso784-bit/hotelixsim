### Task 8: `store` — red poslova (`FOR UPDATE SKIP LOCKED`, zakup, pokušaji)

**Files:**
- Create: `crates/store/src/jobs.rs`; Modify: `crates/store/src/lib.rs` (`mod jobs;`)
- Test: `crates/store/tests/jobs.rs`
- Modify: `.sqlx/`

**Interfaces:**
- Consumes: Task 5 (`app::{JobQueue, Job, MAX_ATTEMPTS}`), Task 7 (tablica `jobs`, `decide` s `enqueue`/`finish_job`).
- Produces: `impl JobQueue for Store`.

- [ ] **Step 1: Testovi (crveni).** `crates/store/tests/jobs.rs` (uvozi i pomoćnike `game` i `ev` kopiraj iz `tests/log.rs`; dodatno `use app::JobQueue;`):

```rust
async fn enqueue(s: &Store, g: GameId, key: &str) {
    let key = key.to_owned();
    s.decide(g, Box::new(move |_: &[RawEvent]| Ok(Decision { events: vec![], finish_job: None,
        enqueue: Some(NewJob { key, kind: "resolve_round".into(), game_id: g, payload: json!({"month": "jun"}) }) })))
        .await
        .unwrap();
}

#[sqlx::test]
async fn empty_queue_gives_none(pool: PgPool) {
    assert!(Store::from_pool(pool).claim(30).await.unwrap().is_none());
}

#[sqlx::test]
async fn claimed_job_is_leased(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let j = s.claim(30).await.unwrap().unwrap();
    assert_eq!((j.key.as_str(), j.attempts, j.game_id), ("a", 1, g));
    assert_eq!(j.payload, json!({"month": "jun"}));
    assert!(s.claim(30).await.unwrap().is_none(), "zakup traje");
}

#[sqlx::test]
async fn concurrent_claims_take_one_job_once(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let tasks: Vec<_> = (0..8).map(|_| { let s = s.clone(); tokio::spawn(async move { s.claim(30).await.unwrap() }) }).collect();
    let mut got = 0;
    for t in tasks {
        if t.await.unwrap().is_some() { got += 1; }
    }
    assert_eq!(got, 1, "dvostruko preuzimanje nije moguće");
}

#[sqlx::test]
async fn expired_lease_is_reclaimed(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let first = s.claim(30).await.unwrap().unwrap(); // radnik „padne" usred posla
    sqlx::query("UPDATE jobs SET locked_until = now() - interval '1 second'").execute(&pool).await.unwrap();
    let again = s.claim(30).await.unwrap().unwrap();
    assert_eq!((again.id, again.attempts), (first.id, 2));
}

#[sqlx::test]
async fn finished_job_is_not_claimed(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let j = s.claim(30).await.unwrap().unwrap();
    s.decide(g, Box::new(move |_: &[RawEvent]| Ok(Decision { events: vec![], enqueue: None, finish_job: Some(j.id) }))).await.unwrap();
    // i kad bi zakup „istekao", gotov posao se više ne preuzima
    sqlx::query("UPDATE jobs SET locked_until = now() - interval '1 second'").execute(&pool).await.unwrap();
    assert!(s.claim(30).await.unwrap().is_none());
}

#[sqlx::test]
async fn failing_job_is_retried_then_failed(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    for i in 1..=app::MAX_ATTEMPTS {
        let j = s.claim(30).await.unwrap().expect("posao se vraća u red");
        assert_eq!(j.attempts, i);
        s.fail(j.id, format!("pokušaj {i}")).await.unwrap();
    }
    assert!(s.claim(30).await.unwrap().is_none());
    let (status, err): (String, Option<String>) =
        sqlx::query_as("SELECT status, last_error FROM jobs").fetch_one(&pool).await.unwrap();
    assert_eq!(status, "failed");
    assert_eq!(err.as_deref(), Some("pokušaj 5"));
}

#[sqlx::test]
async fn expired_last_attempt_becomes_failed(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    sqlx::query("UPDATE jobs SET status = 'running', attempts = $1, locked_until = now() - interval '1 second'")
        .bind(app::MAX_ATTEMPTS).execute(&pool).await.unwrap();
    assert!(s.claim(30).await.unwrap().is_none());
    let status: String = sqlx::query_scalar("SELECT status FROM jobs").fetch_one(&pool).await.unwrap();
    assert_eq!(status, "failed");
}
```
- [ ] **Step 2: Pokreni** `cargo test -p store --test jobs; echo EXIT=$?` → ne prevodi se, `EXIT=101`.

- [ ] **Step 3: `jobs.rs`.**

```rust
use app::{BoxFut, GameId, Job, JobQueue, MAX_ATTEMPTS, StoreError};

use crate::Store;
use crate::log::store_err;

impl JobQueue for Store {
    fn claim(&self, lease_secs: i32) -> BoxFut<'_, Result<Option<Job>, StoreError>> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(store_err)?;
            // posao kojem je istekao zakup nakon zadnjeg dopuštenog pokušaja ne vraća se u red
            sqlx::query!(
                "UPDATE jobs SET status = 'failed', locked_until = NULL, updated_at = now(), \
                 last_error = COALESCE(last_error, 'istekao zakup nakon zadnjeg pokušaja') \
                 WHERE status = 'running' AND locked_until < now() AND attempts >= $1",
                MAX_ATTEMPTS
            )
            .execute(&mut *tx).await.map_err(store_err)?;
            let row = sqlx::query!(
                r#"UPDATE jobs SET status = 'running', attempts = attempts + 1,
                       locked_until = now() + make_interval(secs => $1), updated_at = now()
                   WHERE id = (
                       SELECT id FROM jobs
                       WHERE (status = 'queued' OR (status = 'running' AND locked_until < now()))
                         AND attempts < $2
                       ORDER BY id
                       FOR UPDATE SKIP LOCKED
                       LIMIT 1
                   )
                   RETURNING id, job_key, kind, game_id, payload, attempts"#,
                f64::from(lease_secs),
                MAX_ATTEMPTS
            )
            .fetch_optional(&mut *tx).await.map_err(store_err)?;
            tx.commit().await.map_err(store_err)?;
            Ok(row.map(|r| Job { id: r.id, key: r.job_key, kind: r.kind, game_id: GameId(r.game_id), payload: r.payload, attempts: r.attempts }))
        })
    }

    fn fail(&self, id: i64, error: String) -> BoxFut<'_, Result<(), StoreError>> {
        Box::pin(async move {
            sqlx::query!(
                "UPDATE jobs SET status = CASE WHEN attempts >= $2 THEN 'failed' ELSE 'queued' END, \
                 locked_until = NULL, last_error = $3, updated_at = now() WHERE id = $1",
                id, MAX_ATTEMPTS, error
            )
            .execute(&self.pool).await.map_err(store_err)?;
            Ok(())
        })
    }
}
```
  Ponovni pokušaj je odmah (bez odgode); odgoda za AI i usporedba s apalis/pgmq su korak 4 (BACKLOG, Task 12).

- [ ] **Step 4: Offline cache i testovi.** `set -a; source .env; set +a; SQLX_OFFLINE=false cargo sqlx prepare --workspace; echo EXIT=$?`
  → `EXIT=0`; `cargo test -p store; echo EXIT=$?` → `EXIT=0`.

- [ ] **Step 5: Provjere i commit** — `crates/store .sqlx`, poruka
  „Dnevnik 8/12: store — red poslova sa zakupom i SKIP LOCKED".

---
