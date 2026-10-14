//! OpenAPI document and API reference UI (Scalar).
//!
//! The base document lives here (security schemes); the application supplies
//! the `info` block through [`crate::app::AppInfo`]. The paths and schemas of
//! individual features are NOT listed here: they arrive automatically from the
//! features themselves (the `#[utoipa::path]` handlers collected by
//! `utoipa-axum`), so this module never needs to change when a feature is
//! added.
//!
//! The interactive UI is provided by [Scalar](https://scalar.com): the OpenAPI
//! document is serialized once at startup and served at `/api-docs/openapi.json`,
//! while `/scalar` returns an HTML page that loads the Scalar script from the
//! CDN and points it at that URL.

use axum::Router;
use axum::response::Html;
use axum::routing::get;
use utoipa::openapi::OpenApi as Spec;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::app::AppInfo;

/// Path of the live OpenAPI document, consumed both by the Scalar UI and by any
/// external client.
pub const OPENAPI_PATH: &str = "/api-docs/openapi.json";

/// Path of the HTML page that renders the API reference through Scalar.
pub const SCALAR_PATH: &str = "/scalar";

/// CDN bundle of the Scalar API reference web component.
const SCALAR_CDN: &str = "https://cdn.jsdelivr.net/npm/@scalar/api-reference";

/// Documentation base: shared components only (the `info` block is filled in by
/// [`build`] from the application metadata).
#[derive(OpenApi)]
#[openapi(modifiers(&SecuritySchemes))]
pub struct ApiDoc;

/// Declares the bearer scheme referenced by the `security(...)` annotations of
/// the endpoints: OpenAPI does not create it automatically.
///
/// - `accessToken`: short-lived JWT sent on every protected request.
struct SecuritySchemes;

impl Modify for SecuritySchemes {
    fn modify(&self, openapi: &mut Spec) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme("accessToken", bearer_jwt());
    }
}

fn bearer_jwt() -> SecurityScheme {
    SecurityScheme::Http(
        HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .build(),
    )
}

/// Merges the OpenAPI components produced by individual features into the
/// base document, filling the `info` block with the application metadata.
pub fn build(feature_apis: Vec<Spec>, info: AppInfo<'_>) -> Spec {
    let mut api = ApiDoc::openapi();
    api.info.title = info.title.to_string();
    api.info.version = info.version.to_string();
    api.info.description = Some(info.description.to_string());
    for feature in feature_apis {
        api.merge(feature);
    }
    api
}

/// Mounts the OpenAPI document and the Scalar API reference.
///
/// The specification is serialized once here (instead of on every request), so
/// serving it is a cheap clone of the pre-rendered JSON string. The UI itself is
/// a static HTML page whose `<script id="api-reference" data-url="...">` tells
/// Scalar where to fetch the document.
pub fn router(api: Spec) -> Router {
    let spec = serde_json::to_string(&api).unwrap_or_else(|_| "{}".to_string());
    let html = scalar_html();

    Router::new()
        .route(
            OPENAPI_PATH,
            get(move || {
                let spec = spec.clone();
                async move {
                    (
                        [(axum::http::header::CONTENT_TYPE, "application/json")],
                        spec,
                    )
                }
            }),
        )
        .route(
            SCALAR_PATH,
            get(move || {
                let html = html.clone();
                async move { Html(html) }
            }),
        )
}

/// Static HTML shell: the Scalar web component replaces the placeholder script
/// tag with the rendered reference, fetching the document from [`OPENAPI_PATH`].
fn scalar_html() -> String {
    format!(
        r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>API Reference</title>
  </head>
  <body>
    <script id="api-reference" data-url="{OPENAPI_PATH}"></script>
    <script src="{SCALAR_CDN}"></script>
  </body>
</html>
"#
    )
}
