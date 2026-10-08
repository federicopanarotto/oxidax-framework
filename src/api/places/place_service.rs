use std::collections::HashMap;
use std::sync::Arc;

use di::injectable;

use crate::api::places::place_dto::{CreatePlaceDto, PlaceResponse, UpdatePlaceDto};
use crate::api::places::place_repository::PlaceRepository;
use crate::errors::{AppError, AppResult};

/// Business rules for places.
///
/// The DTOs arrive already converted and validated (id generated, trimmed,
/// `#[validate(...)]` decorators of the `ValidatedJson` extractors): the
/// service only writes ready-made data to the repository.
#[injectable]
pub struct PlaceService {
    repository: Arc<dyn PlaceRepository>,
}

impl PlaceService {
    pub async fn create(&self, dto: CreatePlaceDto) -> AppResult<PlaceResponse> {
        let place = dto.into_entity();

        self.repository.insert(place).await.map(Into::into)
    }

    pub async fn get(&self, id: &str) -> AppResult<PlaceResponse> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| not_found(id))
    }

    pub async fn list(&self) -> AppResult<Vec<PlaceResponse>> {
        self.repository
            .find_all()
            .await
            .map(|places| places.into_iter().map(Into::into).collect())
    }

    pub async fn update(&self, id: &str, dto: UpdatePlaceDto) -> AppResult<PlaceResponse> {
        let place = dto.into_entity(id.to_string());

        self.repository
            .replace(id, place)
            .await?
            .map(Into::into)
            .ok_or_else(|| not_found(id))
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.remove(id).await? {
            Ok(())
        } else {
            Err(not_found(id))
        }
    }

    /// Checks whether a place exists: used by the reports service to validate
    /// the relationship before creating or updating a report.
    pub async fn exists(&self, id: &str) -> AppResult<bool> {
        Ok(self.repository.find_by_id(id).await?.is_some())
    }

    /// Retrieves a place without failing if absent: used by the reports service
    /// to nest the `place` object in the response.
    pub async fn find(&self, id: &str) -> AppResult<Option<PlaceResponse>> {
        Ok(self.repository.find_by_id(id).await?.map(Into::into))
    }

    /// `id → place` index: decorates a list of reports with a single read,
    /// avoiding N+1 queries per element.
    pub async fn by_id(&self) -> AppResult<HashMap<String, PlaceResponse>> {
        let mut index = HashMap::new();
        for place in self.list().await? {
            index.insert(place.id.clone(), place);
        }
        Ok(index)
    }
}

fn not_found(id: &str) -> AppError {
    AppError::NotFound(format!("place {id} not found"))
}
