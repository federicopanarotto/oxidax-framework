//! Full CRUD cycle over HTTP for the generated `color` feature.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use crate::support::{app, authorized_json_request, json_body, mint_access_token};

#[tokio::test]
async fn color_full_crud_cycle() {
    let app = app().await;
    let token = mint_access_token("550e8400-e29b-41d4-a716-446655440000").await;

    let response = app
        .clone()
        .oneshot(authorized_json_request(
            "POST",
            "/api/color",
            json!({
            "name": "sample",
            "hex": "sample",
            }),
            &token,
        ))
        .await
        .unwrap();
    let status = response.status();
    let created = json_body(response).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = created["id"].as_str().expect("id").to_string();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/color/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/color?page=1&limit=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let total = response
        .headers()
        .get("x-total-list")
        .expect("X-TOTAL-LIST header")
        .to_str()
        .unwrap()
        .to_string();
    let list = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    assert_eq!(total, "1");
    assert_eq!(list.as_array().unwrap().len(), 1);

    let response = app
        .clone()
        .oneshot(authorized_json_request(
            "PUT",
            &format!("/api/color/{id}"),
            json!({
            "name": "sample",
            "hex": "sample",
            }),
            &token,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/color/{id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/color/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
