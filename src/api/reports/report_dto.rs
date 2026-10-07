use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::categories::category_dto::CategoryResponse;
use crate::lib::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateReportDto {
    /// Nome del report
    #[schema(example = "Vendite mensili", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "Il nome è obbligatorio e non può essere vuoto"
        ),
        length(max = 200, message = "Il nome deve essere di massimo 200 caratteri"),
    )]
    pub name: String,
    /// Descrizione del report
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
        length(max = 2000, message = "La descrizione deve essere di massimo 2000 caratteri"),
    )]
    pub description: String,
    /// Id della categoria associata (opzionale)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub category_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateReportDto {
    /// Nuovo nome del report
    #[schema(example = "Vendite trimestrali", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "Il nome è obbligatorio e non può essere vuoto"
        ),
        length(max = 200, message = "Il nome deve essere di massimo 200 caratteri"),
    )]
    pub name: String,
    /// Nuova descrizione del report
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
        length(max = 2000, message = "La descrizione deve essere di massimo 2000 caratteri"),
    )]
    pub description: String,
    /// Id della categoria associata (opzionale; `null` o assente rimuove la relazione)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub category_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReportResponse {
    /// Identificativo UUID del report
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// Nome del report
    #[schema(example = "Vendite mensili")]
    pub name: String,
    /// Descrizione del report
    #[schema(example = "Report sulle vendite del mese corrente")]
    pub description: String,
    /// Id della categoria associata (opzionale)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub category_id: Option<String>,
    /// Categoria associata (oggetto completo, presente solo se la relazione esiste)
    pub category: Option<CategoryResponse>,
}

impl From<(crate::api::reports::report_entity::Report, Option<CategoryResponse>)> for ReportResponse {
    fn from(
        (report, category): (crate::api::reports::report_entity::Report, Option<CategoryResponse>),
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