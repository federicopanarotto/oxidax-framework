//! Shared API validation library.
//!
//! Exposes the [`ValidatedJson`], [`ValidatedQuery`] and
//! [`ValidatedParams`] extractors (the validation "middleware" that runs
//! *before* the controller logic) and the shared validators to be reused in
//! every feature.

pub mod middlewares;
pub mod registry;
pub mod validators;

pub use middlewares::validation_middleware::{
    ValidatedJson, ValidatedParams, ValidatedQuery, ValidationErrorResponse,
};
pub use validator::Validate;
pub use validators::{format_validation_errors, not_blank};
