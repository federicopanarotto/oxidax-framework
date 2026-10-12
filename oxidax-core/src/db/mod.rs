//! Database helpers shared across projects.
//!
//! The connection and the low-level SurrealDB primitives live here, so an
//! application only declares its concrete repositories and wires them in its
//! own `db` module. SurrealDB is the default backend, but the repository traits
//! are backend-agnostic: switching database means implementing the traits and
//! registering a different repository in the application composition.

pub mod query;
mod surreal;

pub use surreal::{connect, define_table, record_key_to_string};
