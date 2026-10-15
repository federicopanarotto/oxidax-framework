use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::uploads::upload_entity::Asset;

/// Metadata returned to the client after an upload.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AssetResponse {
    /// UUID identifier of the asset
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// Optional free-text description provided with the upload
    #[schema(example = "Monthly report")]
    pub description: Option<String>,
    /// Original file name
    #[schema(example = "report.pdf")]
    pub original_name: String,
    /// Content type of the file
    #[schema(example = "application/pdf")]
    pub content_type: String,
    /// Size in bytes
    #[schema(example = 1024)]
    pub size: u64,
    /// Id of the uploader, if authenticated
    pub owner_id: Option<String>,
    /// Creation time as a Unix timestamp (seconds)
    #[schema(example = 1_700_000_000u64)]
    pub created_at: u64,
    /// URL the file can be downloaded from
    #[schema(example = "/api/upload/550e8400-e29b-41d4-a716-446655440000")]
    pub url: String,
}

impl From<Asset> for AssetResponse {
    fn from(asset: Asset) -> Self {
        Self {
            url: format!("/api/upload/{}", asset.id),
            id: asset.id,
            description: asset.description,
            original_name: asset.original_name,
            content_type: asset.content_type,
            size: asset.size,
            owner_id: asset.owner_id,
            created_at: asset.created_at,
        }
    }
}

/// Documentation-only schema of the `multipart/form-data` body accepted by
/// `POST /api/upload` (the runtime parsing is done by the `Upload` extractor).
#[derive(Debug, ToSchema)]
pub struct UploadForm {
    /// The file to upload
    #[schema(value_type = String, format = Binary)]
    pub file: String,
    /// Optional free-text description carried by the form
    #[schema(value_type = Option<String>, example = "Monthly report")]
    pub description: Option<String>,
}
