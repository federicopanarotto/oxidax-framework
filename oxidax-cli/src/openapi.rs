//! `cargo xtask openapi`: generate the OpenAPI specification as JSON.

use std::fs;
use std::path::Path;

use oxidax::app;
use oxidax::config::Config;

pub fn generate() -> std::process::ExitCode {
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    rt.block_on(async {
        let config = Config::from_env();
        match app::build_openapi(&config).await {
            Ok(spec) => {
                let json = serde_json::to_string_pretty(&spec)
                    .expect("failed to serialize OpenAPI spec");
                let path = Path::new("openapi.json");
                if let Err(err) = fs::write(path, json) {
                    eprintln!("xtask: failed to write openapi.json: {err}");
                    std::process::ExitCode::from(2)
                } else {
                    eprintln!("xtask: wrote {path}", path = path.display());
                    std::process::ExitCode::SUCCESS
                }
            }
            Err(err) => {
                eprintln!("xtask: failed to build OpenAPI spec: {err}");
                std::process::ExitCode::from(2)
            }
        }
    })
}