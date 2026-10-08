use async_trait::async_trait;

use crate::api::places::place_entity::Place;
use crate::errors::AppResult;

/// Persistence contract for places.
///
/// It is the only point that depends on the database: implementing this
/// trait on a new backend (Postgres, SQLite, MongoDB...) leaves the service
/// and controllers unchanged.
#[async_trait]
pub trait PlaceRepository: Send + Sync {
    async fn insert(&self, place: Place) -> AppResult<Place>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Place>>;
    async fn find_all(&self) -> AppResult<Vec<Place>>;
    async fn replace(&self, id: &str, place: Place) -> AppResult<Option<Place>>;
    async fn remove(&self, id: &str) -> AppResult<bool>;
}
