pub mod report_controller;
pub mod report_dto;
pub mod report_entity;
pub mod report_repository;
pub mod report_response;
pub mod report_service;
pub mod surreal_report_repository;

use di::Injectable;
use di::ServiceCollection;

use crate::framework::Feature;
use report_service::ReportService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `report_controller`. The repository is registered
// centrally in `crate::db`.
fn register(services: &mut ServiceCollection) {
    services.add(ReportService::singleton());
}

// Zero-touch self-registration in the feature registry. The `order` of 3
// guarantees that the dependencies of reports (categories, places and the
// users service for the creator) are registered first.
inventory::submit! {
    Feature::new(3, register)
}
