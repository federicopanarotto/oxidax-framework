use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::{RecordId, SurrealValue};

use crate::api::reports::report_entity::Report;
use crate::api::reports::report_repository::ReportRepository;
use crate::db::{define_table, query, record_key_to_string};
use crate::errors::{AppError, AppResult};

const TABLE: &str = "report";

/// Row as it arrives from the database (the id is a SurrealDB record id).
/// `user_id` is `Option` because rows created before authentication existed
/// do not carry the field at all (a missing field deserializes to `None`).
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct ReportRow {
    id: RecordId,
    name: String,
    description: String,
    category_id: Option<String>,
    place_id: Option<String>,
    user_id: Option<String>,
}

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

/// The API works with the record id key alone (`uuid`), not with `table:uuid`.
fn into_entity(row: ReportRow) -> Report {
    Report {
        id: record_key_to_string(row.id.key),
        name: row.name,
        description: row.description,
        category_id: row.category_id,
        place_id: row.place_id,
        user_id: row.user_id,
    }
}

fn created_or_internal(report: Report, created: Option<ReportRow>) -> AppResult<Report> {
    if created.is_some() {
        Ok(report)
    } else {
        Err(AppError::Internal(format!(
            "failed to create report {}",
            report.id
        )))
    }
}

#[async_trait]
impl ReportRepository for SurrealReportRepository {
    async fn insert(&self, report: Report) -> AppResult<Report> {
        let content = ReportContent {
            name: report.name.clone(),
            description: report.description.clone(),
            category_id: report.category_id.clone(),
            place_id: report.place_id.clone(),
            user_id: report.user_id.clone(),
        };

        let created: Option<ReportRow> = self
            .db
            .create((TABLE, report.id.as_str()))
            .content(content)
            .await?;

        created_or_internal(report, created)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<Report>> {
        let found: Option<ReportRow> = self.db.select((TABLE, id)).await?;
        Ok(found.map(into_entity))
    }

    async fn find_all(&self) -> AppResult<Vec<Report>> {
        let rows: Vec<ReportRow> = self.db.select(TABLE).await?;
        Ok(rows.into_iter().map(into_entity).collect())
    }

    async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<Report>> {
        let rows: Vec<ReportRow> = query::select_page(&self.db, TABLE, offset, limit).await?;
        Ok(rows.into_iter().map(into_entity).collect())
    }

    async fn count(&self) -> AppResult<u64> {
        query::count(&self.db, TABLE).await
    }

    async fn replace(&self, id: &str, report: Report) -> AppResult<Option<Report>> {
        // Merge instead of content: only the mutable fields are rewritten and
        // the creator (`user_id`) is preserved as it is.
        let patch = ReportPatch {
            name: report.name.clone(),
            description: report.description.clone(),
            category_id: report.category_id.clone(),
            place_id: report.place_id.clone(),
        };

        let updated: Option<ReportRow> = self.db.update((TABLE, id)).merge(patch).await?;
        Ok(updated.map(into_entity))
    }

    async fn remove(&self, id: &str) -> AppResult<bool> {
        let deleted: Option<ReportRow> = self.db.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}
