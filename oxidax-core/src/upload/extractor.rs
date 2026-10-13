use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{FromRequest, Multipart, Request};

use crate::errors::AppError;

use super::uploader::{UploadedFile, Uploader};

/// The parsed `multipart/form-data` request: text `fields` plus the `files`
/// already streamed to disk.
///
/// It mirrors what multer leaves on the Node.js request after the middleware
/// has run: the handler then reads `req.file` / `req.files` / `req.body`. The
/// extractor consumes the body, so it must be the **last** argument of the
/// handler.
pub struct Upload {
    fields: HashMap<String, String>,
    files: Vec<UploadedFile>,
}

impl Upload {
    /// Text value of a non-file field.
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }

    /// The one file carried by `name` (the `upload.single(name)` of multer).
    pub fn single(&self, name: &str) -> Option<&UploadedFile> {
        self.files.iter().find(|file| file.field == name)
    }

    /// Every file carried by `name` (the `upload.array(name)` of multer).
    pub fn array(&self, name: &str) -> Vec<&UploadedFile> {
        self.files
            .iter()
            .filter(|file| file.field == name)
            .collect()
    }

    /// Every file in the request (the `upload.any()` of multer).
    pub fn files(&self) -> &[UploadedFile] {
        &self.files
    }

    /// Every text field.
    pub fn fields(&self) -> &HashMap<String, String> {
        &self.fields
    }
}

impl<S> FromRequest<S> for Upload
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let uploader = req
            .extensions()
            .get::<Arc<Uploader>>()
            .cloned()
            .ok_or_else(|| AppError::Internal("upload middleware is not installed".into()))?;

        let mut multipart = Multipart::from_request(req, state).await.map_err(|err| {
            AppError::UnsupportedMediaType(format!("multipart/form-data body expected: {err}"))
        })?;

        let (fields, files) = uploader.consume(&mut multipart).await?;
        Ok(Self { fields, files })
    }
}
