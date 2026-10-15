use async_trait::async_trait;

use crate::api::users::user_entity::User;
use crate::api::users::user_response::UserSummaryResponse;
use crate::errors::AppResult;
use crate::framework::ViewRepository;

/// Persistence contract for users (profiles).
///
/// It inherits the populated reads (`find`, `find_page`, `find_page_by`,
/// `count`, `count_by`, `delete`) from [`ViewRepository`]: the read-model is the
/// [`UserSummaryResponse`], so the service no longer maps the entity by hand.
/// The role filter uses the generic `*_by` reads with the `role` field.
#[async_trait]
pub trait UserRepository: ViewRepository<Response = UserSummaryResponse> {
    /// Persists a new user and returns its summary.
    async fn insert(&self, user: User) -> AppResult<UserSummaryResponse>;
}
