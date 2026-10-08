//! Password hashing (Argon2id, PHC string format).
//!
//! Only hashes are stored: the plaintext never reaches the database. The PHC
//! format (`$argon2id$v=19$...`) embeds algorithm and parameters, so the
//! hashing function can be upgraded later without a data migration.

use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
};

/// Hashes a password with a random salt (Argon2id with the default parameters).
pub fn hash(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}

/// Verifies a password against a stored PHC hash. A malformed hash fails
/// closed (returns `false`) instead of panicking.
pub fn verify(password: &str, password_hash: &str) -> bool {
    PasswordHash::new(password_hash).is_ok_and(|parsed| {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
}
