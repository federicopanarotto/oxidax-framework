//! Libreria condivisa di validazione dell'API.
//!
//! Espone gli estrattori [`ValidatedJson`], [`ValidatedQuery`] e
//! [`ValidatedParams`] (i "middleware" di validazione che girano *prima* della
//! logica del controller) e i validatori condivisi da riusare in ogni feature.

pub mod middlewares;
pub mod validators;

pub use middlewares::validation_middleware::{
    ValidatedJson, ValidatedParams, ValidatedQuery, ValidationErrorResponse,
};
pub use validator::Validate;
pub use validators::{format_validation_errors, not_blank};
