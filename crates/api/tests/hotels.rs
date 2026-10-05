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
