pub mod auth_controller;
pub mod auth_dto;
pub mod auth_service;

use di::Injectable;
use di::ServiceCollection;

use crate::db;
use crate::framework::Feature;
use auth_service::AuthService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `auth_controller`. The identity repository and the JWT
// settings are registered by the auth library (order 0), the users repository
// by the users feature (order 2): this feature only owns the service.
fn register(_repositories: &db::Repositories, services: &mut ServiceCollection) {
    services.add(AuthService::singleton());
}

// Zero-touch self-registration in the feature registry.
inventory::submit! {
    Feature::new(4, register)
}
