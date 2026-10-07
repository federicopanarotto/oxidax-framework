use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::{RecordId, SurrealValue};

use crate::api::categories::category_entity::Category;
use crate::api::categories::category_repository::CategoryRepository;
use crate::db::{define_table, record_key_to_string};
use crate::errors::{AppError, AppResult};

const TABLE: &str = "category";

/// Riga così come arriva dal database (l'id è un record id SurrealDB).
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct CategoryRow {
    id: RecordId,
    name: String,
}

/// Contenuto persistibile: l'id è gestito dal database, non va scritto nel CONTENT.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct CategoryContent {
    name: String,
}

pub struct SurrealCategoryRepository {
    db: Surreal<Any>,
}

impl SurrealCategoryRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Crea la tabella se non esiste: SurrealDB 3 non crea tabelle implicitamente.
    pub async fn ensure_schema(&self) -> AppResult<()> {
        define_table(&self.db, TABLE).await
    }
}

/// L'API lavora con la sola chiave del record id (`uuid`), non con `table:uuid`.
fn into_entity(row: CategoryRow) -> Category {
    Category {
        id: record_key_to_string(row.id.key),
        name: row.name,
    }
}

fn created_or_internal(category: Category, created: Option<CategoryRow>) -> AppResult<Category> {
    if created.is_some() {
        Ok(category)
    } else {
        Err(AppError::Internal(format!(
            "failed to create category {}",
            category.id
        )))
    }
}

#[async_trait]
impl CategoryRepository for SurrealCategoryRepository {
    async fn insert(&self, category: Category) -> AppResult<Category> {
        let content = CategoryContent {
            name: category.name.clone(),
        };

        let created: Option<CategoryRow> = self
            .db
            .create((TABLE, category.id.as_str()))
            .content(content)
            .await?;

        created_or_internal(category, created)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<Category>> {
        let found: Option<CategoryRow> = self.db.select((TABLE, id)).await?;
        Ok(found.map(into_entity))
    }

    async fn find_all(&self) -> AppResult<Vec<Category>> {
        let rows: Vec<CategoryRow> = self.db.select(TABLE).await?;
        Ok(rows.into_iter().map(into_entity).collect())
    }

    async fn replace(&self, id: &str, category: Category) -> AppResult<Option<Category>> {
        let content = CategoryContent {
            name: category.name.clone(),
        };

        let updated: Option<CategoryRow> = self.db.update((TABLE, id)).content(content).await?;
        Ok(updated.map(into_entity))
    }

    async fn remove(&self, id: &str) -> AppResult<bool> {
        let deleted: Option<CategoryRow> = self.db.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}
