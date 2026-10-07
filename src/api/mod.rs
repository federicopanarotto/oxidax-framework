pub mod docs;
pub mod reports;

use axum::Router;
use axum::routing::get;

use crate::api::reports::report_service::ReportService;

pub fn router(service: ReportService) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/api/reports", reports::report_controller::router(service))
        .merge(docs::swagger_router())
}

async fn health() -> &'static str {
    "ok"
}
