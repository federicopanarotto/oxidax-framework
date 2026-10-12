use async_trait::async_trait;

use crate::api::users::user_entity::User;
use crate::errors::AppResult;

/// Persistence contract for users (profiles).
///
/// It is the only point that depends on the database: implementing this trait
/// on a new backend (Postgres, SQLite, MongoDB...) leaves the service and
/// controllers unchanged.
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert(&self, user: User) -> AppResult<User>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<User>>;
    async fn find_all(&self) -> AppResult<Vec<User>>;
    /// Users with the exact role (the query filter of the list: the service
    /// only passes values already validated at the DTO level).
    async fn find_by_role(&self, role: &str) -> AppResult<Vec<User>>;
    /// One page of users: `offset` skipped, at most `limit` returned.
    async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<User>>;
    /// One page of users with the exact role.
    async fn find_page_by_role(&self, role: &str, offset: u64, limit: u64) -> AppResult<Vec<User>>;
    /// Total number of users, used to fill the pagination metadata.
    async fn count(&self) -> AppResult<u64>;
    /// Total number of users with the exact role.
    async fn count_by_role(&self, role: &str) -> AppResult<u64>;
    async fn remove(&self, id: &str) -> AppResult<bool>;
}
