//! The [`UserIdentity`] entity: how an access method identifies a user.
//!
//! Deliberately dissociated from the application's `User` entity:
//!
//! - the **user** is the person inside the software (name, role);
//! - the **identity** is one way to get in, in the spirit of passport.js
//!   "strategies": with `provider = local` the credential is an email used as
//!   username plus a password hash, and the rotating session of the refresh
//!   token lives here.
//!
//! A user can hold several identities (a local one today, an external
//! provider later) without the profile changing.

use serde::{Deserialize, Serialize};

/// Authentication provider of an identity (passport.js "strategy").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthProvider {
    /// Email + password, verified locally through JWTs.
    Local,
}

impl AuthProvider {
    /// Identifier persisted in the database: kept as a plain string in the
    /// rows so the stored format never depends on the serde representation
    /// of the enum.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
        }
    }

    /// Inverse of [`AuthProvider::as_str`]: unknown values map to `None`
    /// and the caller decides the fallback.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "local" => Some(Self::Local),
            _ => None,
        }
    }
}

/// Access method bound to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    /// UUID of the identity.
    pub id: String,
    /// UUID of the application's `User` this identity belongs to.
    pub user_id: String,
    /// Provider ("strategy") used to authenticate.
    pub provider: AuthProvider,
    /// Username of the identity: the email for the local provider.
    pub identifier: String,
    /// Argon2 PHC hash of the password (never the password itself).
    pub password_hash: String,
    /// `jti` of the refresh token currently in force: it is rewritten at
    /// every refresh (rotation) and cleared on logout, so the previous refresh
    /// token stops working.
    pub refresh_jti: Option<String>,
    /// Expiration of that refresh token, checked server-side as well.
    pub refresh_expires_at: Option<usize>,
}

impl UserIdentity {
    /// Whether the stored session is still valid for the given `jti`.
    pub fn accepts_refresh(&self, jti: &str, now: usize) -> bool {
        let stored = self.refresh_jti.as_deref() == Some(jti);
        let unexpired = self.refresh_expires_at.is_some_and(|expires| expires > now);
        stored && unexpired
    }
}
