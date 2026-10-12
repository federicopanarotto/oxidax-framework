pub mod category_controller;
pub mod category_dto;
pub mod category_entity;
pub mod category_repository;
pub mod category_service;
pub mod surreal_category_repository;

use di::Injectable;
use di::ServiceCollection;

use crate::framework::Feature;
use category_service::CategoryService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `category_controller`. The repository is registered
// centrally in `crate::db`.
fn register(services: &mut ServiceCollection) {
    services.add(CategoryService::singleton());
}

// Zero-touch self-registration in the feature registry.
inventory::submit! {
    Feature::new(0, register)
}
