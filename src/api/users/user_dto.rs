use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::ValidationError;

use crate::api::users::user_entity::{ROLE_ADMIN, ROLE_USER, User};
use crate::lib::Validate;

/// Query parameters of `GET /api/users`: all optional, an absent parameter
/// means "no filter".
#[derive(Debug, Clone, Deserialize, ToSchema, Validate)]
pub struct UserListQueryDto {
    /// Role filter: only `admin` or `user` are accepted
    #[schema(example = "admin")]
    #[validate(custom(
        function = "crate::api::users::user_dto::validate_role",
        message = "The role must be either 'admin' or 'user'"
    ))]
    pub role: Option<String>,
}

/// Used with the `#[validate(custom(...))]` decorator of `UserListQueryDto`:
/// accepts only the known roles. On `None` the `validator` crate skips the
/// check, so an absent parameter stays unfiltered.
fn validate_role(value: &str) -> Result<(), ValidationError> {
    if value == ROLE_ADMIN || value == ROLE_USER {
        Ok(())
    } else {
        Err(ValidationError::new("unknown_role"))
    }
}

/// Profile of a user as returned by the API: includes the email of the local
/// identity (the username of the login method) when one exists.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    /// UUID identifier of the user
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// First name
    #[schema(example = "Mario")]
    pub firstname: String,
    /// Last name
    #[schema(example = "Rossi")]
    pub lastname: String,
    /// Role: `user` or `admin`
    #[schema(example = "user")]
    pub role: String,
    /// Email of the local identity, if any
    #[schema(example = "mario.rossi@example.com")]
    pub email: Option<String>,
}

impl UserResponse {
    /// Builds the response joining the profile with the email of its local
    /// identity (a user without identity has no `email`).
    pub fn new(user: User, email: Option<String>) -> Self {
        Self {
            id: user.id,
            firstname: user.firstname,
            lastname: user.lastname,
            role: user.role,
            email,
        }
    }
}

/// Profile of a user nested inside another response (`created_by`): the
/// email is deliberately absent — it is only exposed by `/api/users`
/// (admin) and `/api/users/me`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserSummaryResponse {
    /// UUID identifier of the user
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// First name
    #[schema(example = "Mario")]
    pub firstname: String,
    /// Last name
    #[schema(example = "Rossi")]
    pub lastname: String,
    /// Role: `user` or `admin`
    #[schema(example = "user")]
    pub role: String,
}

impl UserSummaryResponse {
    /// Builds the nested profile from the entity: no email is ever involved.
    pub fn new(user: User) -> Self {
        Self {
            id: user.id,
            firstname: user.firstname,
            lastname: user.lastname,
            role: user.role,
        }
    }
}
