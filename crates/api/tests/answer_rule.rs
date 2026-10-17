//! PRAVILO-c: API nosi `click_capped` (situacija) i `copied` (odgovor) u JSON-u. `openapi.json` se generira (`gen:api`),
//! ovdje se tvrdi samo oblik odgovora.
mod common;

use std::sync::Arc;

use api::{AppState, router};
use app::memory::{FakeAi, deps_with_ai};
use common::{call, cookie_value, run_worker};
use content::{Snapshot, repo_dir};
use serde_json::json;

#[tokio::test]
async fn situations_json_has_click_capped_and_copied() {
    let (deps, _) = deps_with_ai(
        Snapshot::read(&repo_dir()).unwrap(),
        Arc::new(FakeAi::good()),
    );
    let app = router(AppState { deps: deps.clone() });
    let r = call(
        &app,
        "POST",
        "/v1/demo",
        None,
        Some(json!({ "model": "v2", "round_kind": "month" })),
    )
    .await;
    assert_eq!(r.status, 201, "{:?}", r.body);
    let c = cookie_value(r.set_cookie.as_deref().unwrap());
    let hotel = r.body["hotel_id"].as_str().unwrap().to_owned();
    let uri = format!("/v1/hotels/{hotel}/situations");

    let list = call(&app, "GET", &uri, Some(&c), None).await;
    assert_eq!(list.status, 200);
    let items = list.body["items"].as_array().unwrap();
    for i in items {
        assert!(
            i["click_capped"].is_boolean(),
            "{}: click_capped mora biti bool",
            i["code"]
        );
    }
    assert!(
        items
            .iter()
            .any(|i| i["click_capped"] == true && !i["character"].is_null()),
        "lik + vlastiti odgovor => click_capped true"
    );
    assert!(
        items
            .iter()
            .any(|i| i["click_capped"] == false && i["character"].is_null()),
        "bez lika => click_capped false"
    );

    // Prepisan ponuđeni odgovor: nakon presude `response.copied` je true; vlastiti tekst false.
    let s1 = items.iter().find(|i| i["code"] == "S1").unwrap();
    let label = s1["answers"][0].as_str().unwrap().to_owned();
    let r = call(
        &app,
        "POST",
        &format!("{uri}/S1/response"),
        Some(&c),
        Some(json!({ "text": label })),
    )
    .await;
    assert_eq!(r.status, 202, "{:?}", r.body);
    let pending = r.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["code"] == "S1")
        .unwrap();
    assert_eq!(pending["response"]["copied"], false, "{pending}");
    run_worker(&deps).await;
    let after = call(&app, "GET", &uri, Some(&c), None).await;
    let s1 = after.body["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["code"] == "S1")
        .unwrap();
    assert_eq!(s1["response"]["status"], "judged", "{s1}");
    assert_eq!(s1["response"]["copied"], true, "{s1}");
}
