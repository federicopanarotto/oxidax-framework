mod surreal;

use std::sync::Arc;

use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use crate::api::categories::category_repository::CategoryRepository;
use crate::api::categories::surreal_category_repository::SurrealCategoryRepository;
use crate::api::reports::report_repository::ReportRepository;
use crate::api::reports::surreal_report_repository::SurrealReportRepository;
use crate::config::{Config, DbBackend};
use crate::errors::AppResult;

pub use surreal::{define_table, record_key_to_string};

/// All the application repositories, built on the same connection.
pub struct Repositories {
    pub reports: Arc<dyn ReportRepository>,
    pub categories: Arc<dyn CategoryRepository>,
}

/// Creates the repositories based on the configured backend.
///
/// Switching database means implementing the `ReportRepository` and
/// `CategoryRepository` traits and adding a new branch here: the rest of the
/// application (services, controllers) does not change.
pub async fn repositories(config: &Config) -> AppResult<Repositories> {
    let url = match config.db_backend {
        DbBackend::Memory => "mem://",
        DbBackend::Surreal => config.surreal_url.as_str(),
    };

    from_connection(surreal::connect(url, config).await?).await
}

/// Embedded in-memory repositories (SurrealDB `mem://`): used by the tests and
/// available at runtime with `DB_BACKEND=memory`.
pub async fn memory_repositories() -> AppResult<Repositories> {
    repositories(&Config::memory()).await
}

async fn from_connection(db: Surreal<Any>) -> AppResult<Repositories> {
    let reports = SurrealReportRepository::new(db.clone());
    reports.ensure_schema().await?;

    let categories = SurrealCategoryRepository::new(db);
    categories.ensure_schema().await?;

    Ok(Repositories {
        reports: Arc::new(reports),
        categories: Arc::new(categories),
    })
}
