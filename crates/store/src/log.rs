use app::{
    BoxFut, Decide, DecideError, Decided, EventLog, GameId, NewEvent, NewGame, RawEvent, StoreError,
};
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
        .map(|r| RawEvent {
            seq: i64::from(r.stream_seq),
            kind: r.kind,
            schema_ver: r.schema_ver,
            payload: r.payload,
            actor: r.actor,
        })
        .collect())
}

async fn insert_events(
    conn: &mut PgConnection,
    game: GameId,
    stream: &str,
    last: i64,
    events: &[NewEvent],
) -> Result<(), sqlx::Error> {
    for (e, seq) in events.iter().zip(last + 1..) {
        let seq = i32::try_from(seq)
            .map_err(|_| sqlx::Error::Protocol("redni broj izvan INTEGER".into()))?;
        sqlx::query!(
            "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            game.0,
            stream,
            seq,
            e.kind,
            e.schema_ver,
            e.payload,
            e.actor
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
            // Dvostruki `game_id` je povreda primarnog ključa: StoreError, igra se ne prepisuje.
            sqlx::query!(
                "INSERT INTO games (id, created_by) VALUES ($1, $2)",
                g.game_id.0,
                g.created_by.0
            )
            .execute(&mut *tx)
            .await
            .map_err(store_err)?;
            for h in &g.hotels {
                sqlx::query!(
                    "INSERT INTO hotels (id, game_id, slot, guest_id) VALUES ($1, $2, $3, $4)",
                    h.hotel_id.0,
                    g.game_id.0,
                    h.slot,
                    h.guest_id.map(|x| x.0)
                )
                .execute(&mut *tx)
                .await
                .map_err(store_err)?;
            }
            insert_events(&mut tx, g.game_id, &g.game_id.stream(), 0, &g.events)
                .await
                .map_err(store_err)?;
            tx.commit().await.map_err(store_err)
        })
    }

    fn load(&self, game: GameId) -> BoxFut<'_, Result<Vec<RawEvent>, StoreError>> {
        Box::pin(async move {
            let mut conn = self.pool.acquire().await.map_err(store_err)?;
            load_in(&mut conn, &game.stream()).await.map_err(store_err)
        })
    }

    fn decide<'a>(
        &'a self,
        game: GameId,
        decide: Decide<'a>,
    ) -> BoxFut<'a, Result<Decided, DecideError>> {
        Box::pin(async move {
            let st = |e: sqlx::Error| DecideError::Store(store_err(e));
            let stream = game.stream();
            let mut tx = self.pool.begin().await.map_err(st)?;
            // Jedna naredba po igri u isto vrijeme (ADR-015 t. 4); UNIQUE(stream, stream_seq) je zadnja brana.
            // `pg_advisory_xact_lock` vraća `void`, pa je ovo funkcija `sqlx::query`, ne makro.
            sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
                .bind(&stream)
                .execute(&mut *tx)
                .await
                .map_err(st)?;
            let raw = load_in(&mut tx, &stream).await.map_err(st)?;
            let last = raw.last().map_or(0, |r| r.seq);
            let d = decide(&raw).map_err(DecideError::Rejected)?; // `tx` se odbacuje = ROLLBACK
            insert_events(&mut tx, game, &stream, last, &d.events)
                .await
                .map_err(st)?;
            if let Some(j) = &d.enqueue {
                sqlx::query!(
                    "INSERT INTO jobs (job_key, kind, game_id, payload) VALUES ($1, $2, $3, $4) \
                     ON CONFLICT (job_key) DO NOTHING",
                    j.key,
                    j.kind,
                    j.game_id.0,
                    j.payload
                )
                .execute(&mut *tx)
                .await
                .map_err(st)?;
            }
            if let Some(c) = d.finish_job {
                // Samo posao koji TAJ radnik još drži; zastarjeli zahtjev ne radi ništa (događaji se ipak upisuju).
                sqlx::query!(
                    "UPDATE jobs SET status = 'done', locked_until = NULL, updated_at = now() \
                     WHERE id = $1 AND attempts = $2 AND status = 'running'",
                    c.id,
                    c.attempts
                )
                .execute(&mut *tx)
                .await
                .map_err(st)?;
            }
            tx.commit().await.map_err(st)?;
            let appended = d.events.len();
            let added = i64::try_from(appended)
                .map_err(|_| DecideError::Store(StoreError("previše događaja".into())))?;
            Ok(Decided {
                last_seq: last + added,
                appended,
            })
        })
    }
}
