use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_addr: String,
    pub db_backend: DbBackend,
    pub surreal_url: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub surreal_user: String,
    pub surreal_pass: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbBackend {
    Surreal,
    Memory,
}

impl Config {
    pub fn from_env() -> Self {
        let db_backend = match env::var("DB_BACKEND").unwrap_or_else(|_| "surreal".into()) {
            s if s.eq_ignore_ascii_case("memory") => DbBackend::Memory,
            _ => DbBackend::Surreal,
        };

        Self {
            server_addr: env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into()),
            db_backend,
            surreal_url: env::var("SURREAL_URL").unwrap_or_else(|_| "ws://localhost:8000".into()),
            surreal_ns: env::var("SURREAL_NS").unwrap_or_else(|_| "test".into()),
            surreal_db: env::var("SURREAL_DB").unwrap_or_else(|_| "reports".into()),
            surreal_user: env::var("SURREAL_USER").unwrap_or_else(|_| "root".into()),
            surreal_pass: env::var("SURREAL_PASS").unwrap_or_else(|_| "root".into()),
        }
    }
}
