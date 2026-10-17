//! IZGLED-a, test 7: API nosi izgled lika (`character.look`, `character_look`) s ključevima i vrijednostima iz kartice.
mod common;

use std::sync::Arc;

use api::{AppState, router};
use app::memory::{FakeAi, deps_with_ai};
use common::{call, cookie_value};
use content::{Snapshot, repo_dir};
use serde_json::{Value, json};

#[tokio::test]
async fn character_look_has_the_agreed_json_shape() {
    let (deps, _) = deps_with_ai(
        Snapshot::read(&repo_dir()).unwrap(),
        Arc::new(FakeAi::good()),
    );
    let app = router(AppState { deps });
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
    let get = |uri: String| {
        let (app, c) = (app.clone(), c.clone());
        async move {
            let r = call(&app, "GET", &uri, Some(&c), None).await;
            assert_eq!(r.status, 200, "{uri}: {:?}", r.body);
            r.body
        }
    };
    let list = get(format!("/v1/hotels/{hotel}/situations")).await;
    let items = list["items"].as_array().unwrap();

    let check = |look: &Value, what: &str| {
        let o = look
            .as_object()
            .unwrap_or_else(|| panic!("{what}: look nije objekt: {look}"));
        let mut keys: Vec<_> = o.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["age_group", "gender", "origin", "uniform"], "{what}");
        assert!(
            ["f", "m"].contains(&look["gender"].as_str().unwrap()),
            "{what}"
        );
        assert!(
            look["age_group"].is_null()
                || ["young", "adult", "senior"].contains(&look["age_group"].as_str().unwrap()),
            "{what}: {look}"
        );
        assert!(
            ["regional", "foreign"].contains(&look["origin"].as_str().unwrap()),
            "{what}"
        );
        // IZGLED-c: ključ `uniform` je obavezan; danas nijedan predložak nema ulogu službe, pa je `null`.
        assert!(look["uniform"].is_null(), "{what}: {look}");
    };

    let inst = items
        .iter()
        .find(|i| {
            i["code"].as_str().unwrap().contains('.')
                && !i["character"].is_null()
                && i["free_text"] == true
        })
        .expect("primjerak s likom");
    check(&inst["character"]["look"], "situacije");

    // Sidro: ključ `look` postoji (obavezan), vrijednost null.
    if let Some(anchor) = items
        .iter()
        .find(|i| !i["character"].is_null() && !i["code"].as_str().unwrap().contains('.'))
    {
        assert!(
            anchor["character"]
                .as_object()
                .unwrap()
                .contains_key("look")
        );
        assert!(anchor["character"]["look"].is_null());
    }

    let code = inst["code"].as_str().unwrap();
    let conv = get(format!("/v1/hotels/{hotel}/situations/{code}/conversation")).await;
    check(&conv["character_look"], "razgovor");
    assert_eq!(conv["character_look"], inst["character"]["look"]);
}
