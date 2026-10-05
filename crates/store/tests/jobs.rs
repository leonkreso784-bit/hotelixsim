use app::{
    Decision, EventLog, GameId, GuestId, Guests, HotelId, JobQueue, NewEvent, NewGame, NewHotel,
    NewJob, RawEvent,
};
use serde_json::json;
use sqlx::PgPool;
use store::Store;
use uuid::Uuid;

fn ev(kind: &str) -> NewEvent {
    NewEvent {
        kind: kind.into(),
        schema_ver: 1,
        payload: json!({ "k": kind }),
        actor: "system".into(),
    }
}

fn new_game(g: GameId, guest: GuestId, h: HotelId) -> NewGame {
    NewGame {
        game_id: g,
        created_by: guest,
        hotels: vec![NewHotel {
            hotel_id: h,
            slot: 1,
            guest_id: Some(guest),
        }],
        events: vec![ev("GameCreated")],
    }
}

async fn game(store: &Store, n: u128) -> (GameId, GuestId, HotelId) {
    let (g, guest, h) = (
        GameId(Uuid::from_u128(n)),
        GuestId(Uuid::from_u128(n + 1000)),
        HotelId(Uuid::from_u128(n + 2000)),
    );
    store
        .create_guest(guest, format!("sha256:{n}"))
        .await
        .unwrap();
    store.create_game(new_game(g, guest, h)).await.unwrap();
    (g, guest, h)
}

async fn enqueue(s: &Store, g: GameId, key: &str) {
    let key = key.to_owned();
    s.decide(
        g,
        Box::new(move |_: &[RawEvent]| {
            Ok(Decision {
                events: vec![],
                finish_job: None,
                enqueue: Some(NewJob {
                    key,
                    kind: "resolve_round".into(),
                    game_id: g,
                    payload: json!({"month": "jun"}),
                }),
            })
        }),
    )
    .await
    .unwrap();
}

async fn expire(pool: &PgPool) {
    sqlx::query("UPDATE jobs SET locked_until = now() - interval '1 second'")
        .execute(pool)
        .await
        .unwrap();
}

async fn state(pool: &PgPool) -> (String, i32) {
    sqlx::query_as("SELECT status, attempts FROM jobs")
        .fetch_one(pool)
        .await
        .unwrap()
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
    let tasks: Vec<_> = (0..8)
        .map(|_| {
            let s = s.clone();
            tokio::spawn(async move { s.claim(30).await.unwrap() })
        })
        .collect();
    let mut got = 0;
    for t in tasks {
        if t.await.unwrap().is_some() {
            got += 1;
        }
    }
    assert_eq!(got, 1, "dvostruko preuzimanje nije moguće");
}

#[sqlx::test]
async fn expired_lease_is_reclaimed(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let first = s.claim(30).await.unwrap().unwrap(); // radnik „padne" usred posla
    expire(&pool).await;
    let again = s.claim(30).await.unwrap().unwrap();
    assert_eq!((again.id, again.attempts), (first.id, 2));
}

#[sqlx::test]
async fn finished_job_is_not_claimed(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let j = s.claim(30).await.unwrap().unwrap();
    s.decide(
        g,
        Box::new(move |_: &[RawEvent]| {
            Ok(Decision {
                events: vec![],
                enqueue: None,
                finish_job: Some(j.claim()),
            })
        }),
    )
    .await
    .unwrap();
    // i kad bi zakup „istekao", gotov posao se više ne preuzima
    expire(&pool).await;
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
        s.fail(j.claim(), format!("pokušaj {i}")).await.unwrap();
    }
    assert!(s.claim(30).await.unwrap().is_none());
    let (status, err): (String, Option<String>) =
        sqlx::query_as("SELECT status, last_error FROM jobs")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "failed");
    assert_eq!(err.as_deref(), Some("pokušaj 5"));
}

#[sqlx::test]
async fn expired_last_attempt_becomes_failed(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    sqlx::query(
        "UPDATE jobs SET status = 'running', attempts = $1, locked_until = now() - interval '1 second'",
    )
    .bind(app::MAX_ATTEMPTS)
    .execute(&pool)
    .await
    .unwrap();
    assert!(s.claim(30).await.unwrap().is_none());
    assert_eq!(state(&pool).await.0, "failed");
    // i dalje se ne preuzima
    assert!(s.claim(30).await.unwrap().is_none());
}

#[sqlx::test]
async fn stale_worker_fail_is_a_noop(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let a = s.claim(30).await.unwrap().unwrap();
    expire(&pool).await;
    let b = s.claim(30).await.unwrap().unwrap();
    assert_eq!(b.attempts, 2);
    s.fail(a.claim(), "zastarjeli radnik".into()).await.unwrap();
    assert_eq!(state(&pool).await, ("running".to_owned(), 2));
    let err: Option<String> = sqlx::query_scalar("SELECT last_error FROM jobs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(err, None);
}

#[sqlx::test]
async fn fail_on_done_job_keeps_it_done(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    enqueue(&s, g, "a").await;
    let j = s.claim(30).await.unwrap().unwrap();
    let claim = j.claim();
    s.decide(
        g,
        Box::new(move |_: &[RawEvent]| {
            Ok(Decision {
                finish_job: Some(claim),
                ..Decision::default()
            })
        }),
    )
    .await
    .unwrap();
    s.fail(claim, "kasno".into()).await.unwrap();
    assert_eq!(state(&pool).await, ("done".to_owned(), 1));
}
