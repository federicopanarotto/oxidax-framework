use axum::Router;
use axum::routing::get;

use crate::api::docs;
use crate::api::features;
use crate::config::Config;
use crate::errors::{AppResult, pipeline};

/// Overall API router.
///
/// It knows nothing about features: it opens the DI provider, has each
/// feature (self-registered in the registry) build its own router and
/// OpenAPI components, then aggregates everything under `/api`. The collected
/// OpenAPI components feed the Swagger UI.
pub async fn build(config: &Config) -> AppResult<Router> {
    let provider = features::provider(config).await?;
    let bundles = features::feature_bundles(&provider);
    let (routers, feature_apis): (Vec<_>, Vec<_>) = bundles.into_iter().unzip();

    let api = docs::build(feature_apis);
    let mut router = Router::new().route("/api/health", get(health));
    for feature in routers {
        router = router.merge(feature);
    }
    router = router.merge(docs::swagger_router(api));
    Ok(router.fallback(pipeline::route_not_found))
}

async fn health() -> &'static str {
    "ok"
}
