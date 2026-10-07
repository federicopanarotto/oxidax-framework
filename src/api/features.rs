//! Composizione delle feature tramite dependency injection.
//!
//! Usa `more-di`: qui si registrano soltanto repository (istanze esistenti
//! sotto il proprio trait), service e controller; il container genera i
//! costruttori ([`#[injectable]`](di::injectable)) e risolve le dipendenze
//! automaticamente. Il router generale aggrega i sotto-router prodotti qui
//! senza conoscere le classi concrete.

use std::sync::Arc;

use axum::Router;
use di::{Injectable, ServiceCollection, ServiceProvider, singleton_factory};

use crate::api::categories::category_controller;
use crate::api::categories::category_repository::CategoryRepository;
use crate::api::categories::category_service::CategoryService;
use crate::api::reports::report_controller;
use crate::api::reports::report_repository::ReportRepository;
use crate::api::reports::report_service::ReportService;
use crate::config::Config;
use crate::db;
use crate::errors::{AppError, AppResult};

/// Costruisce il provider DI registrando tutte le dipendenze.
///
/// I repository vengono registrati come singoletto sotto il proprio trait
/// (l'apertura della connessione resta un'operazione `async`, fatta qui prima
/// della registrazione); service e controller tramite `#[injectable]`.
pub async fn provider(config: &Config) -> AppResult<ServiceProvider> {
    let repositories = db::repositories(config).await?;

    let reports: Arc<dyn ReportRepository> = repositories.reports.clone();
    let categories: Arc<dyn CategoryRepository> = repositories.categories.clone();

    let mut services = ServiceCollection::new();

    services
        .add(singleton_factory::<dyn ReportRepository, _>(
            move |_| -> di::Ref<dyn ReportRepository> { reports.clone() },
        ))
        .add(singleton_factory::<dyn CategoryRepository, _>(
            move |_| -> di::Ref<dyn CategoryRepository> { categories.clone() },
        ))
        .add(CategoryService::singleton())
        .add(ReportService::singleton());

    services.build_provider().map_err(di_error)
}

/// Risolve i service dal provider DI e costruisce le tabelle delle rotte,
/// passando a ciascuna il proprio service come stato (niente controller
/// oggetto: gli handler usano direttamente il service).
pub async fn build_features(config: &Config) -> AppResult<Vec<Router<()>>> {
    let provider = provider(config).await?;

    let reports = provider.get_required::<ReportService>();
    let categories = provider.get_required::<CategoryService>();

    Ok(vec![
        report_controller::router(reports),
        category_controller::router(categories),
    ])
}

fn di_error(err: impl std::fmt::Display) -> AppError {
    AppError::Internal(format!("dependency injection: {err}"))
}
