use async_trait::async_trait;

use crate::api::reports::report_entity::Report;
use crate::error::AppResult;

/// Contratto di persistenza per i report.
///
/// È l'unico punto che dipende dal database: implementando questo trait su un
/// nuovo backend (Postgres, SQLite, MongoDB...) il service e i controller
/// restano invariati.
#[async_trait]
pub trait ReportRepository: Send + Sync {
    async fn insert(&self, report: Report) -> AppResult<Report>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Report>>;
    async fn find_all(&self) -> AppResult<Vec<Report>>;
    async fn replace(&self, id: &str, report: Report) -> AppResult<Option<Report>>;
    async fn remove(&self, id: &str) -> AppResult<bool>;
}
