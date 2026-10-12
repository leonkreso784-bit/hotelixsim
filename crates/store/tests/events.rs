use sqlx::PgPool;
use store::Store;

fn db_code(e: sqlx::Error) -> String {
    e.as_database_error()
        .and_then(|d| d.code())
        .map(|c| c.into_owned())
        .unwrap_or_default()
}

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
    assert_eq!(
        db_code(err),
        "23505",
        "isti (stream, stream_seq) ne smije proći dvaput"
    );
    insert("game:2")
        .execute(&pool)
        .await
        .expect("isti redni broj u drugom toku je dopušten");
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
    for sql in [
        "UPDATE events SET type = 'Y'",
        "DELETE FROM events",
        "TRUNCATE events",
    ] {
        let err = sqlx::query(sql).execute(&pool).await.unwrap_err();
        assert_eq!(db_code(err), "42501", "{sql}");
    }
    assert_eq!(Store::from_pool(pool).count_events().await.unwrap(), 1);
}

#[sqlx::test]
async fn actor_must_follow_convention(pool: PgPool) {
    let insert = |seq: i32, actor: &str| {
        sqlx::query(
            "INSERT INTO events (game_id, stream, stream_seq, type, schema_ver, payload, actor) \
             VALUES ('00000000-0000-0000-0000-000000000001', 'game:1', $1, 'X', 1, '{}'::jsonb, $2)",
        )
        .bind(seq)
        .bind(actor.to_owned())
    };
    for bad in ["leon", "guest:123", "robot:", "Robot:lovac", "system "] {
        let err = insert(1, bad).execute(&pool).await.unwrap_err();
        assert_eq!(db_code(err), "23514", "{bad}");
    }
    // oblici koje stvarno daje game::Actor::as_db
    for (seq, ok) in [
        (1, "guest:00000000-0000-0000-0000-00000000002a"),
        (2, "robot:stedisa"),
        (3, "robot:nasumicni"),
        (4, "system"),
    ] {
        insert(seq, ok)
            .execute(&pool)
            .await
            .unwrap_or_else(|e| panic!("{ok}: {e}"));
    }
}

/// Sezonski mentor (KS-2) je novi događaj: baza ga prima i vraća bajt-isti JSON (hrvatski tekst, novi redovi, meta).
/// Isti JSON `game/tests/season_mentor.rs` čita kao `GameEvent::SeasonDebriefRecorded`.
#[sqlx::test]
async fn season_debrief_event_roundtrips_through_database(pool: PgPool) {
    use app::{Decision, EventLog, GameId, GuestId, Guests, HotelId, NewEvent, NewGame, NewHotel};
    use uuid::Uuid;
    let s = Store::from_pool(pool);
    let (g, guest, h) = (
        GameId(Uuid::from_u128(1)),
        GuestId(Uuid::from_u128(2)),
        HotelId(Uuid::from_u128(11)),
    );
    s.create_guest(guest, "sha256:sezona".into()).await.unwrap();
    let ev = |kind: &str, payload: serde_json::Value| NewEvent {
        kind: kind.into(),
        schema_ver: 1,
        payload,
        actor: "system".into(),
    };
    s.create_game(NewGame {
        game_id: g,
        created_by: guest,
        hotels: vec![NewHotel {
            hotel_id: h,
            slot: 1,
            guest_id: Some(guest),
        }],
        events: vec![ev("GameCreated", serde_json::json!({}))],
    })
    .await
    .unwrap();
    let payload = serde_json::json!({
        "hotel_id": "00000000-0000-0000-0000-00000000000b",
        "text": "Kako je prošla sezona\nNapredak je +8 bodova.",
        "source": "ai",
        "meta": { "model": "claude-sonnet-5-5", "prompt": "mentor-sezona.v1", "input_tokens": 10, "output_tokens": 20 }
    });
    let sent = ev("SeasonDebriefRecorded", payload.clone());
    s.decide(
        g,
        Box::new(move |_| {
            Ok(Decision {
                events: vec![sent],
                ..Decision::default()
            })
        }),
    )
    .await
    .unwrap();
    let raw = s.load(g).await.unwrap();
    let back = raw.last().unwrap();
    assert_eq!((back.seq, back.kind.as_str()), (2, "SeasonDebriefRecorded"));
    assert_eq!(back.schema_ver, 1);
    assert_eq!(back.payload, payload);
}
