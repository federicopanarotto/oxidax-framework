use axum::Router;
use tracing_subscriber::EnvFilter;

use axum_api::api;
use axum_api::api::reports::report_controller::ReportController;
use axum_api::api::reports::report_service::ReportService;
use axum_api::config::Config;
use axum_api::db;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,axum_api=debug")),
        )
        .init();

    let config = Config::from_env();

    let repository = db::report_repository(&config)
        .await
        .expect("impossibile connettersi al database");

    let app: Router = api::router(ReportController::new(ReportService::new(repository)));

    let listener = tokio::net::TcpListener::bind(&config.server_addr)
        .await
        .expect("impossibile ascoltare sull'indirizzo indicato");

    tracing::info!("listening on http://{}", config.server_addr);
    tracing::info!("swagger su http://{}/swagger-ui", config.server_addr);

    axum::serve(listener, app).await.expect("server interrotto");
}
