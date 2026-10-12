//! OpenAPI document and Swagger UI.
//!
//! The base document lives here (security schemes); the application supplies
//! the `info` block through [`crate::app::AppInfo`]. The paths and schemas of
//! individual features are NOT listed here: they arrive automatically from the
//! features themselves (the `#[utoipa::path]` handlers collected by
//! `utoipa-axum`), so this module never needs to change when a feature is
//! added.

use utoipa::openapi::OpenApi as Spec;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use crate::app::AppInfo;

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

/// Live documentation served by the Swagger UI: the OpenAPI spec is generated
/// at runtime and served at `/api-docs/openapi.json` without writing any file
/// to disk.
pub fn swagger_router(api: Spec) -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api)
}
