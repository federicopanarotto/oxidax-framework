//! Authentication over HTTP: registration, login, refresh rotation, logout
//! and the `#[IsGranted]` role guard.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::support::{app, app_and_provider, json_body, json_request};

fn register_payload(email: &str, password: &str) -> Value {
    json!({
        "email": email,
        "password": password,
        "firstname": "Mario",
        "lastname": "Rossi",
    })
}

/// POST JSON without a token.
async fn post_json(app: &Router, uri: &str, payload: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(json_request("POST", uri, payload))
        .await
        .unwrap();
    let status = response.status();
    (status, json_body(response).await)
}

/// POST without a body, with the bearer token (e.g. `logout`).
async fn post_token(app: &Router, uri: &str, token: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    if status == StatusCode::NO_CONTENT {
        return (status, Value::Null);
    }
    (status, json_body(response).await)
}

/// GET with an optional bearer token.
async fn get(app: &Router, uri: &str, token: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().uri(uri);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    (status, json_body(response).await)
}

async fn register(app: &Router, email: &str, password: &str) -> (StatusCode, Value) {
    post_json(app, "/api/auth/register", register_payload(email, password)).await
}

async fn login(app: &Router, email: &str, password: &str) -> (StatusCode, Value) {
    post_json(
        app,
        "/api/auth/login",
        json!({ "email": email, "password": password }),
    )
    .await
}

/// Registers and logs in, returning the token pair.
async fn registered_session(app: &Router, email: &str, password: &str) -> Value {
    let (status, body) = register(app, email, password).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");

    let (status, body) = login(app, email, password).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

fn access_token(tokens: &Value) -> String {
    tokens["access_token"]
        .as_str()
        .expect("access_token")
        .to_string()
}

fn refresh_token(tokens: &Value) -> String {
    tokens["refresh_token"]
        .as_str()
        .expect("refresh_token")
        .to_string()
}

#[tokio::test]
async fn register_returns_201_message_without_tokens() {
    let app = app().await;

    let (status, body) = register(&app, "mario@example.com", "Str0ngPassword!").await;

    assert_eq!(status, StatusCode::CREATED, "{body}");
    assert_eq!(body["message"], "user registered successfully");
    assert!(body.get("access_token").is_none(), "{body}");
    assert!(body.get("refresh_token").is_none(), "{body}");
}

#[tokio::test]
async fn register_duplicate_returns_generic_conflict() {
    let app = app().await;

    let (status, body) = register(&app, "dup@example.com", "Str0ngPassword!").await;
    assert_eq!(status, StatusCode::CREATED, "{body}");

    let (status, body) = register(&app, "dup@example.com", "AnotherPass1!").await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(body["code"], "conflict");
    assert_eq!(body["message"], "registration could not be completed");
}

#[tokio::test]
async fn register_rejects_invalid_email() {
    let app = app().await;

    let (status, body) = post_json(
        &app,
        "/api/auth/register",
        json!({
            "email": "not-an-email",
            "password": "Str0ngPassword!",
            "firstname": "Mario",
            "lastname": "Rossi",
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["code"], "validation_error", "{body}");
    assert_eq!(body["error"], "422 Unprocessable Entity", "{body}");
    assert!(
        body["message"].as_str().unwrap().contains("email"),
        "{body}"
    );
}

#[tokio::test]
async fn register_rejects_short_password() {
    let app = app().await;

    let (status, body) = register(&app, "short@example.com", "abc").await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert_eq!(body["code"], "validation_error", "{body}");
    assert_eq!(body["error"], "422 Unprocessable Entity", "{body}");
    assert!(
        body["message"].as_str().unwrap().contains("password"),
        "{body}"
    );
}

#[tokio::test]
async fn login_returns_identical_401_for_unknown_email_and_wrong_password() {
    let app = app().await;
    let (status, _) = register(&app, "known@example.com", "Str0ngPassword!").await;
    assert_eq!(status, StatusCode::CREATED);

    let (unknown_status, unknown_body) = login(&app, "unknown@example.com", "Whatever123!").await;
    let (wrong_status, wrong_body) = login(&app, "known@example.com", "WrongPass123!").await;

    assert_eq!(unknown_status, StatusCode::UNAUTHORIZED, "{unknown_body}");
    assert_eq!(wrong_status, StatusCode::UNAUTHORIZED, "{wrong_body}");
    // Same body in both cases: the API never reveals which part was wrong.
    assert_eq!(unknown_body, wrong_body);
    assert_eq!(unknown_body["message"], "invalid credentials");
    assert_eq!(unknown_body["code"], "unauthorized");
}

#[tokio::test]
async fn login_issues_token_pair() {
    let app = app().await;

    let tokens = registered_session(&app, "pair@example.com", "Str0ngPassword!").await;

    assert!(!access_token(&tokens).is_empty());
    assert!(!refresh_token(&tokens).is_empty());
    assert_eq!(tokens["token_type"], "Bearer");
    assert!(
        tokens["expires_in"].as_u64().is_some_and(|value| value > 0),
        "{tokens}"
    );
}

#[tokio::test]
async fn me_requires_a_valid_access_token() {
    let app = app().await;

    let (status, body) = get(&app, "/api/users/me", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["code"], "unauthorized");

    let (status, body) = get(&app, "/api/users/me", Some("not-a-token")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["code"], "unauthorized");
}

#[tokio::test]
async fn me_returns_profile_with_email() {
    let app = app().await;
    let tokens = registered_session(&app, "profile@example.com", "Str0ngPassword!").await;

    let (status, body) = get(&app, "/api/users/me", Some(&access_token(&tokens))).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["email"], "profile@example.com");
    assert_eq!(body["firstname"], "Mario");
    assert_eq!(body["role"], "user");
    assert!(
        body["id"].as_str().is_some_and(|id| !id.is_empty()),
        "{body}"
    );
}

#[tokio::test]
async fn expired_access_token_is_rejected() {
    let (app, provider) = app_and_provider().await;

    let jwt = provider.get_required::<axum_api::lib::auth::JwtConfig>();
    let now = axum_api::lib::auth::JwtConfig::now();
    let expired = jwt
        .encode(&axum_api::lib::auth::jwt::Claims {
            sub: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            identity_id: "d6c1a1b2-1111-2222-3333-444455556666".to_string(),
            role: "user".to_string(),
            token_use: axum_api::lib::auth::jwt::ACCESS.to_string(),
            jti: "expired-test".to_string(),
            // Well past the 60s leeway jsonwebtoken applies by default.
            iat: now - 300,
            exp: now - 120,
        })
        .expect("token scaduto");

    let (status, body) = get(&app, "/api/users/me", Some(&expired)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["code"], "unauthorized");
}

#[tokio::test]
async fn refresh_rotates_the_refresh_token() {
    let app = app().await;
    let tokens = registered_session(&app, "rotate@example.com", "Str0ngPassword!").await;
    let first_refresh = refresh_token(&tokens);

    let (status, rotated) = post_json(
        &app,
        "/api/auth/refresh",
        json!({ "refresh_token": first_refresh }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{rotated}");
    assert_ne!(refresh_token(&rotated), first_refresh);

    // The rotated access token works.
    let (status, body) = get(&app, "/api/users/me", Some(&access_token(&rotated))).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    // The first refresh token was burned by the rotation.
    let (status, body) = post_json(
        &app,
        "/api/auth/refresh",
        json!({ "refresh_token": first_refresh }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
    assert_eq!(body["message"], "invalid or expired refresh token");
}

#[tokio::test]
async fn logout_revokes_the_refresh_token() {
    let app = app().await;
    let tokens = registered_session(&app, "logout@example.com", "Str0ngPassword!").await;

    let (status, body) = post_token(&app, "/api/auth/logout", &access_token(&tokens)).await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

    // Without a token the endpoint is protected too.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // The outstanding refresh token no longer works.
    let (status, body) = post_json(
        &app,
        "/api/auth/refresh",
        json!({ "refresh_token": refresh_token(&tokens) }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
}

#[tokio::test]
async fn users_list_requires_admin_role() {
    let app = app().await;
    let tokens = registered_session(&app, "plain@example.com", "Str0ngPassword!").await;

    let (status, body) = get(&app, "/api/users", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");

    let (status, body) = get(&app, "/api/users", Some(&access_token(&tokens))).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["code"], "forbidden");
    assert_eq!(body["message"], "insufficient role");
}

#[tokio::test]
async fn admin_can_list_users_with_emails() {
    use axum_api::api::users::user_entity::{ROLE_ADMIN, User};
    use axum_api::api::users::user_repository::UserRepository;
    use axum_api::lib::auth::{AuthProvider, UserIdentity, UserIdentityRepository, password};

    let (app, provider) = app_and_provider().await;

    // First admin: no endpoint exists for this, it is assigned on the
    // database exactly like in production.
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

    // A regular user registered through the API.
    let (status, body) = register(&app, "member@example.com", "Str0ngPassword!").await;
    assert_eq!(status, StatusCode::CREATED, "{body}");

    let tokens = registered_session_as(&app, "admin@example.com", "AdminPass123!").await;
    let (status, body) = get(&app, "/api/users", Some(&access_token(&tokens))).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let list = body.as_array().expect("lista di users");
    let admin = list
        .iter()
        .find(|user| user["email"] == "admin@example.com")
        .expect("admin nella lista");
    assert_eq!(admin["role"], "admin");
    let member = list
        .iter()
        .find(|user| user["email"] == "member@example.com")
        .expect("member nella lista");
    assert_eq!(member["role"], "user");
    assert!(list.len() >= 2, "{body}");
}

/// Login with credentials that may already exist (used by the admin test).
async fn registered_session_as(app: &Router, email: &str, password: &str) -> Value {
    let (status, body) = login(app, email, password).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}
