use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::SurrealValue;

use crate::api::places::place_entity::Place;
use crate::api::places::place_repository::PlaceRepository;
use crate::api::places::place_response::PlaceResponse;
use crate::db::{define_table, query};
use crate::errors::AppResult;
use crate::framework::ViewRepository;

const TABLE: &str = "place";

/// Persistable content: the id is managed by the database, it must not be
/// written in the CONTENT.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct PlaceContent {
    city: String,
    address: String,
    cap: String,
    provenance: String,
}

pub struct SurrealPlaceRepository {
    db: Surreal<Any>,
}

impl SurrealPlaceRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Creates the table if it does not exist: SurrealDB 3 does not create
    /// tables implicitly.
    pub async fn ensure_schema(&self) -> AppResult<()> {
        define_table(&self.db, TABLE).await
    }
}

/// Enables all the populated reads (`find`, `find_page`, `count`, `delete`) on
/// top of the shared connection.
#[async_trait]
impl ViewRepository for SurrealPlaceRepository {
    type Response = PlaceResponse;

    fn db(&self) -> &Surreal<Any> {
        &self.db
    }
}

fn content(place: Place) -> PlaceContent {
    PlaceContent {
        city: place.city,
        address: place.address,
        cap: place.cap,
        provenance: place.provenance,
    }
}

#[async_trait]
impl PlaceRepository for SurrealPlaceRepository {
    async fn insert(&self, place: Place) -> AppResult<PlaceResponse> {
        let id = place.id.clone();
        query::insert_view(&self.db, &id, content(place)).await
    }

    async fn replace(&self, id: &str, place: Place) -> AppResult<Option<PlaceResponse>> {
        query::replace_view(&self.db, id, content(place)).await
    }
}
