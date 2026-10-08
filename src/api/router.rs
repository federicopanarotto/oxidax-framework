use axum::Router;
use axum::routing::get;

use crate::api::docs;
use crate::api::features;
use crate::config::Config;
use crate::errors::{AppResult, pipeline};

/// Overall API router.
///
/// It knows nothing about features: it opens the DI provider, has every route
/// (self-registered by `#[route]` in its own controller) build its router and
/// OpenAPI fragment, then aggregates everything. The collected OpenAPI
/// fragments feed the Swagger UI.
pub async fn build(config: &Config) -> AppResult<Router> {
    let provider = features::provider(config).await?;
    let bundles = features::route_bundles(&provider);
    let (routers, route_apis): (Vec<_>, Vec<_>) = bundles.into_iter().unzip();

    let api = docs::build(route_apis);
    let mut router = Router::new().route("/api/health", get(health));
    for route in routers {
        router = router.merge(route);
    }
    router = router.merge(docs::swagger_router(api));
    Ok(router.fallback(pipeline::route_not_found))
}

async fn health() -> &'static str {
    "ok"
}
