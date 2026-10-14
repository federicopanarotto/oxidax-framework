//! Workspace task runner and code generator (`cargo xtask ...`).
//!
//! Running `cargo xtask docker-build` etc. is equivalent to `cargo run
//! --package oxidax-cli -- <task>` thanks to the alias in `.cargo/config.toml`.
//! It launches the `docker` commands from the `docker/prod` folder with no
//! extra tooling, and provides the interactive feature generator
//! (`cargo xtask new-feature`).

use std::process::{Command, ExitCode};

mod generate;
mod openapi;

const IMAGE: &str = "oxidax:latest";
const DOCKERFILE: &str = "docker/prod/Dockerfile";
const COMPOSE_FILE: &str = "docker/prod/docker-compose.yml";
const TAR_PATH: &str = "docker/oxidax.tar";

fn main() -> ExitCode {
    let task = std::env::args().nth(1);
    match task.as_deref() {
        None | Some("help" | "--help" | "-h") => {
            print_usage();
            ExitCode::SUCCESS
        }
        Some("new-feature") => generate::new_feature(),
        Some("openapi") => openapi::generate(),
        Some("docker-build") => run("docker", &["build", "-t", IMAGE, "-f", DOCKERFILE, "."]),
        Some("docker-save") => save_tar(),
        Some("docker-package") => {
            let built = run("docker", &["build", "-t", IMAGE, "-f", DOCKERFILE, "."]);
            if built == ExitCode::SUCCESS {
                save_tar()
            } else {
                built
            }
        }
        Some("docker-run") => run(
            "docker",
            &["compose", "-f", COMPOSE_FILE, "up", "-d", "--build"],
        ),
        Some("docker-stop") => run("docker", &["compose", "-f", COMPOSE_FILE, "down"]),
        Some(other) => {
            eprintln!("xtask: unknown task `{other}`");
            print_usage();
            ExitCode::from(2)
        }
    }
}

/// Exports the built image to `docker/oxidax.tar`.
fn save_tar() -> ExitCode {
    let args = ["save", "-o", TAR_PATH, IMAGE];
    let status = Command::new("docker")
        .args(args)
        .status()
        .map_err(|err| {
            eprintln!("xtask: failed to run `docker save`: {err}");
        })
        .ok();

    match status {
        Some(status) if status.success() => {
            eprintln!("xtask: image saved to {TAR_PATH}");
            ExitCode::SUCCESS
        }
        Some(status) => status_code_of(status),
        None => ExitCode::from(2),
    }
}

fn run(program: &str, args: &[&str]) -> ExitCode {
    eprintln!("$ {program} {}", args.join(" "));
    match Command::new(program).args(args).status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(status) => status_code_of(status),
        Err(err) => {
            eprintln!("xtask: failed to run `{program}`: {err}");
            ExitCode::from(2)
        }
    }
}

fn status_code_of(status: std::process::ExitStatus) -> ExitCode {
    match status.code() {
        Some(code) if (0..=255).contains(&code) => ExitCode::from(code as u8),
        _ => ExitCode::from(1),
    }
}

fn print_usage() {
    eprintln!(
        "xtask: workspace task runner (docker/prod) and code generator\n\n\
         Usage: cargo xtask <task>\n\n\
         Tasks:\n\
         \x20 new-feature     interactive generator of a new CRUD feature\n\
         \x20 openapi         generate openapi.json in the repository root\n\
         \x20 docker-build    cargo build --release && docker build docker/prod/Dockerfile\n\
         \x20 docker-save     save the built image to docker/oxidax.tar\n\
         \x20 docker-package  docker-build + docker-save\n\
         \x20 docker-run      docker compose up -d (build + start the prod stack)\n\
         \x20 docker-stop     docker compose down"
    );
}
