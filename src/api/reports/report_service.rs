use std::sync::Arc;

use di::injectable;

use crate::api::categories::category_dto::CategoryResponse;
use crate::api::categories::category_service::CategoryService;
use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::api::reports::report_entity::Report;
use crate::api::reports::report_repository::ReportRepository;
use crate::errors::{AppError, AppResult};

/// Business rules for reports.
///
/// The DTOs arrive already converted and validated (id generated, trimmed,
/// UUID format checked by the `ValidatedJson` extractors): here the service
/// only writes ready-made data to the repository. Only the domain validation
/// that requires the database remains (existence of the category).
#[injectable]
pub struct ReportService {
    repository: Arc<dyn ReportRepository>,
    categories: Arc<CategoryService>,
}

impl ReportService {
    pub async fn create(&self, dto: CreateReportDto) -> AppResult<ReportResponse> {
        let report = dto.into_entity();
        self.validate_category_id(&report.category_id).await?;

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
        let report = dto.into_entity(id.to_string());
        self.validate_category_id(&report.category_id).await?;

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

    /// Validates the relationship with categories.
    ///
    /// The id format has already been checked at the DTO level; here only the
    /// existence of the category in the database is verified. If the field is
    /// absent (`None`) there is nothing to check.
    async fn validate_category_id(&self, category_id: &Option<String>) -> AppResult<()> {
        if let Some(id) = category_id
            && !self.categories.exists(id).await?
        {
            return Err(AppError::Validation(format!("category {id} not found")));
        }
        Ok(())
    }

    /// Composes the response by nesting the `category` object (if the
    /// relationship exists).
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
