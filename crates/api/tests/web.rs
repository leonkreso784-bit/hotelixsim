//! Rust poslužuje izgrađeni web (pilot, dio 2 b): `/v1/*` ostaje API (nepoznato = 404 Problem),
//! sve ostalo je statična datoteka ili `index.html` (SPA). Vrata (lozinka) vrijede i za web.
mod common;
use std::path::PathBuf;

use api::{AppState, RouterConfig, router_with_config};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::app;
use http_body_util::BodyExt;
use tower::ServiceExt;

const INDEX: &str = "<!doctype html><html><body>HOTELIXSIM-INDEX-MARKER</body></html>";
const JS: &str = "console.log('asset-marker');";
const GOOD: &str = "Basic c3R1ZGVudDp0YWpuYTEyMw=="; // student:tajna123

/// Izgrađeni web u privremenoj mapi; `secret.txt` leži IZVAN mape weba (za probu izlaska iz mape).
struct Dist {
    root: PathBuf,
}

impl Dist {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("hotelixsim-web-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("dist/assets")).unwrap();
        std::fs::write(root.join("dist/index.html"), INDEX).unwrap();
        std::fs::write(root.join("dist/assets/x.js"), JS).unwrap();
        std::fs::write(root.join("dist/assets/x.css"), "body{color:red}").unwrap();
        std::fs::write(root.join("secret.txt"), "IZVAN-MAPE-TAJNA").unwrap();
        Dist { root }
    }
    fn router(&self, password: Option<&str>) -> Router {
        let (_, deps, _) = app();
        router_with_config(
            AppState { deps },
            RouterConfig {
                site_password: password.map(str::to_owned),
                web_dir: Some(self.root.join("dist")),
                ..RouterConfig::default()
            },
        )
    }
}

impl Drop for Dist {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

async fn get(app: &Router, uri: &str, auth: Option<&str>) -> (StatusCode, Option<String>, String) {
    let mut req = Request::get(uri);
    if let Some(a) = auth {
        req = req.header("authorization", a);
    }
    let res = app
        .clone()
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let ctype = res
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_owned());
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (status, ctype, String::from_utf8_lossy(&bytes).into_owned())
}

#[tokio::test]
async fn root_and_spa_routes_return_index_html() {
    let d = Dist::new("spa");
    let app = d.router(None);
    for uri in ["/", "/hotel/x", "/demo", "/likovi", "/hotel/a/b/c"] {
        let (code, ctype, body) = get(&app, uri, None).await;
        assert_eq!(code, StatusCode::OK, "{uri}");
        let ctype = ctype.unwrap_or_default();
        assert!(ctype.starts_with("text/html"), "{uri}: {ctype}");
        assert!(body.contains("HOTELIXSIM-INDEX-MARKER"), "{uri}");
    }
}

#[tokio::test]
async fn existing_static_files_are_served_with_their_type() {
    let d = Dist::new("static");
    let app = d.router(None);
    let (code, ctype, body) = get(&app, "/assets/x.js", None).await;
    assert_eq!(code, StatusCode::OK);
    assert!(ctype.unwrap_or_default().contains("javascript"));
    assert_eq!(body, JS);
    let (code, ctype, body) = get(&app, "/assets/x.css", None).await;
    assert_eq!(code, StatusCode::OK);
    assert!(ctype.unwrap_or_default().starts_with("text/css"));
    assert_eq!(body, "body{color:red}");
}

#[tokio::test]
async fn unknown_v1_route_stays_a_404_problem_not_index() {
    let d = Dist::new("v1");
    let app = d.router(None);
    for uri in [
        "/v1/nepostoji",
        "/v1/demo/profiles/D/indices",
        "/v1/hotels/x/y/z",
    ] {
        let (code, ctype, body) = get(&app, uri, None).await;
        assert_eq!(code, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(ctype.as_deref(), Some("application/problem+json"), "{uri}");
        assert!(!body.contains("HOTELIXSIM-INDEX-MARKER"), "{uri}");
    }
    let (_, _, body) = get(&app, "/v1/nepostoji", None).await;
    assert!(body.contains("unknown_route"), "{body}");
}

#[tokio::test]
async fn api_still_works_next_to_web() {
    let d = Dist::new("api");
    let app = d.router(None);
    let (code, ctype, body) = get(&app, "/v1/health", None).await;
    assert_eq!(code, StatusCode::OK);
    assert!(ctype.unwrap_or_default().starts_with("application/json"));
    assert!(body.contains("\"database\""), "{body}");
}

#[tokio::test]
async fn files_outside_the_web_dir_are_never_served() {
    let d = Dist::new("traversal");
    let app = d.router(None);
    for uri in [
        "/../secret.txt",
        "/..%2fsecret.txt",
        "/%2e%2e/secret.txt",
        "/assets/../../secret.txt",
        "/assets/%2e%2e/%2e%2e/secret.txt",
    ] {
        let (_, _, body) = get(&app, uri, None).await;
        assert!(
            !body.contains("IZVAN-MAPE-TAJNA"),
            "{uri} curi datoteku izvan mape"
        );
    }
}

#[tokio::test]
async fn gate_covers_the_web_too() {
    let d = Dist::new("gate");
    let app = d.router(Some("tajna123"));
    for uri in ["/", "/hotel/x", "/assets/x.js"] {
        let (code, _, body) = get(&app, uri, None).await;
        assert_eq!(code, StatusCode::UNAUTHORIZED, "{uri}");
        assert!(
            !body.contains("HOTELIXSIM-INDEX-MARKER") && body != JS,
            "{uri}"
        );
    }
    let (code, _, body) = get(&app, "/hotel/x", Some(GOOD)).await;
    assert_eq!(code, StatusCode::OK);
    assert!(body.contains("HOTELIXSIM-INDEX-MARKER"));
    let (code, _, body) = get(&app, "/assets/x.js", Some(GOOD)).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body, JS);
    let (code, ctype, _) = get(&app, "/v1/nepostoji", Some(GOOD)).await;
    assert_eq!(code, StatusCode::NOT_FOUND);
    assert_eq!(ctype.as_deref(), Some("application/problem+json"));
    let (code, _, _) = get(&app, "/v1/health", None).await;
    assert_eq!(code, StatusCode::OK, "zdravlje je i uz web bez lozinke");
}

#[tokio::test]
async fn without_web_dir_root_is_404_problem_as_today() {
    let (_, deps, _) = app();
    let app = router_with_config(AppState { deps }, RouterConfig::default());
    for uri in ["/", "/hotel/x"] {
        let (code, ctype, _) = get(&app, uri, None).await;
        assert_eq!(code, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(ctype.as_deref(), Some("application/problem+json"), "{uri}");
    }
}
