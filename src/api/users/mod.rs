pub mod surreal_user_repository;
pub mod user_controller;
pub mod user_dto;
pub mod user_entity;
pub mod user_repository;
pub mod user_response;
pub mod user_service;

use di::Injectable;
use di::ServiceCollection;

use crate::framework::Feature;
use user_service::UserService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `user_controller`. The identity repository is owned by
// the auth library and registered centrally in `crate::db`.
fn register(services: &mut ServiceCollection) {
    services.add(UserService::singleton());
}

// Zero-touch self-registration in the feature registry (order 2: the reports
// feature depends on the users service to decorate the creator).
inventory::submit! {
    Feature::new(2, register)
}
