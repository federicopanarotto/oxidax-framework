use std::sync::Arc;

use di::injectable;

use crate::api::categories::category_dto::{CreateCategoryDto, UpdateCategoryDto};
use crate::api::categories::category_repository::CategoryRepository;
use crate::api::categories::category_response::CategoryResponse;
use crate::errors::{AppError, AppResult};
use crate::lib::pagination::{Paged, Pagination};

/// Business rules for categories.
///
/// The DTOs arrive already converted and validated (id generated, trimmed,
/// `#[validate(...)]` decorators of the `ValidatedJson` extractors): the
/// service only writes ready-made data to the repository.
#[injectable]
pub struct CategoryService {
    repository: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    pub async fn create(&self, dto: CreateCategoryDto) -> AppResult<CategoryResponse> {
        self.repository.insert(dto.into_entity()).await
    }

    pub async fn get(&self, id: &str) -> AppResult<CategoryResponse> {
        self.repository
            .find(id)
            .await?
            .ok_or_else(|| AppError::not_found("category", id))
    }

    /// One page of categories with the total count (sent as `X-TOTAL-LIST`).
    pub async fn list_page(&self, pagination: Pagination) -> AppResult<Paged<CategoryResponse>> {
        let total = self.repository.count().await?;
        let items = self
            .repository
            .find_page(pagination.offset(), pagination.limit())
            .await?;

        Ok(Paged::new(items, total))
    }

    pub async fn update(&self, id: &str, dto: UpdateCategoryDto) -> AppResult<CategoryResponse> {
        let category = dto.into_entity(id.to_string());

        self.repository
            .replace(id, category)
            .await?
            .ok_or_else(|| AppError::not_found("category", id))
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("category", id))
        }
    }

    /// Used by the reports service to validate the related category.
    pub async fn exists(&self, id: &str) -> AppResult<bool> {
        Ok(self.repository.find(id).await?.is_some())
    }
}
