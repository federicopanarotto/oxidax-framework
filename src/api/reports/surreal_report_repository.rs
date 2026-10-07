use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::{RecordId, SurrealValue};

use crate::api::reports::report_entity::Report;
use crate::api::reports::report_repository::ReportRepository;
use crate::db::{define_table, record_key_to_string};
use crate::errors::{AppError, AppResult};

const TABLE: &str = "report";

/// Row as it arrives from the database (the id is a SurrealDB record id).
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct ReportRow {
    id: RecordId,
    name: String,
    description: String,
    category_id: Option<String>,
}

/// Persistable content: the id is managed by the database, it must not be
/// written in the CONTENT.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct ReportContent {
    name: String,
    description: String,
    category_id: Option<String>,
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

    async fn replace(&self, id: &str, report: Report) -> AppResult<Option<Report>> {
        let content = ReportContent {
            name: report.name.clone(),
            description: report.description.clone(),
            category_id: report.category_id.clone(),
        };

        let updated: Option<ReportRow> = self.db.update((TABLE, id)).content(content).await?;
        Ok(updated.map(into_entity))
    }

    async fn remove(&self, id: &str) -> AppResult<bool> {
        let deleted: Option<ReportRow> = self.db.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}
