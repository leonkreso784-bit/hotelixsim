mod common;
use common::*;
use serde_json::json;

#[tokio::test]
async fn game_view_lists_four_hotels_and_open_june() {
    let (app, _, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    let r = call(&app, "GET", &format!("/v1/games/{game}"), Some(&tok), None).await;
    assert_eq!(r.status.as_u16(), 200);
    assert_eq!(r.body["months"], json!(["jun", "jul"]));
    assert_eq!(
        r.body["current"],
        json!({"month": "jun", "status": "open", "unanswered": 2,
               "round": {"code": "jun", "kind": "month", "index": 0, "days": 30, "first_day": 1,
                         "first_month": "jun", "last_day": 30, "last_month": "jun", "month_ends": true}})
    );
    assert_eq!(r.body["my_hotel_id"], hotel.as_str());
    assert_eq!(r.body["hotels"].as_array().unwrap().len(), 4);
    assert!(
        r.body["hotels"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h.get("private").is_none())
    );
}

#[tokio::test]
async fn plan_put_validates_and_is_visible() {
    let (app, _, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    let me = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    let mut plan = me.body["private"]["plan"]["plan"].clone();
    plan["marketing_cents"] = json!(500_000);
    plan["training"] = json!("basic");
    let uri = format!("/v1/hotels/{hotel}/plans/jun");
    let r = call(&app, "PUT", &uri, Some(&tok), Some(plan.clone())).await;
    assert_eq!((r.status.as_u16(), &r.body), (200, &plan));
    let me = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    assert_eq!(
        me.body["private"]["plan"]["plan"], plan,
        "zadnja promjena vrijedi"
    );
    let mut bad = plan.clone();
    bad["price_cents"] = json!(plan["price_cents"].as_i64().unwrap() + 50);
    assert_problem(
        &call(&app, "PUT", &uri, Some(&tok), Some(bad)).await,
        422,
        "price_not_whole_euro",
    );
    assert_problem(
        &call(
            &app,
            "PUT",
            &format!("/v1/hotels/{hotel}/plans/aug"),
            Some(&tok),
            Some(plan.clone()),
        )
        .await,
        404,
        "unknown_month",
    );
    assert_problem(
        &call(
            &app,
            "PUT",
            &format!("/v1/hotels/{hotel}/plans/xyz"),
            Some(&tok),
            Some(plan),
        )
        .await,
        404,
        "unknown_month",
    );
}

#[tokio::test]
async fn bad_plan_body_is_422_problem() {
    let (app, _, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    let uri = format!("/v1/hotels/{hotel}/plans/jun");
    let me = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    let mut plan = me.body["private"]["plan"]["plan"].clone();
    plan["training"] = json!("mega");
    assert_problem(
        &call(&app, "PUT", &uri, Some(&tok), Some(plan)).await,
        422,
        "invalid_body",
    );
    assert_problem(
        &call(
            &app,
            "PUT",
            &uri,
            Some(&tok),
            Some(json!({"price_cents": 1})),
        )
        .await,
        422,
        "invalid_body",
    );
}

#[tokio::test]
async fn situations_hide_quality_and_answer_once() {
    let (app, _, mem) = app();
    let (tok, game, hotel) = demo(&app).await;
    let uri = format!("/v1/hotels/{hotel}/situations");
    let r = call(&app, "GET", &uri, Some(&tok), None).await;
    assert_eq!(r.body["month"], "jun");
    assert_eq!(r.body["unanswered"], 2);
    assert_eq!(r.body["items"].as_array().unwrap().len(), 2);
    assert!(!r.body.to_string().contains("quality") && !r.body.to_string().contains("mechanics"));
    let a = call(
        &app,
        "POST",
        &format!("{uri}/S1/answer"),
        Some(&tok),
        Some(json!({"answer": 0})),
    )
    .await;
    assert_eq!(a.body["items"][0]["chosen"], 0);
    assert_eq!(a.body["unanswered"], 1);
    assert_problem(
        &call(
            &app,
            "POST",
            &format!("{uri}/S1/answer"),
            Some(&tok),
            Some(json!({"answer": 1})),
        )
        .await,
        409,
        "already_answered",
    );
    assert_problem(
        &call(
            &app,
            "POST",
            &format!("{uri}/S3/answer"),
            Some(&tok),
            Some(json!({"answer": 9})),
        )
        .await,
        422,
        "unknown_answer",
    );
    assert_problem(
        &call(
            &app,
            "POST",
            &format!("{uri}/S9/answer"),
            Some(&tok),
            Some(json!({"answer": 0})),
        )
        .await,
        404,
        "unknown_situation",
    );
    let robot = robot_hotel(&mem, &game);
    assert_problem(
        &call(
            &app,
            "GET",
            &format!("/v1/hotels/{robot}/situations"),
            Some(&tok),
            None,
        )
        .await,
        403,
        "not_your_hotel",
    );
}

#[tokio::test]
async fn close_twice_is_202_then_200_and_worker_resolves() {
    let (app, d, _) = app();
    let (tok, game, _) = demo(&app).await;
    let round = format!("/v1/games/{game}/rounds/jun");
    assert_eq!(
        call(&app, "GET", &round, Some(&tok), None).await.body["unanswered"],
        2
    );
    let first = call(&app, "POST", &format!("{round}/close"), Some(&tok), None).await;
    assert_eq!(
        (first.status.as_u16(), &first.body["status"]),
        (202, &json!("closing"))
    );
    let second = call(&app, "POST", &format!("{round}/close"), Some(&tok), None).await;
    assert_eq!(
        (second.status.as_u16(), &second.body["status"]),
        (200, &json!("closing"))
    );
    run_worker(&d).await;
    assert_eq!(
        call(&app, "GET", &round, Some(&tok), None).await.body["status"],
        "resolved"
    );
    let g = call(&app, "GET", &format!("/v1/games/{game}"), Some(&tok), None).await;
    assert_eq!(g.body["current"]["month"], "jul");
    assert_problem(
        &call(
            &app,
            "POST",
            &format!("/v1/games/{game}/rounds/aug/close"),
            Some(&tok),
            None,
        )
        .await,
        404,
        "unknown_month",
    );
}

#[tokio::test]
async fn plan_for_closed_month_is_409() {
    let (app, _, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    let plan = call(&app, "GET", "/v1/me/hotel", Some(&tok), None)
        .await
        .body["private"]["plan"]["plan"]
        .clone();
    call(
        &app,
        "POST",
        &format!("/v1/games/{game}/rounds/jun/close"),
        Some(&tok),
        None,
    )
    .await;
    assert_problem(
        &call(
            &app,
            "PUT",
            &format!("/v1/hotels/{hotel}/plans/jun"),
            Some(&tok),
            Some(plan),
        )
        .await,
        409,
        "round_not_open",
    );
    assert_problem(
        &call(
            &app,
            "POST",
            &format!("/v1/hotels/{hotel}/situations/S1/answer"),
            Some(&tok),
            Some(json!({"answer": 0})),
        )
        .await,
        409,
        "round_not_open",
    );
}

#[tokio::test]
async fn feed_is_owner_only_and_tells_the_month() {
    let (app, d, mem) = app();
    let (tok, game, hotel) = demo(&app).await;
    call(
        &app,
        "POST",
        &format!("/v1/hotels/{hotel}/situations/S1/answer"),
        Some(&tok),
        Some(json!({"answer": 0})),
    )
    .await;
    call(
        &app,
        "POST",
        &format!("/v1/games/{game}/rounds/jun/close"),
        Some(&tok),
        None,
    )
    .await;
    run_worker(&d).await;
    let f = call(
        &app,
        "GET",
        &format!("/v1/hotels/{hotel}/feed"),
        Some(&tok),
        None,
    )
    .await;
    let kinds: Vec<&str> = f.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds[0], "hotel_received");
    for k in [
        "answer_chosen",
        "round_closed",
        "situation_ignored",
        "month_resolved",
        "change",
    ] {
        assert!(kinds.contains(&k), "{k}");
    }
    let robot = robot_hotel(&mem, &game);
    assert_problem(
        &call(
            &app,
            "GET",
            &format!("/v1/hotels/{robot}/feed"),
            Some(&tok),
            None,
        )
        .await,
        403,
        "not_your_hotel",
    );
}

#[tokio::test]
async fn unknown_ids_are_404_and_anonymous_is_401() {
    let (app, _, _) = app();
    let (tok, _, _) = demo(&app).await;
    let big = "00000000-0000-0000-0000-000000009999";
    assert_problem(
        &call(&app, "GET", &format!("/v1/games/{big}"), Some(&tok), None).await,
        404,
        "unknown_game",
    );
    assert_problem(
        &call(
            &app,
            "GET",
            &format!("/v1/hotels/{big}/feed"),
            Some(&tok),
            None,
        )
        .await,
        404,
        "unknown_hotel",
    );
    assert_problem(
        &call(&app, "GET", &format!("/v1/games/{big}"), None, None).await,
        401,
        "guest_required",
    );
}

const BIG: &str = "00000000-0000-0000-0000-000000009999";

/// Svih sedam novih ruta: (metoda, putanja s {g} igrom ili {h} hotelom, ima li tijelo).
fn routes(g: &str, h: &str) -> Vec<(&'static str, String, Option<serde_json::Value>)> {
    let plan = json!({"price_cents": 1});
    vec![
        ("GET", format!("/v1/games/{g}"), None),
        ("GET", format!("/v1/games/{g}/rounds/jun"), None),
        ("PUT", format!("/v1/hotels/{h}/plans/jun"), Some(plan)),
        ("GET", format!("/v1/hotels/{h}/situations"), None),
        (
            "POST",
            format!("/v1/hotels/{h}/situations/S1/answer"),
            Some(json!({"answer": 0})),
        ),
        ("POST", format!("/v1/games/{g}/rounds/jun/close"), None),
        ("GET", format!("/v1/hotels/{h}/feed"), None),
    ]
}

#[tokio::test]
async fn every_new_route_checks_access_in_order() {
    let (app, _, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    let (_, other_game, other_hotel) = demo(&app).await;
    for (m, uri, body) in routes(&game, &hotel) {
        let r = call(&app, m, &uri, None, body).await;
        assert_problem(&r, 401, "guest_required");
    }
    for (m, uri, body) in routes(BIG, BIG) {
        let r = call(&app, m, &uri, Some(&tok), body).await;
        assert_eq!(r.status.as_u16(), 404, "{m} {uri}");
        assert_eq!(r.ctype.as_deref(), Some("application/problem+json"));
    }
    // Tuđa igra i tuđi hotel (drugi gost) -> 403.
    for (m, uri, body) in routes(&other_game, &other_hotel) {
        let r = call(&app, m, &uri, Some(&tok), body).await;
        assert_eq!(r.status.as_u16(), 403, "{m} {uri}");
    }
}

#[tokio::test]
async fn bad_body_never_beats_access_checks() {
    let (app, _, mem) = app();
    let (tok, game, hotel) = demo(&app).await;
    let (_, _, other_hotel) = demo(&app).await;
    let robot = robot_hotel(&mem, &game);
    let bad = [
        ("PUT", "plans/jun", json!({"nope": 1})),
        ("POST", "situations/S1/answer", json!({"answer": "x"})),
    ];
    for (m, tail, body) in bad {
        let u = |h: &str| format!("/v1/hotels/{h}/{tail}");
        assert_problem(
            &call(&app, m, &u(&hotel), None, Some(body.clone())).await,
            401,
            "guest_required",
        );
        assert_problem(
            &call(&app, m, &u(BIG), Some(&tok), Some(body.clone())).await,
            404,
            "unknown_hotel",
        );
        assert_problem(
            &call(&app, m, &u(&other_hotel), Some(&tok), Some(body.clone())).await,
            403,
            "not_in_game",
        );
        assert_problem(
            &call(&app, m, &u(&robot), Some(&tok), Some(body.clone())).await,
            403,
            "not_your_hotel",
        );
        assert_problem(
            &call(&app, m, &u(&hotel), Some(&tok), Some(body)).await,
            422,
            "invalid_body",
        );
    }
}

#[tokio::test]
async fn plan_for_scheduled_july_is_409() {
    let (app, _, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    let plan = call(&app, "GET", "/v1/me/hotel", Some(&tok), None)
        .await
        .body["private"]["plan"]["plan"]
        .clone();
    let uri = format!("/v1/hotels/{hotel}/plans/jul");
    assert_problem(
        &call(&app, "PUT", &uri, Some(&tok), Some(plan)).await,
        409,
        "round_not_open",
    );
}

#[tokio::test]
async fn close_after_resolved_is_200_and_stale_answer_is_rejected() {
    let (app, d, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    let close = format!("/v1/games/{game}/rounds/jun/close");
    assert_eq!(
        call(&app, "POST", &close, Some(&tok), None)
            .await
            .status
            .as_u16(),
        202
    );
    run_worker(&d).await;
    let again = call(&app, "POST", &close, Some(&tok), None).await;
    assert_eq!(
        (again.status.as_u16(), &again.body["status"]),
        (200, &json!("resolved"))
    );
    // Zastarjeli lipanjski ekran: odgovor na lipanjsku situaciju kad je srpanj otvoren.
    let stale = format!("/v1/hotels/{hotel}/situations/S1/answer");
    let r = call(&app, "POST", &stale, Some(&tok), Some(json!({"answer": 0}))).await;
    assert_problem(&r, 409, "round_not_open");
    let s = call(
        &app,
        "GET",
        &format!("/v1/hotels/{hotel}/situations"),
        Some(&tok),
        None,
    )
    .await;
    assert_eq!(s.body["month"], "jul");
    assert!(
        s.body["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["code"] != "S1")
    );
    let f = call(
        &app,
        "GET",
        &format!("/v1/hotels/{hotel}/feed"),
        Some(&tok),
        None,
    )
    .await;
    let chosen = f.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["kind"] == "answer_chosen")
        .count();
    assert_eq!(chosen, 0, "ništa nije zapisano");
}
