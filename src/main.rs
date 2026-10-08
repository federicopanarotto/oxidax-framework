use tracing_subscriber::EnvFilter;

use axum_api::api;
use axum_api::config::Config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,axum_api=debug")),
        )
        .init();

    let config = Config::from_env();

    let app = api::router::build(&config)
        .await
        .expect("failed to start the application");

    let listener = tokio::net::TcpListener::bind(&config.server_addr)
        .await
        .expect("failed to bind the server to the configured address");

    tracing::info!("listening on http://{}", config.server_addr);
    tracing::info!("swagger UI at http://{}/swagger-ui", config.server_addr);

    axum::serve(listener, app).await.expect("server terminated");
}
