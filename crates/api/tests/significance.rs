//! L1-13 dio 2: API nosi razinu značaja situacije (`significance`: `kljucna` | `vazna` | `svakodnevna`; `null` za
//! sidra S1–S4 i igru v1). Razina se izvodi iz `weight` (4 / 3 / 1–2). `openapi.json` se generira (`gen:api`), ovdje
//! se tvrdi oblik odgovora i shema.
mod common;

use common::call;
use serde_json::{Value, json};

async fn demo(body: Value) -> (axum::Router, String, String) {
    let (app, _deps, _mem) = common::app();
    let r = call(&app, "POST", "/v1/demo", None, Some(body)).await;
    assert_eq!(r.status, 201, "{:?}", r.body);
    let c = common::cookie_value(r.set_cookie.as_deref().unwrap());
    let hotel = r.body["hotel_id"].as_str().unwrap().to_owned();
    (app, c, hotel)
}

async fn items(app: &axum::Router, c: &str, hotel: &str) -> Vec<Value> {
    let r = call(
        app,
        "GET",
        &format!("/v1/hotels/{hotel}/situations"),
        Some(c),
        None,
    )
    .await;
    assert_eq!(r.status, 200, "{:?}", r.body);
    r.body["items"].as_array().unwrap().clone()
}

#[tokio::test]
async fn situations_json_has_significance_words() {
    for kind in ["month", "week"] {
        let (app, c, hotel) = demo(json!({ "model": "v2", "round_kind": kind })).await;
        let all = items(&app, &c, &hotel).await;
        assert!(
            all.iter().any(|i| !i["weight"].is_null()),
            "{kind}: primjerci"
        );
        for i in &all {
            assert!(
                i.get("significance").is_some(),
                "{}: polje je uvijek prisutno (null kad ga nema)",
                i["code"]
            );
            match i["weight"].as_u64() {
                None => assert!(
                    i["significance"].is_null(),
                    "{}: sidro bez oznake",
                    i["code"]
                ),
                Some(4) => assert_eq!(i["significance"], "kljucna", "{}", i["code"]),
                Some(3) => assert_eq!(i["significance"], "vazna", "{}", i["code"]),
                Some(1 | 2) => assert_eq!(i["significance"], "svakodnevna", "{}", i["code"]),
                Some(w) => panic!("{}: neočekivana težina {w}", i["code"]),
            }
        }
    }
}

#[tokio::test]
async fn classic_situations_json_has_null_significance() {
    let (app, c, hotel) = demo(json!({ "model": "v1" })).await;
    let all = items(&app, &c, &hotel).await;
    assert!(!all.is_empty());
    for i in &all {
        assert!(i.get("significance").is_some(), "{}", i["code"]);
        assert!(i["significance"].is_null(), "{}: v1 bez razine", i["code"]);
    }
}

#[test]
fn openapi_documents_significance() {
    let doc: Value = serde_json::from_str(&api::openapi_json()).unwrap();
    let schemas = &doc["components"]["schemas"];
    let prop = &schemas["SituationBody"]["properties"]["significance"];
    assert!(prop.is_object(), "SituationBody.significance");
    let required = schemas["SituationBody"]["required"].as_array().unwrap();
    assert!(
        required.contains(&json!("significance")),
        "polje je obavezno (null dopušten), kao weight"
    );
    let want = ["kljucna", "vazna", "svakodnevna"];
    let has_enum = schemas.as_object().unwrap().values().any(|s| {
        s["enum"]
            .as_array()
            .is_some_and(|e| e.len() == want.len() && want.iter().all(|w| e.contains(&json!(w))))
    });
    assert!(has_enum, "razina značaja kao snake_case enum u shemi");
}
