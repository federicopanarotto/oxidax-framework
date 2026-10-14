<p align="center">
  <img src="LOGO.png" alt="Oxidax logo" width="220">
</p>

<h1 align="center">Oxidax Framework</h1>

<p align="center">
  <em>An experimental framework for building REST APIs in Rust with an Express-like developer experience.</em>
</p>

<p align="center">
  <img alt="status: experimental" src="https://img.shields.io/badge/status-experimental-orange">
  <img alt="rust: edition 2024" src="https://img.shields.io/badge/rust-edition%202024-blue">
  <img alt="tests: 52 passing" src="https://img.shields.io/badge/tests-52%20passing-brightgreen">
</p>

---

> **Experimental project.** Oxidax is a personal, learning-oriented framework. It
> is **not production-ready** and its public API (the macros, the registries, the
> crate layout) is still evolving and may change without notice. It exists to
> explore how far a thin convention layer over mature Rust crates can go, while
> keeping the ergonomics of an Express/Node.js project.

## About

Oxidax brings the development style of Express and Node.js to Rust **without
reinventing the ecosystem**. Rather than implementing an HTTP stack, an ORM or a
validation engine from scratch, it selects mature, well-maintained crates and
*combines* them into a single, opinionated project convention:

- **Routing and documentation are declared on the handler** through attributes
  (`#[route]` + `#[utoipa::path]`), not in a separate router file.
- **Validation is declarative** (`#[validate]` decorators on DTOs, applied by
  `ValidatedJson` / `ValidatedQuery` / `ValidatedParams`).
- **Features are self-contained modules**: the generic assembly lives once in
  `oxidax-core` and never has to be modified when a new domain is added.

The project is split into a reusable library (`oxidax-core`), a procedural macro
crate (`oxidax-macro`), a CLI/code generator (`oxidax-cli`) and the example
application (`oxidax`). It is intended for learning and experimentation.

> **Warning:** the public macro API and the internal registries are still
> evolving; expect breaking changes between releases.

## Design principles

1. **Compose, don't reinvent.** Every cross-cutting concern (routing, OpenAPI,
   validation, dependency injection, configuration, errors) is delegated to a
   mature crate and exposed through a single project convention.
2. **Zero-touch wiring.** Adding an API domain (a *feature*) means only creating
   its modules; routes and the OpenAPI document are collected automatically at
   compile time through static registries.
3. **Decorator style.** Routes, documentation and validation are declared on top
   of functions and DTOs, mirroring the decorator-based development familiar from
   Express and NestJS.
4. **One consistent contract.** Every response, success or error, follows a small,
   predictable shape across the whole API.

## Architecture

### Workspace

The repository is a Cargo workspace made of reusable crates plus the application:

| Crate           | Role                                                                                                                        |
| --------------- | --------------------------------------------------------------------------------------------------------------------------- |
| `oxidax-core`   | Reusable library: configuration, errors, authentication, middleware, pagination, file upload, DI/route registries, DB helpers and generic app assembly |
| `oxidax-macro`  | The `#[route]` procedural macro (and the `#[IsGranted]` guard it consumes)                                                   |
| `oxidax-cli`    | Command-line tools: Docker helpers and the `new-feature` code generator (`cargo xtask ...`)                                  |
| `oxidax` (root) | The example application: feature modules and a thin composition layer (database backend, OpenAPI metadata)                   |

Everything that is identical across projects lives in `oxidax-core`, keeping the
application's `src/` limited to application code.

### Tech stack

| Building block      | Crate(s)                                                             | Rationale                                                    |
| ------------------- | -------------------------------------------------------------------- | ------------------------------------------------------------ |
| HTTP / routing      | [axum](https://github.com/tokio-rs/axum)                             | performant, macro-free, part of the tower ecosystem          |
| OpenAPI / UI        | [utoipa](https://github.com/juhaku/utoipa) + `utoipa-axum` + [Scalar](https://scalar.com) | `#[utoipa::path]` decorators and a ready-made API reference UI |
| Route discovery     | `oxidax-macro` (`#[route]`) + [inventory](https://github.com/dtolnay/inventory) | compile-time static route registration            |
| Validation          | [validator](https://github.com/Keats/validator)                      | mature validation decorators                                 |
| Dependency injection| [more-di](https://github.com/AndyShi-more/MoreDi)                    | lazy injection with a `ServiceProvider`                      |
| Database            | [SurrealDB](https://surrealdb.com) + `surrealdb`                     | performant, schema-less, `mem://` backend for tests          |
| Authentication      | `jsonwebtoken` + `argon2`                                            | HS256 JWT (access/refresh) + Argon2id password hashing       |
| File upload         | axum `multipart` + `tokio` + `tokio-util`                            | streaming, multer-style multipart handling                   |
| Errors              | `thiserror` + a unified pipeline                                     | a single error contract across the entire API                |
| Configuration       | `dotenvy` + environment variables                                    | no hardcoded values                                          |

### Project structure

```
oxidax-core/                  # reusable framework library
├── src/
│   ├── app.rs                # generic assembly: DI provider + router + Scalar API reference
│   ├── docs.rs               # OpenAPI base document (security schemes)
│   ├── config.rs             # configuration from the environment
│   ├── errors/               # error contract + HTTP pipeline
│   ├── auth/                 # JWT, password hashing, identity, guard/extractor
│   ├── upload/               # multer-style file upload middleware
│   ├── middlewares/          # extractable middleware, Express style
│   ├── pagination.rs         # PaginationQuery + Pagination + Paged<T> (X-TOTAL-LIST)
│   ├── validators.rs         # shared validators
│   ├── registry.rs           # static Feature (DI) and Route registries
│   ├── db/                   # SurrealDB connection + reusable query helpers
│   └── framework.rs          # facade: single import (`oxidax_core::framework::*`)

oxidax-macro/src/lib.rs      # the #[route] proc-macro
oxidax-cli/src/              # `cargo xtask`: Docker helpers + `new-feature`

oxidax/ (root)
├── src/
│   ├── api/
│   │   ├── categories/       # generic CRUD feature
│   │   ├── places/           # CRUD with value objects (city, address, cap, ...)
│   │   ├── reports/          # feature composing categories, places and users
│   │   ├── users/            # profile + admin user listing
│   │   ├── auth/             # register / login / refresh / logout
│   │   └── uploads/          # file upload / download / delete
│   ├── db.rs                 # concrete repositories + central DI registration
│   ├── app.rs                # application assembly entry point
│   ├── lib.rs                # re-exports of the core (keeps `crate::lib::...`)
│   └── main.rs               # binary entry point
├── tests/report/             # end-to-end integration tests (real HTTP)
└── docker/prod/              # production image + compose stack
```

### How a feature is born

A feature is self-contained; taking `categories` as an example:

1. **`mod.rs`** — the only manual wiring: it registers the service. Concrete
   repositories are registered centrally (see `src/db.rs`), so the feature stays
   database-agnostic.
   ```rust
   fn register(services: &mut ServiceCollection) {
       services.add(CategoryService::singleton());
   }

   inventory::submit! { Feature::new(0, register) }
   ```
   The optional `order` guarantees the registration order when a feature depends
   on another (e.g. `reports` → `categories`).

2. **`category_controller.rs`** — handlers are plain functions with attributes;
   there is no controller object:
   ```rust
   #[route]                                    // auto-registers route + OpenAPI
   #[utoipa::path(post, path = "/api/categories",
       request_body = CreateCategoryDto,
       responses((status = 201, body = CategoryResponse), ...))]
   pub async fn create_category(
       State(service): State<Arc<CategoryService>>,
       ValidatedJson(dto): ValidatedJson<CreateCategoryDto>,
   ) -> AppResult<(StatusCode, Json<CategoryResponse>)> {
       let created = service.create(dto).await?;
       Ok((StatusCode::CREATED, Json(created)))
   }
   ```

3. **`category_dto.rs`** — DTOs carry validation decorators:
   ```rust
   #[derive(Debug, Serialize, Deserialize, ToSchema)]
   #[serde(deny_unknown_fields)]
   pub struct CreateCategoryDto {
       #[validate(length(min = 3, max = 64, message = "The name is required and must be at least 3 characters"))]
       #[schema(max_length = 64)]
       pub name: String,
   }
   ```

No further change is required: the assembly in `oxidax_core::app` iterates the
static registries and automatically collects the routes and the OpenAPI document.

### Code generator

`cargo xtask new-feature` scaffolds a complete, compiling CRUD feature
interactively: it asks for the feature name and its fields (name and type from a
menu) and writes the entity, DTOs, repository trait, SurrealDB repository,
service, controller, the DI registration, the `src/db.rs` wiring and a full CRUD
test.

```bash
cargo new-feature      # alias for: cargo xtask new-feature
```

## Features

### Routing and OpenAPI

- `#[route]` is placed **above** `#[utoipa::path]` and expands into the unchanged
  handler plus a compile-time registration (`inventory::submit!`). The generated
  OpenAPI paths point at `oxidax-core`.
- `oxidax_core::app::assemble` builds the dependency-injection provider, merges
  the collected route bundles, mounts `/api/health`, the Scalar API reference and
  the 404 fallback, and applies the access log as the outermost layer.
- There is no exported `openapi.json`: the specification is served live at
  `/api-docs/openapi.json` and the Scalar UI at `/scalar`.
- **Automatic error examples**: `oxidax_core::errors::ErrorExamples` walks the merged
  OpenAPI tree after collecting all feature paths and injects, for each documented
  error status (`400,401,403,404,409,413,415,422,500,503`), the shared
  `ErrorResponse` schema plus a correct example. The example message is taken
  directly from the response's `description`, so controllers only declare
  `(status = ..., description = "...")` (no `body = ...`). This keeps error docs
  centralized, correct and easy to maintain.

### Validation and response contract

- `ValidatedJson`, `ValidatedQuery` and `ValidatedParams` deserialize **and**
  validate the input before the handler runs.
- **Success responses** are the JSON body of the response DTO (or `204 No
  Content`).
- **Error responses** always have the shape `{code, error, message}` with a
  meaningful HTTP status:
  - `validation_error` (`422`) — a field failed validation; the problems are
    listed in `message`.
  - `bad_request` (`400`) — a malformed or unparsable body/query/params.
  - `not_found` (`404`), `conflict` (`409`), `unauthorized` (`401`),
    `forbidden` (`403`), `payload_too_large` (`413`),
    `unsupported_media_type` (`415`), `database_unavailable` (`503`),
    `internal_error` (`500`).

  The same body is used by domain errors, the error pipeline and DTO validation.

### Pagination

Every list endpoint (`/api/categories`, `/api/places`, `/api/reports`,
`/api/users`, `/api/upload`) is paginated with two optional, validated query
parameters:

| Param   | Default | Constraints           |
| ------- | ------- | --------------------- |
| `page`  | `1`     | `>= 1`                |
| `limit` | `20`    | between `1` and `100` |

The response body stays a plain JSON array (no envelope); the total number of
items travels in the **`X-TOTAL-LIST`** header so the client can compute the page
count:

```http
GET /api/categories?page=2&limit=5
X-TOTAL-LIST: 42
```

```json
[ { "id": "...", "name": "Sale" }, ... ]
```

Out-of-range values return `422 {code: validation_error}`; a non-numeric value
gives `400 {code: bad_request}`. Everything is reusable via
`PaginationQuery`, `Pagination` and `Paged<T>`
(`oxidax-core/src/pagination.rs`) and the query helpers in
`oxidax-core/src/db/query.rs`.

### Authentication and authorization

JWTs are signed with HS256 (`JWT_SECRET`) and come in two flavours: a short-lived
access token and a long, **single-use** refresh token that is rotated on every
refresh and revoked on logout.

| Endpoint                  | Effect                                                                 |
| ------------------------- | ---------------------------------------------------------------------- |
| `POST /api/auth/register` | `201 {message}` — the email is the username (`role = user`)            |
| `POST /api/auth/login`    | `200 {access_token, refresh_token, token_type, expires_in}`            |
| `POST /api/auth/refresh`  | a new token pair; the previous refresh token stops working             |
| `POST /api/auth/logout`   | `204` — clears the refresh token (`jti` revocation)                    |
| `GET /api/users/me`       | the caller's profile, including the email                              |
| `GET /api/users`          | admin-only user list; optional `?role=admin\|user` filter, paginated   |

Authentication is **per route**. A handler adds `#[IsGranted]` (below
`#[utoipa::path]`), which generates the guard layer (`401` without a valid token,
`403` for the wrong role) and exposes the `CurrentUser` extractor
(`user_id`, `identity_id`, `role`):

```rust
#[route]
#[utoipa::path(get, path = "/api/users/me", ...)]
#[IsGranted]
pub async fn me(State(service): State<Arc<UserService>>, user: CurrentUser) -> ... {
    // ...
}
```

To reduce account enumeration, errors are intentionally generic: an unknown email
and a wrong password both return `401 invalid credentials`, and a duplicate
registration returns a generic `409`. The first administrator is created directly
in the database.

### File uploads

`oxidax-core` ships a reusable, multer-style upload middleware
(`oxidax-core/src/upload`, built on axum's `multipart`):

| Node.js / multer                       | Oxidax                                       |
| -------------------------------------- | -------------------------------------------- |
| `multer({ dest, limits, fileFilter })` | `UploadConfig`, installed by `app::assemble` |
| `upload.single('file')`                | `Upload::single("file")`                     |
| `upload.array('photos')`               | `Upload::array("photos")`                    |
| `req.file` / `req.files`               | `UploadedFile`                               |
| `req.body`                             | `Upload::field("name")`                      |

Files are streamed to disk (never buffered in memory) into `ASSETS_DIR` under a
random UUID name plus the original extension. The `Upload` extractor consumes the
body, so it must be the **last** handler argument. Per-file size, file count,
total body size and the extension allow-list are enforced while streaming;
violations return `413` or `415`.

The example application keeps a metadata table (`asset`) with the public id,
owner, original name, content type, size and creation time. The public URL
(`/api/upload/{id}`) is decoupled from storage, so files can later be moved to
object storage without changing the API.

| Endpoint                    | Effect                                                         |
| --------------------------- | -------------------------------------------------------------- |
| `GET /api/upload`           | paginated list of assets (`X-TOTAL-LIST`)                      |
| `POST /api/upload`          | multipart `file` field → `201` with the asset metadata         |
| `GET /api/upload/{id}`      | streams the stored file with its content type and name         |
| `DELETE /api/upload/{id}`   | owner (or admin) deletes the row and the file → `204`          |

## Operations

### Configuration

Configuration comes **only** from environment variables; no value is hardcoded.
Locally the values are read from the git-ignored `.env` file via `dotenvy`, and
variables already present in the environment take precedence over the file.

```bash
cp .env.example .env   # template with all required variables
```

**Required variables:** `SERVER_ADDR`, `DB_BACKEND` (`surreal` or `memory`),
`SURREAL_URL`, `SURREAL_NS`, `SURREAL_DB`, `SURREAL_USER`, `SURREAL_PASS`,
`JWT_SECRET`, `ACCESS_TOKEN_DURATION` (seconds), `REFRESH_TOKEN_DURATION`
(seconds) and `ASSETS_DIR` (root directory for uploaded files, created at
startup). If any is missing, startup fails with an explicit message.

**Optional upload variables:** `UPLOAD_MAX_FILE_SIZE` (bytes, default 10 MiB),
`UPLOAD_MAX_FILES` (default 10), `UPLOAD_MAX_BODY_SIZE` (bytes, default
`UPLOAD_MAX_FILE_SIZE × UPLOAD_MAX_FILES`) and `UPLOAD_ALLOWED_EXTENSIONS`
(comma-separated allow-list without the dot; empty means any extension).

#### CORS

CORS is optional and configured with a single variable, `CORS_ALLOWED_ORIGINS`: a
comma-separated list of **regex** patterns matched against the request's `Origin`
header (*scheme + host + port*). A lone `*` allows any origin; an empty or unset
variable disables CORS entirely (no CORS headers, cross-origin requests blocked
by the browser).

```bash
# localhost on any port + any subdomain of example.com
CORS_ALLOWED_ORIGINS=^http://localhost(:\d+)?$,^https://(.+\.)?example\.com$
```

The layer answers preflight (`OPTIONS`) requests, allows the methods
`GET/POST/PUT/PATCH/DELETE/OPTIONS`, allows the `Authorization`, `Content-Type`
and `Accept` headers, and exposes the `X-TOTAL-LIST` header to browsers. An
invalid regex fails fast at startup.

### Running

```bash
# Dev container (SurrealDB + toolchain included)
docker compose -f .devcontainer/docker-compose.yml up -d

# In-memory backend, no external SurrealDB
DB_BACKEND=memory cargo run

# SurrealDB backend (values from .env)
cargo run
```

#### Auto-reload in development (the nodemon equivalent)

`cargo-watch` is the Rust equivalent of `nodemon`: it recompiles and restarts the
process on every save. It is preinstalled in the dev-container image and two
aliases are provided in `.cargo/config.toml`:

```bash
cargo dev          # = cargo watch --poll -x run   (recompile and restart)
cargo dev-check    # = cargo watch --poll -x check (check only, no restart)
```

or the explicit invocation:

```bash
cargo watch --poll -x run                 # with the .env (SurrealDB)
DB_BACKEND=memory cargo watch --poll -x run
```

`cargo watch` observes `src/`, the workspace crates, the tests and `Cargo.toml`,
ignoring `target/`.

> **Why `--poll`?** With the dev container's `..:/workspace:cached` mount,
> **inotify** events produced by edits made *from the host* (VS Code on Windows)
> do not reach the watcher inside the container: without `--poll` the process
> would only observe the initial modification and miss later saves. `--poll`
> makes it rescan the files every 500 ms and always works; the extra I/O is
> negligible for a project of this size.

### Tests

```bash
cargo test                                     # end-to-end integration tests over real HTTP
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

The suite runs against a real router with the in-memory (`mem://`) SurrealDB
backend and covers authentication, CRUD, validation, pagination, CORS, the file
upload flow and the OpenAPI document.

### Docker

A production image and a compose stack live in `docker/prod/`:

```bash
cargo xtask docker-build      # build the image
cargo xtask docker-run        # start the API + SurrealDB stack
```

Uploaded files are persisted on a named volume, and the container runs as a
non-root user.

## Roadmap and status

The framework is under active development. The checklists below track what has
been implemented and what is planned.

### Implemented

- [x] **Zero-touch routing** — `#[route]` proc-macro + `inventory` static registries
- [x] **Dependency injection** — `more-di`, with per-feature service registration and centralized repository wiring
- [x] OpenAPI document and **live Scalar API reference** generated from decorators (`/scalar`)
- [x] Declarative **validation** on body/query/params with `422 validation_error`
- [x] **Unified error contract** `{code, error, message}` for domain, pipeline and validation errors
- [x] JWT **authentication** (HS256): access token + single-use rotating refresh token, Argon2id passwords
- [x] Per-route **authorization**: `#[IsGranted]` guard with roles and the `CurrentUser` extractor
- [x] Reusable **pagination** (`?page`, `?limit`) with the `X-TOTAL-LIST` header
- [x] **CORS** with a configurable regex origin allow-list
- [x] Multer-style **file upload** middleware with streaming, limits and filters
- [x] Example features: `categories`, `places`, `users`, `reports`, `auth`, `uploads`
- [x] Interactive **code generator** (`cargo xtask new-feature`)
- [x] Production **Docker** image and compose stack
- [x] **52** end-to-end HTTP integration tests on `mem://`

### Backlog

- [ ] Request id + per-request tracing (correlation id in logs and headers)
- [ ] Migrations / schema versioning and indexes for filtered fields
- [ ] Rate limiting and security headers as extractable middleware
- [ ] Health with readiness (DB ping), metrics and graceful shutdown
- [ ] Reusable `created_at` / `updated_at` timestamps and soft delete
- [ ] `?sort=` ordering with a per-entity field whitelist
- [ ] Generic search / filters shared across features

## License

No license has been chosen yet. This project is published for learning and
reference purposes; treat it as experimental and not production-ready.
