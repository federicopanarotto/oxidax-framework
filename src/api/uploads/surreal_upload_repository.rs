use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::{RecordId, SurrealValue};

use crate::api::uploads::upload_entity::Asset;
use crate::api::uploads::upload_repository::AssetRepository;
use crate::db::{define_table, query, record_key_to_string};
use crate::errors::{AppError, AppResult};

const TABLE: &str = "asset";

/// Row as it arrives from the database (the id is a SurrealDB record id).
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct AssetRow {
    id: RecordId,
    stored_name: String,
    original_name: String,
    content_type: String,
    size: i64,
    owner_id: Option<String>,
    created_at: i64,
}

/// Persistable content: the id is managed by the database, it must not be
/// written in the CONTENT.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct AssetContent {
    stored_name: String,
    original_name: String,
    content_type: String,
    size: i64,
    owner_id: Option<String>,
    created_at: i64,
}

pub struct SurrealAssetRepository {
    db: Surreal<Any>,
}

impl SurrealAssetRepository {
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
fn into_entity(row: AssetRow) -> Asset {
    Asset {
        id: record_key_to_string(row.id.key),
        stored_name: row.stored_name,
        original_name: row.original_name,
        content_type: row.content_type,
        size: row.size.max(0) as u64,
        owner_id: row.owner_id,
        created_at: row.created_at.max(0) as u64,
    }
}

fn content(asset: &Asset) -> AssetContent {
    AssetContent {
        stored_name: asset.stored_name.clone(),
        original_name: asset.original_name.clone(),
        content_type: asset.content_type.clone(),
        size: asset.size as i64,
        owner_id: asset.owner_id.clone(),
        created_at: asset.created_at as i64,
    }
}

#[async_trait]
impl AssetRepository for SurrealAssetRepository {
    async fn insert(&self, asset: Asset) -> AppResult<Asset> {
        let created: Option<AssetRow> = self
            .db
            .create((TABLE, asset.id.as_str()))
            .content(content(&asset))
            .await?;

        if created.is_some() {
            Ok(asset)
        } else {
            Err(AppError::Internal(format!(
                "failed to create asset {}",
                asset.id
            )))
        }
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<Asset>> {
        let found: Option<AssetRow> = self.db.select((TABLE, id)).await?;
        Ok(found.map(into_entity))
    }

    async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<Asset>> {
        let rows: Vec<AssetRow> = query::select_page(&self.db, TABLE, offset, limit).await?;
        Ok(rows.into_iter().map(into_entity).collect())
    }

    async fn count(&self) -> AppResult<u64> {
        query::count(&self.db, TABLE).await
    }

    async fn remove(&self, id: &str) -> AppResult<bool> {
        let removed: Option<AssetRow> = self.db.delete((TABLE, id)).await?;
        Ok(removed.is_some())
    }
}
