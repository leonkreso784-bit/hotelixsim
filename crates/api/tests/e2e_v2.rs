mod common;

use serde_json::json;

#[tokio::test]
async fn weekly_v2_season_from_start_to_end() {
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
    let rounds: Vec<String> = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None)
        .await
        .body["rounds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["code"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(rounds.len(), 17);
    for code in &rounds {
        let s = common::call(
            &app,
            "GET",
            &format!("/v1/hotels/{h}/situations"),
            Some(&c),
            None,
        )
        .await;
        for item in s.body["items"].as_array().unwrap() {
            let sc = item["code"].as_str().unwrap();
            common::call(
                &app,
                "POST",
                &format!("/v1/hotels/{h}/situations/{sc}/answer"),
                Some(&c),
                Some(json!({ "answer": 0 })),
            )
            .await;
        }
        let r = common::call(
            &app,
            "POST",
            &format!("/v1/games/{g}/rounds/{code}/close"),
            Some(&c),
            None,
        )
        .await;
        assert_eq!(r.status, 202, "{code}");
        common::run_worker(&deps).await;
    }
    let game = common::call(&app, "GET", &format!("/v1/games/{g}"), Some(&c), None).await;
    assert!(game.body["current"].is_null());
    let pl = common::call(
        &app,
        "GET",
        &format!("/v1/hotels/{h}/pl?scope=season"),
        Some(&c),
        None,
    )
    .await;
    assert_eq!(pl.status, 200);
    assert_eq!(pl.body["days"], 122);
    let kpi = common::call(&app, "GET", &format!("/v1/hotels/{h}/kpi"), Some(&c), None).await;
    assert_eq!(kpi.body["months"].as_array().unwrap().len(), 17);
    let feed = common::call(&app, "GET", &format!("/v1/hotels/{h}/feed"), Some(&c), None).await;
    assert!(
        feed.body["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["round"] == "t17")
    );
}
