use async_trait::async_trait;

use crate::auth::user_identity::UserIdentity;
use crate::errors::AppResult;

/// Persistence contract for identities (login methods).
///
/// It is the only point that depends on the database: implementing this trait
/// on a new backend leaves the auth service and controllers unchanged.
#[async_trait]
pub trait UserIdentityRepository: Send + Sync {
    async fn insert(&self, identity: UserIdentity) -> AppResult<UserIdentity>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<UserIdentity>>;
    /// Looks up an identity by username (email for the local provider).
    async fn find_by_identifier(&self, identifier: &str) -> AppResult<Option<UserIdentity>>;
    /// Looks up the identity a user logs in with: used to read the email of
    /// the profile endpoint.
    async fn find_by_user_id(&self, user_id: &str) -> AppResult<Option<UserIdentity>>;
    async fn find_all(&self) -> AppResult<Vec<UserIdentity>>;
    /// Persists the rotating session (`jti` + expiration) of the refresh
    /// token currently in force.
    async fn set_refresh(&self, id: &str, jti: &str, expires_at: usize) -> AppResult<bool>;
    /// Drops the session: the outstanding refresh token becomes unusable.
    async fn clear_refresh(&self, id: &str) -> AppResult<bool>;
}
