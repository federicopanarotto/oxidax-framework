use async_trait::async_trait;

use crate::api::color::color_entity::Color;
use crate::errors::AppResult;

/// Persistence contract for colors.
///
/// It is the only point that depends on the database: implementing this
/// trait on a new backend leaves the service and controllers unchanged.
#[async_trait]
pub trait ColorRepository: Send + Sync {
    async fn insert(&self, item: Color) -> AppResult<Color>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Color>>;
    async fn find_all(&self) -> AppResult<Vec<Color>>;
    /// One page: `offset` skipped, at most `limit` returned.
    async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<Color>>;
    /// Total number of rows, used to fill the pagination metadata.
    async fn count(&self) -> AppResult<u64>;
    async fn replace(&self, id: &str, item: Color) -> AppResult<Option<Color>>;
    async fn remove(&self, id: &str) -> AppResult<bool>;
}
