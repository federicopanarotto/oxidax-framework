use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::lib::Validate;

/// Registration payload: email as username, password and profile name.
///
/// Field validation runs in the extractor (`ValidatedJson`); the answers of
/// the endpoint are deliberately generic to avoid account enumeration.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct RegisterRequest {
    /// Email used as username for the local login
    #[schema(example = "developer@example.com", max_length = 254)]
    #[validate(
        email(message = "The email must be a valid address"),
        length(max = 254, message = "The email must be at most 254 characters")
    )]
    pub email: String,
    /// Password, at least 8 characters
    #[schema(example = "Str0ngPassword!", min_length = 8, max_length = 128)]
    #[validate(length(
        min = 8,
        max = 128,
        message = "The password must be between 8 and 128 characters"
    ))]
    pub password: String,
    /// First name
    #[schema(example = "Mario", min_length = 1, max_length = 100)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The first name is required and cannot be empty"
        ),
        length(max = 100, message = "The first name must be at most 100 characters")
    )]
    pub firstname: String,
    /// Last name
    #[schema(example = "Rossi", min_length = 1, max_length = 100)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The last name is required and cannot be empty"
        ),
        length(max = 100, message = "The last name must be at most 100 characters")
    )]
    pub lastname: String,
}

/// Credentials for the local login: email + password.
///
/// A wrong email and a wrong password produce the same `401`, so the response
/// never reveals which of the two was wrong (or whether the account exists).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    /// Email used as username
    #[schema(example = "developer@example.com", max_length = 254)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The email is required and cannot be empty"
        ),
        length(max = 254, message = "The email must be at most 254 characters")
    )]
    pub email: String,
    /// Password
    #[schema(example = "Str0ngPassword!", max_length = 128)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The password is required and cannot be empty"
        ),
        length(max = 128, message = "The password must be at most 128 characters")
    )]
    pub password: String,
}

/// Refresh payload: the refresh token issued at login or at a previous
/// refresh (single-use: it is rotated on every call).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct RefreshRequest {
    /// Refresh token previously returned by `login`/`refresh`
    #[schema(
        example = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAiLCJ0b2tlbl91c2UiOiJyZWZyZXNoIn0.signature",
        max_length = 4096
    )]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The refresh token is required and cannot be empty"
        ),
        length(
            max = 4096,
            message = "The refresh token must be at most 4096 characters"
        )
    )]
    pub refresh_token: String,
}

/// Token pair returned by `login` and `refresh`.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    /// Short-lived access token, sent in `Authorization: Bearer` on every
    /// protected request
    #[schema(
        example = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAiLCJ0b2tlbl91c2UiOiJhY2Nlc3MifQ.signature"
    )]
    pub access_token: String,
    /// Long-lived, single-use refresh token: sent back to `/api/auth/refresh`
    /// to obtain a new pair (the previous one stops working)
    #[schema(
        example = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI1NTBlODQwMC1lMjliLTQxZDQtYTcxNi00NDY2NTU0NDAwMDAiLCJ0b2tlbl91c2UiOiJyZWZyZXNoIn0.signature"
    )]
    pub refresh_token: String,
    /// Type of the access token, always `Bearer`
    #[schema(example = "Bearer")]
    pub token_type: String,
    /// Lifetime of the access token in seconds
    #[schema(example = 900)]
    pub expires_in: u64,
}

/// Generic confirmation message.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponse {
    /// Human readable outcome of the operation
    #[schema(example = "user registered successfully")]
    pub message: String,
}

impl MessageResponse {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}
