use std::sync::Arc;

use di::injectable;
use uuid::Uuid;

use crate::api::auth::auth_dto::{
    LoginRequest, MessageResponse, RefreshRequest, RegisterRequest, TokenResponse,
};
use crate::api::users::user_entity::{ROLE_USER, User};
use crate::api::users::user_repository::UserRepository;
use crate::errors::{AppError, AppResult};
use crate::framework::CurrentUser;
use crate::lib::auth::{AuthProvider, JwtConfig, UserIdentity, UserIdentityRepository, password};

/// Generic answers: the API never reveals whether an account exists or which
/// part of the credential was wrong.
const INVALID_CREDENTIALS: &str = "invalid credentials";
const INVALID_REFRESH: &str = "invalid or expired refresh token";
const REGISTRATION_CONFLICT: &str = "registration could not be completed";
const REGISTERED: &str = "user registered successfully";

/// Authentication business rules: registration, login, refresh rotation and
/// logout.
///
/// The password never reaches this service in a reusable form (it is hashed
/// by the auth library) and every credential answer is deliberately uniform,
/// so the endpoints cannot be used to enumerate accounts.
#[injectable]
pub struct AuthService {
    users: Arc<dyn UserRepository>,
    identities: Arc<dyn UserIdentityRepository>,
    jwt: Arc<JwtConfig>,
}

impl AuthService {
    /// Creates the user and its local identity: email (normalized to
    /// lowercase) as username, hashed password as credential.
    pub async fn register(&self, dto: RegisterRequest) -> AppResult<MessageResponse> {
        let identifier = dto.email.trim().to_lowercase();

        if self
            .identities
            .find_by_identifier(&identifier)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict(REGISTRATION_CONFLICT.to_string()));
        }

        let password_hash = password_hash(&dto.password)?;

        let user = User {
            id: Uuid::new_v4().to_string(),
            firstname: dto.firstname.trim().to_string(),
            lastname: dto.lastname.trim().to_string(),
            // The role is never taken from the payload: only the first admin
            // is assigned manually on the database.
            role: ROLE_USER.to_string(),
        };
        self.users.insert(user.clone()).await?;

        let identity = UserIdentity {
            id: Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            provider: AuthProvider::Local,
            identifier,
            password_hash,
            refresh_jti: None,
            refresh_expires_at: None,
        };

        if let Err(err) = self.identities.insert(identity).await {
            // Compensating action: the two writes are not transactional yet,
            // so a failed identity must not leave an orphan user behind.
            if let Err(rollback) = self.users.remove(&user.id).await {
                tracing::warn!(
                    user_id = %user.id,
                    error = %rollback,
                    "user rollback after identity insert failure failed"
                );
            }
            return Err(err);
        }

        Ok(MessageResponse::new(REGISTERED))
    }

    /// Verifies the credentials and opens a session: the `jti` of the refresh
    /// token is persisted on the identity, so there is exactly one active
    /// session per identity (the previous refresh token stops working).
    pub async fn login(&self, dto: LoginRequest) -> AppResult<TokenResponse> {
        let identifier = dto.email.trim().to_lowercase();

        let Some(identity) = self.identities.find_by_identifier(&identifier).await? else {
            return Err(unauthorized(INVALID_CREDENTIALS));
        };

        if !password::verify(&dto.password, &identity.password_hash) {
            return Err(unauthorized(INVALID_CREDENTIALS));
        }

        let Some(user) = self.users.find_by_id(&identity.user_id).await? else {
            return Err(unauthorized(INVALID_CREDENTIALS));
        };

        self.issue_session(&user, &identity).await
    }

    /// Exchanges a refresh token for a new pair: the token must be valid,
    /// still the one stored on the identity (not rotated away) and not
    /// expired. The old refresh token is burned by the rotation.
    pub async fn refresh(&self, dto: RefreshRequest) -> AppResult<TokenResponse> {
        let claims = self
            .jwt
            .verify_refresh(&dto.refresh_token)
            .map_err(|_| unauthorized(INVALID_REFRESH))?;

        let identity = self
            .identities
            .find_by_id(&claims.identity_id)
            .await?
            .ok_or_else(|| unauthorized(INVALID_REFRESH))?;

        if !identity.accepts_refresh(&claims.jti, JwtConfig::now()) {
            // Already rotated by a previous refresh or revoked by logout.
            return Err(unauthorized(INVALID_REFRESH));
        }

        let user = self
            .users
            .find_by_id(&identity.user_id)
            .await?
            .ok_or_else(|| unauthorized(INVALID_REFRESH))?;

        self.issue_session(&user, &identity).await
    }

    /// Revokes the session of the caller: clearing the stored `jti` makes the
    /// outstanding refresh token unusable immediately.
    pub async fn logout(&self, current: &CurrentUser) -> AppResult<()> {
        self.identities.clear_refresh(&current.identity_id).await?;
        Ok(())
    }

    /// Signs the pair of tokens and persists the session `jti` on the
    /// identity: the role is read fresh from the database, so a role change
    /// applies from the next login/refresh.
    async fn issue_session(
        &self,
        user: &User,
        identity: &UserIdentity,
    ) -> AppResult<TokenResponse> {
        let jti = Uuid::new_v4().to_string();
        self.identities
            .set_refresh(&identity.id, &jti, self.jwt.refresh_expiry())
            .await?;

        let access_token = self
            .jwt
            .issue_access(&user.id, &identity.id, &user.role)
            .map_err(token_error)?;
        let refresh_token = self
            .jwt
            .issue_refresh(&user.id, &identity.id, &user.role, &jti)
            .map_err(token_error)?;

        Ok(TokenResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt.access_duration(),
        })
    }
}

/// Hashes the password: a failure here means the server cannot hash at all,
/// so it is reported as an internal error (never as a client error).
fn password_hash(plain: &str) -> AppResult<String> {
    password::hash(plain)
        .map_err(|err| AppError::Internal(format!("failed to hash the password: {err}")))
}

fn unauthorized(message: &str) -> AppError {
    AppError::Unauthorized(message.to_string())
}

/// Signing failure: a server-side problem (bad configuration), not a client
/// one, and the cause is not leaked to the caller.
fn token_error(_: crate::lib::auth::jwt::JwtError) -> AppError {
    AppError::Internal("failed to issue the tokens".to_string())
}
