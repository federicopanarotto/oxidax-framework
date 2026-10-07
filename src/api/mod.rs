pub mod docs;
pub mod reports;

use axum::Router;
use axum::routing::get;

use crate::api::reports::report_controller::ReportController;
use crate::errors::pipeline;

pub fn router(controller: ReportController) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/api/reports", reports::report_router::router(controller))
        .merge(docs::swagger_router())
        .fallback(pipeline::route_not_found)
}

async fn health() -> &'static str {
    "ok"
}
