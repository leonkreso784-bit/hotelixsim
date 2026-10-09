mod common;
use common::*;

#[tokio::test]
async fn demo_sets_httponly_cookie_once() {
    let (app, _, _) = app();
    let r = call(&app, "POST", "/v1/demo", None, None).await;
    let c = r.set_cookie.clone().unwrap();
    for part in [
        "hotelixsim_guest=",
        "HttpOnly",
        "SameSite=Lax",
        "Path=/",
        "Max-Age=604800",
    ] {
        assert!(c.contains(part), "{c}");
    }
    let tok = c
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("hotelixsim_guest=")
        .unwrap();
    let again = call(&app, "POST", "/v1/demo", Some(tok), None).await;
    assert_eq!(again.status.as_u16(), 201);
    assert_eq!(
        again.set_cookie, None,
        "postojeći gost ne dobiva novi kolačić"
    );
    assert_ne!(again.body["game_id"], r.body["game_id"]);
}

#[tokio::test]
async fn me_hotel_shows_private_numbers_and_june_plan() {
    let (app, _, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    let r = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    assert_eq!(r.status.as_u16(), 200);
    assert_eq!(r.body["game_id"], game.as_str());
    assert_eq!(r.body["hotel"]["hotel_id"], hotel.as_str());
    assert_eq!(r.body["hotel"]["seat"]["kind"], "human");
    assert!(r.body["private"]["now"]["cash_cents"].is_i64());
    assert_eq!(r.body["private"]["plan"]["month"], "jun");
    assert_eq!(r.body["hotel"]["start"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn foreign_and_anonymous_requests() {
    let (app, _, mem) = app();
    let (tok, game, hotel) = demo(&app).await;
    let (other, _, _) = demo(&app).await;
    assert_problem(
        &call(&app, "GET", "/v1/me/hotel", None, None).await,
        401,
        "guest_required",
    );
    assert_problem(
        &call(&app, "GET", "/v1/me/hotel", Some("nije-token"), None).await,
        401,
        "guest_required",
    );
    let uri = format!("/v1/hotels/{hotel}");
    assert_problem(
        &call(&app, "GET", &uri, Some(&other), None).await,
        403,
        "not_in_game",
    );
    let robot = call(
        &app,
        "GET",
        &format!("/v1/hotels/{}", robot_hotel(&mem, &game)),
        Some(&tok),
        None,
    )
    .await;
    assert_eq!(robot.status.as_u16(), 200);
    assert!(
        robot.body["private"].is_null(),
        "tuđi hotel bez stvarnih brojki"
    );
    assert!(robot.body["hotel"]["start"].is_array());
}

#[tokio::test]
async fn unknown_ids_and_routes_are_problem_404() {
    let (app, _, _) = app();
    let (tok, _, _) = demo(&app).await;
    let nobody = "00000000-0000-0000-0000-00000000abcd";
    assert_problem(
        &call(
            &app,
            "GET",
            &format!("/v1/hotels/{nobody}"),
            Some(&tok),
            None,
        )
        .await,
        404,
        "unknown_hotel",
    );
    assert_problem(
        &call(&app, "GET", "/v1/hotels/nije-uuid", Some(&tok), None).await,
        404,
        "unknown_hotel",
    );
    assert_problem(
        &call(&app, "GET", "/v1/nema-me", None, None).await,
        404,
        "unknown_route",
    );
}

async fn drain(d: &app::Deps) {
    while app::run_next_job(d).await.unwrap() {}
}

fn msg(h: &str, s: &str) -> String {
    format!("/v1/hotels/{h}/situations/{s}/messages")
}

fn text(t: &str) -> Option<serde_json::Value> {
    Some(serde_json::json!({ "text": t }))
}

#[tokio::test]
async fn message_to_character_is_accepted_and_awaits_reply() {
    let (app, _, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    let r = call(
        &app,
        "POST",
        &msg(&hotel, "S1"),
        Some(&tok),
        text("Dobar dan."),
    )
    .await;
    assert_eq!(r.status.as_u16(), 202, "{:?}", r.body);
    assert_eq!(r.body["awaiting_reply"], true);
    assert_eq!(r.body["remaining"], 5);
    let again = call(
        &app,
        "POST",
        &msg(&hotel, "S1"),
        Some(&tok),
        text("Još jedna."),
    )
    .await;
    assert_problem(&again, 409, "awaiting_reply");
    let conv = call(
        &app,
        "GET",
        &format!("/v1/hotels/{hotel}/situations/S1/conversation"),
        Some(&tok),
        None,
    )
    .await;
    assert_eq!(conv.status.as_u16(), 200);
    assert_eq!(conv.body["lines"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn seventh_message_hits_chat_limit() {
    let (app, d, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    for i in 0..6 {
        let r = call(
            &app,
            "POST",
            &msg(&hotel, "S1"),
            Some(&tok),
            text(&format!("Poruka {i}")),
        )
        .await;
        assert_eq!(r.status.as_u16(), 202, "{:?}", r.body);
        drain(&d).await;
    }
    let r = call(&app, "POST", &msg(&hotel, "S1"), Some(&tok), text("Sedma")).await;
    assert_problem(&r, 409, "chat_limit");
}

#[tokio::test]
async fn free_response_validation_and_conflicts() {
    let (app, _, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    let resp = |s: &str| format!("/v1/hotels/{hotel}/situations/{s}/response");
    assert_problem(
        &call(
            &app,
            "POST",
            &resp("S3"),
            Some(&tok),
            text("Svojim riječima"),
        )
        .await,
        422,
        "not_free_text",
    );
    assert_problem(
        &call(&app, "POST", &resp("S1"), Some(&tok), text("")).await,
        422,
        "bad_text",
    );
    assert_problem(
        &call(
            &app,
            "POST",
            &resp("S1"),
            Some(&tok),
            text(&"a".repeat(2001)),
        )
        .await,
        422,
        "bad_text",
    );
    let ok = call(&app, "POST", &resp("S1"), Some(&tok), text("Ispričam se.")).await;
    assert_eq!(ok.status.as_u16(), 202, "{:?}", ok.body);
    let s1 = &ok.body["items"][0];
    assert_eq!(s1["code"], "S1");
    assert_eq!(s1["response"]["status"], "pending");
    let offered = call(
        &app,
        "POST",
        &format!("/v1/hotels/{hotel}/situations/S1/answer"),
        Some(&tok),
        Some(serde_json::json!({"answer": 0})),
    )
    .await;
    assert_problem(&offered, 409, "already_answered");
}

#[tokio::test]
async fn new_routes_check_access_before_body() {
    let (app, _, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    let (other, _, _) = demo(&app).await;
    for (method, tail, body) in [
        ("POST", "messages", text("x")),
        ("GET", "conversation", None),
        ("POST", "response", text("x")),
    ] {
        let uri = format!("/v1/hotels/{hotel}/situations/S1/{tail}");
        assert_problem(
            &call(&app, method, &uri, None, body.clone()).await,
            401,
            "guest_required",
        );
        assert_problem(
            &call(&app, method, &uri, Some(&other), body).await,
            403,
            "not_in_game",
        );
    }
    // loše tijelo tuđem gostu ne otkriva 422
    let bad = call(
        &app,
        "POST",
        &msg(&hotel, "S1"),
        Some(&other),
        Some(serde_json::json!({"nije": 1})),
    )
    .await;
    assert_problem(&bad, 403, "not_in_game");
    let own_bad = call(
        &app,
        "POST",
        &msg(&hotel, "S1"),
        Some(&tok),
        Some(serde_json::json!({"nije": 1})),
    )
    .await;
    assert_problem(&own_bad, 422, "invalid_body");
}

#[tokio::test]
async fn judged_response_shows_five_criteria() {
    let (app, d, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    call(
        &app,
        "POST",
        &format!("/v1/hotels/{hotel}/situations/S1/response"),
        Some(&tok),
        text("Ispričam se i premjestim obitelj."),
    )
    .await;
    drain(&d).await;
    let s = call(
        &app,
        "GET",
        &format!("/v1/hotels/{hotel}/situations"),
        Some(&tok),
        None,
    )
    .await;
    let item = s.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["code"] == "S1")
        .unwrap();
    assert_eq!(item["free_text"], true);
    assert!(item["character"]["name"].is_string());
    assert_eq!(item["response"]["status"], "judged");
    let crit = item["response"]["criteria"].as_array().unwrap();
    assert_eq!(crit.len(), 5);
    assert!(
        crit.iter()
            .all(|c| c["text"].as_str().is_some_and(|t| !t.is_empty()))
    );
    let s3 = s.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["code"] == "S3")
        .unwrap();
    assert_eq!(s3["free_text"], false);
    assert!(s3["character"].is_null() && s3["response"].is_null());
}

#[tokio::test]
async fn feed_and_last_month_carry_verdict_and_mentor() {
    let (app, d, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    call(
        &app,
        "POST",
        &format!("/v1/hotels/{hotel}/situations/S1/response"),
        Some(&tok),
        text("Ispričam se."),
    )
    .await;
    drain(&d).await;
    let close = call(
        &app,
        "POST",
        &format!("/v1/games/{game}/rounds/jun/close"),
        Some(&tok),
        None,
    )
    .await;
    assert!(close.status.is_success(), "{:?}", close.body);
    drain(&d).await;
    let feed = call(
        &app,
        "GET",
        &format!("/v1/hotels/{hotel}/feed"),
        Some(&tok),
        None,
    )
    .await;
    let kinds: Vec<&str> = feed.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["kind"].as_str().unwrap())
        .collect();
    for k in ["response", "verdict", "mentor"] {
        assert!(kinds.contains(&k), "{k} nedostaje: {kinds:?}");
    }
    let me = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    assert!(
        me.body["private"]["last_month"]["mentor"].is_string(),
        "{:?}",
        me.body["private"]
    );
}
