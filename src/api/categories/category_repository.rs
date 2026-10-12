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
    /// One page of categories: `offset` skipped, at most `limit` returned.
    async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<Category>>;
    /// Total number of categories, used to fill the pagination metadata.
    async fn count(&self) -> AppResult<u64>;
    async fn replace(&self, id: &str, category: Category) -> AppResult<Option<Category>>;
    async fn remove(&self, id: &str) -> AppResult<bool>;
}
