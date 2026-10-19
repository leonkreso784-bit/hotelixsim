//! Rust poslužuje izgrađeni web (pilot, dio 2 b): statična datoteka ili `index.html` (SPA).
//! `/v1/…` nikad ne završi u `index.html`: nepoznato je i dalje 404 Problem. Izlazak iz mape brani `ServeDir`
//! (segmenti `..` se ne razrješuju izvan korijena), a test to pribija.
//!
//! Keširanje (nakon deploya stari `index.html` ne smije tražiti nepostojeće `/assets/…`):
//! - `/assets/*` (Viteovi hashirani nazivi) = `public, max-age=31536000, immutable`;
//! - `index.html`, SPA put i ostale datoteke = `no-cache` (preglednik provjeri, `If-Modified-Since` vrati 304);
//! - put s nastavkom datoteke (`.js`, `.png`…) koji ne postoji = 404, nikad `index.html` (pogrešan MIME).

use std::future::Future;
use std::path::Path;
use std::pin::Pin;

use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

use crate::unknown_route;

type Boxed = Pin<Box<dyn Future<Output = Response> + Send>>;

const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const NO_CACHE: &str = "no-cache";

/// Zadnji segment puta ima nastavak datoteke (`x.js`); `/hotel/x` i `/` nemaju.
fn has_extension(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|last| last.contains('.'))
}

pub fn fallback(dir: &Path) -> impl Fn(Request) -> Boxed + Clone + Send + Sync + 'static {
    let files = ServeDir::new(dir);
    let spa = ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")));
    move |req: Request| -> Boxed {
        let (files, spa) = (files.clone(), spa.clone());
        Box::pin(async move {
            let path = req.uri().path().to_owned();
            if path == "/v1" || path.starts_with("/v1/") {
                return unknown_route().await.into_response();
            }
            let file_like = has_extension(&path);
            let res = if file_like {
                files.oneshot(req).await.map(IntoResponse::into_response)
            } else {
                spa.oneshot(req).await.map(IntoResponse::into_response)
            };
            let mut res = match res {
                Ok(r) => r,
                Err(never) => match never {},
            };
            if file_like && res.status() == StatusCode::NOT_FOUND {
                return unknown_route().await.into_response();
            }
            let ok = res.status().is_success() || res.status() == StatusCode::NOT_MODIFIED;
            if ok {
                let value = if path.starts_with("/assets/") {
                    IMMUTABLE
                } else {
                    NO_CACHE
                };
                res.headers_mut()
                    .insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
            }
            res
        })
    }
}
