//! Zero-touch feature discovery.
//!
//! Every feature self-registers here via [`inventory::submit!`]: it provides
//! how to register it in the DI container (repository + service) and how to
//! build routers and OpenAPI components. `features.rs`, `router.rs` and
//! `docs.rs` know nothing about individual features: they only iterate this
//! registry, so adding a new feature no longer requires any line in the
//! wiring files.

use std::sync::Arc;

use axum::Router;
use di::singleton_factory;
use di::{ServiceCollection, ServiceProvider};
use utoipa::openapi::OpenApi;

use crate::db;

/// Feature self-registered in the registry.
///
/// It is not a runtime object: it is a static value that says *which*
/// dependencies the feature requires in the DI (`register`) and *how* to
/// build it (`build`).
pub struct Feature {
    /// Registration order in the container: a feature's dependencies must be
    /// registered before the code that uses them (lazy injection in `more-di`
    /// makes the order tolerant, but keeping it explicit is safer).
    pub order: u8,
    /// Registers the feature's repository and service in the DI container.
    pub register: fn(repositories: &db::Repositories, services: &mut ServiceCollection),
    /// Builds the axum router and OpenAPI components by resolving the
    /// services from the DI provider.
    pub build: fn(provider: &ServiceProvider) -> (Router, OpenApi),
}

impl Feature {
    pub const fn new(
        order: u8,
        register: fn(repositories: &db::Repositories, services: &mut ServiceCollection),
        build: fn(provider: &ServiceProvider) -> (Router, OpenApi),
    ) -> Self {
        Self {
            order,
            register,
            build,
        }
    }
}

// Collects all occurrences of [`Feature`] in the static registry.
// `collect!` must live in the crate that defines the type: it is here, and all
// features can then register themselves with `inventory::submit!`.
inventory::collect!(Feature);

/// Registers an already-built dependency (a repository) in the DI under its
/// trait, simplifying the registration line of every feature.
pub fn register_repository<S: ?Sized + Send + Sync + 'static>(
    services: &mut ServiceCollection,
    repository: Arc<S>,
) {
    services.add(singleton_factory::<S, _>(move |_| -> di::Ref<S> {
        repository.clone()
    }));
}
