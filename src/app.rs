//! Application assembly entry point.
//!
//! The generic assembly lives in `oxidax_core::app`; here we only supply the
//! application-specific pieces: which repositories to build for the configured
//! backend and the OpenAPI metadata. Nothing else changes when a feature is
//! added.

use axum::Router;
use di::ServiceProvider;
use oxidax_core::app::{self, AppInfo};

use crate::config::Config;
use crate::db;
use crate::errors::AppResult;

/// Builds the application router (fully assembled: DI, routes, Scalar API
/// reference, health endpoint, fallback and access log).
pub async fn build(config: &Config) -> AppResult<Router> {
    Ok(build_with_provider(config).await?.0)
}

/// Builds only the OpenAPI specification (JSON value) for the application.
pub async fn build_openapi(config: &Config) -> AppResult<serde_json::Value> {
    let repositories = db::repositories(config).await?;
    let (_, provider) = app::assemble(config, info(), move |services| {
        db::register_repositories(&repositories, services)
    })?;
    let bundles = oxidax_core::registry::Route::bundles(&provider);
    let (_routers, route_apis): (Vec<_>, Vec<_>) = bundles.into_iter().unzip();
    let api = oxidax_core::docs::build(route_apis, info());
    serde_json::to_value(api).map_err(|err| oxidax_core::errors::AppError::Internal(err.to_string()))
}

fn info() -> AppInfo<'static> {
    AppInfo {
        title: "Oxidax API",
        version: env!("CARGO_PKG_VERSION"),
        description: "Reports, categories and places management API",
    }
}

/// Like [`build`], but also returns the DI provider so callers (tests) can
/// resolve services or write directly to the same database instance.
pub async fn build_with_provider(config: &Config) -> AppResult<(Router, ServiceProvider)> {
    let repositories = db::repositories(config).await?;
    app::assemble(config, info(), move |services| {
        db::register_repositories(&repositories, services)
    })
}
