use async_trait::async_trait;

use crate::api::reports::report_entity::Report;
use crate::api::reports::report_response::ReportResponse;
use crate::errors::AppResult;
use crate::framework::ViewRepository;

/// Persistence contract for reports.
///
/// It inherits the populated reads (`find`, `find_page`, `count`, `delete`)
/// from [`ViewRepository`]: the relations (`category`, `place`, `created_by`)
/// are resolved by the database in a single query, never row by row and never
/// in the service — no N+1, no manual decoration. Only the aggregate-specific
/// writes are declared here.
#[async_trait]
pub trait ReportRepository: ViewRepository<Response = ReportResponse> {
    /// Persists a new report and returns it populated.
    async fn insert(&self, report: Report) -> AppResult<ReportResponse>;
    /// Overwrites the mutable fields and returns the populated report; `None`
    /// if it does not exist.
    async fn replace(&self, id: &str, report: Report) -> AppResult<Option<ReportResponse>>;
}
