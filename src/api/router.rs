use axum::Router;
use axum::routing::get;

use crate::api::categories::category_controller::CategoryController;
use crate::api::categories::category_router;
use crate::api::categories::category_service::CategoryService;
use crate::api::docs;
use crate::api::reports::report_controller::ReportController;
use crate::api::reports::report_router;
use crate::api::reports::report_service::ReportService;
use crate::config::Config;
use crate::db;
use crate::errors::{AppResult, pipeline};

/// Punto di composizione dell'applicazione: il router generale dell'API.
///
/// Configura tutto in sequenza — database → service → controller → rotte — e
/// restituisce l'`Router` pronto da servire. `main` non conosce i controller
/// né i repository: chiama solo questa funzione.
pub async fn build(config: &Config) -> AppResult<Router> {
    let repositories = db::repositories(config).await?;

    let categories = CategoryService::new(repositories.categories);
    let category_controller = CategoryController::new(categories.clone());
    let reports = ReportService::new(repositories.reports, categories);
    let report_controller = ReportController::new(reports);

    Ok(Router::new()
        .route("/health", get(health))
        .nest("/api/reports", report_router::router(report_controller))
        .nest("/api/categories", category_router::router(category_controller))
        .merge(docs::swagger_router())
        .fallback(pipeline::route_not_found))
}

async fn health() -> &'static str {
    "ok"
}
