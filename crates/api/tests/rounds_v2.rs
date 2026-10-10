mod common;

use serde_json::json;

async fn new_game(body: Option<serde_json::Value>) -> (axum::Router, String, String, String) {
    let (app, _, _) = common::app();
    let r = common::call(&app, "POST", "/v1/demo", None, body).await;
    assert_eq!(r.status, 201, "{:?}", r.body);
    let cookie = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let game = r.body["game_id"].as_str().unwrap().to_owned();
    let hotel = r.body["hotel_id"].as_str().unwrap().to_owned();
    (app, cookie, game, hotel)
}

#[tokio::test]
async fn post_demo_without_body_still_creates_two_month_v1_game() {
    let (app, c, g, _) = new_game(None).await;
    let r = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    assert_eq!(r.body["model"], "v1");
    assert_eq!(r.body["round_kind"], "month");
    assert_eq!(r.body["months"], json!(["jun", "jul"]));
    let codes: Vec<&str> = r.body["rounds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["jun", "jul"]);
    assert_eq!(r.body["current"]["round"]["code"], "jun");
}

#[tokio::test]
async fn post_demo_v2_defaults_to_weekly_full_season() {
    let (app, c, g, _) = new_game(Some(json!({ "model": "v2" }))).await;
    let r = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    assert_eq!(r.body["model"], "v2");
    assert_eq!(r.body["round_kind"], "week");
    assert_eq!(r.body["rounds"].as_array().unwrap().len(), 17);
    assert_eq!(
        r.body["rounds"][4],
        json!({ "code": "t05", "kind": "week", "index": 4, "days": 7, "first_day": 29, "first_month": "jun",
                "last_day": 5, "last_month": "jul", "month_ends": true })
    );
    assert_eq!(r.body["current"]["round"]["code"], "t01");
    assert_eq!(r.body["current"]["month"], "jun");
}

#[tokio::test]
async fn post_demo_v2_monthly_has_four_rounds() {
    let (app, c, g, _) = new_game(Some(json!({ "model": "v2", "round_kind": "month" }))).await;
    let r = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    assert_eq!(r.body["rounds"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn post_demo_v1_with_weeks_is_422_invalid_settings() {
    let (app, _, _) = common::app();
    let r = common::call(
        &app,
        "POST",
        "/v1/demo",
        None,
        Some(json!({ "model": "v1", "round_kind": "week" })),
    )
    .await;
    assert_eq!(r.status, 422);
    assert_eq!(r.body["code"], "invalid_settings");
}

#[tokio::test]
async fn post_demo_with_unknown_field_is_422_invalid_body() {
    let (app, _, _) = common::app();
    let r = common::call(
        &app,
        "POST",
        "/v1/demo",
        None,
        Some(json!({ "modell": "v2" })),
    )
    .await;
    assert_eq!(r.status, 422);
    assert_eq!(r.body["code"], "invalid_body");
}

#[tokio::test]
async fn round_paths_accept_week_codes() {
    let (app, c, g, h) = new_game(Some(json!({ "model": "v2" }))).await;
    let r = common::call(
        &app,
        "GET",
        &format!("/v1/games/{g}/rounds/t01"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(r.status, 200);
    assert_eq!(r.body["round"]["code"], "t01");
    let plan = common::call(&app, "GET", "/v1/me/hotel", Some(&c), None)
        .await
        .body["private"]["plan"]["plan"]
        .clone();
    let r = common::call(
        &app,
        "PUT",
        &format!("/v1/hotels/{h}/plans/t01"),
        Some(&c),
        Some(plan),
    )
    .await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    let r = common::call(
        &app,
        "POST",
        &format!("/v1/games/{g}/rounds/t01/close"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(r.status, 202);
}

#[tokio::test]
async fn week_code_in_monthly_game_is_404() {
    let (app, c, g, _) = new_game(None).await;
    let r = common::call(
        &app,
        "GET",
        &format!("/v1/games/{g}/rounds/t03"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(r.status, 404);
    assert_eq!(r.body["code"], "unknown_month");
}

#[tokio::test]
async fn outdated_v2_game_is_409_game_outdated() {
    let (app, deps, mem) = common::app();
    let r = common::call(
        &app,
        "POST",
        "/v1/demo",
        None,
        Some(json!({ "model": "v2" })),
    )
    .await;
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let g = r.body["game_id"].as_str().unwrap().to_owned();
    mem.patch_first_event(&g, |p| p["core_version"] = json!("1.9.9"));
    let r = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    assert_eq!(r.status, 409);
    assert_eq!(r.body["code"], "game_outdated");
    assert_eq!(
        r.body["title"],
        "Igra je stvorena starijom verzijom igre. Započni novu igru."
    );
    drop(deps);
}
