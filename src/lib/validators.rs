//! Shared validators and error formatting for the `validator` crate.
//!
//! The functions here are reusable in every feature via the
//! `#[validate(custom(function = ...))]` decorator on DTO fields.

use uuid::Uuid;
use validator::ValidationError;

/// Formats `validator` errors into a list of human-readable strings in the
/// `field: message, message` format.
pub fn format_validation_errors(errors: validator::ValidationErrors) -> Vec<String> {
    errors
        .field_errors()
        .into_iter()
        .map(|(field, field_errors)| {
            let messages: Vec<String> = field_errors
                .iter()
                .map(|error| {
                    error
                        .message
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| error.code.to_string())
                })
                .collect();
            format!("{field}: {}", messages.join(", "))
        })
        .collect()
}

/// Used with `#[validate(custom(function = "crate::lib::validators::not_blank"))]`:
/// rejects empty strings or strings made only of whitespace.
///
/// The `validator` crate requires a custom validator to return
/// `Result<(), ValidationError>`: the message is overridden by the one
/// declared in the field's decorator.
pub fn not_blank(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::new("not_blank"))
    } else {
        Ok(())
    }
}

/// Used with `#[validate(custom(function = "crate::lib::validators::validate_uuid"))]`
/// on id fields (including `Option<String>`): checks that the value is a valid
/// UUID. On `None` the `validator` crate skips the check.
pub fn validate_uuid(value: &str) -> Result<(), ValidationError> {
    if Uuid::parse_str(value).is_err() {
        return Err(ValidationError::new("not_a_uuid"));
    }
    Ok(())
}
