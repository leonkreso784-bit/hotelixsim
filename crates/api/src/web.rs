//! Rust poslužuje izgrađeni web (pilot, dio 2 b): statična datoteka ili `index.html` (SPA).
//! `/v1/…` nikad ne završi u `index.html`: nepoznato je i dalje 404 Problem. Izlazak iz mape brani `ServeDir`
//! (segmenti `..` se ne razrješuju izvan korijena), a test to pribija.

use std::future::Future;
use std::path::Path;
use std::pin::Pin;

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

use crate::unknown_route;

type Boxed = Pin<Box<dyn Future<Output = Response> + Send>>;

pub fn fallback(dir: &Path) -> impl Fn(Request) -> Boxed + Clone + Send + Sync + 'static {
    let service = ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")));
    move |req: Request| -> Boxed {
        let service = service.clone();
        Box::pin(async move {
            let path = req.uri().path();
            if path == "/v1" || path.starts_with("/v1/") {
                return unknown_route().await.into_response();
            }
            match service.oneshot(req).await {
                Ok(res) => res.into_response(),
                Err(never) => match never {},
            }
        })
    }
}
