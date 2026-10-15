use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::SurrealValue;

use crate::api::reports::report_entity::Report;
use crate::api::reports::report_repository::ReportRepository;
use crate::api::reports::report_response::ReportResponse;
use crate::db::{define_table, query};
use crate::errors::AppResult;
use crate::framework::ViewRepository;

const TABLE: &str = "report";

/// Persistable content: the id is managed by the database, it must not be
/// written in the CONTENT.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct ReportContent {
    name: String,
    description: String,
    category_id: Option<String>,
    place_id: Option<String>,
    user_id: Option<String>,
}

/// Fields rewritten by `replace`: `user_id` is deliberately absent, so the
/// update is a merge that can never change who created the report.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct ReportPatch {
    name: String,
    description: String,
    category_id: Option<String>,
    place_id: Option<String>,
}

pub struct SurrealReportRepository {
    db: Surreal<Any>,
}

impl SurrealReportRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Creates the table if it does not exist: SurrealDB 3 does not create
    /// tables implicitly.
    pub async fn ensure_schema(&self) -> AppResult<()> {
        define_table(&self.db, TABLE).await
    }
}

/// Enables all the populated reads (`find`, `find_page`, `count`, `delete`) on
/// top of the shared connection.
#[async_trait]
impl ViewRepository for SurrealReportRepository {
    type Response = ReportResponse;

    fn db(&self) -> &Surreal<Any> {
        &self.db
    }
}

#[async_trait]
impl ReportRepository for SurrealReportRepository {
    /// Writes the row and returns the read-model already populated: the
    /// relations are resolved by the same database, the service never decorates.
    async fn insert(&self, report: Report) -> AppResult<ReportResponse> {
        let content = ReportContent {
            name: report.name,
            description: report.description,
            category_id: report.category_id,
            place_id: report.place_id,
            user_id: report.user_id,
        };
        query::insert_view(&self.db, &report.id, content).await
    }

    /// Merges the mutable fields and returns the read-model: the creator
    /// (`user_id`) is preserved because it is not part of the patch.
    async fn replace(&self, id: &str, report: Report) -> AppResult<Option<ReportResponse>> {
        let patch = ReportPatch {
            name: report.name,
            description: report.description,
            category_id: report.category_id,
            place_id: report.place_id,
        };
        query::replace_view(&self.db, id, patch).await
    }
}
