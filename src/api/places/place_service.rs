use std::sync::Arc;

use di::injectable;

use crate::api::places::place_dto::{CreatePlaceDto, UpdatePlaceDto};
use crate::api::places::place_repository::PlaceRepository;
use crate::api::places::place_response::PlaceResponse;
use crate::errors::{AppError, AppResult};
use crate::lib::pagination::{Paged, Pagination};

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
        self.repository.insert(dto.into_entity()).await
    }

    pub async fn get(&self, id: &str) -> AppResult<PlaceResponse> {
        self.repository
            .find(id)
            .await?
            .ok_or_else(|| AppError::not_found("place", id))
    }

    /// One page of places with the total count (sent as `X-TOTAL-LIST`).
    pub async fn list_page(&self, pagination: Pagination) -> AppResult<Paged<PlaceResponse>> {
        let total = self.repository.count().await?;
        let items = self
            .repository
            .find_page(pagination.offset(), pagination.limit())
            .await?;

        Ok(Paged::new(items, total))
    }

    pub async fn update(&self, id: &str, dto: UpdatePlaceDto) -> AppResult<PlaceResponse> {
        let place = dto.into_entity(id.to_string());

        self.repository
            .replace(id, place)
            .await?
            .ok_or_else(|| AppError::not_found("place", id))
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("place", id))
        }
    }

    /// Used by the reports service to validate the related place.
    pub async fn exists(&self, id: &str) -> AppResult<bool> {
        Ok(self.repository.find(id).await?.is_some())
    }
}
