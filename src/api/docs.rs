use utoipa::OpenApi;
use utoipa::openapi::OpenApi as Spec;
use utoipa_swagger_ui::SwaggerUi;

/// Documentation base: info, tags and shared components.
///
/// The paths and schemas of individual features are NOT listed here: they
/// arrive automatically from the features themselves (the `#[utoipa::path]`
/// handlers collected by `utoipa-axum`), so `docs.rs` never needs to be
/// touched when adding a feature.
#[derive(OpenApi)]
#[openapi(info(
    title = "Axum Reports API",
    version = "0.1.0",
    description = "CRUD API per la gestione dei report e delle categorie"
))]
pub struct ApiDoc;

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
