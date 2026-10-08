# riepilogo

## Configurazione via `.env` + `dotenvy` (08/10)
- [x] **`dotenvy`** aggiunto a `Cargo.toml`. `Config::from_env()` carica `.env` (`dotenvy::dotenv().ok()`, le env esistenti hanno precedenza) e legge **tutte** le variabili senza default: nessun valore hardcoded (nemmeno di prova). Nuovo helper `required("...")` che va in panico con messaggio esplicito se manca una variabile.
- [x] **`.env`** (git-ignorato) e **`.env.example`** (template, versionato) con `DB_BACKEND`, `SERVER_ADDR`, `SURREAL_URL/NS/DB/USER/PASS`. `.gitignore` aggiornato con `.env`.
- [x] **`main.rs`**: `dotenvy::dotenv().ok()` prima del setup di tracing (così `RUST_LOG` dal file è rispettato) + **doc-comment di modulo e commenti in inglese** su ogni fase (runtime tokio, tracing, config, build app, bind, log, serve).
- [x] **README**: nuova sezione "Configurazione" (solo env, `cp .env.example .env`, variabili richieste, fallimento esplicito se manca) + "Avvio" aggiornato.
- [x] Verifiche: `cargo test` **13/13** + 1 doctest ✓; `cargo clippy --all-targets -- -D warnings` ✓; `cargo fmt` ✓.

## Messaggi in inglese + response report senza `category_id` (08/10)
- [x] **Tutti i messaggi a livello API/log tradotti in inglese** (richiesta esplicita): messaggi di validazione dei DTO (report/category), messaggi degli estrattori `ValidatedJson`/`Query`/`Params` (`Validation error on ...`, `Body JSON is invalid or malformed`, ...), esempio `ValidationErrorResponse`, tutte le `description = "..."` utoipa dei controller, log/`expect` in `main.rs` (`failed to start the application`, `swagger UI at ...`) e `pipeline.rs` (`internal error`, `request rejected`). I commenti restano in inglese; le stringhe-dato nei test/esempi (es. `"Report di ottobre"`) sono invariate. Asserzioni dei test aggiornate ai nuovi messaggi.
- [x] **`ReportResponse` senza `category_id`** (richiesta esplicita: "il campo è di troppo, c'è già l'oggetto nested"): rimosso il campo `category_id` dalla response e dal `From<...>`; la relazione è esposta solo come `category: Option<CategoryResponse>` (con `id`+`name`). Entity/repository/service continuano a gestire internamente `category_id` per validazione e persistenza.
- [x] Verifiche: `cargo test` **13/13** + 1 doctest ✓; `cargo clippy --all-targets -- -D warnings` ✓; `cargo fmt` ✓.

## Framework zero-touch: `#[route]` + registri statici (08/10)
- [x] **Proc-macro crate `framework-macros/`**: attributo `#[route]` su ogni handler → espande l'handler invariato + genera un builder `fn(&ServiceProvider) -> (Router, OpenApi)` (risolve il service dal provider DI via `provider.get_required::<T>()` dove `T` è estratto dal `State<Arc<T>>` della firma) + `inventory::submit!` nel registro `Route`. Verifica di compile-time: presenza `#[utoipa::path(...)]` e argomento `State<Arc<T>>`. Il codice generato usa path fully-qualified (`di::`, `axum::`, `utoipa::`, `utoipa_axum::`, `inventory::`), così gli handler non hanno bisogno di import.
- [x] **`src/lib/registry.rs`**: registri doppi — `Feature { order, register }` (solo DI: repo + service) e `Route { build }` (una rotta per handler); due `inventory::collect!`; `register_repository` resta.
- [x] **`features.rs`**: `provider(config)` (itera `Feature` ordinati → DI) + `route_bundles(&provider) -> Vec<(Router, OpenApi)>` (itera `Route`). **`router.rs`**: niente più `nest("/api")` — ogni `#[route]` monta path assoluti (`/api/reports` ecc.) dichiarati in `#[utoipa::path]`; merge generico + health + swagger + fallback.
- [x] **Controller ripuliti**: `build_router` eliminato (prima l'obbiettivo era "solo la route table"), ogni handler = `#[route]` sopra `#[utoipa::path(...)]`; stato e OpenAPI auto-generati. **`mod.rs` delle feature**: solo wiring DI + `inventory::submit! { Feature::new(order, register) }` (categories 0, reports 1).
- [x] **Facade `src/framework.rs`**: `pub use` di registri, `route`, estrattori di validazione, contratto errori → `use crate::framework::*;`. Usata già in feature e `mod.rs`. Doctest incluso (verifica l'import).
- [x] **Tutti i commenti tradotti in inglese** (richiesta esplicita): `config`, `db`, `errors`, `lib`, `api`, `tests`; le stringhe user-facing (messaggi di validazione, `description = "..."` in utoipa, esempi schema) restano in italiano.
- [x] **`README.md` creato** (in italiano): progetto di prova per un framework personale, filosofia "usare le crate esistenti" (axum, utoipa+utoipa-axum, validator, more-di, inventory, SurrealDB), guida "come nasce una feature", contratto errori, avvio, test, stato/reprossimi passi.
- [x] Verifiche: `cargo check --all-targets` ✓; `cargo test` **13/13** + 1 doctest facade ✓; `cargo clippy --all-targets -- -D warnings` ✓; `cargo fmt --check` ✓.
- Nota: l'errore di rust-analyzer su Windows (`expected (), found ({unknown}, {unknown}, {unknown})` su `routes!`) è un limite dell'host (non vede le crate del container): il codice compila; per RA senza falsi errori aprire la cartella nel Dev Container.
- **Server fermato** (su richiesta: l'utente ha ripreso il controllo della porta 3000; processo terminato con `pkill` nel container).

## Refactor validazione + controller (07/10 sera)
- [x] **Libreria di validazione propria `src/lib/`** (montata con `#[path = "lib/mod.rs"]` in `src/lib.rs` per evitare collisione col crate root): estrattori axum `ValidatedJson`/`ValidatedQuery`/`ValidatedParams` (`T: DeserializeOwned + Send + Validate`) che validano via `validator::Validate` e rispondono 400/422 uniformi `{error, details}`; `ValidationErrorResponse` ToSchema (`#[schema(example=...)]` solo su `error`, non su `details`); helper `format_validation_errors`, `not_blank`, `validate_uuid`.
- [x] **Validazione sui DTO** con `validator 0.20`: `#[derive(Validate)]` + decoratori `custom(not_blank)` e `length(max=...)` su name/description; `validate_uuid` su `category_id` (Option): il formato id viene verificato a livello DTO, non più nel service.
- [x] **Conversioni nei DTO**: `into_entity()` (uuid + trim) in `CreateReportDto`, `UpdateReportDto`, `CreateCategoryDto`, `UpdateCategoryDto` → i service scrivono dati già pronti (solo esistenza categoria, rimane `validate_category_id(&report.category_id) → 400 validation_error`).
- [x] **Niente più classi controller** (richiesta "non fare la classe"): i moduli `report_controller.rs`/`category_controller.rs` sono solo handler liberi `#[utoipa::path]` con `State<Arc<Service>>` (+ `ValidatedJson`) e un builder libero `pub fn router(service: Arc<Service>) -> Router`; `features.rs` risolve i service dal provider DI e costruisce `Vec<Router<()>>`; `ReportController`/`CategoryController` eliminati da DI e da `features.rs`.
- [x] **OpenAPI corretto**: 400 documentato con `BadRequestResponse` (manual `PartialSchema`+`ToSchema` perché `one_of` non esiste su struct; `OneOfBuilder` con `Ref::from_schema_name` di `ErrorResponse`/`ValidationErrorResponse`, `schemas(&mut Vec<(String, RefOr<Schema>)>)`); 422 documentata con `$ref ValidationErrorResponse`; `AppJson`/`src/errors/extract.rs` rimossi.
- [x] **Rimossa l'esportazione di `docs/openapi.json`** (richiesta "voglio solo la swagger UI"): `export_spec()` eliminato; la UI serve la spec a runtime su `/api-docs/openapi.json`. Rimosso anche il test `openapi_spec_is_exported_on_startup`, la cartella `docs/` e la riga in `.gitignore`.
- [x] Pitfall risolti: `#[utoipa::path]` accetta solo funzioni libere (`ItemFn`), non metodi; custom validator `validator` devono restituire `Result<(), ValidationError>` (non `bool`); `OneOf`: from `OneOfBuilder` a `RefOr<Schema>` (non da `OneOf`).
- [x] Verifiche: `cargo check --all-targets` ✓; `cargo clippy --all-targets -- -D warnings` ✓; `cargo fmt --check` ✓; `cargo test` **13/13**; smoke su 3000: health 200, `swagger-ui` 200, POST report senza name → 422 `{"error":"Errore di validazione del Body","details":["name: Il nome è obbligatorio e non può essere vuoto"]}`.

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
- [x] Verifiche: `cargo check --all-targets` ✓; `cargo clippy --all-targets -- -D warnings` ✓; `cargo fmt --all` + `--check` ✓; `cargo test` **13/13** (crud 4, categories 4, docs 1, service 3 + unit).
- [x] Smoke test runtime su porta 3001 (container `smoke-axum`, `-e DB_BACKEND=memory`, volume `axum-api_devcontainer_cargo-target`, bin `/cargo-target/debug/axum-api`, rimosso dopo): `GET /api/health` 200; POST `/api/categories` 201; POST `/api/reports` con `category_id` valido 201; con `does-not-exist` → `{"code":"validation_error","error":"400 Bad Request","message":"category does-not-exist not found"}`; `/api/nope` 404; openapi contiene `/api/categories`, `/api/reports`, `CategoryResponse`, tag `categories`.

## Scartato
- `service-rs 0.1` → richiede nightly (`#![feature(unsize)]`), E0554 su rustc stable 1.99.
- Container DI manuale `src/di.rs` (era verde 13/13 ma giudicato "boilerplate" dall'utente) → sostituito da more-di; `src/di.rs` eliminato, `pub mod di;` rimosso da `src/lib.rs`.

## Note per l'utente
- **Permessi `/cargo-target`**: il container da VS Code compila come utente `vscode`; ogni comando cargo da terminale host va eseguito con `docker exec -u vscode -w /workspace axum-api_devcontainer-app-1 cargo ...`. Se si lancia cargo come root (`docker exec` senza `-u`) coesistono due proprietari sul volume condiviso e VS Code fallisce con `Permission denied (os error 13)` sui `.fingerprint`. Fix applicato: `chown -R vscode:vscode /cargo-target`.
- Server **non più in esecuzione** (stoppato per consegna del controllo); per riavviarlo: `docker exec -w /workspace axum-api_devcontainer-app-1 sh -c "/cargo-target/debug/axum-api > /tmp/app.log 2>&1 &"` oppure `DB_BACKEND=memory cargo run` e aprire http://localhost:3000/swagger-ui (UI Swagger; la spec è servita live su `/api-docs/openapi.json`, non esportata su file).
- SurrealDB `root/root` su localhost:8000; in dev usare `cargo run` con DB backend Surreal (default) o `DB_BACKEND=memory`.

## Prossimi passi possibili
- **Vertical slicing "come aggiungere una feature"** (ora zero-touch: solo `mod` + handler `#[route]` + `Feature::new` nel `mod.rs`; i file di wiring non si toccano). Da documentare anche in README con un esempio completo end-to-end.
- Middleware **chain "stile Express"** (logging, auth) come layer applicabili globalmente o per-feature.
- Paginazione/ordinamento e altre utility condivise da riusare tra le feature.
- Committare (non fatto: serve richiesta esplicita).
- Test e2e HTTP su backend Surreal (finora solo `memory`).