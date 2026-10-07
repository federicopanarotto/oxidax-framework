use async_trait::async_trait;

use crate::api::categories::category_entity::Category;
use crate::errors::AppResult;

/// Persistence contract for categories.
///
/// It is the only point that depends on the database: implementing this
/// trait on a new backend (Postgres, SQLite, MongoDB...) leaves the service
/// and controllers unchanged.
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn insert(&self, category: Category) -> AppResult<Category>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Category>>;
    async fn find_all(&self) -> AppResult<Vec<Category>>;
    async fn replace(&self, id: &str, category: Category) -> AppResult<Option<Category>>;
    async fn remove(&self, id: &str) -> AppResult<bool>;
}
