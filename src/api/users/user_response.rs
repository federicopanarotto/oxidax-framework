use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::users::user_entity::User;
use crate::framework::Response;

/// Profile of the caller decorated with the email of its local identity: the
/// email is a join with another table, so this view is composed in the service
/// and is not a database projection.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
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
    /// Builds the response joining the profile summary with the email of its
    /// local identity (a user without identity has no `email`).
    pub fn new(user: UserSummaryResponse, email: Option<String>) -> Self {
        Self {
            id: user.id,
            firstname: user.firstname,
            lastname: user.lastname,
            role: user.role,
            email,
        }
    }
}

/// Profile of a user nested inside another response (`created_by`): the email
/// is deliberately absent — it is only exposed by `/api/users` (admin) and
/// `/api/users/me`. The derive teaches it how to project itself from the
/// `user` table, so a report resolves its creator in the same query.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Response)]
#[serde(rename_all = "camelCase")]
#[response(entity = User, table = "user")]
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
