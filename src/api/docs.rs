use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::categories::category_controller;
use crate::api::categories::category_dto::{
    CategoryResponse, CreateCategoryDto, UpdateCategoryDto,
};
use crate::api::reports::report_controller;
use crate::api::reports::report_dto::{CreateReportDto, ReportResponse, UpdateReportDto};
use crate::errors::ErrorResponse;

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
    ))
)]
pub struct ApiDoc;

pub fn swagger_router() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi())
}
