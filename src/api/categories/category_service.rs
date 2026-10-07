use std::collections::HashMap;
use std::sync::Arc;

use di::injectable;
use uuid::Uuid;

use crate::api::categories::category_dto::{
    CategoryResponse, CreateCategoryDto, UpdateCategoryDto,
};
use crate::api::categories::category_entity::Category;
use crate::api::categories::category_repository::CategoryRepository;
use crate::errors::{AppError, AppResult};

/// Regole di business delle categorie.
///
/// La validazione dei campi è demandata all'extractor `ValidatedJson` (decoratori
/// `#[validate(...)]` sul DTO); qui resta solo la normalizzazione (trim).
/// `#[injectable]` genera il costruttore con le dipendenze (`Arc<T>`) iniettate
/// automaticamente dal container.
#[injectable]
pub struct CategoryService {
    repository: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    pub async fn create(&self, dto: CreateCategoryDto) -> AppResult<CategoryResponse> {
        let category = Category {
            id: Uuid::new_v4().to_string(),
            name: dto.name.trim().to_string(),
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
            name: dto.name.trim().to_string(),
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

    /// Recupera una categoria senza fallire se assente: usata dal service dei
    /// report per annidare l'oggetto `category` nella response.
    pub async fn find(&self, id: &str) -> AppResult<Option<CategoryResponse>> {
        Ok(self.repository.find_by_id(id).await?.map(Into::into))
    }

    /// Indice `id → categoria`: decora una lista di report con una sola lettura
    /// evitando N+1 query per ogni elemento.
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
