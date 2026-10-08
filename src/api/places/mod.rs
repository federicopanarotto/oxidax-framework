pub mod place_controller;
pub mod place_dto;
pub mod place_entity;
pub mod place_repository;
pub mod place_service;
pub mod surreal_place_repository;

use di::Injectable;
use di::ServiceCollection;

use crate::db;
use crate::framework::{Feature, register_repository};
use place_service::PlaceService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `place_controller`.
fn register(repositories: &db::Repositories, services: &mut ServiceCollection) {
    register_repository(services, repositories.places.clone());
    services.add(PlaceService::singleton());
}

// Zero-touch self-registration in the feature registry.
inventory::submit! {
    Feature::new(1, register)
}
