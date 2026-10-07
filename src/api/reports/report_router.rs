use axum::Router;
use axum::routing::{get, post};

use crate::api::reports::report_controller::{
    ReportController, create_report, delete_report, get_report, list_reports, update_report,
};

/// Tabella delle rotte dei report: `/api/reports` e `/api/reports/{id}`.
///
/// Gli handler sono documentati in OpenAPI e leggono il [`ReportController`]
/// dallo stato di axum.
pub fn router(controller: ReportController) -> Router {
    Router::new()
        .route("/", post(create_report).get(list_reports))
        .route(
            "/{id}",
            get(get_report).put(update_report).delete(delete_report),
        )
        .with_state(controller)
}
