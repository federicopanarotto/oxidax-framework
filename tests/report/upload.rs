//! File upload middleware: multer-style multipart handling, limits, filters,
//! download and delete.

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use oxidax::app;
use oxidax::config::Config;
use tower::ServiceExt;
use uuid::Uuid;

use crate::support::{access_token_for, json_body};

const BOUNDARY: &str = "X-OXIDAX-TEST-BOUNDARY";

/// A single `multipart/form-data` part.
struct Part<'a> {
    name: &'a str,
    filename: Option<&'a str>,
    content_type: Option<&'a str>,
    data: &'a [u8],
}

fn file_part<'a>(
    name: &'a str,
    filename: &'a str,
    content_type: &'a str,
    data: &'a [u8],
) -> Part<'a> {
    Part {
        name,
        filename: Some(filename),
        content_type: Some(content_type),
        data,
    }
}

/// Encodes the parts into a `multipart/form-data` body.
fn multipart(boundary: &str, parts: &[Part<'_>]) -> Vec<u8> {
    let mut body = Vec::new();

    for part in parts {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        match part.filename {
            Some(filename) => {
                body.extend_from_slice(
                    format!(
                        "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                        part.name, filename
                    )
                    .as_bytes(),
                );
                if let Some(content_type) = part.content_type {
                    body.extend_from_slice(format!("Content-Type: {content_type}\r\n").as_bytes());
                }
            }
            None => body.extend_from_slice(
                format!("Content-Disposition: form-data; name=\"{}\"\r\n", part.name).as_bytes(),
            ),
        }
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(part.data);
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    body
}

fn upload_request(boundary: &str, body: Vec<u8>, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("POST").uri("/api/uploads").header(
        header::CONTENT_TYPE,
        format!("multipart/form-data; boundary={boundary}"),
    );

    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    builder.body(Body::from(body)).unwrap()
}

/// In-memory config with an isolated assets directory.
fn test_config() -> Config {
    let mut config = Config::memory();
    config.assets_dir = std::env::temp_dir().join(format!("oxidax-upload-{}", Uuid::new_v4()));
    config
}

async fn upload_app(config: &Config) -> Router {
    app::build(config).await.expect("application")
}

#[tokio::test]
async fn upload_download_delete_roundtrip() {
    let app = upload_app(&test_config()).await;
    let token = access_token_for(&app, "uploader@example.com").await;

    let body = multipart(
        BOUNDARY,
        &[file_part("file", "hello.txt", "text/plain", b"hello world")],
    );
    let response = app
        .clone()
        .oneshot(upload_request(BOUNDARY, body, Some(&token)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let created = json_body(response).await;
    let id = created["id"].as_str().expect("id").to_string();
    assert_eq!(created["originalName"], "hello.txt");
    assert_eq!(created["contentType"], "text/plain");
    assert!(created["description"].is_null());
    assert_eq!(created["size"], 11);
    assert_eq!(
        created["url"].as_str().unwrap(),
        format!("/api/upload/{id}")
    );

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/uploads/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/plain"
    );
    let disposition = response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(disposition.contains("hello.txt"), "{disposition}");
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&bytes[..], b"hello world");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/uploads/{id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/uploads/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn upload_persists_the_description_field() {
    let app = upload_app(&test_config()).await;
    let token = access_token_for(&app, "described@example.com").await;

    let body = multipart(
        BOUNDARY,
        &[
            file_part("file", "report.txt", "text/plain", b"data"),
            Part {
                name: "description",
                filename: None,
                content_type: None,
                data: b"Monthly report",
            },
        ],
    );
    let response = app
        .oneshot(upload_request(BOUNDARY, body, Some(&token)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(json_body(response).await["description"], "Monthly report");
}

#[tokio::test]
async fn upload_requires_authentication() {
    let app = upload_app(&test_config()).await;

    let body = multipart(
        BOUNDARY,
        &[file_part("file", "hello.txt", "text/plain", b"hi")],
    );
    let response = app
        .oneshot(upload_request(BOUNDARY, body, None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn missing_file_field_is_rejected() {
    let app = upload_app(&test_config()).await;
    let token = access_token_for(&app, "nofile@example.com").await;

    let body = multipart(
        BOUNDARY,
        &[Part {
            name: "description",
            filename: None,
            content_type: None,
            data: b"just a text field",
        }],
    );
    let response = app
        .oneshot(upload_request(BOUNDARY, body, Some(&token)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn file_too_large_is_rejected() {
    let mut config = test_config();
    config.upload_max_file_size = 4;
    let app = upload_app(&config).await;
    let token = access_token_for(&app, "big@example.com").await;

    let body = multipart(
        BOUNDARY,
        &[file_part("file", "big.txt", "text/plain", b"0123456789")],
    );
    let response = app
        .oneshot(upload_request(BOUNDARY, body, Some(&token)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn disallowed_extension_is_rejected() {
    let mut config = test_config();
    config.upload_allowed_extensions = vec!["txt".to_string()];
    let app = upload_app(&config).await;
    let token = access_token_for(&app, "filter@example.com").await;

    let body = multipart(
        BOUNDARY,
        &[file_part(
            "file",
            "evil.exe",
            "application/octet-stream",
            b"MZ",
        )],
    );
    let response = app
        .oneshot(upload_request(BOUNDARY, body, Some(&token)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn delete_is_forbidden_for_another_user() {
    let app = upload_app(&test_config()).await;
    let owner = access_token_for(&app, "owner@example.com").await;
    let other = access_token_for(&app, "other@example.com").await;

    let body = multipart(
        BOUNDARY,
        &[file_part("file", "secret.txt", "text/plain", b"secret")],
    );
    let response = app
        .clone()
        .oneshot(upload_request(BOUNDARY, body, Some(&owner)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let id = json_body(response).await["id"]
        .as_str()
        .unwrap()
        .to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/uploads/{id}"))
                .header(header::AUTHORIZATION, format!("Bearer {other}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn list_uploaded_files_is_paginated() {
    let app = upload_app(&test_config()).await;
    let token = access_token_for(&app, "lister@example.com").await;

    for name in ["a.txt", "b.txt"] {
        let body = multipart(BOUNDARY, &[file_part("file", name, "text/plain", b"x")]);
        let response = app
            .clone()
            .oneshot(upload_request(BOUNDARY, body, Some(&token)))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/uploads?page=1&limit=20")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("x-total-list")
            .unwrap()
            .to_str()
            .unwrap(),
        "2"
    );
    let list = json_body(response).await;
    assert_eq!(list.as_array().unwrap().len(), 2);
}
