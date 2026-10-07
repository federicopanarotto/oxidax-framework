pub mod category_controller;
pub mod category_dto;
pub mod category_entity;
pub mod category_repository;
pub mod category_service;
pub mod surreal_category_repository;

use axum::Router;
use di::Injectable;
use di::{ServiceCollection, ServiceProvider};
use utoipa::openapi::OpenApi;

use crate::db;
use crate::lib::registry::{Feature, register_repository};
use category_service::CategoryService;

// All the wiring "dirty work" lives here, keeping the other files clean.
fn register(repositories: &db::Repositories, services: &mut ServiceCollection) {
    register_repository(services, repositories.categories.clone());
    services.add(CategoryService::singleton());
}

fn build(provider: &ServiceProvider) -> (Router, OpenApi) {
    let service = provider.get_required::<CategoryService>();
    category_controller::build_router(service)
}

inventory::submit! {
    Feature::new(0, register, build)
}
