use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::{RecordId, SurrealValue};

use crate::api::color::color_entity::Color;
use crate::api::color::color_repository::ColorRepository;
use crate::db::{define_table, query, record_key_to_string};
use crate::errors::{AppError, AppResult};

const TABLE: &str = "color";

/// Row as it arrives from the database (the id is a SurrealDB record id).
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct ColorRow {
    id: RecordId,
    name: String,
    hex: String,
}

/// Persistable content: the id is managed by the database.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct ColorContent {
    name: String,
    hex: String,
}

pub struct SurrealColorRepository {
    db: Surreal<Any>,
}

impl SurrealColorRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Creates the table if it does not exist: SurrealDB 3 does not
    /// create tables implicitly.
    pub async fn ensure_schema(&self) -> AppResult<()> {
        define_table(&self.db, TABLE).await
    }
}

/// The API works with the record id key alone (`uuid`), not `table:uuid`.
fn into_entity(row: ColorRow) -> Color {
    Color {
        id: record_key_to_string(row.id.key),
        name: row.name,
        hex: row.hex,
    }
}

fn created_or_internal(item: Color, created: Option<ColorRow>) -> AppResult<Color> {
    if created.is_some() {
        Ok(item)
    } else {
        Err(AppError::Internal(format!(
            "failed to create color {}",
            item.id
        )))
    }
}

#[async_trait]
impl ColorRepository for SurrealColorRepository {
    async fn insert(&self, item: Color) -> AppResult<Color> {
        let content = ColorContent {
            name: item.name.clone(),
            hex: item.hex.clone(),
        };

        let created: Option<ColorRow> = self
            .db
            .create((TABLE, item.id.as_str()))
            .content(content)
            .await?;

        created_or_internal(item, created)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<Color>> {
        let found: Option<ColorRow> = self.db.select((TABLE, id)).await?;
        Ok(found.map(into_entity))
    }

    async fn find_all(&self) -> AppResult<Vec<Color>> {
        let rows: Vec<ColorRow> = self.db.select(TABLE).await?;
        Ok(rows.into_iter().map(into_entity).collect())
    }

    async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<Color>> {
        let rows: Vec<ColorRow> = query::select_page(&self.db, TABLE, offset, limit).await?;
        Ok(rows.into_iter().map(into_entity).collect())
    }

    async fn count(&self) -> AppResult<u64> {
        query::count(&self.db, TABLE).await
    }

    async fn replace(&self, id: &str, item: Color) -> AppResult<Option<Color>> {
        let content = ColorContent {
            name: item.name.clone(),
            hex: item.hex.clone(),
        };

        let updated: Option<ColorRow> = self.db.update((TABLE, id)).content(content).await?;
        Ok(updated.map(into_entity))
    }

    async fn remove(&self, id: &str) -> AppResult<bool> {
        let deleted: Option<ColorRow> = self.db.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}
