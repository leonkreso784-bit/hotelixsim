//! Red poslova u PostgreSQL-u: zakup, `FOR UPDATE SKIP LOCKED`, brojač pokušaja (ARHITEKTURA §9.4, ADR-015 t. 8).
use app::{BoxFut, GameId, Job, JobClaim, JobQueue, MAX_ATTEMPTS, StoreError};

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
            .execute(&mut *tx)
            .await
            .map_err(store_err)?;
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
            .fetch_optional(&mut *tx)
            .await
            .map_err(store_err)?;
            tx.commit().await.map_err(store_err)?;
            Ok(row.map(|r| Job {
                id: r.id,
                key: r.job_key,
                kind: r.kind,
                game_id: GameId(r.game_id),
                payload: r.payload,
                attempts: r.attempts,
                age_secs: 0,
            }))
        })
    }

    fn fail(&self, claim: JobClaim, error: String) -> BoxFut<'_, Result<(), StoreError>> {
        Box::pin(async move {
            // samo posao koji TAJ radnik još drži (isti pokušaj); gotov, neuspio ili tuđi zakup se ne dira
            sqlx::query!(
                "UPDATE jobs SET status = CASE WHEN attempts >= $3 THEN 'failed' ELSE 'queued' END, \
                 locked_until = NULL, last_error = $4, updated_at = now() \
                 WHERE id = $1 AND attempts = $2 AND status = 'running'",
                claim.id,
                claim.attempts,
                MAX_ATTEMPTS,
                error
            )
            .execute(&self.pool)
            .await
            .map_err(store_err)?;
            Ok(())
        })
    }

    fn defer(&self, _claim: JobClaim, _secs: i32) -> BoxFut<'_, Result<(), StoreError>> {
        Box::pin(async { Err(StoreError("defer: Task 7".into())) })
    }
}
