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
        ai: Arc::new(app::DisabledAi),
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
    let jul_close = call(
        &app,
        "POST",
        &format!("/v1/games/{game}/rounds/jul/close"),
        Some(&tok),
        None,
    )
    .await;
    assert_eq!(jul_close.status.as_u16(), 202, "{:?}", jul_close.body);
    run_worker(&d).await;
    // ljestvica i feed
    let g = call(&app, "GET", &format!("/v1/games/{game}"), Some(&tok), None).await;
    assert!(g.body["current"].is_null(), "igra je gotova");
    let hotels = g.body["hotels"].as_array().unwrap();
    assert_eq!(hotels.len(), 4);
    for h in hotels {
        let months = h["months"].as_array().unwrap();
        assert_eq!(months.len(), 2, "{h}");
        assert_eq!(months[0]["month"], "jun");
        assert_eq!(months[1]["month"], "jul");
        for m in months {
            assert_eq!(m["indices"].as_array().unwrap().len(), 5, "{h}");
        }
    }
    // roboti s različitim strategijama moraju se razlikovati (nulti ili isti obračun bi pao)
    for mi in 0..2 {
        let distinct: std::collections::BTreeSet<String> = hotels
            .iter()
            .map(|h| {
                format!(
                    "{}|{}",
                    h["months"][mi]["indices"], h["months"][mi]["progress_hundredths"]
                )
            })
            .collect();
        assert!(
            distinct.len() >= 3,
            "mjesec {mi}: hoteli su gotovo isti: {distinct:?}"
        );
        assert!(
            hotels.iter().any(|h| h["months"][mi]["indices"]
                .as_array()
                .unwrap()
                .iter()
                .any(|i| i["hundredths"] != 0)),
            "svi indeksi su nula u mjesecu {mi}"
        );
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
    let items = f.body["items"].as_array().unwrap();
    let months_resolved: Vec<&str> = items
        .iter()
        .filter(|i| i["kind"] == "month_resolved")
        .map(|i| i["month"].as_str().unwrap())
        .collect();
    assert_eq!(months_resolved, ["jun", "jul"]);
    // neodgovorene situacije: S3 u lipnju i S2 u srpnju
    let ignored: Vec<(&str, &str)> = items
        .iter()
        .filter(|i| i["kind"] == "situation_ignored")
        .map(|i| {
            (
                i["month"].as_str().unwrap(),
                i["situation"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(ignored, [("jun", "S3"), ("jul", "S2")]);
    // pohranjeni obračun igračeva hotela jednak je onome što API pokazuje, a plan i ishodi su njegovi
    let stored: Vec<(String, serde_json::Value)> = sqlx::query_as(
        "SELECT payload->>'month', payload FROM events WHERE type = 'RoundResolved' ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(stored.len(), 2);
    for (mi, (month, payload)) in stored.iter().enumerate() {
        let mine = payload["hotels"]
            .as_array()
            .unwrap()
            .iter()
            .find(|h| h["hotel_id"] == hotel.as_str())
            .expect("igračev hotel u obračunu");
        let api_hotel = hotels
            .iter()
            .find(|h| h["hotel_id"] == hotel.as_str())
            .unwrap();
        assert_eq!(api_hotel["months"][mi]["month"], month.as_str());
        assert_eq!(
            api_hotel["months"][mi]["progress_hundredths"],
            mine["progress_hundredths"]
        );
        let api_idx: Vec<_> = api_hotel["months"][mi]["indices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["hundredths"].clone())
            .collect();
        assert_eq!(json!(api_idx), mine["indices"]);
        let un = mine["situations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["answer"].is_null())
            .expect("neodgovorena situacija");
        assert_eq!(un["quality"], 1);
        assert_eq!(un["code"], if mi == 0 { "S3" } else { "S2" });
        if mi == 0 {
            assert_eq!(
                mine["plan"]["marketing_cents"], 500_000,
                "plan igrača je obračunat"
            );
            let s1 = mine["situations"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["code"] == "S1")
                .unwrap();
            assert_eq!(s1["answer"], 0);
        }
    }
    // roboti su djelovali u oba mjeseca: PlanSet i AnswerChosen za svaku strategiju
    for robot in ["stedisa", "brizni", "lovac"] {
        for (kind, month) in [
            ("PlanSet", "jun"),
            ("PlanSet", "jul"),
            ("AnswerChosen", "jun"),
            ("AnswerChosen", "jul"),
        ] {
            let c: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM events WHERE type = $1 AND actor = $2 AND payload->>'month' = $3",
            )
            .bind(kind)
            .bind(format!("robot:{robot}"))
            .bind(month)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(c >= 1, "{robot} nema {kind} za {month}");
        }
    }
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
