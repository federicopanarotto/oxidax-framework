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
    async fn remove(&self, id: &str) -> AppResult<bool>;
}
