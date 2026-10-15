use std::sync::Arc;

use di::injectable;

use crate::api::color::color_dto::{CreateColorDto, UpdateColorDto};
use crate::api::color::color_repository::ColorRepository;
use crate::api::color::color_response::ColorResponse;
use crate::errors::{AppError, AppResult};
use crate::lib::pagination::{Paged, Pagination};

/// Business rules for colors.
#[injectable]
pub struct ColorService {
    repository: Arc<dyn ColorRepository>,
}

impl ColorService {
    pub async fn create(&self, dto: CreateColorDto) -> AppResult<ColorResponse> {
        self.repository.insert(dto.into_entity()).await
    }

    pub async fn get(&self, id: &str) -> AppResult<ColorResponse> {
        self.repository
            .find(id)
            .await?
            .ok_or_else(|| AppError::not_found("color", id))
    }

    /// One page of colors with the total count (sent as `X-TOTAL-LIST`).
    pub async fn list_page(&self, pagination: Pagination) -> AppResult<Paged<ColorResponse>> {
        let total = self.repository.count().await?;
        let items = self
            .repository
            .find_page(pagination.offset(), pagination.limit())
            .await?;

        Ok(Paged::new(items, total))
    }

    pub async fn update(&self, id: &str, dto: UpdateColorDto) -> AppResult<ColorResponse> {
        let item = dto.into_entity(id.to_string());

        self.repository
            .replace(id, item)
            .await?
            .ok_or_else(|| AppError::not_found("color", id))
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("color", id))
        }
    }
}
