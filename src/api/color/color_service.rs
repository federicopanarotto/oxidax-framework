use std::sync::Arc;

use di::injectable;

use crate::api::color::color_dto::{ColorResponse, CreateColorDto, UpdateColorDto};
use crate::api::color::color_repository::ColorRepository;
use crate::errors::{AppError, AppResult};
use crate::lib::pagination::{Paged, Pagination};

/// Business rules for colors.
#[injectable]
pub struct ColorService {
    repository: Arc<dyn ColorRepository>,
}

impl ColorService {
    pub async fn create(&self, dto: CreateColorDto) -> AppResult<ColorResponse> {
        let item = dto.into_entity();
        self.repository.insert(item).await.map(Into::into)
    }

    pub async fn get(&self, id: &str) -> AppResult<ColorResponse> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| not_found(id))
    }

    pub async fn list(&self) -> AppResult<Vec<ColorResponse>> {
        self.repository
            .find_all()
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
    }

    /// One page of colors with the total count (sent as `X-TOTAL-LIST`).
    pub async fn list_page(&self, pagination: Pagination) -> AppResult<Paged<ColorResponse>> {
        let total = self.repository.count().await?;
        let items = self
            .repository
            .find_page(pagination.offset(), pagination.limit())
            .await?;

        let items = items.into_iter().map(Into::into).collect();
        Ok(Paged::new(items, total))
    }

    pub async fn update(&self, id: &str, dto: UpdateColorDto) -> AppResult<ColorResponse> {
        let item = dto.into_entity(id.to_string());
        self.repository
            .replace(id, item)
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
}

fn not_found(id: &str) -> AppError {
    AppError::NotFound(format!("color {id} not found"))
}
