use std::env;

use regex::Regex;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_addr: String,
    pub db_backend: DbBackend,
    pub surreal_url: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    /// Secret used to sign the JWTs.
    pub jwt_secret: String,
    /// Lifetime in seconds of an access token.
    pub access_token_duration: u64,
    /// Lifetime in seconds of a refresh token.
    pub refresh_token_duration: u64,
    /// Regex allow-list matched against the CORS `Origin` header. An empty list
    /// disables CORS entirely (no cross-origin requests allowed).
    pub cors_allowed_origins: Vec<Regex>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbBackend {
    Surreal,
    Memory,
}

impl Config {
    /// Configuration with the in-memory backend (tests and `DB_BACKEND=memory`).
    pub fn memory() -> Self {
        Self {
            db_backend: DbBackend::Memory,
            ..Self::from_env()
        }
    }

    pub fn from_env() -> Self {
        // Load the `.env` file if present. Existing environment variables win
        // (dotenvy does not override them by default), so deployments can inject
        // real values and only fall back to the file for local development.
        dotenvy::dotenv().ok();

        let db_backend = match required("DB_BACKEND") {
            s if s.eq_ignore_ascii_case("memory") => DbBackend::Memory,
            _ => DbBackend::Surreal,
        };

        Self {
            server_addr: required("SERVER_ADDR"),
            db_backend,
            surreal_url: required("SURREAL_URL"),
            surreal_ns: required("SURREAL_NS"),
            surreal_db: required("SURREAL_DB"),
            surreal_user: required("SURREAL_USER"),
            surreal_pass: required("SURREAL_PASS"),
            jwt_secret: required("JWT_SECRET"),
            access_token_duration: required_duration("ACCESS_TOKEN_DURATION"),
            refresh_token_duration: required_duration("REFRESH_TOKEN_DURATION"),
            cors_allowed_origins: optional_regex_list("CORS_ALLOWED_ORIGINS"),
        }
    }
}

/// Reads an optional comma-separated list of regex patterns (CORS origins).
///
/// Each entry is compiled and matched against the request's `Origin` header
/// (scheme + host + port); a lone `*` is a shortcut for "any origin". A missing
/// or empty variable yields an empty list (CORS disabled). An invalid regex
/// fails fast at startup with a clear message.
fn optional_regex_list(key: &str) -> Vec<Regex> {
    let Ok(raw) = env::var(key) else {
        return Vec::new();
    };

    raw.split(',')
        .map(str::trim)
        .filter(|pattern| !pattern.is_empty())
        .map(|pattern| {
            let source = if pattern == "*" { ".*" } else { pattern };
            Regex::new(source).unwrap_or_else(|err| {
                panic!("environment variable `{key}`: invalid origin regex `{pattern}`: {err}")
            })
        })
        .collect()
}

/// Reads a mandatory environment variable holding a duration in seconds.
fn required_duration(key: &str) -> u64 {
    let raw = required(key);
    raw.parse().unwrap_or_else(|_| {
        panic!("environment variable `{key}` must be a positive number of seconds")
    })
}

/// Reads a mandatory environment variable: configuration is never hardcoded in
/// the code, it always comes from the environment/`.env` (see `.env.example`).
fn required(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        panic!("missing required environment variable `{key}` (see .env.example)")
    })
}
