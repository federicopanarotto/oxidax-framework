use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateReportDto {
    /// Nome del report
    #[schema(example = "Vendite mensili")]
    pub name: String,
    /// Descrizione del report
    #[schema(example = "Report sulle vendite del mese corrente")]
    pub description: String,
    /// Id della categoria associata (opzionale)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub category_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateReportDto {
    /// Nuovo nome del report
    #[schema(example = "Vendite trimestrali")]
    pub name: String,
    /// Nuova descrizione del report
    #[schema(example = "Report sulle vendite del trimestre corrente")]
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
}

impl From<crate::api::reports::report_entity::Report> for ReportResponse {
    fn from(report: crate::api::reports::report_entity::Report) -> Self {
        Self {
            id: report.id,
            name: report.name,
            description: report.description,
            category_id: report.category_id,
        }
    }
}
