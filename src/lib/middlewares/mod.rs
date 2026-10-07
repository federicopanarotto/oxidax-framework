//! Validation middleware: extractors run before the controller.
//!
//! In axum the idiomatic equivalent of Express's "middleware chain" for
//! validation is an *extractor*: `ValidatedJson`, `ValidatedQuery` and
//! `ValidatedParams` extract the DTO, validate it with the
//! `#[validate(...)]` decorators and, if anything fails, respond with the
//! uniform error body before the controller method is called.

pub mod validation_middleware;
