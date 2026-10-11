mod common;

use serde_json::json;

async fn v2_after_first_week() -> (axum::Router, String, String, String) {
    let (app, deps, _) = common::app();
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
    let h = r.body["hotel_id"].as_str().unwrap().to_owned();
    let close = common::call(
        &app,
        "POST",
        &format!("/v1/games/{g}/rounds/t01/close"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(close.status, 202);
    common::run_worker(&deps).await; // Task 7: premješten iz e2e.rs/games.rs u common/mod.rs
    (app, c, g, h)
}

#[tokio::test]
async fn departments_list_has_rooms_with_last_round() {
    let (app, c, _, h) = v2_after_first_week().await;
    let r = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{h}/departments"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    assert_eq!(r.body["round"], "t01");
    let rooms = r.body["departments"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["code"] == "rooms")
        .unwrap();
    assert_eq!(rooms["name"], "Smještaj");
    assert_eq!(rooms["class"], "rooms");
    assert!(rooms["last"]["revenue_cents"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn department_detail_and_unknown_department() {
    let (app, c, _, h) = v2_after_first_week().await;
    let r = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{h}/departments/restaurant"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(r.status, 200);
    assert_eq!(r.body["history"].as_array().unwrap().len(), 1);
    let r = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{h}/departments/casino"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (404, Some("unknown_department"))
    );
}

#[tokio::test]
async fn pl_has_sections_in_usali_order_and_aligned_lines() {
    let (app, c, _, h) = v2_after_first_week().await;
    let r = common::call(&app, "GET", &format!("/v1/hotels/{h}/pl"), Some(&c), None).await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    let sections: Vec<&str> = r.body["sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["section"].as_str().unwrap())
        .collect();
    assert_eq!(
        sections,
        [
            "revenue",
            "departmental_expenses",
            "departmental_profit",
            "undistributed",
            "gop",
            "fixed_charges",
            "ebitda"
        ]
    );
    for s in r.body["sections"].as_array().unwrap() {
        for l in s["lines"].as_array().unwrap() {
            assert_eq!(
                l["variance_cents"].as_i64().unwrap(),
                l["actual_cents"].as_i64().unwrap() - l["budget_cents"].as_i64().unwrap()
            );
            assert!(!l["name"].as_str().unwrap().is_empty());
        }
    }
    assert_eq!(r.body["rounds"], json!(["t01"]));
}

#[tokio::test]
async fn pl_before_any_resolution_is_409() {
    let (app, _, _) = common::app();
    let r = common::call(
        &app,
        "POST",
        "/v1/demo",
        None,
        Some(json!({ "model": "v2" })),
    )
    .await;
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let h = r.body["hotel_id"].as_str().unwrap().to_owned();
    let r = common::call(&app, "GET", &format!("/v1/hotels/{h}/pl"), Some(&c), None).await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (409, Some("no_resolved_round"))
    );
}

#[tokio::test]
async fn v1_game_has_no_departments() {
    let (app, _, _) = common::app();
    let r = common::call(&app, "POST", "/v1/demo", None, None).await;
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let h = r.body["hotel_id"].as_str().unwrap().to_owned();
    let r = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{h}/departments"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (409, Some("model_v1"))
    );
}

#[tokio::test]
async fn other_hotel_departments_are_forbidden() {
    let (app, c, g, _) = v2_after_first_week().await;
    let game = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    let other = game.body["hotels"][1]["hotel_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let r = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{other}/departments"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(r.status, 403);
}

#[tokio::test]
async fn plan_with_departments_round_trips_and_bad_closing_is_422() {
    let (app, c, _, h) = v2_after_first_week().await;
    let me = common::call(&app, "GET", "/v1/me/hotel", Some(&c), None).await;
    let mut plan = me.body["private"]["plan"]["plan"].clone();
    assert!(plan["departments"]["restaurant"].is_object()); // restoran ima svaki profil i ne smije se zatvoriti
    // Profil gosta ovisi o sjemenu (`game:profile`), pa bar biramo iz granica: svaki profil ima odjel koji se smije zatvoriti.
    let limits = me.body["private"]["plan"]["limits"]["departments"]
        .as_object()
        .unwrap();
    let bar = limits
        .iter()
        .find(|(_, l)| l["can_close"] == true)
        .map(|(k, _)| k.clone())
        .unwrap();
    plan["departments"][&bar]["price_level_pct"] = json!(110);
    let r = common::call(
        &app,
        "PUT",
        &format!("/v1/hotels/{h}/plans/t02"),
        Some(&c),
        Some(plan.clone()),
    )
    .await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    assert_eq!(r.body["departments"][&bar]["price_level_pct"], 110);
    plan["departments"]["restaurant"]["open"] = json!(false);
    plan["departments"]["restaurant"]["seasonal_target"] = json!(0);
    let r = common::call(
        &app,
        "PUT",
        &format!("/v1/hotels/{h}/plans/t02"),
        Some(&c),
        Some(plan),
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (422, Some("department_plan_invalid"))
    );
}

#[tokio::test]
async fn v1_plan_with_departments_is_invalid_body() {
    let (app, _, _) = common::app();
    let r = common::call(&app, "POST", "/v1/demo", None, None).await;
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let h = r.body["hotel_id"].as_str().unwrap().to_owned();
    let me = common::call(&app, "GET", "/v1/me/hotel", Some(&c), None).await;
    assert!(
        me.body["private"]["plan"]["plan"]
            .get("departments")
            .is_none()
    );
    assert!(
        me.body["private"]["plan"]["limits"]
            .get("departments")
            .is_none()
    );
    let round = me.body["private"]["plan"]["round"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut plan = me.body["private"]["plan"]["plan"].clone();
    plan["departments"] = json!({ "restaurant": { "open": true, "seasonal_target": 0, "price_level_pct": 100, "maintenance": "regular" } });
    let r = common::call(
        &app,
        "PUT",
        &format!("/v1/hotels/{h}/plans/{round}"),
        Some(&c),
        Some(plan),
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (422, Some("invalid_body"))
    );
}

#[tokio::test]
async fn pl_season_scope_and_unknown_round() {
    let (app, c, _, h) = v2_after_first_week().await;
    let r = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{h}/pl?round=t01&scope=season"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    assert_eq!(r.body["scope"], "season");
    let r = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{h}/pl?round=t99"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (404, Some("unknown_month"))
    );
    let r = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{h}/pl?round=t02"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(
        (r.status.as_u16(), r.body["code"].as_str()),
        (409, Some("round_not_open"))
    );
}

/// ZP-6: PUT plana vraća plan kakav je spremljen, ne poslano tijelo. S odjelima se ukupni sezonci iz tijela ne čitaju
/// (spremi se zbroj odjela); bez odjela se odjeli uzimaju iz važećeg plana.
#[tokio::test]
async fn put_plan_returns_the_saved_plan() {
    let (app, c, _, h) = v2_after_first_week().await;
    let me = common::call(&app, "GET", "/v1/me/hotel", Some(&c), None).await;
    let mut plan = me.body["private"]["plan"]["plan"].clone();
    let limits = me.body["private"]["plan"]["limits"].clone();
    let depts = plan["departments"].as_object().unwrap();
    let sum: u64 = depts
        .values()
        .map(|d| d["seasonal_target"].as_u64().unwrap())
        .sum();
    assert_eq!(plan["seasonal_target"].as_u64(), Some(sum));
    // Restoran ±1 unutar granica (ukupno i odjel), a ukupni broj u tijelu ostaje star.
    let rest = plan["departments"]["restaurant"]["seasonal_target"]
        .as_u64()
        .unwrap();
    let rest_max = limits["departments"]["restaurant"]["seasonal_max"]
        .as_u64()
        .unwrap();
    let up = sum < limits["seasonal_max"].as_u64().unwrap() && rest < rest_max;
    let (new_rest, new_sum) = if up {
        (rest + 1, sum + 1)
    } else {
        (rest - 1, sum - 1)
    };
    plan["departments"]["restaurant"]["seasonal_target"] = json!(new_rest);
    let r = common::call(
        &app,
        "PUT",
        &format!("/v1/hotels/{h}/plans/t02"),
        Some(&c),
        Some(plan.clone()),
    )
    .await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    assert_eq!(r.body["seasonal_target"].as_u64(), Some(new_sum));
    assert_eq!(
        r.body["departments"]["restaurant"]["seasonal_target"].as_u64(),
        Some(new_rest)
    );
    let me = common::call(&app, "GET", "/v1/me/hotel", Some(&c), None).await;
    assert_eq!(r.body, me.body["private"]["plan"]["plan"]);

    // Tijelo bez odjela: odgovor ima odjele (iz važećeg plana) i iste brojke kao spremljeni plan.
    let mut body = r.body.clone();
    body.as_object_mut().unwrap().remove("departments");
    let r = common::call(
        &app,
        "PUT",
        &format!("/v1/hotels/{h}/plans/t02"),
        Some(&c),
        Some(body),
    )
    .await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    assert!(r.body["departments"]["restaurant"].is_object());
    let me = common::call(&app, "GET", "/v1/me/hotel", Some(&c), None).await;
    assert_eq!(r.body, me.body["private"]["plan"]["plan"]);
}
