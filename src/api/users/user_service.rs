use std::collections::HashMap;
use std::sync::Arc;

use di::injectable;

use crate::api::users::user_dto::{UserResponse, UserSummaryResponse};
use crate::api::users::user_repository::UserRepository;
use crate::errors::{AppError, AppResult};
use crate::lib::auth::UserIdentityRepository;
use crate::lib::pagination::{Paged, Pagination};

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

    /// Profile of a user without the email (email is only ever exposed to
    /// the owner and to admins): used by other services to nest a user in
    /// their responses without failing if the user is gone.
    pub async fn find(&self, id: &str) -> AppResult<Option<UserSummaryResponse>> {
        Ok(self
            .users
            .find_by_id(id)
            .await?
            .map(UserSummaryResponse::new))
    }

    /// `id → user` index: decorates a list of entities with a single read,
    /// avoiding N+1 queries per element.
    pub async fn by_id(&self) -> AppResult<HashMap<String, UserSummaryResponse>> {
        let mut index = HashMap::new();
        for user in self.users.find_all().await? {
            let response = UserSummaryResponse::new(user);
            index.insert(response.id.clone(), response);
        }
        Ok(index)
    }

    /// All users with their local email, resolved through a single read of
    /// the identities (index instead of N+1 queries). With `role` present
    /// only the users with that role are returned (the value was already
    /// validated by the `ValidatedQuery` extractor).
    pub async fn list(&self, role: Option<&str>) -> AppResult<Vec<UserResponse>> {
        let emails: HashMap<String, String> = self
            .identities
            .find_all()
            .await?
            .into_iter()
            .map(|identity| (identity.user_id, identity.identifier))
            .collect();

        let users = match role {
            Some(role) => self.users.find_by_role(role).await?,
            None => self.users.find_all().await?,
        };

        Ok(users
            .into_iter()
            .map(|user| {
                let email = emails.get(&user.id).cloned();
                UserResponse::new(user, email)
            })
            .collect())
    }

    /// One page of users (optionally filtered by role) with the total count,
    /// resolved with a single read of the identities (index instead of N+1
    /// queries).
    pub async fn list_page(
        &self,
        role: Option<&str>,
        pagination: Pagination,
    ) -> AppResult<Paged<UserResponse>> {
        let emails: HashMap<String, String> = self
            .identities
            .find_all()
            .await?
            .into_iter()
            .map(|identity| (identity.user_id, identity.identifier))
            .collect();

        let offset = pagination.offset();
        let limit = pagination.limit();
        let (users, total) = match role {
            Some(role) => (
                self.users.find_page_by_role(role, offset, limit).await?,
                self.users.count_by_role(role).await?,
            ),
            None => (
                self.users.find_page(offset, limit).await?,
                self.users.count().await?,
            ),
        };

        let items = users
            .into_iter()
            .map(|user| {
                let email = emails.get(&user.id).cloned();
                UserResponse::new(user, email)
            })
            .collect();

        Ok(Paged::new(items, total))
    }
}

fn not_found(id: &str) -> AppError {
    AppError::NotFound(format!("user {id} not found"))
}
