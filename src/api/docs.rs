use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::reports::report_controller;
use crate::api::reports::report_dto::{
    CreateReportDto, ErrorResponse, ReportResponse, UpdateReportDto,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Axum Reports API",
        version = "0.1.0",
        description = "CRUD API per la gestione dei report"
    ),
    tags((name = "reports", description = "Gestione dei report")),
    paths(
        report_controller::create_report,
        report_controller::list_reports,
        report_controller::get_report,
        report_controller::update_report,
        report_controller::delete_report,
    ),
    components(schemas(ReportResponse, CreateReportDto, UpdateReportDto, ErrorResponse))
)]
pub struct ApiDoc;

pub fn swagger_router() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi())
}
