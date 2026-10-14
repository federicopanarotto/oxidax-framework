//! Access log middleware.
//!
//! Logs every incoming HTTP request as a single compact line, e.g.
//! `POST /api/reports 201 3ms`. It is installed outermost on the router so it
//! wraps *all* requests, including unmatched routes (the fallback handler) and
//! the Scalar API reference.

use std::time::Instant;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

/// Times the request, runs the rest of the stack, then logs
/// `METHOD path status elapsed_ms`.
pub async fn access_log(request: Request, next: Next) -> Response {
    // Capture method and full path (path + query string) before the request is
    // moved into the inner layers.
    let start = Instant::now();
    let method = request.method().clone();
    let path = request
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    let response = next.run(request).await;
    let status = response.status().as_u16();
    let elapsed_ms = start.elapsed().as_millis();

    tracing::info!("{method} {path} {status} {elapsed_ms}ms");

    response
}
