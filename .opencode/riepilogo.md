# Project memory — oxidax (working memory)

> Consolidated working memory. Facts, conventions and decisions of the project:
> it lets a new session restart without re-reading the whole history. Update it
> whenever something structural changes.

## Identity

- **What it is**: a personal (experimental) framework for writing APIs in Rust,
  bringing the Express/Node development style to Rust. It rewrites nothing from
  scratch: it **combines** mature crates into a project convention.
- **Goal**: adding a feature = creating only its modules. The generic assembly
  lives once in `oxidax-core` and is never touched.
- **Language**: everything (conversation, docs, this file, code comments,
  error/log messages) is in **English**. Test data strings stay as they are.

## Workspace

The repo is a Cargo workspace: reusable crates + the application.

| Crate          | Role                                                                       |
| -------------- | -------------------------------------------------------------------------- |
| `oxidax-core`  | Reusable library: config, errors, auth, middleware, pagination, DI/route registries, DB helpers, generic assembly |
| `oxidax-macro` | The `#[route]` proc-macro (consumes `#[IsGranted]`)                        |
| `oxidax-cli`   | Task runner + code generator (`cargo xtask ...`, binary `oxidax-cli`)      |
| `oxidax` (root)| The application: feature modules + thin composition                        |

`Cargo.toml` (root): `package = "oxidax"`, `members = ["oxidax-core",
"oxidax-macro", "oxidax-cli"]`, `default-members = ["."]`. The app depends on
`oxidax-core` (path). The app's `src/lib.rs` re-exports the core so feature code
keeps using `crate::lib::...`, `crate::framework::...`, `crate::errors::...`,
`crate::config::...` unchanged:
`pub use oxidax_core as lib; pub use oxidax_core::{config, errors, framework};`.
`src/db.rs` re-exports `oxidax_core::db::{query, define_table, record_key_to_string}`.

## Stack

| Building block | Choice                                                     |
| -------------- | ---------------------------------------------------------- |
| HTTP/route     | `axum 0.8` (+ `tower`)                                     |
| OpenAPI/UI     | `utoipa 6` + `utoipa-axum 0.3` + Scalar (CDN)             |
| Routes         | own `#[route]` proc-macro (`oxidax-macro/`) + `inventory`  |
| Validation     | `validator 0.21` (derive)                                  |
| DI             | `more-di 3` (feature `async` → `Ref<T> = Arc<T>`)          |
| DB             | `surrealdb 3.3` (`mem://` in tests) — default backend, configurable |
| Auth           | `jsonwebtoken 11` (HS256) + `argon2 0.5` (Argon2id)         |
| Errors         | `thiserror 2` + unified pipeline                           |
| Log            | `tracing` + `tracing-subscriber`                           |
| Upload         | axum `multipart` (i.e. `multer`) + `tokio` (fs/io-util) + `tokio-util` (ReaderStream) |
| Config         | `dotenvy 0.15` + environment variables                     |

The app dropped `jsonwebtoken`, `argon2` (they now live in `oxidax-core`). The
UI crate `utoipa-swagger-ui` was removed in favour of [Scalar](https://scalar.com)
(a CDN script loaded by an HTML endpoint). The app keeps `utoipa-axum`,
`inventory`, `more-di` because the
macro-generated code refers to them by name (`utoipa_axum`, `inventory`, `di`).

## Layout

```
oxidax-core/src/
├── lib.rs             # modules: app, auth, config, db, docs, errors, framework,
│                      #   middlewares, pagination, registry, validators + pub use validator::Validate
├── app.rs             # assemble(config, AppInfo, register_repositories) → (Router, ServiceProvider)
├── docs.rs            # ApiDoc (security scheme accessToken) + build + Scalar router
├── config.rs          # Config::from_env() (env/.env, no default) + Config::memory()
├── framework.rs       # facade for features: `use crate::framework::*` + `pub use oxidax_macro::route`
├── registry.rs        # static Feature (DI) and Route registries + register_repository
├── validators.rs      # format_validation_errors, not_blank, validate_uuid
├── pagination.rs      # PaginationQuery + Pagination + Paged<T> (X-TOTAL-LIST)
├── upload/            # file upload middleware (multer-style): config, uploader, extractor
├── auth/              # auth core (order 0): jwt, password, user_identity,
│                      #   identity_repository (trait + surreal), extractor, guard, mod
├── middlewares/
│   ├── validation_middleware.rs  # ValidatedJson/Query/Params
│   └── access_log.rs             # "POST /api/x 200 3ms" request log
├── errors/            # AppError, ErrorResponse, BadRequestResponse, pipeline::respond, fallback 404
└── db/                # surreal.rs (connect/define_table/record_key_to_string) + query.rs + mod.rs

oxidax-macro/src/lib.rs  # #[route] proc-macro (consumes #[IsGranted])
oxidax-cli/src/          # main.rs (tasks) + generate.rs (new-feature generator)

src/                     # the application
├── main.rs             # dotenv → tracing → Config::from_env → app::build → bind → serve
├── lib.rs              # pub mod api/app/db + re-exports of oxidax-core (lib/config/errors/framework)
├── app.rs              # build() / build_with_provider() / info() (title "Oxidax API")
├── db.rs               # Repositories + repositories() + memory_repositories()
│                       #   + register_repositories() (central DI) + from_connection()
└── api/
    ├── mod.rs          # feature module declarations (one line per feature)
    ├── categories/     # feature: entity, dto, repository, surreal_*, service, controller, mod
    ├── places/         # feature: city, address, cap, provenance
    ├── reports/        # feature (order 3): depends on categories, places and users
    ├── users/          # feature (order 2): /api/users/me, /api/users (admin)
    ├── auth/           # feature (order 4): register/login/refresh/logout
    └── uploads/        # feature (order 5): /api/upload (asset table + local files)

tests/report/           # end-to-end HTTP integration tests (mem:// backend)
README.md               # English project overview
ADD-FEATURE.md          # English step-by-step guide to add a feature (example: Places)
```

## Architecture

### Zero-touch routing + OpenAPI
- `#[route]` (in `oxidax-macro`) must go **above** `#[utoipa::path(...)]`.
  It expands to: unchanged handler + `fn __build_route_<handler>(&ServiceProvider)
  -> (Router, OpenApi)` + `inventory::submit!`. Emitted paths point to
  `oxidax_core::...`.
- Compile-time requirements: presence of `#[utoipa::path]` and a
  `State<Arc<T>>` argument (from which it derives `T`).
- **Absolute** paths (`/api/...`) declared in `#[utoipa::path]`: no `nest`.
- `oxidax_core::app::assemble` builds the provider, merges the route bundles,
  mounts `/api/health`, the Scalar API reference and the fallback, and applies
  `access_log` as the outermost layer.
- No `docs/openapi.json` export: the spec is served live at
  `/api-docs/openapi.json`, UI at `/scalar`.

### Dependency injection
- Static registries in `oxidax-core/src/registry.rs`: `Feature { order, register:
  fn(&mut ServiceCollection) }` (services only) and `Route { build }` (one per
  handler), both with `inventory::collect!`.
- **Centralized repository registration**: the app registers every concrete
  repository in `src/db.rs::register_repositories(&Repositories, &mut
  ServiceCollection)` (via `register_repository(services, repositories.X.clone())`);
  this closure is passed to `oxidax_core::app::assemble`. Features never know the
  concrete repositories, only the `...Repository` traits.
- Each feature: `inventory::submit! { Feature::new(order, register) }` in `mod.rs`;
  `register` only does `services.add(XService::singleton())`.
- `#[injectable]` (more-di) generates the constructor; fields are `Arc<...>`.
  `use di::Injectable;` is needed for `.singleton()`.
- **Current order**: auth core = 0, categories = 0, places = 1, users = 2,
  reports = 3, auth = 4 (dependencies first); `oxidax_core::auth` core = order 0
  (registers the identity repo + lazy `JwtConfig` from `Arc<Config>`). Injection
  is lazy, so the order is a safety belt.

### Request flow
`access_log` → axum router → (routes with `#[IsGranted]`: bearer guard → extends
`CurrentUser`) → extractor (`State`/`ValidatedJson`) → handler → service →
repository → SurrealDB. Errors and validation bubble up as `AppError` or as the
validation extractor's `ErrorResponse`.

### Authentication (JWT)
- `oxidax-core/src/auth/` is the "core" (order 0): it does **not** depend on the
  users/auth features (no feature↔feature coupling); it registers
  `UserIdentityRepository` and `JwtConfig` (lazy from `Arc<Config>`: `JWT_SECRET`,
  `ACCESS_TOKEN_DURATION`, `REFRESH_TOKEN_DURATION`).
- Tokens: short access + long **single-use** refresh (jti in
  `user_identity.refresh_jti`; rotated on every refresh, `clear_refresh` on logout
  → 401). `verify_*` uses `jsonwebtoken` (default `leeway: 60`, `validate_exp: true`).
- Password: argon2 (`hash`/`verify` fail-closed) in `auth/password.rs`.
- `#[IsGranted]` / `#[IsGranted(["admin"])]` goes **below** `#[route]`: the macro
  adds the guard layer on the single route (`from_fn_with_state`) and shadows
  `__router`; without the attribute the route is public and does not depend on
  `JwtConfig`. Guard → `401`/`403`, then extends `CurrentUser` (`user_id`,
  `identity_id`, `role`), extractor in `framework`.
- Tables: `user` (roles `ROLE_USER`/`ROLE_ADMIN`) + `user_identity`
  (provider/identifier/password_hash + refresh jti/exp). The admin is created by
  hand in the DB (register forces `role = user`).
- Anti-enumeration generic errors: login 401 identical for unknown email or wrong
  password; duplicate register `409 registration could not be completed`; refresh
  `401 invalid or expired refresh token`.
- Register spans 2 tables non-transactionally → manual compensation (removes the
  user if the identity fails, `tracing::warn` if the rollback fails).
- OpenAPI security: scheme `accessToken` (`utoipa::Modify` in `oxidax-core/src/docs.rs`,
  HTTP bearer JWT); each protected handler declares `security(...)` and 401/403.
  (The auth refresh handler references `refreshToken`, but only `accessToken` is
  currently declared in the spec — a known gap.)

### Error contract
- Domain/pipeline: `{code, error, message}` with the right status
  (`not_found` 404, `validation_error` 400, `conflict` 409, `unauthorized` 401,
  `forbidden` 403, `payload_too_large` 413, `unsupported_media_type` 415,
  `database_unavailable` 503, `internal_error` 500).
- Validation extractor: invalid fields → **422** code `validation_error` (details
  joined into `message`); malformed/unparsable body/query/params → **400** code
  `bad_request`. Same `ErrorResponse` shape as everything else.
- In the write-method 400s the OpenAPI schema is `BadRequestResponse`, an **alias**
  of `ErrorResponse` (`Ref::from_schema_name("ErrorResponse")`), doc-only.

### Validation
- `ValidatedJson`/`ValidatedQuery`/`ValidatedParams` (`T: DeserializeOwned + Send
  + Validate`): deserialize and validate **before** the handler.
- DTOs use `#[derive(Validate)]` and `#[validate(custom = not_blank, length(max=...))]`;
  `validate_uuid` on `Option<String>` id fields.
- Query DTOs: `XxxQueryDto` (e.g. `UserListQueryDto`) + `ValidatedQuery<...>`;
  enum-like allowed values are validated with a module-local custom validator
  (`validate_role`, feature-local: uses `user_entity` constants).
- `into_entity()` in the DTOs: generates the uuid and `.trim()`s → the service
  receives ready-made data.

### Access log
- `oxidax-core/src/middlewares/access_log.rs` (`axum::middleware::from_fn`):
  `tracing::info!` formatted as `METHOD path status elapsed_ms` (e.g.
  `POST /api/reports 201 3ms`), status via `as_u16()`, path+query.

### CORS
- `oxidax-core/src/cors.rs`: `layer(patterns: &[Regex]) -> Option<CorsLayer>`
  (built on `tower-http` `CorsLayer` + `AllowOrigin::predicate`, because
  `tower-http` only supports exact origins out of the box). Applied outermost in
  `app::assemble` so preflight `OPTIONS` is answered before routing.
- Config: `Config::cors_allowed_origins: Vec<Regex>`, from the **optional**
  `CORS_ALLOWED_ORIGINS` env var (comma-separated regexes matched against the
  full `Origin` header; a lone `*` → `.*`). Empty/unset = CORS disabled. Invalid
  regex panics at startup with a clear message.
- Allows methods `GET/POST/PUT/PATCH/DELETE/OPTIONS`, headers
  `Authorization`/`Content-Type`/`Accept`, and exposes `X-TOTAL-LIST`.

### File uploads (multer-style)
- `oxidax-core/src/upload/`: `config.rs` (`UploadConfig`, `FileNameStrategy`),
  `uploader.rs` (`Uploader` + `UploadedFile`, streaming write, limits/extension
  filter, `remove`), `extractor.rs` (`Upload`, must be the **last** handler arg,
  consumes the body), `mod.rs` (`stream_file` for downloads + filename
  sanitization). Built on axum `multipart` (i.e. `multer`).
- `Uploader` is built in `app::assemble` from `UploadConfig::from(config)`,
  **registered in DI** (services resolve it) and installed on the router as an
  `axum::Extension` layer; the body limit uses `DefaultBodyLimit::max(
  UPLOAD_MAX_BODY_SIZE)`. The assets dir is created at startup.
- Files are streamed to disk (never buffered) under a random UUID name + the
  original extension; `Upload::single/array/files` + `field` mirror
  `upload.single/array/any` + `req.file(s)`/`req.body`.
- App feature `uploads` (order 5): entity `Asset`, trait `AssetRepository` +
  `SurrealAssetRepository` (table `asset`), `UploadService` (compensates the file
  if the DB insert fails; delete = owner or admin), routes `GET/POST /api/upload`
  and `GET/DELETE /api/upload/{id}` (writes require `#[IsGranted]`).

### Configuration
- `Config::from_env()` loads `.env` (`dotenvy::dotenv().ok()`, existing env vars
  win) and reads **all** variables with **no default** (helper `required` →
  explicit panic if missing).
- Variables: `SERVER_ADDR`, `DB_BACKEND` (`surreal` | `memory`), `SURREAL_URL`,
  `SURREAL_NS`, `SURREAL_DB`, `SURREAL_USER`, `SURREAL_PASS`, `JWT_SECRET`,
  `ACCESS_TOKEN_DURATION` (900), `REFRESH_TOKEN_DURATION` (604800), `ASSETS_DIR`
  (required: root directory of the uploaded files). Optional:
  `CORS_ALLOWED_ORIGINS` (comma-separated regex allow-list; empty = CORS off),
  `UPLOAD_MAX_FILE_SIZE` (10 MiB), `UPLOAD_MAX_FILES` (10),
  `UPLOAD_MAX_BODY_SIZE` (default = file-size × max-files), and
  `UPLOAD_ALLOWED_EXTENSIONS` (comma-separated, no dot; empty = any).
- `.env` is git-ignored; `.env.example` is the template. `main.rs` loads dotenv
  **before** setting up tracing (so `RUST_LOG` from the file applies).
- `Config::memory()` uses the memory backend (tests) but still reads the rest from
  the environment.

## Features and data model

| Feature     | Order | Table           | Notes                                                       |
| ----------- | ----- | --------------- | ----------------------------------------------------------- |
| (auth core) | 0     | `user_identity` | `oxidax_core::auth`: JWT, password, guard, `CurrentUser`     |
| categories  | 0     | `category`      | CRUD `/api/categories`                                       |
| places      | 1     | `place`         | CRUD `/api/places` (city, address, cap, provenance)          |
| users       | 2     | `user`          | `GET /api/users/me`; `GET /api/users?role=` (admin only)     |
| reports     | 3     | `report`        | CRUD `/api/reports`; creator + relations to categories/places |
| auth        | 4     | —               | register/login/refresh/logout (`/api/auth/*`)                |
| uploads     | 5     | `asset`         | upload/download/delete files (`/api/upload`)                 |

- `Report` (entity) has `category_id: Option<String>`, `place_id: Option<String>`
  and `user_id: Option<String>` (creator; `None` only on legacy rows).
- The creator is **not** in the request DTOs: it comes from the token
  (`#[IsGranted]` → `CurrentUser.user_id` → `service.create(&user_id, dto)`).
- Responses do **not** expose the `*_id` fields: only the nested objects
  `category`, `place` and `created_by` (`Option<UserSummaryResponse>` — id,
  firstname, lastname, role: **no email field**, not even `null`). Email stays
  only for `/me` and the admin list (`UserResponse` with `email`).
- `GET /api/users` accepts the **`role`** query param (`admin`/`user`): DTO
  `UserListQueryDto` validated by `ValidatedQuery` (value outside the list → **422**
  with details on the `role` field); the filter reaches the repository
  (`find_by_role`, `WHERE role = $value`) only for already-validated values. The
  admin guard runs **before** the extractor → a non-admin always sees 403.
- The report write methods (`POST`, `PUT`, `DELETE`) are protected
  (`#[IsGranted]` + 401/security in OpenAPI); reads stay public and still report
  `created_by`.
- `ReportService` validates the existence of category/place (→ 400
  `validation_error`), does not validate the creator (just verified by the guard;
  if the user disappears the decoration degrades to `null`) and "decorates" the
  single item via `find` and lists via `by_id()` (`HashMap` index, no N+1);
  it injects `UserService` (service → service).
- `replace` of reports is a **merge** patch without `user_id`: an update can never
  rewrite the creator.
- **Pagination (reusable)**: lists accept optional `?page=` (default 1) and
  `?limit=` (default 20, max 100) via `ValidatedQuery<PaginationQuery>`; the body
  stays the JSON array and the total travels in the `X-TOTAL-LIST` header
  (`Paged<T>`, `oxidax-core/src/pagination.rs` + `db/query.rs`). Out of range →
  422; non-numeric → 400.

## Conventions (to respect)

1. Comments, API/log messages and docs in **English**.
2. **1 handler = 1 service**; the controller is transport only (no logic).
   Multi-service logic → a service orchestrating other services (acyclic graph).
3. Responses do not expose relation ids, only the nested object.
4. New feature = its modules + `pub mod X;` in `src/api/mod.rs` + repository in
   `src/db.rs` (`Repositories` + `register_repositories` + `from_connection` +
   `ensure_schema`) + `Feature::new(order, register)` (service only). Everything
   else is automatic. (`cargo xtask new-feature` does all of this.)
5. `#[route]` always **above** `#[utoipa::path]`; handlers with `State<Arc<T>>`.
6. Absolute paths in utoipa attributes; no `nest("/api")`.
7. No `openapi.json` export; live UI only.
8. Protected routes: `#[IsGranted]` **below** `#[route]` and below
   `#[utoipa::path]`; protected handlers use the `CurrentUser` extractor; in
   `utoipa::path` declare `security(...)` + documented 401/403.
9. Auth: email = identifier; generic login/register responses (no enumeration);
   single-use refresh.
10. Query params: `XxxQueryDto` + `ValidatedQuery<...>`; allowed values validated
    with a custom validator (module-local when domain-bound).
11. Uploads: the `Upload` extractor must be the **last** handler argument (it
    consumes the body); the core only writes files, the feature persists the
    metadata (`Asset`) via its service/repository.

## Status and verification

- **Tests**: 52 integration tests over real HTTP with the `mem://` backend —
  `auth.rs` 13, `upload.rs` 7, `categories.rs` 5, `crud.rs` 5, `places.rs` 4,
  `service.rs` 4, `cors.rs` 4, `creator.rs` 3, `users.rs` 3, `pagination.rs` 2,
  `color.rs` 1 (generated feature kept), `docs.rs` 1.
  `cargo test --workspace` → 52 passed.
- `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets
  -- -D warnings` and `cargo fmt --all` all clean.
- The `new-feature` generator was verified end-to-end: it scaffolds a compiling
  feature whose generated CRUD test passes; the test feature was then removed and
  the shared files restored.
- `git`: branch `main`, working tree **not clean** (the whole workspace split
  `oxidax-core`/`oxidax-macro`/`oxidax-cli` + app rename to `oxidax` + docs
  translation, not yet committed). Last commit:
  `53abf46 refactor: unify error response structure and remove ValidationErrorResponse`.
- **Port 3000 busy**: it is the user's debug server inside the container (Docker
  publish) — do not touch; temporary servers use other ports.
- **Do not commit without an explicit user request.**

## Commands (dev container)

Always as user `vscode` (see Gotchas). Replace `<container>` with your dev
container name (the `docker-compose` service is `app`):

```bash
docker exec -u vscode -w /workspace <container> cargo check --workspace --all-targets
docker exec -u vscode -w /workspace <container> cargo fmt --all
docker exec -u vscode -w /workspace <container> cargo clippy --workspace --all-targets -- -D warnings
docker exec -u vscode -w /workspace <container> cargo test --workspace
```

When running inside the Dev Container directly, just use `cargo ...`.

Aliases (`.cargo/config.toml`): `cargo dev`, `cargo dev-check`, `cargo xtask
<task>`, `cargo new-feature`, `cargo docker-{build,save,package,run,stop}`.

Run locally: `cp .env.example .env`; then `DB_BACKEND=memory cargo run` (no Surreal)
or `cargo run` (SurrealDB from `.env`). UI: http://localhost:3000/scalar.

## Gotchas

- **`/cargo-target` permissions**: compile from the host terminal with `-u vscode`
  (as above). Running cargo as root creates mixed-ownership files →
  `Permission denied (os error 13)` on `.fingerprint` files in VS Code. Fix:
  `chown -R vscode:vscode /cargo-target`.
- **rust-analyzer on the Windows host** shows false errors like
  `expected (), found ({unknown}, {unknown}, {unknown})` on macros: the host does
  not see the container crates. It still compiles; for correct RA, open in the
  Dev Container.
- SurrealDB 3 tables are not created automatically: `ensure_schema` is required →
  `DEFINE TABLE IF NOT EXISTS ... SCHEMALESS` (helper `db::define_table`, in
  `oxidax-core`).
- **Only `ensure_schema` owns the real DB schema**: `DEFINE FIELD`s added by hand
  (console/Studio) conflict with the code and break writes with
  `503 Couldn't coerce ... Expected uuid/record<> but found '...'` (the `mem://`
  tests do not see it because the DB is clean). Symptoms: place/category/report
  cannot be created, auth/user fine. Fix: `REMOVE FIELD ...` (data is kept).
  Verified 2026-10-08: 11 manual fields removed from `test/reports`, E2E 15/15 on
  the real DB.
- **Studio can create `SCHEMAFULL` tables** (table designer): with no defined
  fields, writes are rejected with `503 Found field 'x', but no such field exists
  for table 'y'` (register or login broken). `DEFINE TABLE IF NOT EXISTS ...
  SCHEMALESS` does **not** fix an existing table: use `ALTER TABLE <t> SCHEMALESS`
  (data is kept). Verified 2026-10-08 on `test/reports` (`user_identity`).
- `#[utoipa::path]` only accepts free functions, not methods; hence no controller
  classes.
- `validator` custom validators must return `Result<(), ValidationError>`.
- **SurrealDB 3** renamed `type::thing` → **`type::record`**: in manual SQL
  (console/HTTP) `type::thing(...)` gives the parse error
  `Invalid function/constant path, did you maybe mean type::record`.
- The tests rely on `.env` (because `Config::from_env` requires the variables): on
  environments without `.env`, pass the env vars or copy `.env.example`.

## Rejected decisions (and why)

- `service-rs` → requires nightly (`#![feature(unsize)]`): rejected.
- Manual DI container (`src/di.rs`) → too much boilerplate: replaced by `more-di`.
- `Inject<T>` / multiple services directly in the handler: **no by default** — the
  service holds all the logic and the controller calls one function; at most an
  escape hatch for aggregation-only endpoints (YAGNI for now).

## Backlog

- `?sort=` (ordering) with a per-entity field whitelist — **not essential**.
- Richer relation rendering via `oneOf`/`$ref` in OpenAPI.
- Zero-touch `src/db.rs` (dynamic repository registry) to remove the last manual
  shared edit.
- E2E tests on a real SurrealDB backend (today only `mem://` is covered).
- More "Express style" middleware beyond the access log, CORS and the per-route
  guard (e.g. rate limiting, security headers, request id).
- Commit only on explicit request.
