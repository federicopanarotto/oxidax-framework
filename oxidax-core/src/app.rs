//! Application assembly: dependency injection, routes and OpenAPI.
//!
//! Generic glue shared by every project. The application calls [`assemble`]
//! with the [`Config`], its OpenAPI metadata and a closure that registers the
//! concrete repositories; everything else (config injection, feature services,
//! self-registered routes, Swagger UI, health endpoint, fallback and access
//! log) is handled here.

use std::sync::Arc;

use axum::Router;
use axum::middleware::from_fn;
use axum::routing::get;
use di::ServiceCollection;
use di::ServiceProvider;
use di::singleton_factory;
use utoipa::openapi::OpenApi;

use crate::config::Config;
use crate::cors;
use crate::docs;
use crate::errors::{AppError, AppResult, pipeline};
use crate::middlewares::access_log::access_log;
use crate::registry::{Feature, Route};

/// Metadata of the generated OpenAPI document.
#[derive(Debug, Clone, Copy)]
pub struct AppInfo<'a> {
    /// Title shown in the Swagger UI and in the OpenAPI `info` block.
    pub title: &'a str,
    /// Version shown in the OpenAPI `info` block.
    pub version: &'a str,
    /// Short description of the API.
    pub description: &'a str,
}

/// Assembles the DI provider and the axum router of an application.
///
/// `register_repositories` is the single point where the application registers
/// its concrete repositories, whatever the database backend: features only
/// register their services, so this crate never enumerates project-specific
/// types. The returned provider is shared with the router; handing it back lets
/// tests (and other callers) resolve services directly.
pub fn assemble(
    config: &Config,
    info: AppInfo<'_>,
    register_repositories: impl FnOnce(&mut ServiceCollection),
) -> AppResult<(Router, ServiceProvider)> {
    let mut services = ServiceCollection::new();

    // The configuration is a shared dependency available to every feature: the
    // auth library resolves its own `JwtConfig` from it without any feature
    // knowing how the configuration is loaded.
    let owned = config.clone();
    services.add(singleton_factory::<Config, _>(
        move |_| -> di::Ref<Config> { Arc::new(owned.clone()) },
    ));

    register_repositories(&mut services);

    for feature in registered() {
        (feature.register)(&mut services);
    }

    let provider = services.build_provider().map_err(di_error)?;

    let bundles = route_bundles(&provider);
    let (routers, route_apis): (Vec<_>, Vec<_>) = bundles.into_iter().unzip();

    let api = docs::build(route_apis, info);
    let mut router = Router::new().route("/api/health", get(health));
    for route in routers {
        router = router.merge(route);
    }
    router = router.merge(docs::swagger_router(api));

    let router = router
        .fallback(pipeline::route_not_found)
        .layer(from_fn(access_log));

    // CORS sits outermost so preflight (`OPTIONS`) requests are answered before
    // routing and the headers are added to every response, errors included. It
    // is only installed when an origin allow-list is configured.
    let router = match cors::layer(&config.cors_allowed_origins) {
        Some(cors) => router.layer(cors),
        None => router,
    };

    Ok((router, provider))
}

/// Registered features, ordered by dependencies.
fn registered() -> Vec<&'static Feature> {
    let mut features: Vec<_> = inventory::iter::<Feature>().collect();
    features.sort_by_key(|feature| feature.order);
    features
}

/// Builds the axum router and OpenAPI fragment of every self-registered
/// route, resolving the services from the DI provider.
fn route_bundles(provider: &ServiceProvider) -> Vec<(Router, OpenApi)> {
    inventory::iter::<Route>()
        .map(|route| (route.build)(provider))
        .collect()
}

fn di_error(err: impl std::fmt::Display) -> AppError {
    AppError::Internal(format!("dependency injection: {err}"))
}

async fn health() -> &'static str {
    "ok"
}
