pub mod surreal_upload_repository;
pub mod upload_controller;
pub mod upload_dto;
pub mod upload_entity;
pub mod upload_repository;
pub mod upload_service;

use di::Injectable;
use di::ServiceCollection;

use crate::framework::Feature;
use upload_service::UploadService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `upload_controller`. The repository is registered
// centrally in `crate::db`.
fn register(services: &mut ServiceCollection) {
    services.add(UploadService::singleton());
}

// Zero-touch self-registration in the feature registry. Order 5: it runs after
// the other features, so it can depend on anything they register.
inventory::submit! {
    Feature::new(5, register)
}
