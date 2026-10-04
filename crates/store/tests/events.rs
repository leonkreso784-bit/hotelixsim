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
    assert!(
        sqlx::query(insert).execute(&pool).await.is_err(),
        "isti (stream, stream_seq) ne smije proći dvaput"
    );
}
