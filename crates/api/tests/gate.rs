//! Vrata stranice (pilot, dio 2 a): Basic Auth za cijeli poslužitelj. Provjerava se samo lozinka
//! (korisničko ime je bilo koje); `/v1/health` je uvijek otvoren; bez postavljene lozinke vrata ne postoje.
mod common;
use api::{AppState, RouterConfig, router_with_config};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::app;
use http_body_util::BodyExt;
use tower::ServiceExt;

// base64 od "student:tajna123", ":tajna123" (prazno ime), "u:ta:jna" (lozinka s dvotočkom),
// "student:kriva", "student:tajna12" (prekratka), "student:tajna1234" (predugačka)
const GOOD: &str = "Basic c3R1ZGVudDp0YWpuYTEyMw==";
const EMPTY_USER: &str = "Basic OnRham5hMTIz";
const COLON_USER: &str = "Basic dTp0YTpqbmE=";
const WRONG: &str = "Basic c3R1ZGVudDprcml2YQ==";
const SHORT: &str = "Basic c3R1ZGVudDp0YWpuYTEy";
const LONG: &str = "Basic c3R1ZGVudDp0YWpuYTEyMzQ=";

fn gated(password: Option<&str>) -> Router {
    let (_, deps, _) = app();
    router_with_config(
        AppState { deps },
        RouterConfig {
            site_password: password.map(str::to_owned),
            ..RouterConfig::default()
        },
    )
}

struct Sent {
    status: StatusCode,
    challenge: Option<String>,
    cookie: Option<String>,
    ctype: Option<String>,
}

async fn send(app: &Router, method: &str, uri: &str, auth: Option<&str>) -> Sent {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(a) = auth {
        req = req.header("authorization", a);
    }
    let res = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let header = |n: &str| res.headers().get(n).map(|v| v.to_str().unwrap().to_owned());
    let sent = Sent {
        status: res.status(),
        challenge: header("www-authenticate"),
        cookie: header("set-cookie"),
        ctype: header("content-type"),
    };
    let _ = res.into_body().collect().await.unwrap();
    sent
}

#[tokio::test]
async fn no_password_header_is_401_with_basic_challenge() {
    let app = gated(Some("tajna123"));
    let r = send(&app, "GET", "/v1/me/hotel", None).await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    let c = r.challenge.expect("WWW-Authenticate");
    assert!(c.starts_with("Basic realm="), "{c}");
}

#[tokio::test]
async fn wrong_malformed_or_other_scheme_credentials_are_401() {
    let app = gated(Some("tajna123"));
    for auth in [
        WRONG,
        SHORT,
        LONG,
        "Basic !!!nije-base64!!!",
        "Basic ",
        "Bearer tajna123",
        "tajna123",
    ] {
        let r = send(&app, "GET", "/v1/me/hotel", Some(auth)).await;
        assert_eq!(r.status, StatusCode::UNAUTHORIZED, "{auth}");
        assert!(r.challenge.is_some(), "{auth}: treba WWW-Authenticate");
    }
}

#[tokio::test]
async fn right_password_passes_whatever_the_user_name() {
    let app = gated(Some("tajna123"));
    for auth in [GOOD, EMPTY_USER] {
        // Iza vrata je običan odgovor poslužitelja: nepoznat put je 404 Problem i NEMA WWW-Authenticate.
        let r = send(&app, "GET", "/v1/nepostoji", Some(auth)).await;
        assert_eq!(r.status, StatusCode::NOT_FOUND, "{auth}");
        assert!(r.challenge.is_none(), "{auth}");
        assert_eq!(r.ctype.as_deref(), Some("application/problem+json"));
    }
}

#[tokio::test]
async fn password_may_contain_a_colon() {
    // Dijeli se na PRVOJ dvotočki: ime "u", lozinka "ta:jna".
    let app = gated(Some("ta:jna"));
    let r = send(&app, "GET", "/v1/nepostoji", Some(COLON_USER)).await;
    assert_eq!(r.status, StatusCode::NOT_FOUND);
    let r = send(&app, "GET", "/v1/nepostoji", Some(GOOD)).await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn health_is_always_open() {
    let app = gated(Some("tajna123"));
    for auth in [None, Some(WRONG), Some(GOOD)] {
        let r = send(&app, "GET", "/v1/health", auth).await;
        assert_eq!(r.status, StatusCode::OK, "{auth:?}");
        assert!(r.challenge.is_none());
    }
}

#[tokio::test]
async fn gate_blocks_writes_before_anything_happens() {
    let app = gated(Some("tajna123"));
    for auth in [None, Some(WRONG)] {
        let r = send(&app, "POST", "/v1/demo", auth).await;
        assert_eq!(r.status, StatusCode::UNAUTHORIZED, "{auth:?}");
        assert!(r.cookie.is_none(), "bez lozinke nema ni kolačića ni gosta");
    }
}

#[tokio::test]
async fn without_configured_password_there_is_no_gate() {
    for password in [None, Some(""), Some("   ")] {
        let app = gated(password);
        let r = send(&app, "GET", "/v1/nepostoji", None).await;
        assert_eq!(r.status, StatusCode::NOT_FOUND, "{password:?}");
        assert!(r.challenge.is_none(), "{password:?}");
        // i zaglavlje koje netko ipak pošalje ne smeta
        let r = send(&app, "GET", "/v1/nepostoji", Some(WRONG)).await;
        assert_eq!(r.status, StatusCode::NOT_FOUND, "{password:?}");
    }
}

#[tokio::test]
async fn default_config_equals_todays_local_router() {
    let (_, deps, _) = app();
    let app = router_with_config(AppState { deps }, RouterConfig::default());
    let r = send(&app, "GET", "/v1/health", None).await;
    assert_eq!(r.status, StatusCode::OK);
    assert!(r.challenge.is_none());
    let r = send(&app, "GET", "/nista", None).await;
    assert_eq!(
        r.status,
        StatusCode::NOT_FOUND,
        "bez weba i / je 404 Problem"
    );
    assert_eq!(r.ctype.as_deref(), Some("application/problem+json"));
}

#[tokio::test]
async fn secure_cookie_flag_still_works_through_config() {
    let (_, deps, _) = app();
    let app = router_with_config(
        AppState { deps },
        RouterConfig {
            secure_cookie: true,
            ..RouterConfig::default()
        },
    );
    let r = send(&app, "POST", "/v1/demo", None).await;
    assert!(r.status.is_success(), "{}", r.status);
    let c = r.cookie.expect("kolačić gosta");
    assert!(c.contains("Secure"), "{c}");
}
