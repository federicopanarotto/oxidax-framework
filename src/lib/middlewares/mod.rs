//! Middleware di validazione: estrattori eseguiti prima del controller.
//!
//! In axum l'equivalente idiomatico della "middleware chain" di Express per la
//! validazione è un *extractor*: `ValidatedJson`, `ValidatedQuery` e
//! `ValidatedParams` estraggono il DTO, lo validano con i decoratori
//! `#[validate(...)]` e, se qualcosa non passa, rispondono con il corpo
//! uniforme di errore prima che il metodo del controller venga chiamato.

pub mod validation_middleware;
