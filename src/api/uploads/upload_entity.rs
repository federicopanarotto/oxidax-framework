use serde::{Deserialize, Serialize};

/// Database-independent metadata of an uploaded asset.
///
/// The file content lives on disk (`stored_name` inside the assets directory);
/// the row keeps everything needed to serve it back, authorize its access and
/// delete it. The public id is decoupled from the storage, so the files can be
/// moved (e.g. to object storage) without changing the exposed URLs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub description: Option<String>,
    /// Name of the file inside the assets directory.
    pub stored_name: String,
    /// Original client file name.
    pub original_name: String,
    pub content_type: String,
    pub size: u64,
    /// Uploader id, when the request was authenticated.
    pub owner_id: Option<String>,
    /// Creation time as a Unix timestamp (seconds).
    pub created_at: u64,
}
