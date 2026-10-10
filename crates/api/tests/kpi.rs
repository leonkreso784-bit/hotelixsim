//! Sloj 0 — `GET /v1/hotels/{hotel_id}/kpi`: samo obračunati mjeseci, pristup kao dnevnik, isti brojevi kao „Moj hotel".
mod common;
use common::*;
use serde_json::Value;

async fn drain(d: &app::Deps) {
    while app::run_next_job(d).await.unwrap() {}
}

fn uri(hotel: &str) -> String {
    format!("/v1/hotels/{hotel}/kpi")
}

async fn close(app: &axum::Router, d: &app::Deps, tok: &str, game: &str, month: &str) {
    let r = call(
        app,
        "POST",
        &format!("/v1/games/{game}/rounds/{month}/close"),
        Some(tok),
        None,
    )
    .await;
    assert!(r.status.is_success(), "{:?}", r.body);
    drain(d).await;
}

fn int(v: &Value) -> i64 {
    v.as_i64()
        .unwrap_or_else(|| panic!("nije cijeli broj: {v}"))
}

#[tokio::test]
async fn before_first_settlement_there_are_no_months() {
    let (app, _, _) = app();
    let (tok, _, hotel) = demo(&app).await;
    let r = call(&app, "GET", &uri(&hotel), Some(&tok), None).await;
    assert_eq!(r.status.as_u16(), 200, "{:?}", r.body);
    assert_eq!(r.body["months"], serde_json::json!([]));
}

#[tokio::test]
async fn only_settled_months_with_numbers_matching_my_hotel() {
    let (app, d, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    close(&app, &d, &tok, &game, "jun").await;
    let r = call(&app, "GET", &uri(&hotel), Some(&tok), None).await;
    assert_eq!(r.status.as_u16(), 200, "{:?}", r.body);
    let months = r.body["months"].as_array().unwrap();
    assert_eq!(months.len(), 1, "srpanj je otvoren, nije obračunat");
    let k = &months[0];
    assert_eq!(k["month"], "jun");

    let me = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    let lm = &me.body["private"]["last_month"];
    assert_eq!(lm["month"], "jun");
    for (kpi_field, lm_field) in [
        ("occupancy_bp", "occupancy_bp"),
        ("adr_cents", "adr_cents"),
        ("revpar_cents", "revpar_cents"),
        ("goppar_cents", "goppar_cents"),
        ("gop_cents", "gop_cents"),
        ("rooms_revenue_cents", "rooms_revenue_cents"),
        ("sold_nights", "sold"),
    ] {
        assert_eq!(k[kpi_field], lm[lm_field], "{kpi_field}");
    }
    // Sve sobe × dani: 30 lipanjskih dana, izgubljene noći zasebno.
    assert_eq!(int(&k["room_nights"]) % 30, 0);
    assert_eq!(
        int(&k["room_nights"]) - int(&k["lost_nights"]),
        int(&lm["available"])
    );
    let (adr, occ, revpar) = (
        int(&k["adr_cents"]),
        int(&k["occupancy_bp"]),
        int(&k["revpar_cents"]),
    );
    assert!((adr * occ - revpar * 10_000).abs() <= 10_000 + adr);
    for f in [
        "trevpar_cents",
        "gop_margin_bp",
        "labor_share_bp",
        "cpor_cents",
        "break_even_occupancy_bp",
        "total_revenue_cents",
        "labor_cents",
    ] {
        assert!(k[f].is_i64(), "{f}: {}", k[f]);
    }
    // Samo brojke: nema teksta.
    assert!(
        k.as_object()
            .unwrap()
            .values()
            .all(|v| !v.is_string() || v == "jun"),
        "{k}"
    );
}

#[tokio::test]
async fn second_month_is_added_in_season_order() {
    let (app, d, _) = app();
    let (tok, game, hotel) = demo(&app).await;
    close(&app, &d, &tok, &game, "jun").await;
    close(&app, &d, &tok, &game, "jul").await;
    let r = call(&app, "GET", &uri(&hotel), Some(&tok), None).await;
    let months: Vec<&str> = r.body["months"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["month"].as_str().unwrap())
        .collect();
    assert_eq!(months, ["jun", "jul"]);
}

#[tokio::test]
async fn only_own_hotel_like_feed() {
    let (app, d, mem) = app();
    let (tok, game, hotel) = demo(&app).await;
    let (other, _, _) = demo(&app).await;
    close(&app, &d, &tok, &game, "jun").await;
    let u = uri(&hotel);
    assert_problem(
        &call(&app, "GET", &u, None, None).await,
        401,
        "guest_required",
    );
    assert_problem(
        &call(&app, "GET", &u, Some(&other), None).await,
        403,
        "not_in_game",
    );
    let robot = robot_hotel(&mem, &game);
    let feed = call(
        &app,
        "GET",
        &format!("/v1/hotels/{robot}/feed"),
        Some(&tok),
        None,
    )
    .await;
    let r = call(&app, "GET", &uri(&robot), Some(&tok), None).await;
    assert_problem(&r, 403, "not_your_hotel");
    assert_eq!(r.body["code"], feed.body["code"], "isto kao dnevnik");
    let nobody = "00000000-0000-0000-0000-00000000abcd";
    assert_problem(
        &call(&app, "GET", &uri(nobody), Some(&tok), None).await,
        404,
        "unknown_hotel",
    );
    assert_problem(
        &call(&app, "GET", &uri("nije-uuid"), Some(&tok), None).await,
        404,
        "unknown_hotel",
    );
}

#[tokio::test]
async fn openapi_describes_kpi_and_new_last_month_fields() {
    let doc: Value = serde_json::from_str(&api::openapi_json()).unwrap();
    assert!(doc["paths"]["/v1/hotels/{hotel_id}/kpi"]["get"].is_object());
    let s = &doc["components"]["schemas"];
    let required = |name: &str| -> Vec<String> {
        s[name]["required"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}"))
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect()
    };
    for f in ["adr_cents", "break_even_occupancy_bp", "occupancy_bp"] {
        assert!(required("KpiMonthBody").contains(&f.to_owned()), "{f}");
    }
    for f in ["occupancy_bp", "adr_cents", "revpar_cents"] {
        assert!(required("LastMonthBody").contains(&f.to_owned()), "{f}");
    }
}
