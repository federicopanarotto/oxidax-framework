use std::sync::Arc;

use di::injectable;

use crate::api::categories::category_dto::CategoryResponse;
use crate::api::categories::category_service::CategoryService;
use crate::api::places::place_dto::PlaceResponse;
use crate::api::places::place_service::PlaceService;
use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::api::reports::report_entity::Report;
use crate::api::reports::report_repository::ReportRepository;
use crate::api::users::user_dto::UserSummaryResponse;
use crate::api::users::user_service::UserService;
use crate::errors::{AppError, AppResult};

/// Business rules for reports.
///
/// The DTOs arrive already converted and validated (id generated, trimmed,
/// UUID format checked by the `ValidatedJson` extractors): here the service
/// only writes ready-made data to the repository. Only the domain validation
/// that requires the database remains (existence of the category); the
/// creator comes from the authenticated caller and is never validated against
/// the database (it was just verified by the guard).
#[injectable]
pub struct ReportService {
    repository: Arc<dyn ReportRepository>,
    categories: Arc<CategoryService>,
    places: Arc<PlaceService>,
    users: Arc<UserService>,
}

impl ReportService {
    pub async fn create(&self, user_id: &str, dto: CreateReportDto) -> AppResult<ReportResponse> {
        let report = dto.into_entity(user_id.to_string());
        self.validate_category_id(&report.category_id).await?;
        self.validate_place_id(&report.place_id).await?;

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
        let places = self.places.by_id().await?;
        let users = self.users.by_id().await?;

        Ok(reports
            .into_iter()
            .map(|report| {
                let category = report
                    .category_id
                    .as_ref()
                    .and_then(|id| categories.get(id).cloned());
                let place = report
                    .place_id
                    .as_ref()
                    .and_then(|id| places.get(id).cloned());
                let created_by = report
                    .user_id
                    .as_ref()
                    .and_then(|id| users.get(id).cloned());
                (report, category, place, created_by).into()
            })
            .collect())
    }

    pub async fn update(&self, id: &str, dto: UpdateReportDto) -> AppResult<ReportResponse> {
        let report = dto.into_entity(id.to_string());
        self.validate_category_id(&report.category_id).await?;
        self.validate_place_id(&report.place_id).await?;

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

    /// Validates the relationship with places (existence of the place).
    async fn validate_place_id(&self, place_id: &Option<String>) -> AppResult<()> {
        if let Some(id) = place_id
            && !self.places.exists(id).await?
        {
            return Err(AppError::Validation(format!("place {id} not found")));
        }
        Ok(())
    }

    /// Composes the response by nesting the `category`, `place` and
    /// `created_by` objects (if the relationships exist).
    async fn decorate(&self, report: Report) -> AppResult<ReportResponse> {
        let category = self.category_of(&report).await?;
        let place = self.place_of(&report).await?;
        let created_by = self.creator_of(&report).await?;
        Ok((report, category, place, created_by).into())
    }

    async fn category_of(&self, report: &Report) -> AppResult<Option<CategoryResponse>> {
        match &report.category_id {
            Some(category_id) => self.categories.find(category_id).await,
            None => Ok(None),
        }
    }

    async fn place_of(&self, report: &Report) -> AppResult<Option<PlaceResponse>> {
        match &report.place_id {
            Some(place_id) => self.places.find(place_id).await,
            None => Ok(None),
        }
    }

    async fn creator_of(&self, report: &Report) -> AppResult<Option<UserSummaryResponse>> {
        match &report.user_id {
            Some(user_id) => self.users.find(user_id).await,
            None => Ok(None),
        }
    }
}

fn not_found(id: &str) -> AppError {
    AppError::NotFound(format!("report {id} not found"))
}
