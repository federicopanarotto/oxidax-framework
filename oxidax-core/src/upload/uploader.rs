use std::collections::HashMap;
use std::path::{Path, PathBuf};

use axum::extract::Multipart;
use axum::extract::multipart::Field;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

use super::config::{FileNameStrategy, UploadConfig};

/// Metadata of a file already streamed to disk.
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// Name of the multipart field that carried the file.
    pub field: String,
    /// Original client file name (never used as-is on disk).
    pub original_name: String,
    /// Content type declared by the client, if any.
    pub content_type: Option<String>,
    /// Size of the stored file, in bytes.
    pub size: u64,
    /// Name of the file inside the storage directory.
    pub stored_name: String,
    /// Full path of the stored file.
    pub path: PathBuf,
}

impl UploadedFile {
    /// Lowercase extension of the original name, without the dot.
    pub fn extension(&self) -> Option<String> {
        extension_of(&self.original_name)
    }
}

/// The multer instance: holds the storage configuration and persists the
/// multipart fields. Installed once at startup (as an axum layer) and shared
/// across requests, so it is cheap to clone through an `Arc`.
pub struct Uploader {
    config: UploadConfig,
}

impl Uploader {
    pub fn new(config: UploadConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &UploadConfig {
        &self.config
    }

    /// Directory where the files are stored.
    pub fn dir(&self) -> &Path {
        &self.config.dir
    }

    /// Path of a stored file, given its on-disk name.
    pub fn path_of(&self, stored_name: &str) -> PathBuf {
        self.config.dir.join(stored_name)
    }

    /// Best-effort, idempotent removal of a stored file.
    pub async fn remove(&self, stored_name: &str) -> AppResult<()> {
        match tokio::fs::remove_file(self.path_of(stored_name)).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(AppError::Internal(format!(
                "failed to delete asset `{stored_name}`: {err}"
            ))),
        }
    }

    /// Consumes the whole multipart body: text fields are collected, files are
    /// streamed to disk. Enforces the configured limits and filters.
    pub(crate) async fn consume(
        &self,
        multipart: &mut Multipart,
    ) -> AppResult<(HashMap<String, String>, Vec<UploadedFile>)> {
        let mut fields = HashMap::new();
        let mut files = Vec::new();

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|err| AppError::Validation(format!("invalid multipart body: {err}")))?
        {
            if field.file_name().is_some() {
                if files.len() >= self.config.max_files {
                    return Err(AppError::PayloadTooLarge(format!(
                        "too many files: at most {} are allowed",
                        self.config.max_files
                    )));
                }
                files.push(self.store(field).await?);
            } else {
                let name = field.name().unwrap_or_default().to_string();
                let value = field.text().await.map_err(|err| {
                    AppError::Validation(format!("invalid multipart field: {err}"))
                })?;
                fields.insert(name, value);
            }
        }

        Ok((fields, files))
    }

    /// Streams a single file field to disk, returning its metadata.
    async fn store(&self, mut field: Field<'_>) -> AppResult<UploadedFile> {
        let field_name = field.name().unwrap_or_default().to_string();
        let original_name = field.file_name().unwrap_or_default().to_string();
        let content_type = field.content_type().map(str::to_string);

        self.check_extension(&original_name)?;

        let stored_name = self.stored_name(&original_name);
        let path = self.path_of(&stored_name);

        let mut file = tokio::fs::File::create(&path).await.map_err(|err| {
            AppError::Internal(format!("failed to create `{}`: {err}", path.display()))
        })?;

        let mut size: u64 = 0;
        loop {
            match field.chunk().await {
                Ok(Some(chunk)) => {
                    size += chunk.len() as u64;
                    if size > self.config.max_file_size {
                        drop(file);
                        let _ = tokio::fs::remove_file(&path).await;
                        return Err(AppError::PayloadTooLarge(format!(
                            "file `{original_name}` exceeds the maximum size of {} bytes",
                            self.config.max_file_size
                        )));
                    }
                    if let Err(err) = file.write_all(&chunk).await {
                        drop(file);
                        let _ = tokio::fs::remove_file(&path).await;
                        return Err(AppError::Internal(format!(
                            "failed to write asset `{stored_name}`: {err}"
                        )));
                    }
                }
                Ok(None) => break,
                Err(err) => {
                    drop(file);
                    let _ = tokio::fs::remove_file(&path).await;
                    return Err(AppError::Validation(format!(
                        "invalid multipart field: {err}"
                    )));
                }
            }
        }

        file.flush().await.map_err(|err| {
            AppError::Internal(format!("failed to flush asset `{stored_name}`: {err}"))
        })?;
        drop(file);

        Ok(UploadedFile {
            field: field_name,
            original_name,
            content_type,
            size,
            stored_name,
            path,
        })
    }

    /// Applies the optional extension allow-list (the `fileFilter` of multer).
    fn check_extension(&self, original_name: &str) -> AppResult<()> {
        if self.config.allowed_extensions.is_empty() {
            return Ok(());
        }

        let allowed = extension_of(original_name).is_some_and(|ext| {
            self.config
                .allowed_extensions
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(&ext))
        });

        if allowed {
            Ok(())
        } else {
            Err(AppError::UnsupportedMediaType(format!(
                "file `{original_name}` has a disallowed extension"
            )))
        }
    }

    fn stored_name(&self, original_name: &str) -> String {
        match self.config.file_name {
            FileNameStrategy::Uuid => match extension_of(original_name) {
                Some(ext) => format!("{}.{ext}", Uuid::new_v4()),
                None => Uuid::new_v4().to_string(),
            },
            FileNameStrategy::Original => {
                let sanitized = sanitize(original_name);
                if sanitized.is_empty() {
                    Uuid::new_v4().to_string()
                } else {
                    sanitized
                }
            }
        }
    }
}

/// Lowercase extension of a file name, without the dot.
fn extension_of(name: &str) -> Option<String> {
    Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .filter(|ext| !ext.is_empty())
        .map(str::to_ascii_lowercase)
}

/// Reduces a client-supplied name to a safe base name: it strips the directory
/// components and keeps only alphanumerics, dots, dashes and underscores.
fn sanitize(name: &str) -> String {
    let base = Path::new(name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    base.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('.')
        .to_string()
}
