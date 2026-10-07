use axum::Router;
use axum::routing::get;

use crate::api::docs;
use crate::api::features;
use crate::config::Config;
use crate::errors::{AppResult, pipeline};

/// Router generale dell'API.
///
/// Non conosce le classi delle feature: riceve dal container DI i sotto-router
/// già costruiti e li aggrega sotto `/api`. `main` chiama solo `build`.
pub async fn build(config: &Config) -> AppResult<Router> {
    let features = features::build_features(config).await?;

    let mut router = Router::new().nest("/api", api_router(features));
    router = router.merge(docs::swagger_router());
    Ok(router.fallback(pipeline::route_not_found))
}

/// Aggrega i sotto-router delle feature sotto `/api`: ogni feature registra le
/// proprie rotte relative (es. `/reports`, `/categories`).
fn api_router(features: Vec<Router<()>>) -> Router {
    let mut router = Router::new().route("/health", get(health));
    for feature in features {
        router = router.merge(feature);
    }
    router
}

async fn health() -> &'static str {
    "ok"
}
