//! Query parameters of the users list: the optional `role` filter is
//! validated by the `ValidatedQuery` extractor before the controller runs.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;

use crate::support::{access_token_for, app_and_provider, authorized_get, json_body, json_request};

/// Seeds an admin directly on the database (no endpoint exists for this,
/// exactly like in production) and returns its access token through a real
/// login.
async fn admin_token(app: &Router, provider: &di::ServiceProvider) -> String {
    use axum_api::api::users::user_entity::{ROLE_ADMIN, User};
    use axum_api::api::users::user_repository::UserRepository;
    use axum_api::lib::auth::{AuthProvider, UserIdentity, UserIdentityRepository, password};

    let admin_id = uuid::Uuid::new_v4().to_string();
    provider
        .get_required::<dyn UserRepository>()
        .insert(User {
            id: admin_id.clone(),
            firstname: "Ada".to_string(),
            lastname: "Lovelace".to_string(),
            role: ROLE_ADMIN.to_string(),
        })
        .await
        .expect("admin user");
    provider
        .get_required::<dyn UserIdentityRepository>()
        .insert(UserIdentity {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: admin_id,
            provider: AuthProvider::Local,
            identifier: "admin@example.com".to_string(),
            password_hash: password::hash("AdminPass123!").expect("hash"),
            refresh_jti: None,
            refresh_expires_at: None,
        })
        .await
        .expect("admin identity");

    login(app, "admin@example.com", "AdminPass123!").await
}

/// Real login, returning the access token.
async fn login(app: &Router, email: &str, password: &str) -> String {
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/auth/login",
            json!({ "email": email, "password": password }),
        ))
        .await
        .unwrap();
    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["access_token"]
        .as_str()
        .expect("access_token")
        .to_string()
}

#[tokio::test]
async fn users_list_filters_by_role() {
    let (app, provider) = app_and_provider().await;
    let admin = admin_token(&app, &provider).await;

    // Two members registered through the API (role = user).
    access_token_for(&app, "one@example.com").await;
    access_token_for(&app, "two@example.com").await;

    // No parameter: every user, emails included (admin only view).
    let (status, body) = authorized_get(&app, "/api/users", &admin).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body.as_array().expect("lista").len(), 3, "{body}");

    // role=user: only the registered members.
    let (status, body) = authorized_get(&app, "/api/users?role=user", &admin).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let users = body.as_array().expect("lista filtrata");
    assert_eq!(users.len(), 2, "{body}");
    assert!(users.iter().all(|user| user["role"] == "user"), "{body}");
    let emails: Vec<&str> = users
        .iter()
        .filter_map(|user| user["email"].as_str())
        .collect();
    assert!(emails.contains(&"one@example.com"), "{body}");
    assert!(emails.contains(&"two@example.com"), "{body}");
    assert!(!emails.contains(&"admin@example.com"), "{body}");

    // role=admin: only the admin.
    let (status, body) = authorized_get(&app, "/api/users?role=admin", &admin).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let admins = body.as_array().expect("lista admin");
    assert_eq!(admins.len(), 1, "{body}");
    assert_eq!(admins[0]["role"], "admin");
    assert_eq!(admins[0]["email"], "admin@example.com");
}

#[tokio::test]
async fn users_list_is_paginated_and_reports_the_total() {
    let (app, provider) = app_and_provider().await;
    let admin = admin_token(&app, &provider).await;
    access_token_for(&app, "p1@example.com").await;
    access_token_for(&app, "p2@example.com").await;

    // 3 users total (1 admin + 2 members), first page of two.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/users?page=1&limit=2")
                .header("authorization", format!("Bearer {admin}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let total = response
        .headers()
        .get("x-total-list")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    assert_eq!(total.as_deref(), Some("3"));
    let body = json_body(response).await;
    assert_eq!(body.as_array().expect("lista").len(), 2, "{body}");

    // The role filter combines with the pagination and its own total.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/users?role=user&page=2&limit=1")
                .header("authorization", format!("Bearer {admin}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let total = response
        .headers()
        .get("x-total-list")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);
    assert_eq!(total.as_deref(), Some("2"));
    let body = json_body(response).await;
    let users = body.as_array().expect("lista filtrata");
    assert_eq!(users.len(), 1, "{body}");
    assert_eq!(users[0]["role"], "user", "{body}");
}

#[tokio::test]
async fn users_list_rejects_unknown_role() {
    let (app, provider) = app_and_provider().await;
    let admin = admin_token(&app, &provider).await;

    let (status, body) = authorized_get(&app, "/api/users?role=superuser", &admin).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["code"], "validation_error", "{body}");
    assert_eq!(body["error"], "422 Unprocessable Entity", "{body}");
    assert!(
        body["message"]
            .as_str()
            .is_some_and(|m| m.contains("Validation error on Query") && m.contains("role")),
        "{body}"
    );

    // Empty value: present, but not a known role.
    let (status, body) = authorized_get(&app, "/api/users?role=", &admin).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");

    // The role guard runs before the query validation: a member cannot
    // probe the filter (403, not 422).
    let member = access_token_for(&app, "member@example.com").await;
    let (status, body) = authorized_get(&app, "/api/users?role=whatever", &member).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");

    // A present-but-valid parameter is not affected by the other checks.
    let (status, body) = authorized_get(&app, "/api/users?role=admin", &admin).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}
