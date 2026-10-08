use utoipa::openapi::OpenApi as Spec;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

/// Documentation base: info, tags and shared components.
///
/// The paths and schemas of individual features are NOT listed here: they
/// arrive automatically from the features themselves (the `#[utoipa::path]`
/// handlers collected by `utoipa-axum`), so `docs.rs` never needs to be
/// touched when adding a feature. Only the security schemes live here, because
/// they are a global component shared by every protected endpoint.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Axum Reports API",
        version = "0.1.0",
        description = "CRUD API per la gestione dei report e delle categorie"
    ),
    modifiers(&SecuritySchemes)
)]
pub struct ApiDoc;

/// Declares the two bearer schemes referenced by the `security(...)`
/// annotations of the endpoints: OpenAPI does not create them automatically.
///
/// - `accessToken`: short-lived JWT sent on every protected request;
/// - `refreshToken`: single-use JWT sent only to `/api/auth/refresh`.
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
/// base document (info + tags + shared schemas).
pub fn build(feature_apis: Vec<Spec>) -> Spec {
    let mut api = ApiDoc::openapi();
    for feature in feature_apis {
        api.merge(feature);
    }
    api
}

/// Live documentation served by the Swagger UI: the OpenAPI spec is
/// generated at runtime and served at `/api-docs/openapi.json` without
/// writing any file to disk.
pub fn swagger_router(api: Spec) -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", api)
}
