use crate::lib::auth::password;

/// Role of a user created through the registration endpoint.
pub const ROLE_USER: &str = "user";
/// Administrative role: only assigned directly on the database.
pub const ROLE_ADMIN: &str = "admin";

/// The person inside the software: profile data and role.
///
/// Dissociated from [`crate::lib::auth::UserIdentity`] (the access method):
/// the user says *who* the person is, the identity says *how* they get in.
#[derive(Debug, Clone)]
pub struct User {
    /// UUID of the user.
    pub id: String,
    pub firstname: String,
    pub lastname: String,
    /// One of [`ROLE_USER`]/[`ROLE_ADMIN`]: read by the role guard through
    /// the JWT claim, so a change applies at the next login.
    pub role: String,
}

impl User {
    /// Verifies a password against a hash stored on one of its identities.
    ///
    /// A convenience kept here so the auth service never touches the hashing
    /// primitives directly when it only needs to check a credential.
    pub fn verify_password(password: &str, password_hash: &str) -> bool {
        password::verify(password, password_hash)
    }
}
