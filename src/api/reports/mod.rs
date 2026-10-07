pub mod report_controller;
pub mod report_dto;
pub mod report_entity;
pub mod report_repository;
pub mod report_service;
pub mod surreal_report_repository;

use axum::Router;
use di::Injectable;
use di::{ServiceCollection, ServiceProvider};
use utoipa::openapi::OpenApi;

use crate::db;
use crate::lib::registry::{Feature, register_repository};
use report_service::ReportService;

// All the wiring "dirty work" lives here, keeping the other files clean.
fn register(repositories: &db::Repositories, services: &mut ServiceCollection) {
    register_repository(services, repositories.reports.clone());
    services.add(ReportService::singleton());
}

fn build(provider: &ServiceProvider) -> (Router, OpenApi) {
    let service = provider.get_required::<ReportService>();
    report_controller::build_router(service)
}

// Zero-touch self-registration in the feature registry. The `order` of 1
// guarantees that categories (a dependency of reports) are registered first.
inventory::submit! {
    Feature::new(1, register, build)
}
