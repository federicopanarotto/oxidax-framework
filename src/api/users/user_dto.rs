use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::users::user_entity::User;

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
