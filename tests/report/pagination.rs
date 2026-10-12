//! Pagination of the list endpoints: the body stays a bare JSON array and
//! the total number of elements travels in the `X-TOTAL-LIST` response header.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use crate::support::{app, json_body, json_request};

/// Creates a category and returns its id.
async fn create_category(app: &axum::Router, name: &str) {
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/categories",
            json!({ "name": name }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

/// GET returning status, parsed body and the `X-TOTAL-LIST` header (if any).
async fn get(app: &axum::Router, uri: &str) -> (StatusCode, serde_json::Value, Option<String>) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let total = response
        .headers()
        .get("x-total-list")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    (status, json_body(response).await, total)
}

#[tokio::test]
async fn list_slices_the_array_and_reports_the_total() {
    let app = app().await;
    for index in 1..=5 {
        create_category(&app, &format!("Category {index}")).await;
    }

    // Default page: the whole (small) set, total in the header.
    let (status, body, total) = get(&app, "/api/categories").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().expect("array").len(), 5, "{body}");
    assert_eq!(total.as_deref(), Some("5"), "header X-TOTAL-LIST");

    // First page of two.
    let (status, body, total) = get(&app, "/api/categories?page=1&limit=2").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().expect("array").len(), 2, "{body}");
    assert_eq!(total.as_deref(), Some("5"));

    // Last (partial) page.
    let (status, body, total) = get(&app, "/api/categories?page=3&limit=2").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().expect("array").len(), 1, "{body}");
    assert_eq!(total.as_deref(), Some("5"));

    // Page beyond the end: empty body, total still reported.
    let (status, body, total) = get(&app, "/api/categories?page=10&limit=2").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().expect("array").is_empty(), "{body}");
    assert_eq!(total.as_deref(), Some("5"));
}

#[tokio::test]
async fn list_rejects_invalid_pagination() {
    let app = app().await;

    // Out-of-range values are a validation error (422).
    for uri in [
        "/api/categories?page=0",
        "/api/categories?limit=0",
        "/api/categories?limit=101",
    ] {
        let (status, body, _) = get(&app, uri).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{uri}: {body}");
        assert_eq!(body["code"], "validation_error", "{uri}: {body}");
        assert_eq!(body["error"], "422 Unprocessable Entity", "{uri}: {body}");
        assert!(
            body["message"]
                .as_str()
                .unwrap()
                .contains("Validation error on Query"),
            "{uri}: {body}"
        );
    }

    // A non-numeric value is a malformed query (400), not a validation error.
    let (status, body, _) = get(&app, "/api/categories?page=abc").await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["code"], "bad_request", "{body}");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("Query parameters are invalid"),
        "{body}"
    );
}
