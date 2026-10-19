//! Pilot-2 r2 (I2): keširanje weba i 404 za nepostojeće datoteke (nakon deploya stari `index.html`
//! ne smije dobiti `index.html` umjesto `/assets/…` s pogrešnim tipom).
mod common;
use std::path::PathBuf;

use api::{AppState, RouterConfig, router_with_config};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::app;
use http_body_util::BodyExt;
use tower::ServiceExt;

struct Dist(PathBuf);

impl Dist {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("hotelixsim-webcache-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("assets")).unwrap();
        std::fs::write(root.join("index.html"), "<html>INDEX</html>").unwrap();
        std::fs::write(root.join("assets/x-abc123.js"), "console.log(1)").unwrap();
        std::fs::write(root.join("logo.png"), "png").unwrap();
        Dist(root)
    }
    fn router(&self) -> Router {
        let (_, deps, _) = app();
        router_with_config(
            AppState { deps },
            RouterConfig {
                web_dir: Some(self.0.clone()),
                ..RouterConfig::default()
            },
        )
    }
}

impl Drop for Dist {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

async fn get(app: &Router, uri: &str) -> (StatusCode, Option<String>, String) {
    let res = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let cache = res
        .headers()
        .get("cache-control")
        .map(|v| v.to_str().unwrap().to_owned());
    let body = res.into_body().collect().await.unwrap().to_bytes();
    (status, cache, String::from_utf8_lossy(&body).into_owned())
}

#[tokio::test]
async fn index_and_spa_routes_must_be_revalidated() {
    let d = Dist::new("revalidated");
    let app = d.router();
    for uri in ["/", "/hotel/x", "/index.html", "/logo.png"] {
        let (code, cache, _) = get(&app, uri).await;
        assert_eq!(code, StatusCode::OK, "{uri}");
        assert_eq!(cache.as_deref(), Some("no-cache"), "{uri}");
    }
}

#[tokio::test]
async fn hashed_assets_are_cached_for_a_year_immutable() {
    let d = Dist::new("immutable");
    let (code, cache, body) = get(&d.router(), "/assets/x-abc123.js").await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        cache.as_deref(),
        Some("public, max-age=31536000, immutable")
    );
    assert_eq!(body, "console.log(1)");
}

#[tokio::test]
async fn missing_files_are_404_never_index_html() {
    let d = Dist::new("missing");
    let app = d.router();
    for uri in [
        "/assets/x-STARI.js",
        "/assets/nema.css",
        "/nema.png",
        "/favicon.ico",
    ] {
        let (code, cache, body) = get(&app, uri).await;
        assert_eq!(code, StatusCode::NOT_FOUND, "{uri}");
        assert!(!body.contains("INDEX"), "{uri}");
        assert_ne!(
            cache.as_deref(),
            Some("public, max-age=31536000, immutable"),
            "{uri}"
        );
    }
}
