use std::sync::Arc;

use di::injectable;

use crate::api::categories::category_service::CategoryService;
use crate::api::places::place_service::PlaceService;
use crate::api::reports::report_dto::{CreateReportDto, UpdateReportDto};
use crate::api::reports::report_repository::ReportRepository;
use crate::api::reports::report_response::ReportResponse;
use crate::errors::{AppError, AppResult};
use crate::lib::pagination::{Paged, Pagination};

/// Business rules for reports.
///
/// The DTOs arrive already converted and validated (id generated, trimmed,
/// UUID format checked by the `ValidatedJson` extractors): here the service
/// only writes ready-made data to the repository. Only the domain validation
/// that requires the database remains (existence of the category and the
/// place); the creator comes from the authenticated caller and is never
/// validated against the database (it was just verified by the guard).
///
/// Reads never decorate by hand: the repository resolves the relations
/// (`category`, `place`, `created_by`) inside the database with a single
/// query, so there is no N+1 even on a page of reports.
#[injectable]
pub struct ReportService {
    repository: Arc<dyn ReportRepository>,
    categories: Arc<CategoryService>,
    places: Arc<PlaceService>,
}

impl ReportService {
    pub async fn create(&self, user_id: &str, dto: CreateReportDto) -> AppResult<ReportResponse> {
        let report = dto.into_entity(user_id.to_string());
        self.validate_category_id(&report.category_id).await?;
        self.validate_place_id(&report.place_id).await?;

        self.repository.insert(report).await
    }

    pub async fn get(&self, id: &str) -> AppResult<ReportResponse> {
        self.repository
            .find(id)
            .await?
            .ok_or_else(|| AppError::not_found("report", id))
    }

    /// One page of reports with the total count (sent as `X-TOTAL-LIST`). The
    /// related objects (category, place, creator) are resolved by the database
    /// in the same query, not per element and not here.
    pub async fn list_page(&self, pagination: Pagination) -> AppResult<Paged<ReportResponse>> {
        let total = self.repository.count().await?;
        let items = self
            .repository
            .find_page(pagination.offset(), pagination.limit())
            .await?;
        Ok(Paged::new(items, total))
    }

    pub async fn update(&self, id: &str, dto: UpdateReportDto) -> AppResult<ReportResponse> {
        let report = dto.into_entity(id.to_string());
        self.validate_category_id(&report.category_id).await?;
        self.validate_place_id(&report.place_id).await?;

        self.repository
            .replace(id, report)
            .await?
            .ok_or_else(|| AppError::not_found("report", id))
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.delete(id).await? {
            Ok(())
        } else {
            Err(AppError::not_found("report", id))
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

    /// Validates the relationship with places (existence of the place).
    async fn validate_place_id(&self, place_id: &Option<String>) -> AppResult<()> {
        if let Some(id) = place_id
            && !self.places.exists(id).await?
        {
            return Err(AppError::Validation(format!("place {id} not found")));
        }
        Ok(())
    }
}
