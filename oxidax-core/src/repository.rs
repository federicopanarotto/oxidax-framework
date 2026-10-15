//! Repository building block that resolves relations in the database.
//!
//! A feature that exposes an aggregate with relations derives
//! [`Response`](crate::response::Response) on its read-model and implements
//! this trait once: the populated reads (`find`, `find_page`, `count`,
//! `delete`) and the writes come for free, so the service never fetches a
//! related row by hand and there is no N+1.

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use crate::db::query;
use crate::errors::AppResult;
use crate::response::Response;

/// A SurrealDB-backed repository whose every read returns the populated
/// [`Response`](crate::response::Response) of the aggregate.
///
/// Implement it by exposing the shared connection; all the read methods have a
/// default implementation that projects the read-model (and its relations) in a
/// single query. The writes are thin wrappers over
/// [`query::insert_view`](crate::db::query::insert_view) and
/// [`query::replace_view`](crate::db::query::replace_view).
#[async_trait]
pub trait ViewRepository: Send + Sync {
    /// The read-model returned by every read.
    type Response: Response + DeserializeOwned + Send + 'static;

    /// The shared SurrealDB connection.
    fn db(&self) -> &Surreal<Any>;

    /// A single populated record; `None` if it does not exist.
    async fn find(&self, id: &str) -> AppResult<Option<Self::Response>> {
        query::find_view(self.db(), id).await
    }

    /// One page of populated records (`offset` skipped, at most `limit`).
    async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<Self::Response>> {
        query::select_view(self.db(), offset, limit).await
    }

    /// One page of populated records matching `field = value`: the populated
    /// shape is the same, only the filter changes. `field` is an internal
    /// compile-time constant, `value` goes through a binding.
    async fn find_page_by(
        &self,
        field: &str,
        value: &str,
        offset: u64,
        limit: u64,
    ) -> AppResult<Vec<Self::Response>> {
        query::select_view_by(self.db(), field, value, offset, limit).await
    }

    /// Total number of records (for the pagination header).
    async fn count(&self) -> AppResult<u64> {
        query::count(self.db(), Self::Response::TABLE).await
    }

    /// Total number of records matching `field = value`.
    async fn count_by(&self, field: &str, value: &str) -> AppResult<u64> {
        query::count_by(self.db(), Self::Response::TABLE, field, value).await
    }

    /// Deletes a record; `false` if it did not exist.
    async fn delete(&self, id: &str) -> AppResult<bool> {
        let deleted: Option<serde_json::Value> =
            self.db().delete((Self::Response::TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}
