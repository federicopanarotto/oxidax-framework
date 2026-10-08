# riepilogo — axum-api (memoria di progetto)

> Memoria di lavoro consolidata. Fatti, convenzioni e decisioni del progetto:
> serve a ripartire senza rileggere tutta la cronologia. Aggiornarla quando
> cambia qualcosa di strutturale.

## Identità

- **Cos'è**: framework personale (sperimentale) per scrivere API in Rust con lo
  stile Express/Node portato in Rust. Non riscrive nulla da zero: **combina**
  crate mature in una convenzione di progetto.
- **Obiettivo**: aggiungere una feature = creare solo i suoi moduli. I file di
  cablaggio principali (`features.rs`, `router.rs`, `docs.rs`) **non si toccano**.
- **Lingua**: conversazione, `README.md`, `ADD-FEATURE.md` e questo file in
  italiano; **commenti nel codice e messaggi di errore/log in inglese**
  (le stringhe-dato dei test restano pure loro).

## Stack

| Tassello      | Scelta                                                     |
| ------------- | ---------------------------------------------------------- |
| HTTP/rotta    | `axum 0.8` (+ `tower`)                                     |
| OpenAPI/UI    | `utoipa 6` + `utoipa-axum 0.3` + `utoipa-swagger-ui 10`    |
| Rotte         | proc-macro propria `#[route]` (`framework-macros/`) + `inventory` |
| Validazione   | `validator 0.21` (derive)                                  |
| DI            | `more-di 3` (feature `async` → `Ref<T> = Arc<T>`)          |
| DB            | `surrealdb 3.3` (`mem://` nei test)                        |
| Errori        | `thiserror 2` + pipeline unificata                         |
| Log           | `tracing` + `tracing-subscriber`                           |
| Config        | `dotenvy 0.15` + variabili d'ambiente                      |

## Layout

```
src/
├── main.rs            # entry point: dotenv → tracing → Config::from_env → build → bind → serve (commentato fase per fase)
├── lib.rs             # pub mod api/config/db/errors/framework + #[path="lib/mod.rs"] pub mod lib
├── framework.rs       # facade per le feature: use crate::framework::*
├── config.rs          # Config::from_env() (env/.env, nessun default) + Config::memory()
├── lib/               # "core" del framework
│   ├── registry.rs    # registri statici Feature (DI) e Route + register_repository
│   ├── validators.rs  # format_validation_errors, not_blank, validate_uuid
│   └── middlewares/
│       ├── validation_middleware.rs  # ValidatedJson/Query/Params + ValidationErrorResponse
│       └── access_log.rs             # log richieste "POST /api/x 200 3ms"
├── errors/            # AppError, ErrorResponse, BadRequestResponse, pipeline::respond, fallback 404
├── db/                # surreal.rs (connect/define_table/record_key_to_string) + mod.rs (Repositories)
└── api/
    ├── mod.rs         # dichiarazione moduli feature (una riga per feature)
    ├── features.rs    # GENERICO: provider(config) + route_bundles(&provider)
    ├── router.rs      # GENERICO: merge rotte + health + swagger + fallback + access_log
    ├── docs.rs        # GENERICO: build OpenApi da frammenti + swagger_router
    ├── categories/    # feature: entity, dto, repository, surreal_*, service, controller, mod
    ├── places/        # feature: city, address, cap, provenance
    └── reports/       # feature: dipende da categories e places
framework-macros/      # proc-macro #[route]
tests/report/          # integration test HTTP reali (backend mem://)
ADD-FEATURE.md         # guida: come aggiungere una feature (esempio Places)
```

## Architettura

### Routing + OpenAPI zero-touch
- `#[route]` (in `framework-macros`) va messo **sopra** `#[utoipa::path(...)]`.
  Espande: handler invariato + `fn __build_route_<handler>(&ServiceProvider) ->
  (Router, OpenApi)` + `inventory::submit!`.
- Requisiti controllati a compile-time: presenza di `#[utoipa::path]` e di un
  argomento `State<Arc<T>>` (da cui ricava `T`).
- Path **assoluti** (`/api/...`) dichiarati in `#[utoipa::path]`: niente `nest`.
- `router.rs` unisce i `route_bundles`, monta `/api/health`, la Swagger UI e il
  fallback, e applica outermost l'`access_log`.
- Niente export `docs/openapi.json`: spec servita live su `/api-docs/openapi.json`,
  UI su `/swagger-ui`.

### Dependency injection
- Registri statici in `lib/registry.rs`: `Feature { order, register }` (solo DI:
  repo + service) e `Route { build }` (una per handler), entrambi con
  `inventory::collect!`.
- Ogni feature: `inventory::submit! { Feature::new(order, register) }` nel `mod.rs`;
  `register` fa `register_repository(services, repositories.X.clone())` +
  `services.add(XService::singleton())`.
- `#[injectable]` (more-di) genera il costruttore; i campi sono `Arc<...>`.
  Serve `use di::Injectable;` per `.singleton()`.
- **Order attuale**: categories = 0, places = 1, reports = 2 (dipendenze prima).
  L'iniezione è lazy, quindi l'ordine è una cintura di sicurezza.

### Flusso di una richiesta
`access_log` → router axum → extractor (`State`/`ValidatedJson`) → handler →
service → repository → SurrealDB. Errori e validazione risalgono come `AppError`
o `ValidationErrorResponse`.

### Contratto errori
- Dominio/pipeline: `{code, error, message}` con status giusto
  (`not_found` 404, `validation_error` 400, `database_unavailable` 503,
  `internal_error` 500).
- Validazione (extractor): `{error, details}` con 422; body/query malformati → 400
  (`{"error":"Validation error on Body", "details":[...]}`).
- Nei 400 dei write method lo schema è `BadRequestResponse` = `oneOf`
  (`ErrorResponse` | `ValidationErrorResponse`), definito a mano con
  `OneOfBuilder` + `Ref::from_schema_name` (non esiste `one_of` per struct).

### Validazione
- `ValidatedJson`/`ValidatedQuery`/`ValidatedParams` (`T: DeserializeOwned + Send + Validate`):
  deserializzano e validano **prima** dell'handler.
- DTO con `#[derive(Validate)]` e `#[validate(custom = not_blank, length(max=...))]`;
  `validate_uuid` sugli `Option<String>` id.
- `into_entity()` nei DTO: genera l'uuid e fa `.trim()` → il service riceve dati pronti.

### Access log
- `lib/middlewares/access_log.rs` (`axum::middleware::from_fn`): `tracing::info!`
  con formata `METODO path status elapsed_ms` (es. `POST /api/reports 201 3ms`),
  status come `as_u16()`, path+query.

### Configurazione
- `Config::from_env()` carica `.env` (`dotenvy::dotenv().ok()`, le env esistenti
  vincono) e legge **tutte** le variabili **senza default** (helper `required`
  → panico esplicito se manca).
- Variabili: `SERVER_ADDR`, `DB_BACKEND` (`surreal` | `memory`), `SURREAL_URL`,
  `SURREAL_NS`, `SURREAL_DB`, `SURREAL_USER`, `SURREAL_PASS`.
- `.env` è git-ignorato; `.env.example` è il template. `main.rs` carica il dotenv
  **prima** del setup tracing (così `RUST_LOG` dal file vale).
- `Config::memory()` usa il backend memory (test) ma legge il resto dall'ambiente.

## Feature e modello dati

| Feature    | Order | Tabella     | Note                                                        |
| ---------- | ----- | ----------- | ----------------------------------------------------------- |
| categories | 0     | `category`  | CRUD `/api/categories`                                       |
| places     | 1     | `place`     | CRUD `/api/places` (city, address, cap, provenance)         |
| reports    | 2     | `report`    | CRUD `/api/reports`; relazioni con categories e places      |

- `Report` (entity) ha `category_id: Option<String>` e `place_id: Option<String>`.
- In **risposta** NON si espongono gli `*_id`: solo gli oggetti annidati
  `category: Option<CategoryResponse>` e `place: Option<PlaceResponse>`
  (`From<(Report, Option<CategoryResponse>, Option<PlaceResponse>)>`).
- `ReportService` valida l'esistenza di category/place (→ 400 `validation_error`)
  e "decora" il singolo via `find` e le liste via `by_id()` (index `HashMap`,
  no N+1).

## Convenzioni (da rispettare)

1. Commenti e messaggi API/log in **inglese**; docs di progetto in italiano.
2. **1 handler = 1 service**; il controller è solo trasporto (niente logica).
   Logica multi-servizio → service che orchestra altri service (grafo aciclico).
3. Le response non espongono gli id delle relazioni, solo l'oggetto annidato.
4. Nuova feature = suoi moduli + `pub mod X;` in `src/api/mod.rs` +
   repository in `src/db/mod.rs` (`Repositories` + `from_connection` +
   `ensure_schema`) + `Feature::new(order, register)`. Altro è automatico.
5. `#[route]` sempre **sopra** `#[utoipa::path]`; handler con `State<Arc<T>>`.
6. Path assoluti negli attributi utoipa; niente `nest("/api")`.
7. Niente export di `openapi.json`; solo UI live.

## Stato e verifica

- **Test**: 17 integration (HTTP reali, backend `mem://`) + 1 doctest (facade).
- `cargo clippy --all-targets -- -D warnings` e `cargo fmt` puliti.
- `git`: branch `main`, working tree **pulito**. Ultimo commit
  `72bff5e feat: add places feature with CRUD operations and integration with reports`.
- **Server fermo** (porta 3000 libera).

## Comandi (dev container)

Sempre come utente `vscode` (vedi Gotchas):

```bash
docker exec -u vscode -w /workspace axum-api_devcontainer-app-1 cargo check --all-targets
docker exec -u vscode -w /workspace axum-api_devcontainer-app-1 cargo fmt
docker exec -u vscode -w /workspace axum-api_devcontainer-app-1 cargo clippy --all-targets -- -D warnings
docker exec -u vscode -w /workspace axum-api_devcontainer-app-1 cargo test
```

Avvio locale: `cp .env.example .env`; poi `DB_BACKEND=memory cargo run` (senza
Surreal) o `cargo run` (SurrealDB da `.env`). UI: http://localhost:3000/swagger-ui.

## Gotchas

- **Permessi `/cargo-target`**: compilare da terminale host con `-u vscode`
  (come sopra). Lanciare cargo come root crea file di proprietà mista →
  `Permission denied (os error 13)` sui `.fingerprint` in VS Code. Fix:
  `chown -R vscode:vscode /cargo-target`.
- **rust-analyzer su host Windows** mostra falsi errori tipo
  `expected (), found ({unknown}, {unknown}, {unknown})` sulle macro: l'host non
  vede le crate del container. Compila comunque; per RA corretto aprire nel
  Dev Container.
- Le tabelle SurrealDB 3 non nascono da sole: serve `ensure_schema` →
  `DEFINE TABLE IF NOT EXISTS ... SCHEMALESS` (helper `db::define_table`).
- `#[utoipa::path]` accetta solo funzioni libere, non metodi; quindi niente
  classi controller.
- I custom validator di `validator` devono ritornare `Result<(), ValidationError>`.
- I test si appoggiano al `.env` (perché `Config::from_env` richiede le variabili):
  su ambienti senza `.env` vanno passate le env o copiato `.env.example`.

## Decisioni scartate (perché)

- `service-rs` → richiede nightly (`#![feature(unsize)]`): scartato.
- Container DI manuale (`src/di.rs`) → troppo boilerplate: sostituito da `more-di`.
- `Inject<T>` / più service direttamente nell'handler: **no di default** — il
  service fa tutta la logica e il controller ne chiama una funzione; al più
  valvola di sfogo per endpoint di sola aggregazione (YAGNI per ora).

## Backlog

- Middleware "stile Express" (auth, ecc.) globali o per-feature.
- Paginazione/ordinamento e utility condivise tra feature.
- Eventuale rendering della relazione tramite `oneOf`/`$ref` più ricco in OpenAPI.
- Eventuale `db/mod.rs` zero-touch (registro dinamico dei repository) per togliere
  l'ultima modifica manuale condivisa.
- Test e2e su backend Surreal reale (oggi coperto solo `mem://`).
- Committare solo su richiesta esplicita.