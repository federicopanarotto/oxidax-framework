//! Reusable SurrealDB read helpers shared by the repositories.
//!
//! They implement the two low-level primitives every paginated list needs
//! (total count and a page of rows) so the repositories only declare their row
//! type instead of repeating the same SurrealQL. `table`/`field` are internal
//! compile-time constants, never user input; the values always go through
//! bindings.

use serde::Deserialize;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::SurrealValue;

use crate::errors::AppResult;

/// Shape of the single row returned by `SELECT count() ... GROUP ALL`.
#[derive(Debug, Deserialize, SurrealValue)]
struct CountRow {
    total: i64,
}

/// Total number of rows in the table.
pub async fn count(db: &Surreal<Any>, table: &str) -> AppResult<u64> {
    let mut response = db
        .query(format!("SELECT count() AS total FROM {table} GROUP ALL"))
        .await?;
    total_of(&mut response)
}

/// Total number of rows matching `field = value`.
pub async fn count_by(db: &Surreal<Any>, table: &str, field: &str, value: &str) -> AppResult<u64> {
    let mut response = db
        .query(format!(
            "SELECT count() AS total FROM {table} WHERE {field} = $value GROUP ALL"
        ))
        .bind(("value", value.to_string()))
        .await?;
    total_of(&mut response)
}

/// One page of rows (`offset` skipped, at most `limit` returned).
pub async fn select_page<T>(
    db: &Surreal<Any>,
    table: &str,
    offset: u64,
    limit: u64,
) -> AppResult<Vec<T>>
where
    T: SurrealValue,
{
    let mut response = db
        .query(format!("SELECT * FROM {table} LIMIT $limit START $start"))
        .bind(("limit", limit as i64))
        .bind(("start", offset as i64))
        .await?;
    Ok(response.take(0)?)
}

/// One page of rows matching `field = value`.
pub async fn select_page_by<T>(
    db: &Surreal<Any>,
    table: &str,
    field: &str,
    value: &str,
    offset: u64,
    limit: u64,
) -> AppResult<Vec<T>>
where
    T: SurrealValue,
{
    let mut response = db
        .query(format!(
            "SELECT * FROM {table} WHERE {field} = $value LIMIT $limit START $start"
        ))
        .bind(("value", value.to_string()))
        .bind(("limit", limit as i64))
        .bind(("start", offset as i64))
        .await?;
    Ok(response.take(0)?)
}

/// `GROUP ALL` produces no row when the table (or the filter) matches nothing.
fn total_of(response: &mut surrealdb::IndexedResults) -> AppResult<u64> {
    let row: Option<CountRow> = response.take(0)?;
    Ok(row.map(|row| row.total.max(0) as u64).unwrap_or(0))
}
