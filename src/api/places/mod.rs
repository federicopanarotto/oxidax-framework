pub mod place_controller;
pub mod place_dto;
pub mod place_entity;
pub mod place_repository;
pub mod place_service;
pub mod surreal_place_repository;

use di::Injectable;
use di::ServiceCollection;

use crate::framework::Feature;
use place_service::PlaceService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `place_controller`. The repository is registered centrally
// in `crate::db`.
fn register(services: &mut ServiceCollection) {
    services.add(PlaceService::singleton());
}

// Zero-touch self-registration in the feature registry.
inventory::submit! {
    Feature::new(1, register)
}
