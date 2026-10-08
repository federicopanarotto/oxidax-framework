pub mod surreal_user_repository;
pub mod user_controller;
pub mod user_dto;
pub mod user_entity;
pub mod user_repository;
pub mod user_service;

use di::Injectable;
use di::ServiceCollection;

use crate::db;
use crate::framework::{Feature, register_repository};
use user_service::UserService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `user_controller`. The identity repository is owned by
// the auth library (registered with the JWT settings at order 0).
fn register(repositories: &db::Repositories, services: &mut ServiceCollection) {
    register_repository(services, repositories.users.clone());
    services.add(UserService::singleton());
}

// Zero-touch self-registration in the feature registry.
inventory::submit! {
    Feature::new(3, register)
}
