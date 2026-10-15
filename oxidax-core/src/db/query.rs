//! Reusable SurrealDB read helpers shared by the repositories.
//!
//! They implement the two low-level primitives every paginated list needs
//! (total count and a page of rows) so the repositories only declare their row
//! type instead of repeating the same SurrealQL. `table`/`field` are internal
//! compile-time constants, never user input; the values always go through
//! bindings.

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::SurrealValue;

use crate::errors::{AppError, AppResult};
use crate::response::Response;

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

/// One page of projected responses: a single `SELECT VALUE` resolves the row
/// and its relations (no N+1). `T` declares its table and projection.
pub async fn select_view<T>(db: &Surreal<Any>, offset: u64, limit: u64) -> AppResult<Vec<T>>
where
    T: Response + DeserializeOwned,
{
    let mut response = db
        .query(format!(
            "SELECT VALUE {} FROM {} LIMIT $limit START $start",
            T::select_fields(),
            T::TABLE
        ))
        .bind(("limit", limit as i64))
        .bind(("start", offset as i64))
        .await?;
    decode(response.take(0)?)
}

/// One page of projected responses matching `field = value`: the relations are
/// still resolved in the same query. `field` is an internal compile-time
/// constant, `value` always goes through a binding.
pub async fn select_view_by<T>(
    db: &Surreal<Any>,
    field: &str,
    value: &str,
    offset: u64,
    limit: u64,
) -> AppResult<Vec<T>>
where
    T: Response + DeserializeOwned,
{
    let mut response = db
        .query(format!(
            "SELECT VALUE {} FROM {} WHERE {field} = $value LIMIT $limit START $start",
            T::select_fields(),
            T::TABLE
        ))
        .bind(("value", value.to_string()))
        .bind(("limit", limit as i64))
        .bind(("start", offset as i64))
        .await?;
    decode(response.take(0)?)
}

/// A single projected response by id, `None` if the row does not exist. The
/// relations are resolved in the same query.
pub async fn find_view<T>(db: &Surreal<Any>, id: &str) -> AppResult<Option<T>>
where
    T: Response + DeserializeOwned,
{
    let mut response = db
        .query(format!(
            "SELECT VALUE {} FROM {} WHERE id = type::record('{}', $id)",
            T::select_fields(),
            T::TABLE,
            T::TABLE
        ))
        .bind(("id", id.to_string()))
        .await?;
    let rows: Vec<Value> = response.take(0)?;
    rows.into_iter().next().map(decode_one).transpose()
}

/// Inserts `content` under `id` and returns the projected response, so the
/// caller never reads the row back by hand. `content` is a database content
/// struct (`#[derive(SurrealValue)]`), `T` the read-model.
pub async fn insert_view<T, C>(db: &Surreal<Any>, id: &str, content: C) -> AppResult<T>
where
    T: Response + DeserializeOwned,
    C: SurrealValue,
{
    let created: Option<Value> = db.create((T::TABLE, id)).content(content).await?;
    if created.is_none() {
        return Err(AppError::Internal(format!(
            "failed to create {} {id}",
            T::TABLE
        )));
    }
    find_view(db, id)
        .await?
        .ok_or_else(|| AppError::Internal(format!("{} {id} vanished after create", T::TABLE)))
}

/// Merges `patch` into `id` and returns the projected response; `None` if the
/// record does not exist. Only the fields present in `patch` are rewritten.
pub async fn replace_view<T, C>(db: &Surreal<Any>, id: &str, patch: C) -> AppResult<Option<T>>
where
    T: Response + DeserializeOwned,
    C: SurrealValue,
{
    let updated: Option<Value> = db.update((T::TABLE, id)).merge(patch).await?;
    if updated.is_none() {
        return Ok(None);
    }
    find_view(db, id).await
}

/// Decodes a projected row, mapping a decode failure to an internal error.
fn decode_one<T: DeserializeOwned>(value: Value) -> AppResult<T> {
    serde_json::from_value(value).map_err(|error| {
        AppError::Internal(format!("failed to decode response projection: {error}"))
    })
}

fn decode<T: DeserializeOwned>(rows: Vec<Value>) -> AppResult<Vec<T>> {
    rows.into_iter().map(decode_one).collect()
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
