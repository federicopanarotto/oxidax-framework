//! File upload middleware (the "multer" of the framework).
//!
//! It parses `multipart/form-data` requests and streams the files to a
//! configurable directory, leaving their metadata to the handler. The mapping
//! from the Node.js `multer` API is direct:
//!
//! | Node.js / multer                     | Oxidax                                             |
//! | ------------------------------------ | -------------------------------------------------- |
//! | `multer({ dest, limits, fileFilter })` | [`UploadConfig`] installed by `app::assemble`     |
//! | `upload.single('file')`              | [`Upload::single`]                                 |
//! | `upload.array('photos')`             | [`Upload::array`]                                  |
//! | `upload.any()`                       | [`Upload::files`]                                  |
//! | `req.file` / `req.files`             | [`UploadedFile`]                                   |
//! | `req.body`                           | [`Upload::field`]                                  |
//!
//! The core is storage-agnostic: it knows nothing about the database. The
//! application decides whether to keep just the files or to add a metadata
//! table (recommended: an `asset` row with id, owner, name, size and type).
//!
//! The [`Uploader`] is registered once in the DI container (so services can
//! resolve it) and installed on the router as an axum layer, together with a
//! [`DefaultBodyLimit`](axum::extract::DefaultBodyLimit) derived from the
//! configured maximum body size.

pub mod config;
pub mod extractor;
pub mod uploader;

use std::path::Path;

use axum::body::Body;
use axum::http::Response;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub use config::{FileNameStrategy, UploadConfig};
pub use extractor::Upload;
pub use uploader::{UploadedFile, Uploader};

use crate::errors::{AppError, AppResult};

/// Streams a stored file back to the client with the given content type and
/// original file name (`Content-Disposition: inline`).
pub async fn stream_file(
    path: impl AsRef<Path>,
    content_type: &str,
    original_name: &str,
) -> AppResult<Response<Body>> {
    let path = path.as_ref();
    let file = File::open(path).await.map_err(|err| {
        AppError::Internal(format!(
            "failed to open stored asset `{}`: {err}",
            path.display()
        ))
    })?;

    let mut response = Response::new(Body::from_stream(ReaderStream::new(file)));

    let content_type = content_type
        .parse()
        .unwrap_or_else(|_| axum::http::HeaderValue::from_static("application/octet-stream"));
    response.headers_mut().insert(CONTENT_TYPE, content_type);

    let disposition = format!(
        "inline; filename=\"{}\"",
        sanitize_header_filename(original_name)
    );
    if let Ok(value) = disposition.parse() {
        response.headers_mut().insert(CONTENT_DISPOSITION, value);
    }

    Ok(response)
}

/// Strips anything that could break or inject into the `Content-Disposition`
/// header (quotes, control characters, path separators).
fn sanitize_header_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| c.is_ascii_graphic() && !matches!(c, '"' | '\\' | '/'))
        .collect();

    if cleaned.is_empty() {
        "download".to_string()
    } else {
        cleaned
    }
}
