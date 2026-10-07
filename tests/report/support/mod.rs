//! Shared test utilities: app application, in-memory repository,
//! JSON body parsing and request building.

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::Request;
use axum_api::api;
use axum_api::api::reports::report_service::ReportService;
use axum_api::config::Config;
use serde_json::Value;

/// Report service with the real dependencies (category service included),
/// resolved from the DI provider with an in-memory backend.
pub async fn report_service() -> Arc<ReportService> {
    let provider = api::features::provider(&Config::memory())
        .await
        .expect("provider");
    provider.get_required::<ReportService>()
}

/// App with an in-memory backend, built from the general router the same way `main` does.
pub async fn app() -> Router {
    api::router::build(&Config::memory())
        .await
        .expect("applicazione")
}

/// Reads the response body as JSON.
pub async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// Builds a JSON request.
pub fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}
