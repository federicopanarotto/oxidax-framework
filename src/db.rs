//! Concrete repositories of the application and their DI registration.
//!
//! This is the single place that knows the concrete repository types and the
//! configured database backend. The reusable helpers come from `oxidax-core`;
//! here we keep the project-specific `Repositories` aggregate and register it
//! in the dependency-injection container, so features only deal with the
//! `...Repository` traits.

use std::sync::Arc;

use di::ServiceCollection;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use crate::api::categories::category_repository::CategoryRepository;
use crate::api::categories::surreal_category_repository::SurrealCategoryRepository;
use crate::api::color::color_repository::ColorRepository;
use crate::api::color::surreal_color_repository::SurrealColorRepository;
use crate::api::places::place_repository::PlaceRepository;
use crate::api::places::surreal_place_repository::SurrealPlaceRepository;
use crate::api::reports::report_repository::ReportRepository;
use crate::api::reports::surreal_report_repository::SurrealReportRepository;
use crate::api::uploads::surreal_upload_repository::SurrealAssetRepository;
use crate::api::uploads::upload_repository::AssetRepository;
use crate::api::users::surreal_user_repository::SurrealUserRepository;
use crate::api::users::user_repository::UserRepository;
use crate::config::{Config, DbBackend};
use crate::errors::AppResult;
use crate::framework::register_repository;
use crate::lib::auth::{SurrealUserIdentityRepository, UserIdentityRepository};

pub use oxidax_core::db::query;
pub use oxidax_core::db::{define_table, record_key_to_string};

/// All the application repositories, built on the same connection.
pub struct Repositories {
    pub reports: Arc<dyn ReportRepository>,
    pub categories: Arc<dyn CategoryRepository>,
    pub places: Arc<dyn PlaceRepository>,
    pub color: Arc<dyn ColorRepository>,
    pub users: Arc<dyn UserRepository>,
    pub user_identities: Arc<dyn UserIdentityRepository>,
    pub assets: Arc<dyn AssetRepository>,
}

/// Creates the repositories based on the configured backend.
///
/// SurrealDB is the default backend; switching database means implementing the
/// repository traits and adding a new branch here. The rest of the application
/// (services, controllers) does not change.
pub async fn repositories(config: &Config) -> AppResult<Repositories> {
    let url = match config.db_backend {
        DbBackend::Memory => "mem://",
        DbBackend::Surreal => config.surreal_url.as_str(),
    };

    from_connection(oxidax_core::db::connect(url, config).await?).await
}

/// Embedded in-memory repositories (SurrealDB `mem://`): used by the tests and
/// available at runtime with `DB_BACKEND=memory`.
pub async fn memory_repositories() -> AppResult<Repositories> {
    repositories(&Config::memory()).await
}

/// Registers every repository in the DI container: the single wired point that
/// features no longer need to know about.
pub fn register_repositories(repositories: &Repositories, services: &mut ServiceCollection) {
    register_repository(services, repositories.reports.clone());
    register_repository(services, repositories.categories.clone());
    register_repository(services, repositories.places.clone());
    register_repository(services, repositories.color.clone());
    register_repository(services, repositories.users.clone());
    register_repository(services, repositories.user_identities.clone());
    register_repository(services, repositories.assets.clone());
}

async fn from_connection(db: Surreal<Any>) -> AppResult<Repositories> {
    let reports = SurrealReportRepository::new(db.clone());
    reports.ensure_schema().await?;

    let categories = SurrealCategoryRepository::new(db.clone());
    categories.ensure_schema().await?;

    let places = SurrealPlaceRepository::new(db.clone());
    places.ensure_schema().await?;

    let color = SurrealColorRepository::new(db.clone());
    color.ensure_schema().await?;

    let users = SurrealUserRepository::new(db.clone());
    users.ensure_schema().await?;

    let assets = SurrealAssetRepository::new(db.clone());
    assets.ensure_schema().await?;

    let user_identities = SurrealUserIdentityRepository::new(db);
    user_identities.ensure_schema().await?;

    Ok(Repositories {
        reports: Arc::new(reports),
        categories: Arc::new(categories),
        places: Arc::new(places),
        color: Arc::new(color),
        users: Arc::new(users),
        user_identities: Arc::new(user_identities),
        assets: Arc::new(assets),
    })
}
