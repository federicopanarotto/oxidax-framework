//! JSON Web Tokens: issuing and verification of access and refresh tokens.
//!
//! The library signs with HS256 (shared secret from the configuration) and
//! keeps the algorithm pinned on verification, so a token forged with another
//! algorithm is rejected. Two token kinds exist, distinguished by the
//! `token_use` claim: an access token is never accepted where a refresh token
//! is expected and vice versa.

use jsonwebtoken::errors::Error as JsonWebTokenError;
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::Config;

/// Value of the `token_use` claim for the short-lived access token.
pub const ACCESS: &str = "access";
/// Value of the `token_use` claim for the refresh token.
pub const REFRESH: &str = "refresh";

/// Uniform internal error: expired tokens, wrong signatures and wrong token
/// kinds are deliberately collapsed into one variant so that no detail leaks
/// to the caller (the API always answers with a generic 401).
#[derive(Debug, thiserror::Error)]
#[error("invalid token")]
pub struct JwtError;

impl From<JsonWebTokenError> for JwtError {
    fn from(_: JsonWebTokenError) -> Self {
        Self
    }
}

/// JWT signing/verification settings, derived from the application
/// [`Config`] and registered in the DI container by this module.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    secret: String,
    access_duration: u64,
    refresh_duration: u64,
}

impl From<&Config> for JwtConfig {
    fn from(config: &Config) -> Self {
        Self {
            secret: config.jwt_secret.clone(),
            access_duration: config.access_token_duration,
            refresh_duration: config.refresh_token_duration,
        }
    }
}

/// Token payload shared by access and refresh tokens.
///
/// `sub` (the user) and `identity_id` (the login method that owns the
/// session) are kept separate: the refresh token is bound to the identity so
/// that a logout can invalidate it server-side through `jti`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// User id.
    pub sub: String,
    /// [`crate::auth::UserIdentity`] id owning the session.
    pub identity_id: String,
    /// Role of the user, checked by the role guard.
    pub role: String,
    /// [`ACCESS`] or [`REFRESH`].
    pub token_use: String,
    /// Session id stored on the identity: it changes at every refresh
    /// (rotation) and is cleared on logout, which is what invalidates the
    /// previous refresh token.
    pub jti: String,
    /// Issued at (seconds since epoch).
    pub iat: usize,
    /// Expiration (seconds since epoch).
    pub exp: usize,
}

impl JwtConfig {
    /// Current unix time in seconds, shared with the auth service for
    /// server-side session checks.
    pub fn now() -> usize {
        get_current_timestamp() as usize
    }

    /// Lifetime of an access token in seconds (`ACCESS_TOKEN_DURATION`).
    pub fn access_duration(&self) -> u64 {
        self.access_duration
    }

    /// When a refresh session persisted now would expire
    /// (`REFRESH_TOKEN_DURATION`): stored on the identity and checked again
    /// on every refresh, independently from the JWT.
    pub fn refresh_expiry(&self) -> usize {
        get_current_timestamp() as usize + self.refresh_duration as usize
    }

    /// Signs a new access token for the given user.
    pub fn issue_access(
        &self,
        user_id: &str,
        identity_id: &str,
        role: &str,
    ) -> Result<String, JwtError> {
        let claims = self.new_claims(ACCESS, user_id, identity_id, role, self.access_duration);
        self.encode(&claims)
    }

    /// Signs a new refresh token bound to `jti`, the session id just persisted
    /// on the identity.
    pub fn issue_refresh(
        &self,
        user_id: &str,
        identity_id: &str,
        role: &str,
        jti: &str,
    ) -> Result<String, JwtError> {
        let mut claims =
            self.new_claims(REFRESH, user_id, identity_id, role, self.refresh_duration);
        claims.jti = jti.to_string();
        self.encode(&claims)
    }

    /// Verifies an access token: signature, expiration and token kind.
    pub fn verify_access(&self, token: &str) -> Result<Claims, JwtError> {
        self.verify(token, ACCESS)
    }

    /// Verifies a refresh token: signature, expiration and token kind.
    pub fn verify_refresh(&self, token: &str) -> Result<Claims, JwtError> {
        self.verify(token, REFRESH)
    }

    /// Signs arbitrary claims: exposed so the tests can build edge cases such
    /// as an expired access token.
    pub fn encode(&self, claims: &Claims) -> Result<String, JwtError> {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(Into::into)
    }

    fn verify(&self, token: &str, expected_use: &str) -> Result<Claims, JwtError> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )?;

        if data.claims.token_use != expected_use {
            return Err(JwtError);
        }
        Ok(data.claims)
    }

    fn new_claims(
        &self,
        token_use: &str,
        user_id: &str,
        identity_id: &str,
        role: &str,
        duration: u64,
    ) -> Claims {
        let iat = get_current_timestamp() as usize;
        Claims {
            sub: user_id.to_string(),
            identity_id: identity_id.to_string(),
            role: role.to_string(),
            token_use: token_use.to_string(),
            jti: Uuid::new_v4().to_string(),
            iat,
            exp: iat + duration as usize,
        }
    }
}
