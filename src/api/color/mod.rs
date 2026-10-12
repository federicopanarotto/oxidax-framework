pub mod color_controller;
pub mod color_dto;
pub mod color_entity;
pub mod color_repository;
pub mod color_service;
pub mod surreal_color_repository;

use di::Injectable;
use di::ServiceCollection;

use crate::framework::Feature;
use color_service::ColorService;

// All the DI wiring lives here; every route is auto-registered by
// `#[route]` on the handlers in `color_controller`. The repository is
// registered centrally in `crate::db`.
fn register(services: &mut ServiceCollection) {
    services.add(ColorService::singleton());
}

// Zero-touch self-registration in the feature registry.
inventory::submit! {
    Feature::new(0, register)
}
