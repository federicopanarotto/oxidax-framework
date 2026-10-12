//! Custom authentication library of the framework (the passport.js layer).
//!
//! It owns everything that is *not* feature-specific: issuing and verifying
//! JWTs, password hashing, the [`UserIdentity`] entity and its repository, and
//! the request guard/extractor that `#[route]` wires through `#[IsGranted]`.
//!
//! The application features stay thin: they only expose the endpoints
//! (dto + service + controller) and delegate every credential operation here.
//! The identity repository is registered by the application composition (see
//! the application's `db` module), so this crate never enumerates the concrete
//! repositories of a project.

pub mod extractor;
pub mod guard;
pub mod identity_repository;
pub mod jwt;
pub mod password;
pub mod surreal_identity_repository;
pub mod user_identity;

use std::sync::Arc;

use di::ServiceCollection;
use di::singleton_factory;

use crate::config::Config;
use crate::framework::Feature;

pub use extractor::CurrentUser;
pub use identity_repository::UserIdentityRepository;
pub use jwt::{Claims, JwtConfig};
pub use surreal_identity_repository::SurrealUserIdentityRepository;
pub use user_identity::{AuthProvider, UserIdentity};

/// Registers the JWT settings.
///
/// The JWT values are resolved lazily from the shared [`Config`]: no feature
/// and no global static knows how the configuration is loaded, and any handler
/// protected by `#[IsGranted]` obtains them through the provider. The identity
/// repository is registered separately by the application composition.
fn register(services: &mut ServiceCollection) {
    services.add(singleton_factory::<JwtConfig, _>(
        |provider| -> di::Ref<JwtConfig> {
            let config = provider.get_required::<Config>();
            Arc::new(JwtConfig::from(config.as_ref()))
        },
    ));
}

// Zero-touch self-registration: the auth core participates in the feature
// registry exactly like the API features do.
inventory::submit! {
    Feature::new(0, register)
}
