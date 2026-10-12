#![allow(dead_code)]
use std::sync::Arc;

use api::{AppState, router};
use app::memory::{Memory, deps};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use content::{Snapshot, repo_dir};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

pub struct Resp {
    pub status: StatusCode,
    pub ctype: Option<String>,
    pub set_cookie: Option<String>,
    pub body: Value,
}

pub fn app() -> (Router, app::Deps, Arc<Memory>) {
    let (d, mem) = deps(Snapshot::read(&repo_dir()).unwrap());
    (router(AppState { deps: d.clone() }), d, mem)
}

pub async fn call(
    app: &Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> Resp {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(t) = cookie {
        req = req.header("cookie", format!("hotelixsim_guest={t}"));
    }
    let req = match body {
        Some(b) => req
            .header("content-type", "application/json")
            .body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let header = |n: &str| res.headers().get(n).map(|v| v.to_str().unwrap().to_owned());
    let (status, ctype, set_cookie) = (res.status(), header("content-type"), header("set-cookie"));
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    Resp {
        status,
        ctype,
        set_cookie,
        body,
    }
}

/// Novi gost s igrom: (token, game_id, hotel_id).
pub async fn demo(app: &Router) -> (String, String, String) {
    let r = call(app, "POST", "/v1/demo", None, None).await;
    assert_eq!(r.status, StatusCode::CREATED);
    let c = r.set_cookie.expect("novi gost dobiva kolačić");
    let tok = c
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("hotelixsim_guest=")
        .unwrap()
        .to_owned();
    (
        tok,
        r.body["game_id"].as_str().unwrap().into(),
        r.body["hotel_id"].as_str().unwrap().into(),
    )
}

/// Vrijednost kolačića `hotelixsim_guest=…` do `;`.
pub fn cookie_value(set_cookie: &str) -> String {
    set_cookie
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("hotelixsim_guest=")
        .unwrap()
        .to_owned()
}

pub async fn run_worker(d: &app::Deps) {
    while app::run_next_job(d).await.unwrap() {}
}

pub fn robot_hotel(mem: &Memory, game: &str) -> String {
    let g = app::GameId(game.parse().unwrap());
    mem.events(g)[0].payload["hotels"][1]["hotel_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

pub fn assert_problem(r: &Resp, status: u16, code: &str) {
    assert_eq!(r.status.as_u16(), status, "{:?}", r.body);
    assert_eq!(r.ctype.as_deref(), Some("application/problem+json"));
    assert_eq!(r.body["code"], code);
    assert_eq!(r.body["status"], status);
}

/// Snimak v2 sadržaja s obnovom (brojke O1, O2, O4, O5, O9 iz obnova-v2-spec §0); isti kao u testovima igre.
pub fn renewal_snapshot() -> Snapshot {
    use serde_json::json;
    let renewal = |half: i64, max: i64, step: i64, life: u16| {
        json!({ "gain_pts": 5000, "half_cents": half * 100, "max_monthly_cents": max * 100,
                "step_cents": step * 100, "life_months": life })
    };
    let mut v = Snapshot::read_v2(&repo_dir()).unwrap().to_value();
    for (code, spec) in [
        ("rooms", renewal(150_000, 150_000, 5_000, 84)),
        ("restaurant", renewal(55_000, 50_000, 5_000, 84)),
        ("lobby_bar", renewal(25_000, 50_000, 5_000, 84)),
        ("rooftop_bar", renewal(25_000, 50_000, 5_000, 84)),
        ("pool_bar", renewal(20_000, 50_000, 5_000, 84)),
        ("spa", renewal(40_000, 50_000, 5_000, 120)),
        ("parking", renewal(60_000, 50_000, 5_000, 180)),
    ] {
        v["v2"]["departments"][code]["renewal"] = spec;
    }
    v["v2"]["economy"]["building"] = json!({
        "name": "Zgrada",
        "wear_pts_per_month": 50,
        "satisfaction_share_bp": 2000,
        "renewal": renewal(150_000, 200_000, 10_000, 480),
    });
    v["v2"]["economy"]["ffe_reserve_bp"] = json!(400);
    v["v2"]["economy"]["finance_index"] = json!("ebit");
    for (p, pts) in [("A", 8000), ("B", 4500), ("C", 6500)] {
        v["v2"]["profiles"][p]["building_pts"] = json!(pts);
    }
    Snapshot::from_value(&v).unwrap()
}

/// Aplikacija kojoj je sadržaj v2 snimak s obnovom.
pub fn renewal_app() -> (Router, app::Deps) {
    let (mut d, _) = deps(Snapshot::read(&repo_dir()).unwrap());
    d.snapshot_v2 = Arc::new(renewal_snapshot());
    (router(AppState { deps: d.clone() }), d)
}
