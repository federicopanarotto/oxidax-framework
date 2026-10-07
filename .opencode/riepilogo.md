# riepilogo

## Obiettivo
- Estendere l'API Axum con la feature `Category` (relazione opzionale `Report.category_id`, CRUD completo con validazione), un router generale per le feature collegato al `main`, un `main` snello (solo config + avvio), export automatico di `docs/openapi.json` allo startup, e infine **dependency injection "in automatico"** (framework) così che il router generale **non importi le classi delle feature** e il codice non sia pieno di boilerplate manuale.
- Vincoli dell'utente: niente classi feature in `src/api/router.rs`; `main` snellissimo; DI fatta da un framework, non a mano.

## Stato lavoro (07/10)
- [x] Feature `src/api/categories/`: entity, dto, repository trait, service, controller+utoipa, router, adapter Surreal su tabella `category`. CRUD REST + validazione name (non vuoto, ≤200).
- [x] Relazione: `Report.category_id: Option<String>` in entity/DTO/response/persistenza; service valida l'esistenza della categoria al create/update → `400 validation_error` `category <id> not found`; `null`/vuoto normalizza a `None`.
- [x] **Response annidata**: `ReportResponse` include `category: Option<CategoryResponse>` con l'oggetto completo (`id`, `name`) oltre a `category_id`. `ReportService` decora le response (singolo via `CategoryService.find`, liste via `CategoryService.by_id()` → index `HashMap`, evitando N+1); `From<(Report, Option<CategoryResponse>)>`. OpenAPI aggiornato (`oneOf` → `$ref CategoryResponse`). Test aggiornati (nested in create/get, `category: null` senza relazione e dopo rimozione).
- [x] Refactor db: connessione condivisa; `db::Repositories { reports, categories }`; `db::repositories(config)`, `db::memory_repositories()`; `db::surreal::{connect → Surreal<Any>, define_table, record_key_to_string}`; `DEFINE TABLE IF NOT EXISTS report|category SCHEMALESS`.
- [x] `Config::memory()`; `main.rs` snello (config → `api::router::build(&config)` → bind/serve, senza import di controller/service).
- [x] `src/api/router.rs` generico: importa solo `docs`, `features`, `config`, `errors`; aggregazione feature sotto `/api` + swagger + fallback `route_not_found`.
- [x] `src/api/docs.rs::export_spec()` → scrive `docs/openapi.json` a ogni `router::build` (best-effort con `tracing::warn!`); paths/schemas/tag categories; `.gitignore` con `/docs/openapi.json`.
- [x] **DI automatica con `more-di 3.2.1`** (niente nightly, niente boilerplate manuale):
  - `more-di = { version = "3", features = ["async"] }` in Cargo.toml (la feature `async` fa sì che `Ref<T>` = `Arc<T>`, altrimenti è `Rc`).
  - `#[injectable]` (attribute macro) su `ReportService`, `CategoryService`, `ReportController`, `CategoryController`; i campi restano `Arc<T>`; i costruttori `new()` manuali eliminati.
  - `src/api/features.rs`: `ServiceCollection` con `singleton_factory::<dyn ReportRepository, _>` / `::<dyn CategoryRepository, _>` (registrano i repo esistenti sotto il trait), poi `.add(CategoryService::singleton())`, `ReportService`, `CategoryController`, `ReportController`; `build_provider()` (sync, ok perché la connessione si apre prima); `get_required`.
  - `provider(config)` esposto per i test; `build_features(config) → Vec<Router<()>>` con prefissi `/reports` e `/categories`.
  - Pitfall risolti: `existing()` di more-di vuole `Box<TSvc>` (incompatibile coi nostri `Arc<dyn>`) → usato `singleton_factory`; serve `use di::Injectable;` per `.singleton()`; serve `use di::injectable;` nei file; i controller hanno bisogno di `#[derive(Clone)]` accanto a `#[injectable]` (state axum `S: Clone`).
- [x] Layout rotte finale: `/api/health`, `/api/reports`+`/api/reports/{id}`, `/api/categories`+`/api/categories/{id}`, `/swagger-ui`, `/api-docs/openapi.json` (health spostata su `/api/health` per l'edit dell'utente).
- [x] Verifiche: `cargo check --all-targets` ✓; `cargo clippy --all-targets -- -D warnings` ✓; `cargo fmt --all` + `--check` ✓; `cargo test` **13/13** (crud 4, categories 4, docs 2 incl. `openapi_spec_is_exported_on_startup`, service 3).
- [x] Smoke test runtime su porta 3001 (container `smoke-axum`, `-e DB_BACKEND=memory`, volume `axum-api_devcontainer_cargo-target`, bin `/cargo-target/debug/axum-api`, rimosso dopo): `GET /api/health` 200; POST `/api/categories` 201; POST `/api/reports` con `category_id` valido 201; con `does-not-exist` → `{"code":"validation_error","error":"400 Bad Request","message":"category does-not-exist not found"}`; `/api/nope` 404; openapi contiene `/api/categories`, `/api/reports`, `CategoryResponse`, tag `categories`.

## Scartato
- `service-rs 0.1` → richiede nightly (`#![feature(unsize)]`), E0554 su rustc stable 1.99.
- Container DI manuale `src/di.rs` (era verde 13/13 ma giudicato "boilerplate" dall'utente) → sostituito da more-di; `src/di.rs` eliminato, `pub mod di;` rimosso da `src/lib.rs`.

## Note per l'utente
- Riavviare l'istanza sulla porta 3000 (gira ancora il vecchio binario pre-DI): `docker compose restart` o `docker exec -d axum-api_devcontainer-app-1 cargo run` nel container dev.
- SurrealDB `root/root` su localhost:8000; in dev usare `cargo run` con DB backend Surreal (default) o `DB_BACKEND=memory`.

## Prossimi passi possibili
- **Framework minimale "definisci le route e basta"** (richiesto, scelte fatte): DSL `routes! {}` per-feature + middleware chain (stile Express), auto-registrazione scouting nel container DI (`#[injectable]`), **discovery zero-touch** delle feature via registry statico (`inventory`) così `features.rs`/`router.rs` non si toccano più. Da implementare.
- Committare la feature (non fatto: serve richiesta esplicita).
- Eventuale endpoint di health che includa lo stato del DB/uscite esplicite.
- Test e2e HTTP su backend Surreal (le verifiche finora sono state solo `memory`).