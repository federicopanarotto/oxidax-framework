use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use di::injectable;
use uuid::Uuid;

use crate::api::uploads::upload_dto::AssetResponse;
use crate::api::uploads::upload_entity::Asset;
use crate::api::uploads::upload_repository::AssetRepository;
use crate::errors::{AppError, AppResult};
use crate::framework::{CurrentUser, Upload, Uploader};
use crate::lib::pagination::{Paged, Pagination};

/// Business rules for the uploaded assets.
///
/// The file is already on disk when the service runs (the [`Upload`] extractor
/// streamed it), so here we only persist the metadata row, serve the file back
/// and delete it. The DB is the source of truth: if the insert fails the stored
/// file is removed, so a failed upload leaves nothing behind.
#[injectable]
pub struct UploadService {
    repository: Arc<dyn AssetRepository>,
    uploader: Arc<Uploader>,
}

impl UploadService {
    /// Persists the file carried by the `file` field of the multipart request.
    pub async fn create(&self, form: Upload, owner_id: Option<String>) -> AppResult<AssetResponse> {
        let file = form.single("file").cloned().ok_or_else(|| {
            AppError::Validation("the multipart field `file` is required".to_string())
        })?;

        let asset = Asset {
            id: Uuid::new_v4().to_string(),
            description: form.field("description").map(str::to_string),
            stored_name: file.stored_name.clone(),
            original_name: file.original_name.clone(),
            content_type: file
                .content_type
                .clone()
                .unwrap_or_else(|| "application/octet-stream".to_string()),
            size: file.size,
            owner_id,
            created_at: now(),
        };

        match self.repository.insert(asset).await {
            Ok(asset) => Ok(asset.into()),
            Err(err) => {
                // Compensate: do not leave an orphan file on disk.
                let _ = self.uploader.remove(&file.stored_name).await;
                Err(err)
            }
        }
    }

    pub async fn get(&self, id: &str) -> AppResult<AssetResponse> {
        Ok(self.find(id).await?.into())
    }

    /// Asset metadata plus the path of its file on disk.
    pub async fn open(&self, id: &str) -> AppResult<(Asset, PathBuf)> {
        let asset = self.find(id).await?;
        let path = self.uploader.path_of(&asset.stored_name);
        Ok((asset, path))
    }

    pub async fn list_page(&self, pagination: Pagination) -> AppResult<Paged<AssetResponse>> {
        let total = self.repository.count().await?;
        let assets = self
            .repository
            .find_page(pagination.offset(), pagination.limit())
            .await?;

        let items = assets.into_iter().map(Into::into).collect();
        Ok(Paged::new(items, total))
    }

    /// Deletes the metadata row and the file. Only the owner (or an admin) can
    /// delete an asset.
    pub async fn delete(&self, id: &str, current: &CurrentUser) -> AppResult<()> {
        let asset = self.find(id).await?;

        let is_owner = asset.owner_id.as_deref() == Some(current.user_id.as_str());
        if !is_owner && current.role != "admin" {
            return Err(AppError::Forbidden(format!(
                "asset {id} does not belong to the current user"
            )));
        }

        if self.repository.remove(id).await? {
            self.uploader.remove(&asset.stored_name).await?;
            Ok(())
        } else {
            Err(AppError::not_found("asset", id))
        }
    }

    async fn find(&self, id: &str) -> AppResult<Asset> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("asset", id))
    }
}

/// Current time as a Unix timestamp (seconds).
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}
