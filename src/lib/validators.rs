//! Validatori condivisi e formattazione degli errori del crate `validator`.
//!
//! Le funzioni qui sono riutilizzabili in ogni feature via decoratore
//! `#[validate(custom(function = ...))]` sui campi dei DTO.

use uuid::Uuid;
use validator::ValidationError;

/// Formatta gli errori di `validator` in una lista di stringhe leggibili nel
/// formato `campo: messaggio, messaggio`.
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

/// Usato con `#[validate(custom(function = "crate::lib::validators::not_blank"))]`:
/// rifiuta stringhe vuote o composte solo da spazi.
///
/// Il crate `validator` vuole che un custom validator restituisca
/// `Result<(), ValidationError>`: il messaggio viene sovrascritto da quello
/// dichiarato nel decoratore del campo.
pub fn not_blank(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::new("not_blank"))
    } else {
        Ok(())
    }
}

/// Usato con `#[validate(custom(function = "crate::lib::validators::validate_uuid"))]`
/// sui campi id (anche `Option<String>`): controlla che il valore sia un UUID
/// valido. Su `None` il crate `validator` non esegue il controllo.
pub fn validate_uuid(value: &str) -> Result<(), ValidationError> {
    if Uuid::parse_str(value).is_err() {
        return Err(ValidationError::new("not_a_uuid"));
    }
    Ok(())
}
