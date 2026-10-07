use std::fs;

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::categories::category_controller;
use crate::api::categories::category_dto::{
    CategoryResponse, CreateCategoryDto, UpdateCategoryDto,
};
use crate::api::reports::report_controller;
use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::errors::{BadRequestResponse, ErrorResponse};
use crate::lib::ValidationErrorResponse;

/// Percorso del file OpenAPI esportato a ogni avvio del server.
const OPENAPI_PATH: &str = "docs/openapi.json";

/// Esporta la specifica OpenAPI su file: la documentazione resta allineata al
/// codice a ogni `cargo run` (e a ogni esecuzione dei test).
///
/// È best-effort: se la scrittura fallisce il server parte comunque.
pub fn export_spec() {
    if let Err(err) = fs::create_dir_all("docs") {
        tracing::warn!(?err, "impossibile creare la cartella della documentazione");
        return;
    }

    let spec = match serde_json::to_string_pretty(&ApiDoc::openapi()) {
        Ok(spec) => spec,
        Err(err) => {
            tracing::warn!(?err, "impossibile serializzare la specifica OpenAPI");
            return;
        }
    };

    match fs::write(OPENAPI_PATH, spec) {
        Ok(()) => tracing::info!(path = OPENAPI_PATH, "documentazione OpenAPI esportata"),
        Err(err) => {
            tracing::warn!(
                ?err,
                path = OPENAPI_PATH,
                "impossibile esportare la specifica OpenAPI"
            )
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Axum Reports API",
        version = "0.1.0",
        description = "CRUD API per la gestione dei report e delle categorie"
    ),
    tags(
        (name = "reports", description = "Gestione dei report"),
        (name = "categories", description = "Gestione delle categorie"),
    ),
    paths(
        report_controller::create_report,
        report_controller::list_reports,
        report_controller::get_report,
        report_controller::update_report,
        report_controller::delete_report,
        category_controller::create_category,
        category_controller::list_categories,
        category_controller::get_category,
        category_controller::update_category,
        category_controller::delete_category,
    ),
    components(schemas(
        ReportResponse,
        CreateReportDto,
        UpdateReportDto,
        CategoryResponse,
        CreateCategoryDto,
        UpdateCategoryDto,
        ErrorResponse,
        ValidationErrorResponse,
        BadRequestResponse,
    ))
)]
pub struct ApiDoc;

pub fn swagger_router() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi())
}
