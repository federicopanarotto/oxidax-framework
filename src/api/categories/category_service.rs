use std::collections::HashMap;
use std::sync::Arc;

use di::injectable;

use crate::api::categories::category_dto::{
    CategoryResponse, CreateCategoryDto, UpdateCategoryDto,
};
use crate::api::categories::category_repository::CategoryRepository;
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
        let category = dto.into_entity();

        self.repository.insert(category).await.map(Into::into)
    }

    pub async fn get(&self, id: &str) -> AppResult<CategoryResponse> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| not_found(id))
    }

    pub async fn list(&self) -> AppResult<Vec<CategoryResponse>> {
        self.repository
            .find_all()
            .await
            .map(|categories| categories.into_iter().map(Into::into).collect())
    }

    /// One page of categories with the total count (sent as `X-TOTAL-LIST`).
    pub async fn list_page(&self, pagination: Pagination) -> AppResult<Paged<CategoryResponse>> {
        let total = self.repository.count().await?;
        let categories = self
            .repository
            .find_page(pagination.offset(), pagination.limit())
            .await?;

        let items = categories.into_iter().map(Into::into).collect();
        Ok(Paged::new(items, total))
    }

    pub async fn update(&self, id: &str, dto: UpdateCategoryDto) -> AppResult<CategoryResponse> {
        let category = dto.into_entity(id.to_string());

        self.repository
            .replace(id, category)
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

    /// Checks whether a category exists: used by the reports service to
    /// validate the relationship before creating or updating a report.
    pub async fn exists(&self, id: &str) -> AppResult<bool> {
        Ok(self.repository.find_by_id(id).await?.is_some())
    }

    /// Retrieves a category without failing if absent: used by the reports
    /// service to nest the `category` object in the response.
    pub async fn find(&self, id: &str) -> AppResult<Option<CategoryResponse>> {
        Ok(self.repository.find_by_id(id).await?.map(Into::into))
    }

    /// `id → category` index: decorates a list of reports with a single read,
    /// avoiding N+1 queries per element.
    pub async fn by_id(&self) -> AppResult<HashMap<String, CategoryResponse>> {
        let mut index = HashMap::new();
        for category in self.list().await? {
            index.insert(category.id.clone(), category);
        }
        Ok(index)
    }
}

fn not_found(id: &str) -> AppError {
    AppError::NotFound(format!("category {id} not found"))
}
