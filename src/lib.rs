//! Oxidax application: the concrete features and the thin composition on top
//! of the reusable `oxidax-core` library.
//!
//! Only the application code lives in `src/`: the feature modules (`api`), the
//! concrete repositories (`db`) and the assembly entry point (`app`). The
//! reusable parts (config, errors, auth, middleware, pagination, registry and
//! generic app assembly) come from `oxidax-core` and are re-exported here so
//! the features keep their natural `crate::...` import paths.

pub mod api;
pub mod app;
pub mod db;

/// The reusable core library, exposed under the historical `lib` path.
pub use oxidax_core as lib;
pub use oxidax_core::{config, errors, framework};
