use serde::Deserialize;
use utoipa::ToSchema;
use validator::ValidationError;

use crate::api::users::user_entity::{ROLE_ADMIN, ROLE_USER};
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
fn validate_role(role: &str) -> Result<(), ValidationError> {
    if role == ROLE_ADMIN || role == ROLE_USER {
        Ok(())
    } else {
        Err(ValidationError::new("The role must be 'admin' or 'user'"))
    }
}
