//! Place CRUD over HTTP and the report → place relation.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use crate::support::{app, json_body, json_request};

#[tokio::test]
async fn places_full_crud_cycle() {
    let app = app().await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/places",
            json!({
                "city": "Milano",
                "address": "Via Roma 1",
                "cap": "20100",
                "provenance": "Comune di Milano",
            }),
        ))
        .await
        .unwrap();
    let status = response.status();
    let created = json_body(response).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = created["id"].as_str().expect("id presente").to_string();
    assert_eq!(created["city"], "Milano");
    assert_eq!(created["cap"], "20100");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/places/{id}"))
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
                .uri("/api/places")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let list = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{list}");
    let list = list.as_array().expect("lista di places");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], id.as_str());

    let response = app
        .clone()
        .oneshot(json_request(
            "PUT",
            &format!("/api/places/{id}"),
            json!({
                "city": "Torino",
                "address": "Corso Francia 10",
                "cap": "10100",
                "provenance": "Comune di Torino",
            }),
        ))
        .await
        .unwrap();
    let status = response.status();
    let updated = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["city"], "Torino");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/places/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/places/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn place_rejects_blank_city() {
    let app = app().await;

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/places",
            json!({
                "city": "   ",
                "address": "Via Roma 1",
                "cap": "20100",
                "provenance": "Comune di Milano",
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = json_body(response).await;
    assert_eq!(body["error"], "Validation error on Body");
    let details = body["details"].as_array().expect("details come lista");
    assert!(
        details.iter().any(|d| d.as_str().unwrap().contains("city")),
        "{details:?}"
    );
}

#[tokio::test]
async fn report_accepts_existing_place() {
    let app = app().await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/places",
            json!({
                "city": "Napoli",
                "address": "Via Toledo 2",
                "cap": "80100",
                "provenance": "Comune di Napoli",
            }),
        ))
        .await
        .unwrap();
    let place = json_body(response).await;
    let place_id = place["id"].as_str().expect("id place").to_string();

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/reports",
            json!({
                "name": "Report con luogo",
                "description": "desc",
                "place_id": place_id,
            }),
        ))
        .await
        .unwrap();
    let status = response.status();
    let created = json_body(response).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert!(created.get("place_id").is_none(), "{created}");
    assert_eq!(created["place"]["id"], place_id.as_str());
    assert_eq!(created["place"]["city"], "Napoli");
}

#[tokio::test]
async fn report_rejects_unknown_place() {
    let app = app().await;

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/reports",
            json!({
                "name": "Orfano",
                "description": "Senza place valido",
                "place_id": "00000000-0000-0000-0000-000000000000",
            }),
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
