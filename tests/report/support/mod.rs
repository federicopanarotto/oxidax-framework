//! Utilità condivise dai test: applicazione app, repository in-memory,
//! parsing dei body JSON e costruzione delle richieste.

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::Request;
use axum_api::api;
use axum_api::api::reports::report_service::ReportService;
use axum_api::config::Config;
use serde_json::Value;

/// Service dei report con le dipendenze reali (category service incluso),
/// risolto dal provider DI con backend in-memory.
pub async fn report_service() -> Arc<ReportService> {
    let provider = api::features::provider(&Config::memory())
        .await
        .expect("provider");
    provider.get_required::<ReportService>()
}

/// App con backend in-memory, costruita dal router generale come fa `main`.
pub async fn app() -> Router {
    api::router::build(&Config::memory())
        .await
        .expect("applicazione")
}

/// Legge il body della risposta come JSON.
pub async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// Costruisce una richiesta JSON.
pub fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}
