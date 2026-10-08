//! Shared test utilities: app application, in-memory repository,
//! JSON body parsing and request building.

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum_api::api;
use axum_api::api::reports::report_service::ReportService;
use axum_api::config::Config;
use serde_json::{Value, json};
use tower::ServiceExt;

/// Report service with the real dependencies (category service included),
/// resolved from the DI provider with an in-memory backend.
pub async fn report_service() -> Arc<ReportService> {
    let provider = api::features::provider(&Config::memory())
        .await
        .expect("provider");
    provider.get_required::<ReportService>()
}

/// App with an in-memory backend, built from the general router the same way `main` does.
pub async fn app() -> Router {
    api::router::build(&Config::memory())
        .await
        .expect("applicazione")
}

/// App and its DI provider sharing the same in-memory database.
///
/// Required when a test must write directly to the database (e.g. the first
/// admin, which has no endpoint) and then call the API over HTTP: the two
/// `mem://` connections would otherwise be two separate databases.
pub async fn app_and_provider() -> (Router, di::ServiceProvider) {
    let provider = api::features::provider(&Config::memory())
        .await
        .expect("provider");
    let bundles = api::features::route_bundles(&provider);
    let (routers, route_apis): (Vec<_>, Vec<_>) = bundles.into_iter().unzip();

    let mut router = Router::new();
    for route in routers {
        router = router.merge(route);
    }
    router = router.merge(api::docs::swagger_router(api::docs::build(route_apis)));

    (router, provider)
}

/// Reads the response body as JSON.
pub async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// Builds a JSON request.
pub fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// Builds a JSON request authenticated with the bearer access token.
pub fn authorized_json_request(method: &str, uri: &str, body: Value, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// GET with the bearer access token, returning status and parsed body.
pub async fn authorized_get(app: &Router, uri: &str, token: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(uri)
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    (status, json_body(response).await)
}

/// Access token signed directly with the JWT settings: for tests that only
/// need "some authenticated caller" and do not care about the user behind it
/// (the guard verifies the signature only, without database round trips).
pub async fn mint_access_token(user_id: &str) -> String {
    let provider = api::features::provider(&Config::memory())
        .await
        .expect("provider");
    let jwt = provider.get_required::<axum_api::lib::auth::JwtConfig>();
    let now = axum_api::lib::auth::JwtConfig::now();

    jwt.encode(&axum_api::lib::auth::jwt::Claims {
        sub: user_id.to_string(),
        identity_id: uuid::Uuid::new_v4().to_string(),
        role: "user".to_string(),
        token_use: axum_api::lib::auth::jwt::ACCESS.to_string(),
        jti: uuid::Uuid::new_v4().to_string(),
        iat: now,
        exp: now + 900,
    })
    .expect("access token")
}

/// Registers a new user through the API and logs in, returning its access
/// token. Every `app()` has its own in-memory database, so a fixed email is
/// safe across tests.
pub async fn access_token_for(app: &Router, email: &str) -> String {
    let password = "Str0ngPassword!";

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/auth/register",
            json!({
                "email": email,
                "password": password,
                "firstname": "Mario",
                "lastname": "Rossi",
            }),
        ))
        .await
        .unwrap();
    let status = response.status();
    let body = json_body(response).await;
    assert_eq!(status, StatusCode::CREATED, "{body}");

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
