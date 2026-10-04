use std::sync::Arc;

use api::{AppState, openapi_json, router};
use app::{BoxFut, StatusError, SystemStatus};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

struct Fake(Result<i64, String>);

impl SystemStatus for Fake {
    fn events_count(&self) -> BoxFut<'_, Result<i64, StatusError>> {
        let r = self.0.clone().map_err(StatusError);
        Box::pin(async move { r })
    }
}

async fn get(status: Fake, uri: &str) -> (StatusCode, Option<String>, Value) {
    let app = router(AppState {
        status: Arc::new(status),
    });
    let res = app
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let code = res.status();
    let ctype = res
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_owned());
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (code, ctype, serde_json::from_slice(&bytes).unwrap())
}

#[tokio::test]
async fn health_ok_reports_event_count() {
    let (code, _, body) = get(Fake(Ok(0)), "/v1/health").await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body, json!({"database": "ok", "events": 0}));
}

#[tokio::test]
async fn health_with_broken_database_is_503_not_panic() {
    let (code, _, body) = get(Fake(Err("veza odbijena".into())), "/v1/health").await;
    assert_eq!(code, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body, json!({"database": "error", "events": null}));
}

#[tokio::test]
async fn profile_c_indices_are_fifty_in_fixed_order() {
    let (code, _, body) = get(Fake(Ok(0)), "/v1/demo/profiles/C/indices").await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(
        body,
        json!({"profile": "C", "indices": [
            {"code": "G", "hundredths": 5000},
            {"code": "R", "hundredths": 5000},
            {"code": "Z", "hundredths": 5000},
            {"code": "O", "hundredths": 5000},
            {"code": "F", "hundredths": 5000}
        ]})
    );
}

#[tokio::test]
async fn unknown_profile_is_404_problem_json() {
    for uri in ["/v1/demo/profiles/D/indices", "/v1/demo/profiles/a/indices"] {
        let (code, ctype, body) = get(Fake(Ok(0)), uri).await;
        assert_eq!(code, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(ctype.as_deref(), Some("application/problem+json"), "{uri}");
        assert_eq!(body["code"], "unknown_profile", "{uri}");
        assert_eq!(body["status"], 404, "{uri}");
    }
}

#[test]
fn openapi_is_version_3_1_and_lists_paths() {
    let doc: Value = serde_json::from_str(&openapi_json()).unwrap();
    assert_eq!(doc["openapi"], "3.1.0", "TS generator čita samo 3.0 i 3.1");
    assert!(doc["paths"]["/v1/health"]["get"].is_object());
    assert!(doc["paths"]["/v1/demo/profiles/{profile}/indices"]["get"].is_object());
}

#[test]
fn openapi_output_is_stable() {
    assert_eq!(openapi_json(), openapi_json());
    assert!(openapi_json().ends_with('\n'));
}
