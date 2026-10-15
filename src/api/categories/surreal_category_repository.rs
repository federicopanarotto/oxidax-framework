use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::SurrealValue;

use crate::api::categories::category_entity::Category;
use crate::api::categories::category_repository::CategoryRepository;
use crate::api::categories::category_response::CategoryResponse;
use crate::db::{define_table, query};
use crate::errors::AppResult;
use crate::framework::ViewRepository;

const TABLE: &str = "category";

/// Persistable content: the id is managed by the database, it must not be
/// written in the CONTENT.
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

    /// Creates the table if it does not exist: SurrealDB 3 does not create
    /// tables implicitly.
    pub async fn ensure_schema(&self) -> AppResult<()> {
        define_table(&self.db, TABLE).await
    }
}

/// Enables all the populated reads (`find`, `find_page`, `count`, `delete`) on
/// top of the shared connection.
#[async_trait]
impl ViewRepository for SurrealCategoryRepository {
    type Response = CategoryResponse;

    fn db(&self) -> &Surreal<Any> {
        &self.db
    }
}

#[async_trait]
impl CategoryRepository for SurrealCategoryRepository {
    async fn insert(&self, category: Category) -> AppResult<CategoryResponse> {
        let content = CategoryContent {
            name: category.name,
        };

        query::insert_view(&self.db, &category.id, content).await
    }

    async fn replace(&self, id: &str, category: Category) -> AppResult<Option<CategoryResponse>> {
        let content = CategoryContent {
            name: category.name,
        };

        query::replace_view(&self.db, id, content).await
    }
}
