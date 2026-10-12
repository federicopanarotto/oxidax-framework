# Oxidax — a personal framework for building APIs in Rust

> **Experimental project.** This repository is a "proving ground" where I build,
> piece by piece, **my own** personal framework for writing APIs in Rust. The
> goal is not to invent everything from scratch, but to bring to Rust the
> development style I was comfortable with in **Express / Node**, taking
> advantage of Rust's speed and safety and of a performant database like
> **SurrealDB**.

## Philosophy

1. **Reuse existing libraries.** Rust already has mature ecosystems for
   practically every building block of an API: routing, error handling,
   validation, OpenAPI documentation, dependency injection. The framework
   chooses the best ones and *combines* them into a convenient project
   convention instead of rewriting them.
2. **Zero-touch.** Adding a feature (= a domain of the API) means only creating
   its modules. The "wiring" is generic and **never needs to be touched**.
3. **Decorator style.** Handlers are declared as functions with attributes: the
   route and its documentation are written "on" the function (`#[route]` +
   `#[utoipa::path]`), DTOs are validated with decorators (`#[validate]` inside
   `ValidatedJson<...>`), just like the decorators I was used to.

## Workspace layout

The project is a Cargo workspace split into reusable crates plus the
application:

| Crate         | Role                                                                        |
| ------------- | --------------------------------------------------------------------------- |
| `oxidax-core` | Reusable library: config, errors, auth, middleware, pagination, DI/route registries, DB helpers and generic app assembly |
| `oxidax-macro`| The `#[route]` procedural macro                                              |
| `oxidax-cli`  | Command-line tools: Docker helpers and the `new-feature` code generator (`cargo xtask ...`) |
| `oxidax` (root) | The application: the feature modules and the thin composition (which database backend, OpenAPI metadata) |

Everything that is identical across projects lives in `oxidax-core`; the
application's `src/` only contains the application code.

## Stack

| Building block | Chosen crate                                       | Why                                                          |
| -------------- | -------------------------------------------------- | ------------------------------------------------------------ |
| HTTP/route     | [axum](https://github.com/tokio-rs/axum)           | performant, macro-free, tower ecosystem                      |
| OpenAPI/UI     | [utoipa](https://github.com/juhaku/utoipa) + `utoipa-axum` + `utoipa-swagger-ui` | `#[utoipa::path]` decorators, ready-made Swagger UI |
| Self-registered route | `oxidax-macro` (my `#[route]` proc-macro) + [inventory](https://github.com/dtolnay/inventory) | compile-time static route discovery      |
| Validation     | [validator](https://github.com/Keats/validator)    | mature validation decorators                                 |
| DI             | [more-di](https://github.com/AndyShi-more/MoreDi)  | lazy injection + `ServiceProvider`                           |
| Database       | [SurrealDB](https://surrealdb.com) + `surrealdb`   | performant database, `mem://` for tests, schema-less         |
| Auth           | `jsonwebtoken` + `argon2`                          | HS256 JWT (access/refresh) + Argon2id password hashing       |
| Errors         | `thiserror` + unified pipeline                     | uniform error contract across the whole API                  |

## Project structure

```
oxidax-core/                  # reusable framework library
├── src/
│   ├── app.rs                # generic assembly: DI provider + router + Swagger UI
│   ├── docs.rs              # OpenAPI base document (security schemes)
│   ├── config.rs            # configuration from the environment
│   ├── errors/              # error contract + HTTP pipeline
│   ├── auth/                # JWT, password hashing, identity, guard/extractor
│   ├── middlewares/         # extractable middleware, Express style
│   ├── pagination.rs        # PaginationQuery + Pagination + Paged<T> (X-TOTAL-LIST)
│   ├── validators.rs        # shared validators
│   ├── registry.rs          # static Feature (DI) and Route registries
│   ├── db/                  # SurrealDB connection + reusable query helpers
│   └── framework.rs         # facade: single import (`oxidax_core::framework::*`)

oxidax-macro/src/lib.rs      # the #[route] proc-macro
oxidax-cli/src/              # `cargo xtask`: Docker helpers + `new-feature` generator

src/                         # the application
├── api/                     # the API features
│   ├── categories/          # "categories" feature
│   │   ├── mod.rs           # DI wiring (service) + registration
│   │   ├── category_dto.rs  # DTOs used in body/query/params
│   │   ├── category_entity.rs       # persistence model
│   │   ├── category_repository.rs   # repository trait
│   │   ├── surreal_category_repository.rs  # SurrealDB implementation
│   │   ├── category_service.rs      # domain logic
│   │   └── category_controller.rs   # HTTP handlers with #[route]
│   ├── places/              # "places" feature (city, address, cap, provenance)
│   ├── reports/             # "reports" feature (depends on categories, places and users)
│   ├── users/               # "users" feature (/api/users/me, admin list)
│   └── auth/                # "auth" feature (register/login/refresh/logout)
├── db.rs                    # concrete repositories + central DI registration
├── app.rs                   # application assembly entry point
├── lib.rs                   # re-exports of the core (keeps `crate::lib::...` paths)
└── main.rs                  # binary entry point

tests/report/                # end-to-end integration tests (real HTTP)
docker/prod/                 # production image + compose stack
```

## How a feature is born

A new feature is *only* its module. Taking "categories":

1. **`mod.rs`** — the DI wiring (the only "manual" line): it registers the
   service. Concrete repositories are registered centrally (see `src/db.rs`),
   so the feature stays database-agnostic:
   ```rust
   fn register(services: &mut ServiceCollection) {
       services.add(CategoryService::singleton());
   }

   inventory::submit! { Feature::new(0, register) }
   ```
   The optional `order` guarantees the registration order when a feature depends
   on another (reports → categories).

2. **`category_controller.rs`** — the handlers: no controller object, just
   functions with two attributes:
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

3. **`category_dto.rs`** — the DTOs with validation decorators:
   ```rust
   #[derive(Debug, Serialize, Deserialize, ToSchema)]
   #[serde(deny_unknown_fields)]
   pub struct CreateCategoryDto {
       #[validate(length(min = 3, max = 64, message = "The name is required and must be at least 3 characters"))]
       #[schema(max_length = 64)]
       pub name: String,
   }
   ```

No other change: the assembly in `oxidax-core::app` iterates the static
registries and automatically collects routes and the OpenAPI spec.

## Code generator (CLI)

`cargo xtask new-feature` scaffolds a complete, compiling CRUD feature
interactively: it asks for the feature name and its fields (name + type from a
menu) and writes the entity, DTOs, repository trait, SurrealDB repository,
service, controller, the DI registration, the `src/db.rs` wiring and a full CRUD
test. Complex logic (relations, custom validation, extra endpoints) is left to
be added afterwards — opencode understands the generated code and the CLI
scripts, so it can evolve them.

```bash
cargo new-feature        # or: cargo xtask new-feature
```

Docker shortcuts (see `docker/prod`): `cargo docker-build`, `cargo docker-save`,
`cargo docker-package`, `cargo docker-run`, `cargo docker-stop`.

## Protecting a handler

An authenticated handler (or one restricted to some roles) adds one attribute,
consumed by `#[route]`:

```rust
#[route]                                  // authenticated users only
#[utoipa::path(get, path = "/api/users/me", ...)]
#[IsGranted]
pub async fn get_me(service: ..., current: CurrentUser) -> ... { ... }

#[route]                                  // only the allowed roles
#[IsGranted(["admin"])]
pub async fn list_users(...) -> ... { ... }
```

The macro generates the guard layer (`401` without a valid token, `403` with the
wrong role) and the `CurrentUser` extractor exposes `user_id`, `identity_id` and
`role`: auth is zero-touch too, with no global layer to configure.

## Response contract

- **Success**: JSON body of the response DTO (or `204`).
- **Error**: always `{code, error, message}` with a meaningful HTTP status
  (`not_found`, `database_unavailable`, ...). This holds for domain errors, the
  pipeline and DTO validation: for the latter `code` is `validation_error`
  (invalid field, `422`) or `bad_request` (malformed body/query/params, `400`);
  the field-by-field problems are listed in `message`, in a "kind and
  professional" format meant to be readable by the end user too.

## Pagination

The lists (`/api/categories`, `/api/places`, `/api/reports`, `/api/users`) are
paginated with two **optional**, validated query params:

| Param   | Default | Constraints      |
| ------- | ------- | ---------------- |
| `page`  | `1`     | `>= 1`           |
| `limit` | `20`    | between `1` and `100` |

The **response body stays the usual JSON array**, with no envelope. The total
number of items (across every page) travels in the **`X-TOTAL-LIST`** header, so
the client can compute the number of pages:

```http
GET /api/categories?page=2&limit=5
X-TOTAL-LIST: 42
```

```json
[ { "id": "...", "name": "Sale" }, ... ]
```

Out-of-range values give `422 {code: validation_error}` (like every validated
DTO); a non-numeric value gives `400 {code: bad_request}`. Everything is
reusable: the `PaginationQuery` DTO, the `Pagination` value object and the
`Paged<T>` payload live in `oxidax-core/src/pagination.rs`, and the query
helpers (count + page) in `oxidax-core/src/db/query.rs`. A paginated feature only
adds an extractor and returns `Paged<T>` from the service; the controller calls
`page.into_response()` to compose the body + header.

## Authentication

JWTs signed with HS256 (`JWT_SECRET`) with two distinct tokens: a short-lived
access token and a long, **single-use** refresh token (rotated on every refresh,
burned by logout).

| Endpoint                  | Effect                                                              |
| ------------------------- | ------------------------------------------------------------------- |
| `POST /api/auth/register` | `201 {message}`: no token; the email is the username (`role = user`) |
| `POST /api/auth/login`    | `200 {access_token, refresh_token, token_type, expires_in}`          |
| `POST /api/auth/refresh`  | new pair; the old refresh token stops working                        |
| `POST /api/auth/logout`   | `204`: clears the `jti` → the outstanding refresh token is revoked   |
| `GET /api/users/me`       | the caller's profile with email                                      |
| `GET /api/users`          | user list with email, `admin` only; optional `?role=admin\|user` filter, paginated (`?page=`, `?limit=`) |

Errors are **generic** by design: an unknown email and a wrong password give the
same `401 invalid credentials`, a duplicate register gives a generic `409` (no
account enumeration). The first admin is assigned directly in the database. In
Swagger the two schemes `accessToken`/`refreshToken` (bearer JWT) are ready to
use with "Authorize".

## Documentation

The Swagger UI is served, live, at **`/swagger-ui`** (full spec at
`/api-docs/openapi.json`). No export file: the framework composes the OpenAPI
spec from the macros while it mounts the routes.

## Configuration

Configuration comes **only** from environment variables: no value is hardcoded.
Locally the values are read from the `.env` file (git-ignored) via `dotenvy`;
variables already present in the environment take precedence over the file.

```bash
cp .env.example .env   # template with all required variables
```

Required variables: `SERVER_ADDR`, `DB_BACKEND` (`surreal` or `memory`),
`SURREAL_URL`, `SURREAL_NS`, `SURREAL_DB`, `SURREAL_USER`, `SURREAL_PASS`,
`JWT_SECRET`, `ACCESS_TOKEN_DURATION` (seconds), `REFRESH_TOKEN_DURATION`
(seconds). If a variable is missing, startup fails with an explicit message.

## Running

```bash
# Dev container (SurrealDB + toolchain included)
docker compose -f .devcontainer/docker-compose.yml up -d

# In-memory backend, no external SurrealDB
DB_BACKEND=memory cargo run

# SurrealDB backend (values from .env)
cargo run
```

### Auto-reload in development (the nodemon equivalent)

`cargo-watch` is the Rust equivalent of `nodemon`: it recompiles and restarts
the process on every save. It is already installed in the dev container image
and two aliases are ready in `.cargo/config.toml`:

```bash
cargo dev          # = cargo watch --poll -x run   (recompile and restart the server)
cargo dev-check    # = cargo watch --poll -x check (check only, no restart)
```

or the explicit invocation:

```bash
cargo watch --poll -x run                 # with the .env (SurrealDB)
DB_BACKEND=memory cargo watch --poll -x run
```

`cargo watch` observes `src/`, the workspace crates, the tests, `Cargo.toml` and
the rest of the project, ignoring `target/`.

> **Why `--poll`?** With the dev container's `..:/workspace:cached` mount the
> **inotify** events generated by edits made *from the host* (VS Code on
> Windows) do not reach the watcher inside the container: without `--poll` the
> process would only see the initial modification and never the following saves.
> `--poll` makes it check the files every 500 ms and always works. It is a bit
> "heavier" (I/O) but negligible for a project of this size.

## Tests

```bash
cargo test                                  # end-to-end integration tests over real HTTP (40 tests)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

## Status

- [x] Routing + DI + OpenAPI with **zero-touch** discovery (`#[route]` on every
      handler, `inventory` registries)
- [x] Decorator validation on body/query/params with `422 {code: validation_error}`
- [x] Uniform error pipeline + 404 fallback
- [x] **All** errors in the same `{code, error, message}` format: domain, pipeline
      and DTO validation share the body (`ErrorResponse`, `bad_request` for
      malformed body/query/params)
- [x] Example features: `categories`, `reports` and `places` on SurrealDB
      (`mem://` in tests)
- [x] Live Swagger UI
- [x] Access log (outer middleware) + JWT auth with **per-route** guard
      (`#[IsGranted]`, roles included)
- [x] Report with creator: `POST /api/reports` authenticated, the token's user is
      stored and returned as `created_by` on reads; `PUT`/`DELETE` are
      authenticated too, reads stay public
- [x] Validated query params: `GET /api/users?role=admin|user` (`UserListQueryDto`
      with `ValidatedQuery`, `422` on out-of-list values)
- [x] Reusable pagination on every list: validated `?page=&limit=`, array in the
      body and total in `X-TOTAL-LIST` (`Paged<T>` + `oxidax-core/src/db/query.rs`);
      `422` out of range
- [x] Workspace split into `oxidax-core` (reusable library), `oxidax-macro`
      (the `#[route]` macro) and `oxidax-cli` (Docker helpers + code generator)
- [x] Interactive feature generator: `cargo xtask new-feature`

### Backlog (to be developed)

Features not implemented yet, in approximate priority order. The first ones are
"foundational" to make the framework complete; sorting is **not essential** at
the moment:

- [ ] Request ID + per-request tracing (correlation id in logs/headers)
- [ ] Migrations/schema versioning + indexes for filtered fields
- [ ] Rate limiting, CORS and security headers as extractable middleware
- [ ] Health with readiness (DB ping) + metrics + graceful shutdown
- [ ] Reusable `created_at`/`updated_at` timestamps and soft delete
- [ ] `?sort=` (ordering) — **not essential**: `-` prefix for desc or a separate
      `order=asc|desc` param, with a per-entity field whitelist
- [ ] Generic search/filters shared across features

> **Note**: the framework is under construction and is meant for learning: the
> public macro API and the registries' structure may change.
