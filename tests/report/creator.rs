//! Association between a report and the user who created it: the id comes
//! from the access token (never from the body) and reads return the creator
//! nested as `created_by`.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::support::{
    access_token_for, app, authorized_get, authorized_json_request, json_body, json_request,
};

/// GET without a token: reads stay public and still return the creator.
async fn get(app: &Router, uri: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    (status, json_body(response).await)
}

/// User id of the caller, as seen by the API.
async fn user_id_of(app: &Router, token: &str) -> String {
    let (status, me) = authorized_get(app, "/api/users/me", token).await;
    assert_eq!(status, StatusCode::OK, "{me}");
    me["id"].as_str().expect("id utente").to_string()
}

/// Creates a report with the given token, returning its id.
async fn create_report(app: &Router, token: &str) -> String {
    let response = app
        .clone()
        .oneshot(authorized_json_request(
            "POST",
            "/api/reports",
            json!({"name": "Vendite mensili", "description": "Report di ottobre"}),
            token,
        ))
        .await
        .unwrap();
    let status = response.status();
    let created = json_body(response).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    created["id"].as_str().expect("id report").to_string()
}

#[tokio::test]
async fn report_creation_requires_authentication() {
    let app = app().await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/reports",
            json!({"name": "Riservato", "description": "Senza token"}),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = json_body(response).await;
    assert_eq!(body["code"], "unauthorized");
}

#[tokio::test]
async fn report_stores_and_returns_the_creator() {
    let app = app().await;
    let token = access_token_for(&app, "creator@example.com").await;
    let user_id = user_id_of(&app, &token).await;

    let response = app
        .clone()
        .oneshot(authorized_json_request(
            "POST",
            "/api/reports",
            json!({"name": "Vendite mensili", "description": "Report di ottobre"}),
            &token,
        ))
        .await
        .unwrap();
    let status = response.status();
    let created = json_body(response).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    // The relation id is never exposed, only the nested object.
    assert!(created.get("user_id").is_none(), "{created}");
    assert_eq!(created["createdBy"]["id"], user_id.as_str());
    assert_eq!(created["createdBy"]["firstname"], "Mario");
    // The email of the creator never appears in the response: only the profile.
    assert!(created["createdBy"].get("email").is_none(), "{created}");

    let report_id = created["id"].as_str().expect("id report").to_string();

    let (status, fetched) = get(&app, &format!("/api/reports/{report_id}")).await;
    assert_eq!(status, StatusCode::OK, "{fetched}");
    assert_eq!(fetched["createdBy"]["id"], user_id.as_str());

    let (status, list) = get(&app, "/api/reports").await;
    assert_eq!(status, StatusCode::OK, "{list}");
    let list = list.as_array().expect("lista di report");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["createdBy"]["id"], user_id.as_str());
}

#[tokio::test]
async fn update_preserves_the_creator() {
    let app = app().await;
    let token = access_token_for(&app, "owner@example.com").await;
    let user_id = user_id_of(&app, &token).await;
    let report_id = create_report(&app, &token).await;

    // Updates require the token too: whatever the caller sends, the merge
    // can never rewrite who created the report.
    let response = app
        .clone()
        .oneshot(authorized_json_request(
            "PUT",
            &format!("/api/reports/{report_id}"),
            json!({"name": "Aggiornato", "description": "Nuova descrizione"}),
            &token,
        ))
        .await
        .unwrap();
    let status = response.status();
    let updated = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{updated}");
    assert_eq!(updated["name"], "Aggiornato");
    assert_eq!(updated["createdBy"]["id"], user_id.as_str());
}
