use std::collections::HashMap;
use std::sync::Arc;

use di::injectable;

use crate::api::users::user_dto::UserResponse;
use crate::api::users::user_repository::UserRepository;
use crate::errors::{AppError, AppResult};
use crate::lib::auth::UserIdentityRepository;

/// Business rules for users (profiles and roles).
///
/// The endpoints only expose two views: the profile of the caller
/// (`/api/users/me`, authenticated) and the full list (`/api/users`,
/// admin only).
#[injectable]
pub struct UserService {
    users: Arc<dyn UserRepository>,
    identities: Arc<dyn UserIdentityRepository>,
}

impl UserService {
    /// Profile of the authenticated caller, decorated with the email of its
    /// local identity.
    pub async fn me(&self, user_id: &str) -> AppResult<UserResponse> {
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| not_found(user_id))?;

        let email = self
            .identities
            .find_by_user_id(user_id)
            .await?
            .map(|identity| identity.identifier);

        Ok(UserResponse::new(user, email))
    }

    /// All users with their local email, resolved through a single read of
    /// the identities (index instead of N+1 queries).
    pub async fn list(&self) -> AppResult<Vec<UserResponse>> {
        let emails: HashMap<String, String> = self
            .identities
            .find_all()
            .await?
            .into_iter()
            .map(|identity| (identity.user_id, identity.identifier))
            .collect();

        Ok(self
            .users
            .find_all()
            .await?
            .into_iter()
            .map(|user| {
                let email = emails.get(&user.id).cloned();
                UserResponse::new(user, email)
            })
            .collect())
    }
}

fn not_found(id: &str) -> AppError {
    AppError::NotFound(format!("user {id} not found"))
}
