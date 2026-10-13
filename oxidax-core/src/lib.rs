//! Reusable core library of the Oxidax framework.
//!
//! Everything that is identical across projects lives here: configuration,
//! the uniform error contract, authentication, the validation middleware,
//! pagination, the zero-touch DI/route registries, the database helpers and
//! the generic application assembly. A concrete application only owns its
//! `src/`: the feature modules and the thin composition that picks the
//! database backend and the OpenAPI metadata.

pub mod app;
pub mod auth;
pub mod config;
pub mod cors;
pub mod db;
pub mod docs;
pub mod errors;
pub mod framework;
pub mod middlewares;
pub mod pagination;
pub mod registry;
pub mod upload;
pub mod validators;

/// Re-export of the validation derive used by the DTOs
/// (`#[derive(Validate)]`), so features only depend on the framework.
pub use validator::Validate;
