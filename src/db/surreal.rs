use std::sync::Arc;

use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;

use crate::api::reports::report_repository::ReportRepository;
use crate::api::reports::surreal_report_repository::SurrealReportRepository;
use crate::config::Config;
use crate::errors::{AppError, AppResult};

pub async fn connect(url: &str, config: &Config) -> AppResult<Arc<dyn ReportRepository>> {
    let db = Surreal::<Any>::init();

    db.connect(url).await.map_err(into_db_error)?;

    if needs_auth(url) && !config.surreal_user.is_empty() {
        db.signin(Root {
            username: config.surreal_user.clone(),
            password: config.surreal_pass.clone(),
        })
        .await
        .map_err(into_db_error)?;
    }

    db.use_ns(&config.surreal_ns)
        .use_db(&config.surreal_db)
        .await
        .map_err(into_db_error)?;

    tracing::info!(url, ns = %config.surreal_ns, db = %config.surreal_db, "connected to SurrealDB");

    let repository = SurrealReportRepository::new(db);
    repository.ensure_schema().await?;

    Ok(Arc::new(repository))
}

fn needs_auth(url: &str) -> bool {
    ["ws://", "wss://", "http://", "https://"]
        .iter()
        .any(|scheme| url.starts_with(scheme))
}

fn into_db_error(err: surrealdb::Error) -> AppError {
    AppError::Database(err.to_string())
}
