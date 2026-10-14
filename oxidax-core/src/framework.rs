//! Framework facade.
//!
//! Single import surface for feature authors (and for external users of the
//! framework). It re-exports the registries, the route macro, the validation
//! extractors and the error contract, so a feature can import everything from
//! one place:
//!
//! ```ignore
//! use oxidax_core::framework::*;
//! ```

pub use crate::auth::CurrentUser;
pub use crate::errors::{AppError, AppResult, ErrorResponse};
pub use crate::middlewares::validation_middleware::{
    ValidatedJson, ValidatedParams, ValidatedQuery,
};
pub use crate::pagination::{Paged, Pagination, PaginationQuery, TOTAL_HEADER};
pub use crate::registry::{Feature, Route, register_repository};
pub use crate::upload::{Upload, UploadConfig, UploadedFile, Uploader};
pub use oxidax_macro::route;
