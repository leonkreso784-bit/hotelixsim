//! Kraj do kraja nad pravim Postgresom: gost igra lipanj i srpanj protiv triju robota (ARHITEKTURA §9).
mod common;
use std::sync::Arc;

use api::{AppState, router};
use common::{assert_problem, call};
use content::{Snapshot, repo_dir};
use game::Game;
use serde_json::json;
use sqlx::PgPool;

fn deps(pool: PgPool) -> app::Deps {
    let s = Arc::new(store::Store::from_pool(pool));
    app::Deps {
        status: s.clone(),
        log: s.clone(),
        guests: s.clone(),
        jobs: s,
        ids: Arc::new(app::RandomIds),
        snapshot: Arc::new(Snapshot::read(&repo_dir()).unwrap()),
    }
}

async fn run_worker(d: &app::Deps) {
    while app::run_next_job(d).await.unwrap() {}
}

fn token(set_cookie: &str) -> String {
    set_cookie
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("hotelixsim_guest=")
        .unwrap()
        .to_owned()
}

#[sqlx::test(migrations = "../store/migrations")]
async fn guest_plays_june_and_july_against_three_robots(pool: PgPool) {
    let d = deps(pool.clone());
    let app = router(AppState { deps: d.clone() });
    // gost → igra
    let r = call(&app, "POST", "/v1/demo", None, None).await;
    let tok = token(&r.set_cookie.unwrap());
    let (game, hotel) = (
        r.body["game_id"].as_str().unwrap().to_owned(),
        r.body["hotel_id"].as_str().unwrap().to_owned(),
    );
    // lipanj: plan + jedan odgovor, S3 ostaje neodgovorena
    let me = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    let mut plan = me.body["private"]["plan"]["plan"].clone();
    plan["marketing_cents"] = json!(500_000);
    let put = call(
        &app,
        "PUT",
        &format!("/v1/hotels/{hotel}/plans/jun"),
        Some(&tok),
        Some(plan),
    )
    .await;
    assert_eq!(put.status.as_u16(), 200, "{:?}", put.body);
    let ans = call(
        &app,
        "POST",
        &format!("/v1/hotels/{hotel}/situations/S1/answer"),
        Some(&tok),
        Some(json!({"answer": 0})),
    )
    .await;
    assert!(ans.status.is_success(), "{:?}", ans.body);
    let rd = call(
        &app,
        "GET",
        &format!("/v1/games/{game}/rounds/jun"),
        Some(&tok),
        None,
    )
    .await;
    assert_eq!(rd.body["unanswered"], 1, "{:?}", rd.body);
    let close = format!("/v1/games/{game}/rounds/jun/close");
    assert_eq!(
        call(&app, "POST", &close, Some(&tok), None)
            .await
            .status
            .as_u16(),
        202
    );
    assert_eq!(
        call(&app, "POST", &close, Some(&tok), None)
            .await
            .status
            .as_u16(),
        200
    );
    run_worker(&d).await;
    // srpanj
    let g = call(&app, "GET", &format!("/v1/games/{game}"), Some(&tok), None).await;
    assert_eq!(
        g.body["current"],
        json!({"month": "jul", "status": "open", "unanswered": 1}),
        "{:?}",
        g.body
    );
    let s = call(
        &app,
        "GET",
        &format!("/v1/hotels/{hotel}/situations"),
        Some(&tok),
        None,
    )
    .await;
    assert_eq!(s.body["items"][0]["code"], "S2", "{:?}", s.body);
    call(
        &app,
        "POST",
        &format!("/v1/games/{game}/rounds/jul/close"),
        Some(&tok),
        None,
    )
    .await;
    run_worker(&d).await;
    // ljestvica i feed
    let g = call(&app, "GET", &format!("/v1/games/{game}"), Some(&tok), None).await;
    assert!(g.body["current"].is_null(), "igra je gotova");
    for h in g.body["hotels"].as_array().unwrap() {
        assert_eq!(h["months"].as_array().unwrap().len(), 2, "{h}");
    }
    let f = call(
        &app,
        "GET",
        &format!("/v1/hotels/{hotel}/feed"),
        Some(&tok),
        None,
    )
    .await;
    let resolved = f.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["kind"] == "month_resolved")
        .count();
    assert_eq!(resolved, 2);
    assert_problem(
        &call(
            &app,
            "PUT",
            &format!("/v1/hotels/{hotel}/plans/jul"),
            Some(&tok),
            Some(me.body["private"]["plan"]["plan"].clone()),
        )
        .await,
        409,
        "round_not_open",
    );
    // baza: dva obračuna, nijedan posao nije ostao
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE type = 'RoundResolved'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 2);
    let open: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs WHERE status <> 'done'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(open, 0);
}

#[sqlx::test(migrations = "../store/migrations")]
async fn replay_from_database_matches(pool: PgPool) {
    let d = deps(pool);
    let app = router(AppState { deps: d.clone() });
    let r = call(&app, "POST", "/v1/demo", None, None).await;
    let tok = token(&r.set_cookie.unwrap());
    let game = app::GameId(r.body["game_id"].as_str().unwrap().parse().unwrap());
    call(
        &app,
        "POST",
        &format!("/v1/games/{}/rounds/jun/close", game.0),
        Some(&tok),
        None,
    )
    .await;
    run_worker(&d).await;
    // pročitaj iz baze (JSONB je promijenio redoslijed ključeva), obračunaj lipanj ponovno i usporedi tipizirano
    let raw = d.log.load(game).await.unwrap();
    let recs: Vec<game::Recorded> = raw
        .iter()
        .map(|r| game::Recorded {
            seq: r.seq,
            actor: game::Actor::parse(&r.actor).unwrap(),
            event: game::GameEvent::from_record(&r.kind, r.schema_ver, r.payload.clone()).unwrap(),
        })
        .collect();
    let at = recs
        .iter()
        .position(|r| r.event.kind() == "RoundResolved")
        .unwrap();
    let again = Game::fold(&recs[..at])
        .unwrap()
        .resolve_round(game::dto::MonthCode::Jun.into())
        .unwrap();
    let game::GameEvent::RoundResolved(stored) = &recs[at].event else {
        panic!("očekivan RoundResolved")
    };
    assert_eq!(
        serde_json::to_vec(&again).unwrap(),
        serde_json::to_vec(stored).unwrap()
    );
}
