use async_trait::async_trait;

use crate::api::places::place_entity::Place;
use crate::api::places::place_response::PlaceResponse;
use crate::errors::AppResult;
use crate::framework::ViewRepository;

/// Persistence contract for places.
///
/// It inherits the populated reads (`find`, `find_page`, `count`, `delete`)
/// from [`ViewRepository`]; only the aggregate-specific writes are declared
/// here.
#[async_trait]
pub trait PlaceRepository: ViewRepository<Response = PlaceResponse> {
    async fn insert(&self, place: Place) -> AppResult<PlaceResponse>;
    async fn replace(&self, id: &str, place: Place) -> AppResult<Option<PlaceResponse>>;
}
