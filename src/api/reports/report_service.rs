use std::sync::Arc;

use uuid::Uuid;

use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::api::reports::report_entity::Report;
use crate::api::reports::report_repository::ReportRepository;
use crate::error::{AppError, AppResult};

const NAME_MAX_LEN: usize = 200;
const DESCRIPTION_MAX_LEN: usize = 2000;

#[derive(Clone)]
pub struct ReportService {
    repository: Arc<dyn ReportRepository>,
}

impl ReportService {
    pub fn new(repository: Arc<dyn ReportRepository>) -> Self {
        Self { repository }
    }

    pub async fn create(&self, dto: CreateReportDto) -> AppResult<ReportResponse> {
        let report = Report {
            id: Uuid::new_v4().to_string(),
            name: validate_name(&dto.name)?,
            description: validate_description(&dto.description)?,
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
        let report = Report {
            id: id.to_string(),
            name: validate_name(&dto.name)?,
            description: validate_description(&dto.description)?,
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
        return Err(AppError::Validation(
            "description must not be empty".into(),
        ));
    }
    if description.chars().count() > DESCRIPTION_MAX_LEN {
        return Err(AppError::Validation(format!(
            "description must be at most {DESCRIPTION_MAX_LEN} characters"
        )));
    }
    Ok(description.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::reports::report_dto::CreateReportDto;
    use crate::error::AppError;

    async fn service() -> ReportService {
        let repository = crate::db::memory_repository()
            .await
            .expect("memory repository");
        ReportService::new(repository)
    }

    #[tokio::test]
    async fn create_rejects_empty_name() {
        let service = service().await;
        let err = service
            .create(CreateReportDto {
                name: "   ".into(),
                description: "desc".into(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, AppError::Validation(_)), "{err:?}");
    }

    #[tokio::test]
    async fn get_missing_report_is_not_found() {
        let service = service().await;
        let err = service.get("does-not-exist").await.unwrap_err();

        assert!(matches!(err, AppError::NotFound(_)), "{err:?}");
    }
}
