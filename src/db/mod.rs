mod surreal;

use std::sync::Arc;

use crate::api::reports::report_repository::ReportRepository;
use crate::config::{Config, DbBackend};
use crate::errors::AppResult;

/// Crea il repository dei report in base al backend configurato.
///
/// Cambiare database significa implementare `ReportRepository` e aggiungere
/// un branch qui: il resto dell'applicazione (service, controller) non cambia.
pub async fn report_repository(config: &Config) -> AppResult<Arc<dyn ReportRepository>> {
    match config.db_backend {
        DbBackend::Memory => surreal::connect("mem://", config).await,
        DbBackend::Surreal => surreal::connect(&config.surreal_url, config).await,
    }
}

/// Repository embedded in-memory (SurrealDB `mem://`): usato dai test e
/// disponibile a runtime con `DB_BACKEND=memory`.
pub async fn memory_repository() -> AppResult<Arc<dyn ReportRepository>> {
    surreal::connect("mem://", &Config::from_env()).await
}
