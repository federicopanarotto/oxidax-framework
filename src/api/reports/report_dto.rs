use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::categories::category_dto::CategoryResponse;
use crate::api::reports::report_entity::Report;
use crate::lib::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateReportDto {
    /// Report name
    #[schema(example = "Vendite mensili", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "Il nome è obbligatorio e non può essere vuoto"
        ),
        length(max = 200, message = "Il nome deve essere di massimo 200 caratteri")
    )]
    pub name: String,
    /// Report description
    #[schema(
        example = "Report sulle vendite del mese corrente",
        min_length = 1,
        max_length = 2000
    )]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "La descrizione è obbligatoria e non può essere vuota"
        ),
        length(
            max = 2000,
            message = "La descrizione deve essere di massimo 2000 caratteri"
        )
    )]
    pub description: String,
    /// Id of the associated category (optional; `null` or absent removes the relationship)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    #[validate(custom(
        function = "crate::lib::validators::validate_uuid",
        message = "L'id della categoria deve essere un UUID valido"
    ))]
    pub category_id: Option<String>,
}

impl CreateReportDto {
    /// Converts the DTO into the entity ready for the database: generates the
    /// id and normalizes the fields (validation was already done by the
    /// `ValidatedJson` extractor with the `#[validate(...)]` decorators).
    pub fn into_entity(self) -> Report {
        Report {
            id: Uuid::new_v4().to_string(),
            name: self.name.trim().to_string(),
            description: self.description.trim().to_string(),
            category_id: self.category_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateReportDto {
    /// New report name
    #[schema(example = "Vendite trimestrali", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "Il nome è obbligatorio e non può essere vuoto"
        ),
        length(max = 200, message = "Il nome deve essere di massimo 200 caratteri")
    )]
    pub name: String,
    /// New report description
    #[schema(
        example = "Report sulle vendite del trimestre corrente",
        min_length = 1,
        max_length = 2000
    )]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "La descrizione è obbligatoria e non può essere vuota"
        ),
        length(
            max = 2000,
            message = "La descrizione deve essere di massimo 2000 caratteri"
        )
    )]
    pub description: String,
    /// Id of the associated category (optional; `null` or absent removes the relationship)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    #[validate(custom(
        function = "crate::lib::validators::validate_uuid",
        message = "L'id della categoria deve essere un UUID valido"
    ))]
    pub category_id: Option<String>,
}

impl UpdateReportDto {
    /// Converts the DTO into the entity ready for the database: reuses the id
    /// from the path and normalizes the fields (validation was already done by
    /// the extractor).
    pub fn into_entity(self, id: String) -> Report {
        Report {
            id,
            name: self.name.trim().to_string(),
            description: self.description.trim().to_string(),
            category_id: self.category_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReportResponse {
    /// UUID identifier of the report
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// Report name
    #[schema(example = "Vendite mensili")]
    pub name: String,
    /// Report description
    #[schema(example = "Report sulle vendite del mese corrente")]
    pub description: String,
    /// Id of the associated category (optional)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub category_id: Option<String>,
    /// Associated category (full object, present only if the relationship exists)
    pub category: Option<CategoryResponse>,
}

impl
    From<(
        crate::api::reports::report_entity::Report,
        Option<CategoryResponse>,
    )> for ReportResponse
{
    fn from(
        (report, category): (
            crate::api::reports::report_entity::Report,
            Option<CategoryResponse>,
        ),
    ) -> Self {
        Self {
            id: report.id,
            name: report.name,
            description: report.description,
            category_id: report.category_id,
            category,
        }
    }
}
