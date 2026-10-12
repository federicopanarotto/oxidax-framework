use async_trait::async_trait;

use crate::api::reports::report_entity::Report;
use crate::errors::AppResult;

/// Persistence contract for reports.
///
/// It is the only point that depends on the database: implementing this
/// trait on a new backend (Postgres, SQLite, MongoDB...) leaves the service
/// and controllers unchanged.
#[async_trait]
pub trait ReportRepository: Send + Sync {
    async fn insert(&self, report: Report) -> AppResult<Report>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Report>>;
    async fn find_all(&self) -> AppResult<Vec<Report>>;
    /// One page of reports: `offset` skipped, at most `limit` returned.
    async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<Report>>;
    /// Total number of reports, used to fill the pagination metadata.
    async fn count(&self) -> AppResult<u64>;
    async fn replace(&self, id: &str, report: Report) -> AppResult<Option<Report>>;
    async fn remove(&self, id: &str) -> AppResult<bool>;
}
