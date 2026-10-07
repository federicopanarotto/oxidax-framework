use std::sync::Arc;

use uuid::Uuid;

use crate::api::categories::category_service::CategoryService;
use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::api::reports::report_entity::Report;
use crate::api::reports::report_repository::ReportRepository;
use crate::errors::{AppError, AppResult};

const NAME_MAX_LEN: usize = 200;
const DESCRIPTION_MAX_LEN: usize = 2000;

#[derive(Clone)]
pub struct ReportService {
    repository: Arc<dyn ReportRepository>,
    categories: CategoryService,
}

impl ReportService {
    pub fn new(repository: Arc<dyn ReportRepository>, categories: CategoryService) -> Self {
        Self {
            repository,
            categories,
        }
    }

    pub async fn create(&self, dto: CreateReportDto) -> AppResult<ReportResponse> {
        let name = validate_name(&dto.name)?;
        let description = validate_description(&dto.description)?;
        let category_id = self.validate_category_id(dto.category_id).await?;

        let report = Report {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            category_id,
        };

        self.repository.insert(report).await.map(Into::into)
    }

    pub async fn get(&self, id: &str) -> AppResult<ReportResponse> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| not_found(id))
    }

    pub async fn list(&self) -> AppResult<Vec<ReportResponse>> {
        self.repository
            .find_all()
            .await
            .map(|reports| reports.into_iter().map(Into::into).collect())
    }

    pub async fn update(&self, id: &str, dto: UpdateReportDto) -> AppResult<ReportResponse> {
        let name = validate_name(&dto.name)?;
        let description = validate_description(&dto.description)?;
        let category_id = self.validate_category_id(dto.category_id).await?;

        let report = Report {
            id: id.to_string(),
            name,
            description,
            category_id,
        };

        self.repository
            .replace(id, report)
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

    /// Valida la relazione con le categorie.
    ///
    /// Un `category_id` vuoto o assente normalizza a `None`; se presente deve
    /// riferire a una categoria esistente, altrimenti l'errore è una
    /// `validation_error` (400) perché il campo non passa la validazione.
    async fn validate_category_id(&self, category_id: Option<String>) -> AppResult<Option<String>> {
        let Some(category_id) = category_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };

        if !self.categories.exists(&category_id).await? {
            return Err(AppError::Validation(format!(
                "category {category_id} not found"
            )));
        }

        Ok(Some(category_id))
    }
}

fn not_found(id: &str) -> AppError {
    AppError::NotFound(format!("report {id} not found"))
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

fn validate_description(description: &str) -> AppResult<String> {
    let description = description.trim();
    if description.is_empty() {
        return Err(AppError::Validation("description must not be empty".into()));
    }
    if description.chars().count() > DESCRIPTION_MAX_LEN {
        return Err(AppError::Validation(format!(
            "description must be at most {DESCRIPTION_MAX_LEN} characters"
        )));
    }
    Ok(description.to_string())
}
