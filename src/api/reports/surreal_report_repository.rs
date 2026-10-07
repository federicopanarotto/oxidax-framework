use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::engine::any::Any;
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue, ToSql};
use surrealdb::Surreal;

use crate::api::reports::report_entity::Report;
use crate::api::reports::report_repository::ReportRepository;
use crate::error::{AppError, AppResult};

const TABLE: &str = "report";

/// Riga così come arriva dal database (l'id è un record id SurrealDB).
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct ReportRow {
    id: RecordId,
    name: String,
    description: String,
}

/// Contenuto persistibile: l'id è gestito dal database, non va scritto nel CONTENT.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct ReportContent {
    name: String,
    description: String,
}

pub struct SurrealReportRepository {
    db: Surreal<Any>,
}

impl SurrealReportRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Crea la tabella se non esiste: SurrealDB 3 non crea tabelle implicitamente.
    pub async fn ensure_schema(&self) -> AppResult<()> {
        self.db
            .query(format!("DEFINE TABLE IF NOT EXISTS {TABLE} SCHEMALESS"))
            .await?
            .check()?;
        Ok(())
    }
}

/// L'API lavora con la sola chiave del record id (`uuid`), non con `table:uuid`.
fn into_entity(row: ReportRow) -> Report {
    Report {
        id: record_key_to_string(row.id.key),
        name: row.name,
        description: row.description,
    }
}

fn record_key_to_string(key: RecordIdKey) -> String {
    match key {
        RecordIdKey::String(value) => value,
        RecordIdKey::Uuid(value) => value.to_string(),
        RecordIdKey::Number(value) => value.to_string(),
        other => other.to_sql(),
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
        };

        let updated: Option<ReportRow> = self.db.update((TABLE, id)).content(content).await?;
        Ok(updated.map(into_entity))
    }

    async fn remove(&self, id: &str) -> AppResult<bool> {
        let deleted: Option<ReportRow> = self.db.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}
