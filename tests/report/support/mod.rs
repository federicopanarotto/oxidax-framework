//! Utilità condivise dai test: applica app, parsing dei body JSON e
//! costruzione delle richieste.

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::Request;
use axum_api::api;
use axum_api::api::reports::report_controller::ReportController;
use axum_api::api::reports::report_service::ReportService;
use axum_api::db;
use serde_json::Value;

/// App con repository in-memory (SurrealDB `mem://`).
pub async fn app() -> Router {
    let repository = db::memory_repository().await.expect("memory repository");
    api::router(ReportController::new(ReportService::new(repository)))
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
