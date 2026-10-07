//! Feature composition via dependency injection.
//!
//! This file is **generic**: features self-register in the static
//! [`Feature`](crate::lib::registry::Feature) registry and here we only
//! iterate over it. Adding a new feature means creating its own modules
//! (dto, service, controller) and registering `inventory::submit!` there:
//! `features.rs`, `router.rs` and `docs.rs` are never touched again.

use axum::Router;
use di::ServiceCollection;
use di::ServiceProvider;
use utoipa::openapi::OpenApi;

use crate::config::Config;
use crate::db;
use crate::errors::{AppError, AppResult};
use crate::lib::registry::Feature;

/// Builds the DI provider by registering every recognized feature.
///
/// First *all* of the feature's dependencies (repository and service) are
/// registered, then the provider is built: services are created on the first
/// `get_required` (lazy injection by `more-di`), so registration order is not
/// critical; as a safety measure each feature's `order` is still used.
pub async fn provider(config: &Config) -> AppResult<ServiceProvider> {
    let repositories = db::repositories(config).await?;

    let mut services = ServiceCollection::new();
    for feature in registered() {
        (feature.register)(&repositories, &mut services);
    }

    services.build_provider().map_err(di_error)
}

/// Registered features, ordered by dependencies.
fn registered() -> Vec<&'static Feature> {
    let mut features: Vec<_> = inventory::iter::<Feature>().collect();
    features.sort_by_key(|feature| feature.order);
    features
}

/// Builds each feature's axum router and OpenAPI components, resolving the
/// services from the DI provider.
pub fn feature_bundles(provider: &ServiceProvider) -> Vec<(Router, OpenApi)> {
    registered()
        .into_iter()
        .map(|feature| (feature.build)(provider))
        .collect()
}

fn di_error(err: impl std::fmt::Display) -> AppError {
    AppError::Internal(format!("dependency injection: {err}"))
}
