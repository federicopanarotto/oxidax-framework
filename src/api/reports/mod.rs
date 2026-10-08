pub mod report_controller;
pub mod report_dto;
pub mod report_entity;
pub mod report_repository;
pub mod report_service;
pub mod surreal_report_repository;

use di::Injectable;
use di::ServiceCollection;

use crate::db;
use crate::framework::{Feature, register_repository};
use report_service::ReportService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `report_controller`.
fn register(repositories: &db::Repositories, services: &mut ServiceCollection) {
    register_repository(services, repositories.reports.clone());
    services.add(ReportService::singleton());
}

// Zero-touch self-registration in the feature registry. The `order` of 1
// guarantees that categories (a dependency of reports) are registered first.
inventory::submit! {
    Feature::new(1, register)
}
