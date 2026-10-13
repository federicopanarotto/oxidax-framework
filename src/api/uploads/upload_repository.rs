use async_trait::async_trait;

use crate::api::uploads::upload_entity::Asset;
use crate::errors::AppResult;

/// Persistence contract for the uploaded assets' metadata.
///
/// The files themselves live on disk (managed by the upload middleware); this
/// is the only point that depends on the database.
#[async_trait]
pub trait AssetRepository: Send + Sync {
    async fn insert(&self, asset: Asset) -> AppResult<Asset>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Asset>>;
    /// One page of assets: `offset` skipped, at most `limit` returned.
    async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<Asset>>;
    async fn count(&self) -> AppResult<u64>;
    async fn remove(&self, id: &str) -> AppResult<bool>;
}
