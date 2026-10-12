//! Framework facade.
//!
//! Single import surface for feature authors (and for external users of the
//! framework). It re-exports the registries, the route macro, the validation
//! extractors and the error contract, so a feature can import everything from
//! one place:
//!
//! ```
//! use axum_api::framework::*;
//! ```

pub use crate::errors::{AppError, AppResult, BadRequestResponse, ErrorResponse};
pub use crate::lib::auth::CurrentUser;
pub use crate::lib::middlewares::validation_middleware::{
    ValidatedJson, ValidatedParams, ValidatedQuery,
};
pub use crate::lib::pagination::{Paged, Pagination, PaginationQuery, TOTAL_HEADER};
pub use crate::lib::registry::{Feature, Route, register_repository};
pub use framework_macros::route;
