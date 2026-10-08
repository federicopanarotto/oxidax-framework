//! Feature composition via dependency injection.
//!
//! This file is **generic**: features self-register in the static registries
//! ([`Feature`] for the DI dependencies, [`Route`] for every handler) and here
//! we only iterate over them. Adding a new feature means creating its own
//! modules (dto, service, controller with `#[route]` handlers) and registering
//! `inventory::submit!` for the DI part: `features.rs`, `router.rs` and
//! `docs.rs` are never touched again.

use std::sync::Arc;

use axum::Router;
use di::ServiceCollection;
use di::ServiceProvider;
use di::singleton_factory;
use utoipa::openapi::OpenApi;

use crate::config::Config;
use crate::db;
use crate::errors::{AppError, AppResult};
use crate::lib::registry::{Feature, Route};

/// Builds the DI provider by registering every discovered feature.
///
/// First *all* of the feature's dependencies (repository and service) are
/// registered, then the provider is built: services are created on the first
/// `get_required` (lazy injection by `more-di`), so registration order is not
/// critical; as a safety measure each feature's `order` is still used.
pub async fn provider(config: &Config) -> AppResult<ServiceProvider> {
    let repositories = db::repositories(config).await?;

    let mut services = ServiceCollection::new();

    // The configuration is a shared dependency available to every feature: the
    // auth library resolves its own `JwtConfig` from it without any feature
    // knowing how the configuration is loaded.
    let owned = config.clone();
    services.add(singleton_factory::<Config, _>(
        move |_| -> di::Ref<Config> { Arc::new(owned.clone()) },
    ));

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

/// Builds the axum router and OpenAPI fragment of every self-registered
/// route, resolving the services from the DI provider.
pub fn route_bundles(provider: &ServiceProvider) -> Vec<(Router, OpenApi)> {
    inventory::iter::<Route>()
        .map(|route| (route.build)(provider))
        .collect()
}

fn di_error(err: impl std::fmt::Display) -> AppError {
    AppError::Internal(format!("dependency injection: {err}"))
}
