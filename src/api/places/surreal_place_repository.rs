use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::{RecordId, SurrealValue};

use crate::api::places::place_entity::Place;
use crate::api::places::place_repository::PlaceRepository;
use crate::db::{define_table, record_key_to_string};
use crate::errors::{AppError, AppResult};

const TABLE: &str = "place";

/// Row as it arrives from the database (the id is a SurrealDB record id).
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct PlaceRow {
    id: RecordId,
    city: String,
    address: String,
    cap: String,
    provenance: String,
}

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

/// The API works with the record id key alone (`uuid`), not with `table:uuid`.
fn into_entity(row: PlaceRow) -> Place {
    Place {
        id: record_key_to_string(row.id.key),
        city: row.city,
        address: row.address,
        cap: row.cap,
        provenance: row.provenance,
    }
}

fn created_or_internal(place: Place, created: Option<PlaceRow>) -> AppResult<Place> {
    if created.is_some() {
        Ok(place)
    } else {
        Err(AppError::Internal(format!(
            "failed to create place {}",
            place.id
        )))
    }
}

#[async_trait]
impl PlaceRepository for SurrealPlaceRepository {
    async fn insert(&self, place: Place) -> AppResult<Place> {
        let content = PlaceContent {
            city: place.city.clone(),
            address: place.address.clone(),
            cap: place.cap.clone(),
            provenance: place.provenance.clone(),
        };

        let created: Option<PlaceRow> = self
            .db
            .create((TABLE, place.id.as_str()))
            .content(content)
            .await?;

        created_or_internal(place, created)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<Place>> {
        let found: Option<PlaceRow> = self.db.select((TABLE, id)).await?;
        Ok(found.map(into_entity))
    }

    async fn find_all(&self) -> AppResult<Vec<Place>> {
        let rows: Vec<PlaceRow> = self.db.select(TABLE).await?;
        Ok(rows.into_iter().map(into_entity).collect())
    }

    async fn replace(&self, id: &str, place: Place) -> AppResult<Option<Place>> {
        let content = PlaceContent {
            city: place.city.clone(),
            address: place.address.clone(),
            cap: place.cap.clone(),
            provenance: place.provenance.clone(),
        };

        let updated: Option<PlaceRow> = self.db.update((TABLE, id)).content(content).await?;
        Ok(updated.map(into_entity))
    }

    async fn remove(&self, id: &str) -> AppResult<bool> {
        let deleted: Option<PlaceRow> = self.db.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}
