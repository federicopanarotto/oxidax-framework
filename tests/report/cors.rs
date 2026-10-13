//! CORS: regex origin allow-list, preflight requests and the exposed
//! pagination header.

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use oxidax::app;
use oxidax::config::Config;
use regex::Regex;
use tower::ServiceExt;

/// In-memory config with the given CORS origin regex allow-list.
fn config_with_origins(patterns: &[&str]) -> Config {
    let mut config = Config::memory();
    config.cors_allowed_origins = patterns.iter().map(|p| Regex::new(p).unwrap()).collect();
    config
}

#[tokio::test]
async fn allowed_origin_gets_cors_headers() {
    let config = config_with_origins(&[r"^https://(.+\.)?example\.com$"]);
    let app = app::build(&config).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header(header::ORIGIN, "https://app.example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap(),
        "https://app.example.com"
    );

    let exposed = response
        .headers()
        .get(header::ACCESS_CONTROL_EXPOSE_HEADERS)
        .expect("X-TOTAL-LIST exposed")
        .to_str()
        .unwrap();
    assert!(exposed.contains("x-total-list"), "{exposed}");
}

#[tokio::test]
async fn disallowed_origin_gets_no_cors_header() {
    let config = config_with_origins(&[r"^https://(.+\.)?example\.com$"]);
    let app = app::build(&config).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header(header::ORIGIN, "https://evil.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none()
    );
}

#[tokio::test]
async fn preflight_is_answered_for_allowed_origin() {
    let config = config_with_origins(&[".*"]);
    let app = app::build(&config).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/reports")
                .header(header::ORIGIN, "https://anywhere.test")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                .header(
                    header::ACCESS_CONTROL_REQUEST_HEADERS,
                    "content-type, authorization",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .unwrap(),
        "https://anywhere.test"
    );
    let methods = response
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_METHODS)
        .expect("allowed methods")
        .to_str()
        .unwrap();
    assert!(methods.contains("POST"), "{methods}");
}

#[tokio::test]
async fn no_origins_configured_means_no_cors() {
    let app = app::build(&Config::memory()).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .header(header::ORIGIN, "https://example.com")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .is_none()
    );
}
