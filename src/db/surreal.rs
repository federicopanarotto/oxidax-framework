use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::opt::auth::Root;
use surrealdb::types::{RecordIdKey, ToSql};

use crate::config::Config;
use crate::errors::{AppError, AppResult};

/// Apre la connessione a SurrealDB e imposta il namespace/database.
///
/// La connessione è condivisa da tutti i repository dell'applicazione.
pub async fn connect(url: &str, config: &Config) -> AppResult<Surreal<Any>> {
    let db = Surreal::<Any>::init();

    db.connect(url).await.map_err(into_db_error)?;

    if needs_auth(url) && !config.surreal_user.is_empty() {
        db.signin(Root {
            username: config.surreal_user.clone(),
            password: config.surreal_pass.clone(),
        })
        .await
        .map_err(into_db_error)?;
    }

    db.use_ns(&config.surreal_ns)
        .use_db(&config.surreal_db)
        .await
        .map_err(into_db_error)?;

    tracing::info!(url, ns = %config.surreal_ns, db = %config.surreal_db, "connected to SurrealDB");

    Ok(db)
}

/// Crea la tabella se non esiste: SurrealDB 3 non crea tabelle implicitamente.
pub async fn define_table(db: &Surreal<Any>, table: &str) -> AppResult<()> {
    db.query(format!("DEFINE TABLE IF NOT EXISTS {table} SCHEMALESS"))
        .await?
        .check()?;
    Ok(())
}

/// Converte la chiave di un record id nel solo valore usato dall'API (`uuid`),
/// non nel formato completo `table:uuid`.
pub fn record_key_to_string(key: RecordIdKey) -> String {
    match key {
        RecordIdKey::String(value) => value,
        RecordIdKey::Uuid(value) => value.to_string(),
        RecordIdKey::Number(value) => value.to_string(),
        other => other.to_sql(),
    }
}

fn needs_auth(url: &str) -> bool {
    ["ws://", "wss://", "http://", "https://"]
        .iter()
        .any(|scheme| url.starts_with(scheme))
}

fn into_db_error(err: surrealdb::Error) -> AppError {
    AppError::Database(err.to_string())
}
