use std::sync::Arc;

use uuid::Uuid;

use crate::api::categories::category_dto::{
    CategoryResponse, CreateCategoryDto, UpdateCategoryDto,
};
use crate::api::categories::category_entity::Category;
use crate::api::categories::category_repository::CategoryRepository;
use crate::errors::{AppError, AppResult};

const NAME_MAX_LEN: usize = 200;

/// Regole di business delle categorie.
///
/// Il campo `repository` è `Arc`: il service è `Clone` e può essere condiviso
/// fra il controller delle category e quello dei report (che ne valida
/// l'esistenza per la relazione `Report.category_id`).
#[derive(Clone)]
pub struct CategoryService {
    repository: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    pub fn new(repository: Arc<dyn CategoryRepository>) -> Self {
        Self { repository }
    }

    pub async fn create(&self, dto: CreateCategoryDto) -> AppResult<CategoryResponse> {
        let category = Category {
            id: Uuid::new_v4().to_string(),
            name: validate_name(&dto.name)?,
        };

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

    pub async fn update(&self, id: &str, dto: UpdateCategoryDto) -> AppResult<CategoryResponse> {
        let category = Category {
            id: id.to_string(),
            name: validate_name(&dto.name)?,
        };

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

    /// Verifica l'esistenza di una categoria: usata dal service dei report
    /// per validare la relazione prima di creare o aggiornare un report.
    pub async fn exists(&self, id: &str) -> AppResult<bool> {
        Ok(self.repository.find_by_id(id).await?.is_some())
    }
}

fn not_found(id: &str) -> AppError {
    AppError::NotFound(format!("category {id} not found"))
}

fn validate_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("name must not be empty".into()));
    }
    if name.chars().count() > NAME_MAX_LEN {
        return Err(AppError::Validation(format!(
            "name must be at most {NAME_MAX_LEN} characters"
        )));
    }
    Ok(name.to_string())
}
