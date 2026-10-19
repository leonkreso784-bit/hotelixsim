//! Pilot, dio 4 (ADR-062 t. 9): `POST /v1/demo` odbija 4. igru istog gosta s 409 problem+json `game_limit`;
//! odbijena igra ne stvara igru, gosta ni kolačić. Zahtjev bez kolačića je novi gost (bez granice po IP-u).
//! Pretpostavka sučelja: `Deps.max_games_per_guest: Option<u32>`.
mod common;
use api::{AppState, router};
use app::memory::deps;
use axum::http::StatusCode;
use common::{assert_problem, call, cookie_value};
use content::{Snapshot, repo_dir};
use serde_json::json;

fn limited(max: Option<u32>) -> axum::Router {
    let (mut d, _) = deps(Snapshot::read(&repo_dir()).unwrap());
    d.max_games_per_guest = max;
    router(AppState { deps: d })
}

#[tokio::test]
async fn fourth_demo_of_the_same_guest_is_409_game_limit_without_cookie() {
    let app = limited(Some(3));
    let r = call(&app, "POST", "/v1/demo", None, None).await;
    assert_eq!(r.status, StatusCode::CREATED);
    let tok = cookie_value(&r.set_cookie.expect("novi gost dobiva kolačić"));
    // druga i treća: različite vrste igre, sve se broje
    let week = json!({ "model": "v2", "round_kind": "week" });
    let month = json!({ "model": "v2", "round_kind": "month" });
    for body in [week.clone(), month] {
        let r = call(&app, "POST", "/v1/demo", Some(&tok), Some(body)).await;
        assert_eq!(r.status, StatusCode::CREATED, "{:?}", r.body);
        assert_eq!(r.set_cookie, None);
    }
    let me_before = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    assert_eq!(me_before.status, StatusCode::OK);
    let r4 = call(&app, "POST", "/v1/demo", Some(&tok), None).await;
    assert_problem(&r4, 409, "game_limit");
    assert_eq!(r4.set_cookie, None, "odbijanje ne postavlja kolačić");
    let me_after = call(&app, "GET", "/v1/me/hotel", Some(&tok), None).await;
    assert_eq!(
        me_after.body, me_before.body,
        "zadnja igra gosta je ista: četvrta nije stvorena"
    );
    let r5 = call(&app, "POST", "/v1/demo", Some(&tok), Some(week)).await;
    assert_problem(&r5, 409, "game_limit");
}

#[tokio::test]
async fn second_guest_has_his_own_counter() {
    let app = limited(Some(3));
    let a = call(&app, "POST", "/v1/demo", None, None).await;
    let ta = cookie_value(&a.set_cookie.unwrap());
    for _ in 0..2 {
        let r = call(&app, "POST", "/v1/demo", Some(&ta), None).await;
        assert_eq!(r.status, StatusCode::CREATED);
    }
    assert_problem(
        &call(&app, "POST", "/v1/demo", Some(&ta), None).await,
        409,
        "game_limit",
    );
    let b = call(&app, "POST", "/v1/demo", None, None).await;
    assert_eq!(b.status, StatusCode::CREATED);
    let tb = cookie_value(&b.set_cookie.unwrap());
    let b2 = call(&app, "POST", "/v1/demo", Some(&tb), None).await;
    assert_eq!(b2.status, StatusCode::CREATED);
}

#[tokio::test]
async fn request_without_cookie_is_never_limited() {
    let app = limited(Some(3));
    for _ in 0..5 {
        let r = call(&app, "POST", "/v1/demo", None, None).await;
        assert_eq!(r.status, StatusCode::CREATED);
        assert!(r.set_cookie.is_some());
    }
}

#[tokio::test]
async fn no_limit_set_means_unlimited() {
    let app = limited(None);
    let r = call(&app, "POST", "/v1/demo", None, None).await;
    let tok = cookie_value(&r.set_cookie.unwrap());
    for _ in 0..5 {
        let r = call(&app, "POST", "/v1/demo", Some(&tok), None).await;
        assert_eq!(r.status, StatusCode::CREATED);
    }
}
