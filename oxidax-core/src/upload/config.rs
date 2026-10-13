use std::path::PathBuf;

use crate::config::Config;

/// How the on-disk file name is generated for an uploaded file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileNameStrategy {
    /// A random UUID, keeping the original extension (default). The client
    /// file name is never used on disk, so there is no path traversal or
    /// collision risk; the original name is still kept in the metadata.
    Uuid,
    /// The sanitized original file name (directory components stripped,
    /// special characters replaced). Falls back to a UUID when empty.
    Original,
}

/// Configuration of the upload middleware: where files are stored and which
/// limits/filters apply. Mirrors the object passed to `multer(...)` in Node.js.
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// Directory where the files are written.
    pub dir: PathBuf,
    /// Maximum size, in bytes, of a single file.
    pub max_file_size: u64,
    /// Maximum number of files accepted in a single request.
    pub max_files: usize,
    /// Optional allow-list of file extensions (without the dot,
    /// case-insensitive). When empty, every extension is accepted.
    pub allowed_extensions: Vec<String>,
    /// How the on-disk file name is generated.
    pub file_name: FileNameStrategy,
}

impl From<&Config> for UploadConfig {
    fn from(config: &Config) -> Self {
        Self {
            dir: config.assets_dir.clone(),
            max_file_size: config.upload_max_file_size,
            max_files: config.upload_max_files,
            allowed_extensions: config.upload_allowed_extensions.clone(),
            file_name: FileNameStrategy::Uuid,
        }
    }
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("assets"),
            max_file_size: 10 * 1024 * 1024,
            max_files: 10,
            allowed_extensions: Vec::new(),
            file_name: FileNameStrategy::Uuid,
        }
    }
}
