pub mod category_controller;
pub mod category_dto;
pub mod category_entity;
pub mod category_repository;
pub mod category_service;
pub mod surreal_category_repository;

use di::Injectable;
use di::ServiceCollection;

use crate::db;
use crate::framework::{Feature, register_repository};
use category_service::CategoryService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `category_controller`.
fn register(repositories: &db::Repositories, services: &mut ServiceCollection) {
    register_repository(services, repositories.categories.clone());
    services.add(CategoryService::singleton());
}

// Zero-touch self-registration in the feature registry.
inventory::submit! {
    Feature::new(0, register)
}
