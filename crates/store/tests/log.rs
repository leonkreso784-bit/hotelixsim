use app::{
    DecideError, Decision, EventLog, GameId, GuestId, Guests, HotelId, NewEvent, NewGame, NewHotel,
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

fn append(kind: &'static str) -> app::Decide<'static> {
    Box::new(move |_: &[RawEvent]| {
        Ok(Decision {
            events: vec![ev(kind)],
            ..Decision::default()
        })
    })
}

#[sqlx::test]
async fn create_then_decide_continues_the_stream(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    let d = s.decide(g, append("PlanSet")).await.unwrap();
    assert_eq!((d.last_seq, d.appended), (2, 1));
    let raw = s.load(g).await.unwrap();
    assert_eq!(
        raw.iter()
            .map(|r| (r.seq, r.kind.as_str()))
            .collect::<Vec<_>>(),
        [(1, "GameCreated"), (2, "PlanSet")]
    );
    assert_eq!(raw[1].payload, json!({ "k": "PlanSet" }));
    assert!(
        s.load(GameId(Uuid::from_u128(77)))
            .await
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test]
async fn duplicate_game_id_is_a_store_error(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, guest, h) = game(&s, 1).await;
    let again = NewGame {
        hotels: vec![NewHotel {
            hotel_id: HotelId(Uuid::from_u128(9)),
            slot: 1,
            guest_id: Some(guest),
        }],
        ..new_game(g, guest, h)
    };
    assert!(s.create_game(again).await.is_err());
    assert_eq!(
        s.load(g).await.unwrap().len(),
        1,
        "prva igra ostaje netaknuta"
    );
}

#[sqlx::test]
async fn decide_rejection_writes_nothing(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g, _, _) = game(&s, 1).await;
    let r = s
        .decide(
            g,
            Box::new(|_: &[RawEvent]| Err(app::AppError::new(app::ErrorCode::RoundNotOpen))),
        )
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
            tokio::spawn(async move {
                s.decide(g, append("X"))
                    .await
                    .map(|d| d.last_seq)
                    .map_err(|e| format!("{e:?}"))
            })
        })
        .collect();
    for t in tasks {
        t.await
            .unwrap()
            .expect("zaključavanje igre: nijedan sukob rednog broja");
    }
    let seqs: Vec<i64> = s.load(g).await.unwrap().iter().map(|r| r.seq).collect();
    assert_eq!(seqs, (1..=21).collect::<Vec<i64>>());
}

fn enqueue(g: GameId) -> app::Decide<'static> {
    Box::new(move |_: &[RawEvent]| {
        Ok(Decision {
            events: vec![],
            enqueue: vec![NewJob {
                key: "resolve:x:jun".into(),
                kind: "resolve_round".into(),
                game_id: g,
                payload: json!({"month": "jun"}),
            }],
            finish_job: None,
        })
    })
}

#[sqlx::test]
async fn decide_enqueues_job_once_per_key(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    for _ in 0..2 {
        s.decide(g, enqueue(g)).await.unwrap();
    }
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 1);
}

#[sqlx::test]
async fn finish_job_requires_matching_claim(pool: PgPool) {
    let s = Store::from_pool(pool.clone());
    let (g, _, _) = game(&s, 1).await;
    s.decide(g, enqueue(g)).await.unwrap();
    let id: i64 = sqlx::query_scalar("SELECT id FROM jobs")
        .fetch_one(&pool)
        .await
        .unwrap();
    // zahtjev preuzimanja (claim dolazi u Tasku 8): posao je running, pokušaj 2
    sqlx::query("UPDATE jobs SET status = 'running', attempts = 2")
        .execute(&pool)
        .await
        .unwrap();
    let finish = |attempts: i32| -> app::Decide<'static> {
        Box::new(move |_: &[RawEvent]| {
            Ok(Decision {
                events: vec![ev("RoundResolved")],
                enqueue: vec![],
                finish_job: Some(app::JobClaim { id, attempts }),
            })
        })
    };
    let status = || async {
        sqlx::query_scalar::<_, String>("SELECT status FROM jobs")
            .fetch_one(&pool)
            .await
            .unwrap()
    };
    // zastarjeli pokušaj: događaji se upisuju, posao se ne dira
    s.decide(g, finish(1)).await.unwrap();
    assert_eq!(status().await, "running");
    assert_eq!(s.load(g).await.unwrap().len(), 2);
    s.decide(g, finish(2)).await.unwrap();
    assert_eq!(status().await, "done");
    // gotov posao se ne mijenja ponovno
    s.decide(g, finish(2)).await.unwrap();
    assert_eq!(status().await, "done");
}

#[sqlx::test]
async fn guests_and_hotels_lookup(pool: PgPool) {
    let s = Store::from_pool(pool);
    let (g1, guest, h1) = game(&s, 1).await;
    assert_eq!(
        s.guest_by_token_hash("sha256:1".into()).await.unwrap(),
        Some(guest)
    );
    assert_eq!(
        s.guest_by_token_hash("sha256:nema".into()).await.unwrap(),
        None
    );
    assert_eq!(s.game_of_hotel(h1).await.unwrap(), Some(g1));
    assert_eq!(s.latest_hotel_of(guest).await.unwrap(), Some((g1, h1)));
    let (g2, h2) = (GameId(Uuid::from_u128(5)), HotelId(Uuid::from_u128(6)));
    s.create_game(new_game(g2, guest, h2)).await.unwrap();
    assert_eq!(
        s.latest_hotel_of(guest).await.unwrap(),
        Some((g2, h2)),
        "zadnja igra po games.seq"
    );
}
