use std::env;
use std::path::PathBuf;

use regex::Regex;

/// Default per-file upload limit: 10 MiB.
const DEFAULT_UPLOAD_MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
/// Default maximum number of files accepted in a single request.
const DEFAULT_UPLOAD_MAX_FILES: usize = 10;

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
    /// Root directory where uploaded files are stored (`ASSETS_DIR`).
    pub assets_dir: PathBuf,
    /// Maximum size, in bytes, of a single uploaded file.
    pub upload_max_file_size: u64,
    /// Maximum number of files accepted in a single multipart request.
    pub upload_max_files: usize,
    /// Maximum size, in bytes, of the whole multipart request body.
    pub upload_max_body_size: u64,
    /// Optional allow-list of file extensions (without the dot). When empty,
    /// every extension is accepted.
    pub upload_allowed_extensions: Vec<String>,
    /// Serve `/scalar` and `/api-docs/openapi.json` when true (`OPENAPI_DOC`,
    /// default true). Set to `false` in production.
    pub openapi_doc: bool,
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
            openapi_doc: true,
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
            assets_dir: PathBuf::from(required("ASSETS_DIR")),
            upload_max_file_size: optional_parsed(
                "UPLOAD_MAX_FILE_SIZE",
                DEFAULT_UPLOAD_MAX_FILE_SIZE,
            ),
            upload_max_files: optional_parsed("UPLOAD_MAX_FILES", DEFAULT_UPLOAD_MAX_FILES),
            upload_max_body_size: optional_parsed(
                "UPLOAD_MAX_BODY_SIZE",
                DEFAULT_UPLOAD_MAX_FILE_SIZE * DEFAULT_UPLOAD_MAX_FILES as u64,
            ),
            upload_allowed_extensions: optional_list("UPLOAD_ALLOWED_EXTENSIONS"),
            openapi_doc: optional_bool("OPENAPI_DOC", true),
        }
    }
}

/// Reads an optional boolean variable, falling back to `default` when unset.
/// A present but unparsable value is a configuration error and panics.
fn optional_bool(key: &str, default: bool) -> bool {
    match optional(key) {
        Some(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            other => panic!("environment variable `{key}` must be a boolean (got `{other}`)"),
        },
        None => default,
    }
}

/// Reads an optional environment variable, returning `None` when unset or
/// empty.
fn optional(key: &str) -> Option<String> {
    env::var(key).ok().filter(|value| !value.trim().is_empty())
}

/// Reads an optional numeric variable, falling back to `default` when unset.
/// A present but unparsable value is a configuration error and panics.
fn optional_parsed<T>(key: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    match optional(key) {
        Some(raw) => raw
            .parse()
            .unwrap_or_else(|_| panic!("environment variable `{key}` must be a valid number")),
        None => default,
    }
}

/// Reads an optional comma-separated list of values (e.g. file extensions).
fn optional_list(key: &str) -> Vec<String> {
    optional(key)
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
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
