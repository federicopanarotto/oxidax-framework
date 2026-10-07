use std::sync::Arc;

use di::injectable;
use uuid::Uuid;

use crate::api::categories::category_dto::CategoryResponse;
use crate::api::categories::category_service::CategoryService;
use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::api::reports::report_entity::Report;
use crate::api::reports::report_repository::ReportRepository;
use crate::errors::{AppError, AppResult};

/// Regole di business dei report.
///
/// La validazione dei campi (nome/descrizione) è demandata agli estrattori
/// `ValidatedJson`/`ValidatedQuery`/`ValidatedParams` (con decoratori
/// `#[validate(...)]` sui DTO); qui resta solo la normalizzazione (trim) e la
/// validazione di dominio che richiede il database (esistenza della categoria).
#[injectable]
pub struct ReportService {
    repository: Arc<dyn ReportRepository>,
    categories: Arc<CategoryService>,
}

impl ReportService {
    pub async fn create(&self, dto: CreateReportDto) -> AppResult<ReportResponse> {
        let report = Report {
            id: Uuid::new_v4().to_string(),
            name: dto.name.trim().to_string(),
            description: dto.description.trim().to_string(),
            category_id: self.validate_category_id(dto.category_id).await?,
        };

        let report = self.repository.insert(report).await?;
        self.decorate(report).await
    }

    pub async fn get(&self, id: &str) -> AppResult<ReportResponse> {
        let report = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| not_found(id))?;
        self.decorate(report).await
    }

    pub async fn list(&self) -> AppResult<Vec<ReportResponse>> {
        let reports = self.repository.find_all().await?;
        let categories = self.categories.by_id().await?;

        Ok(reports
            .into_iter()
            .map(|report| {
                let category = report
                    .category_id
                    .as_ref()
                    .and_then(|id| categories.get(id).cloned());
                (report, category).into()
            })
            .collect())
    }

    pub async fn update(&self, id: &str, dto: UpdateReportDto) -> AppResult<ReportResponse> {
        let report = Report {
            id: id.to_string(),
            name: dto.name.trim().to_string(),
            description: dto.description.trim().to_string(),
            category_id: self.validate_category_id(dto.category_id).await?,
        };

        let report = self
            .repository
            .replace(id, report)
            .await?
            .ok_or_else(|| not_found(id))?;
        self.decorate(report).await
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

    /// Compone la response annidando l'oggetto `category` (se la relazione esiste).
    async fn decorate(&self, report: Report) -> AppResult<ReportResponse> {
        let category = self.category_of(&report).await?;
        Ok((report, category).into())
    }

    async fn category_of(&self, report: &Report) -> AppResult<Option<CategoryResponse>> {
        match &report.category_id {
            Some(category_id) => self.categories.find(category_id).await,
            None => Ok(None),
        }
    }
}

fn not_found(id: &str) -> AppError {
    AppError::NotFound(format!("report {id} not found"))
}
