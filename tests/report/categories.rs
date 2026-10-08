//! Category CRUD over HTTP and the report → category relation.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use crate::support::{app, authorized_json_request, json_body, json_request, mint_access_token};

#[tokio::test]
async fn categories_full_crud_cycle() {
    let app = app().await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/categories",
            json!({"name": "Vendite"}),
        ))
        .await
        .unwrap();
    let status = response.status();
    let created = json_body(response).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = created["id"].as_str().expect("id presente").to_string();
    assert_eq!(created["name"], "Vendite");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/categories/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let fetched = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{fetched}");
    assert_eq!(fetched["id"], id.as_str());

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/categories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let list = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    let list = list.as_array().expect("lista di categorie");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], id.as_str());

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/categories/{id}"),
            json!({"name": "Marketing"}),
        ))
        .await
        .unwrap();
    let status = response.status();
    let updated = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["name"], "Marketing");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/categories/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/categories/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn report_accepts_existing_category() {
    let app = app().await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/categories",
            json!({"name": "Finanza"}),
        ))
        .await
        .unwrap();
    let category = json_body(response).await;
    let category_id = category["id"].as_str().expect("id categoria").to_string();

    let token = mint_access_token("550e8400-e29b-41d4-a716-446655440000").await;
    let response = app
        .clone()
        .oneshot(authorized_json_request(
            "POST",
            "/api/reports",
            json!({
                "name": "Bilancio",
                "description": "Report di prova",
                "category_id": category_id,
            }),
            &token,
        ))
        .await
        .unwrap();
    let status = response.status();
    let created = json_body(response).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert!(created.get("category_id").is_none(), "{created}");
    assert_eq!(created["category"]["id"], category_id.as_str());
    assert_eq!(created["category"]["name"], "Finanza");

    let report_id = created["id"].as_str().expect("id report").to_string();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/reports/{report_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let fetched = json_body(response).await;
    assert!(fetched.get("category_id").is_none(), "{fetched}");
    assert_eq!(fetched["category"]["id"], category_id.as_str());
    assert_eq!(fetched["category"]["name"], "Finanza");
}

#[tokio::test]
async fn category_rejects_blank_name() {
    let app = app().await;

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/categories",
            json!({"name": "   "}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    assert_eq!(body["error"], "Validation error on Body");
    let details = body["details"].as_array().expect("details come lista");
    assert!(
        details.iter().any(|d| d.as_str().unwrap().contains("name")),
        "{details:?}"
    );
}

#[tokio::test]
async fn report_rejects_unknown_category() {
    let app = app().await;
    let token = mint_access_token("550e8400-e29b-41d4-a716-446655440000").await;

    let response = app
        .oneshot(authorized_json_request(
            "POST",
            "/api/reports",
            json!({
                "name": "Orfano",
                "description": "Senza categoria valida",
                "category_id": "00000000-0000-0000-0000-000000000000",
            }),
            &token,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["code"], "validation_error");
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("00000000-0000-0000-0000-000000000000")
    );
}

#[tokio::test]
async fn update_can_remove_category() {
    let app = app().await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/categories",
            json!({"name": "Temporanea"}),
        ))
        .await
        .unwrap();
    let category = json_body(response).await;
    let category_id = category["id"].as_str().unwrap().to_string();

    let token = mint_access_token("550e8400-e29b-41d4-a716-446655440000").await;
    let response = app
        .clone()
        .oneshot(authorized_json_request(
            "POST",
            "/api/reports",
            json!({
                "name": "Report con categoria",
                "description": "desc",
                "category_id": category_id,
            }),
            &token,
        ))
        .await
        .unwrap();
    let created = json_body(response).await;
    let report_id = created["id"].as_str().unwrap().to_string();
    assert!(created.get("category_id").is_none(), "{created}");

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/reports/{report_id}"),
            json!({
                "name": "Report senza categoria",
                "description": "desc",
                "category_id": null,
            }),
        ))
        .await
        .unwrap();
    let status = response.status();
    let updated = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert!(updated.get("category_id").is_none(), "{updated}");
    assert!(updated["category"].is_null(), "{updated}");
}
