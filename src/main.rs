//! Binary entry point of the API server.
//!
//! It wires together the three startup phases, in order:
//! 1. configuration (`.env` + environment variables);
//! 2. application assembly (database, dependency injection, routes, OpenAPI);
//! 3. HTTP server (bind the address and serve until shutdown).

use tracing_subscriber::EnvFilter;

use oxidax::app;
use oxidax::config::Config;

/// Async runtime entry point (`#[tokio::main]` expands to a multi-threaded
/// runtime and drives the future returned by this `async fn` to completion).
#[tokio::main]
async fn main() {
    // Load the `.env` file into the process environment if it exists. This is
    // done before the tracing setup so that a `RUST_LOG` defined in the file is
    // picked up below. Existing environment variables win (dotenvy does not
    // override them), so real deployments can inject values directly.
    dotenvy::dotenv().ok();

    // Initialize the global tracing subscriber: it prints the logs to stdout.
    // The filter comes from `RUST_LOG` when set, otherwise it defaults to
    // `info` for every crate and `debug` for this application's crate. This
    // must be called once, before any log is emitted.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,oxidax=debug,oxidax_core=debug")),
        )
        .init();

    // Read the configuration from the environment (and `.env`). No value is
    // hardcoded: if a required variable is missing this panics with a clear
    // message pointing at `.env.example`.
    let config = Config::from_env();

    // Build the whole application: connects to the database backend, assembles
    // the dependency-injection provider from the registered features, collects
    // the routes self-registered by the `#[route]` macro (via `inventory`),
    // merges them with the OpenAPI spec/Scalar API reference and installs the
    // fallback handler. Everything is discovered automatically: this call does
    // not need to change when a new feature is added.
    let app = app::build(&config)
        .await
        .expect("failed to start the application");

    // Bind the TCP listener to the configured address (`SERVER_ADDR`). Binding
    // eagerly here means a port already in use fails fast at startup instead of
    // silently later.
    let listener = tokio::net::TcpListener::bind(&config.server_addr)
        .await
        .expect("failed to bind the server to the configured address");

    // Announce the reachable endpoints: the API base and the Scalar API reference.
    tracing::info!("listening on http://{}", config.server_addr);
    if config.openapi_doc {
        tracing::info!("API reference at http://{}/scalar", config.server_addr);
    }

    // Serve the application on the listener until the process is stopped. This
    // await never returns under normal operation (only on a fatal error).
    axum::serve(listener, app).await.expect("server terminated");
}
