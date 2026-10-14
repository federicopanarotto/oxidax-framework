# Adding a feature — step-by-step guide (example: `Places`)

This guide explains, component by component, how to add a new feature to the
API. As a concrete example we build the **`Places`** feature (a place: `city`,
`address`, `cap`, `provenance`) and connect it to the existing **`reports`**
feature with an optional relation (a report may point to a place), exactly as is
already done with `categories`.

> **Fast path:** most of what this guide describes is generated for you by
> `cargo xtask new-feature` (or the alias `cargo new-feature`): it scaffolds a
> complete, compiling CRUD feature (entity, DTOs, repository, SurrealDB
> implementation, service, controller, wiring and a test) from the feature name
> and its fields. Read this guide to understand what it produces and to evolve
> it by hand (relations, custom validation, extra endpoints).

> Project convention: prose and code comments are in English.

---

## 0. Mental model: the layers of a feature

A feature is always made of the same pieces, stacked from the bottom up. Every
layer depends **only** on the one below it (never the other way around):

```
                 HTTP (axum)
   place_controller.rs   ← #[route] + #[utoipa::path] handlers; talks ONLY to the service
            │
   place_service.rs      ← domain logic; talks to the repository (and to other services)
            │
   place_repository.rs   ← trait: the persistence contract (DB-agnostic)
            │
   surreal_place_repository.rs ← SurrealDB implementation (the only DB-specific part)
            │
   place_entity.rs       ← the pure model the repository persists/returns
            │
   place_dto.rs          ← the input/output DTOs (validation, JSON serialization)
```

Golden rules:

- The **controller** holds no logic: it calls **one** service function.
- The **service** is the only orchestrator; if it needs other data, it talks to
  other services injected via DI (as `ReportService` does with `CategoryService`).
- The **repository** is the only place that "knows" the database.

### What is automatic and what is manual

Thanks to the framework (the `#[route]` macro + `inventory`) the generic
assembly lives in `oxidax-core` (`oxidax_core::app::assemble`) and **you never
touch it**: it iterates the static registries and collects routes and the
OpenAPI spec. The only **manual** things when adding a feature are:

1. create the feature files;
2. declare the module in `src/api/mod.rs` (one line);
3. register the repository in `src/db.rs` (the repository aggregate: the single
   place that knows the concrete repositories).

Inside the feature, the `inventory::submit!` line in `mod.rs` self-registers the
DI wiring; the `#[route]` attributes on the handlers self-register routes and
OpenAPI. (All of this is produced by `cargo xtask new-feature`.)

---

## 1. Files to create

Create the folder `src/api/places/` with these files:

```
src/api/places/
├── mod.rs                        # DI wiring + inventory::submit!  (self-registration)
├── place_entity.rs               # domain/persistence entity
├── place_dto.rs                  # input DTOs (validate) and output DTOs (response)
├── place_repository.rs           # repository trait
├── surreal_place_repository.rs   # SurrealDB implementation of the trait
├── place_service.rs              # domain logic
└── place_controller.rs           # HTTP handlers with #[route] + #[utoipa::path]
```

Then you edit two shared files (`src/api/mod.rs`, `src/db.rs`) and the `reports`
feature to add the relation.

---

## 2. The entity — `src/api/places/place_entity.rs`

The **entity** is the pure model, independent of the database: any repository
(whether SurrealDB, Postgres, an in-memory mock…) must persist and return exactly
this type. It holds neither validation nor logic.

```rust
use serde::{Deserialize, Serialize};

/// Database-independent entity: any repository must persist and return
/// exactly this type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    pub id: String,
    pub city: String,
    pub address: String,
    pub cap: String,
    pub provenance: String,
}
```

Why keep it separate from the DTOs? Because the DTO describes what **enters**
from HTTP (with its validation rules), while the entity describes what lives in
the database. Keeping them apart avoids "leaking" HTTP details into the domain
and vice versa.

---

## 3. DTOs and validation — `src/api/places/place_dto.rs`

Three types live here:

- `CreatePlaceDto` / `UpdatePlaceDto`: what arrives in the body (with the
  `#[validate(...)]` decorators);
- `PlaceResponse`: what goes back to the client;
- the DTO → entity conversion (`into_entity`) and entity → response (`From`).

Are the fields **private**? No: in this project the DTOs have `pub` fields
(convenient for tests and services); validation happens earlier, in the
`ValidatedJson` extractor.

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::places::place_entity::Place;
use crate::lib::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreatePlaceDto {
    /// City name
    #[schema(example = "Milan", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The city is required and cannot be empty"
        ),
        length(max = 200, message = "The city must be at most 200 characters")
    )]
    pub city: String,

    /// Street address
    #[schema(example = "1 Main Street", min_length = 1, max_length = 300)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The address is required and cannot be empty"
        ),
        length(max = 300, message = "The address must be at most 300 characters")
    )]
    pub address: String,

    /// Postal code
    #[schema(example = "20100", min_length = 1, max_length = 10)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The postal code is required and cannot be empty"
        ),
        length(max = 10, message = "The postal code must be at most 10 characters")
    )]
    pub cap: String,

    /// Provenance
    #[schema(example = "City of Milan", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The provenance is required and cannot be empty"
        ),
        length(max = 200, message = "The provenance must be at most 200 characters")
    )]
    pub provenance: String,
}

impl CreatePlaceDto {
    /// Converts the DTO into the entity ready for the database: generates the
    /// id and normalizes the fields (validation already done by the extractor).
    pub fn into_entity(self) -> Place {
        Place {
            id: Uuid::new_v4().to_string(),
            city: self.city.trim().to_string(),
            address: self.address.trim().to_string(),
            cap: self.cap.trim().to_string(),
            provenance: self.provenance.trim().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdatePlaceDto {
    /// New city name
    #[schema(example = "Turin", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The city is required and cannot be empty"
        ),
        length(max = 200, message = "The city must be at most 200 characters")
    )]
    pub city: String,

    /// New street address
    #[schema(example = "10 France Avenue", min_length = 1, max_length = 300)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The address is required and cannot be empty"
        ),
        length(max = 300, message = "The address must be at most 300 characters")
    )]
    pub address: String,

    /// New postal code
    #[schema(example = "10100", min_length = 1, max_length = 10)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The postal code is required and cannot be empty"
        ),
        length(max = 10, message = "The postal code must be at most 10 characters")
    )]
    pub cap: String,

    /// New provenance
    #[schema(example = "City of Turin", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The provenance is required and cannot be empty"
        ),
        length(max = 200, message = "The provenance must be at most 200 characters")
    )]
    pub provenance: String,
}

impl UpdatePlaceDto {
    /// Converts the DTO into the entity ready for the database: reuses the id
    /// from the path and normalizes the fields.
    pub fn into_entity(self, id: String) -> Place {
        Place {
            id,
            city: self.city.trim().to_string(),
            address: self.address.trim().to_string(),
            cap: self.cap.trim().to_string(),
            provenance: self.provenance.trim().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlaceResponse {
    /// UUID identifier of the place
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// City name
    #[schema(example = "Milan")]
    pub city: String,
    /// Street address
    #[schema(example = "1 Main Street")]
    pub address: String,
    /// Postal code
    #[schema(example = "20100")]
    pub cap: String,
    /// Provenance
    #[schema(example = "City of Milan")]
    pub provenance: String,
}

impl From<crate::api::places::place_entity::Place> for PlaceResponse {
    fn from(place: crate::api::places::place_entity::Place) -> Self {
        Self {
            id: place.id,
            city: place.city,
            address: place.address,
            cap: place.cap,
            provenance: place.provenance,
        }
    }
}
```

Key points:

- `#[derive(Validate)]` + `#[validate(...)]` are what the `ValidatedJson`
  extractor runs **before** the handler is called: if they fail, it responds
  `422 {code: validation_error, error, message}` without ever entering the
  service (same `{code, error, message}` format as the pipeline: malformed
  body/query/params give `400 {code: bad_request}`).
- `not_blank` is a shared validator (`crate::lib::validators`, re-exported from
  `oxidax-core`): it rejects empty or whitespace-only strings.
- `into_entity` normalizes (`.trim()`) and generates the `id`, so the service
  receives ready-made data.
- `ToSchema` makes the DTO visible in the API reference.

---

## 4. The repository trait — `src/api/places/place_repository.rs`

The **trait** is the persistence contract. It is the only thing the service
knows about the database: switching backend = implementing this trait, without
touching service and controllers.

```rust
use async_trait::async_trait;

use crate::api::places::place_entity::Place;
use crate::errors::AppResult;

/// Persistence contract for places.
///
/// It is the only point that depends on the database: implementing this
/// trait on a new backend (Postgres, SQLite, MongoDB...) leaves the service
/// and controllers unchanged.
#[async_trait]
pub trait PlaceRepository: Send + Sync {
    async fn insert(&self, place: Place) -> AppResult<Place>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Place>>;
    async fn find_all(&self) -> AppResult<Vec<Place>>;
    async fn replace(&self, id: &str, place: Place) -> AppResult<Option<Place>>;
    async fn remove(&self, id: &str) -> AppResult<bool>;
}
```

Notes:

- `#[async_trait]` allows `async` methods in traits (natively in Rust they are
  only available with GATs; `async-trait` is the trade-off used here).
- `Send + Sync` is required because the repository lives in an `Arc` shared
  across tokio tasks.
- The methods return `AppResult<...>`: database errors are converted into
  `AppError` centrally (see `impl From<surrealdb::Error> for AppError`).

---

## 5. The SurrealDB repository — `src/api/places/surreal_place_repository.rs`

This is the **only DB-specific code** of the feature. The typical structure
(identical to `surreal_category_repository.rs`) is:

- a `TABLE` constant;
- a `...Row`: how the data arrives from the database (the `id` is a SurrealDB
  `RecordId`);
- a `...Content`: what is written in the `CONTENT` (without `id`, managed by the
  database);
- `new`, `ensure_schema`, an `into_entity` (record id → API `uuid`) and the trait
  methods.

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::{RecordId, SurrealValue};

use crate::api::places::place_entity::Place;
use crate::api::places::place_repository::PlaceRepository;
use crate::db::{define_table, record_key_to_string};
use crate::errors::{AppError, AppResult};

const TABLE: &str = "place";

/// Row as it arrives from the database (the id is a SurrealDB record id).
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct PlaceRow {
    id: RecordId,
    city: String,
    address: String,
    cap: String,
    provenance: String,
}

/// Persistable content: the id is managed by the database, it must not be
/// written in the CONTENT.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct PlaceContent {
    city: String,
    address: String,
    cap: String,
    provenance: String,
}

pub struct SurrealPlaceRepository {
    db: Surreal<Any>,
}

impl SurrealPlaceRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Creates the table if it does not exist: SurrealDB 3 does not create
    /// tables implicitly.
    pub async fn ensure_schema(&self) -> AppResult<()> {
        define_table(&self.db, TABLE).await
    }
}

/// The API works with the record id key alone (`uuid`), not with `table:uuid`.
fn into_entity(row: PlaceRow) -> Place {
    Place {
        id: record_key_to_string(row.id.key),
        city: row.city,
        address: row.address,
        cap: row.cap,
        provenance: row.provenance,
    }
}

fn created_or_internal(place: Place, created: Option<PlaceRow>) -> AppResult<Place> {
    if created.is_some() {
        Ok(place)
    } else {
        Err(AppError::Internal(format!(
            "failed to create place {}",
            place.id
        )))
    }
}

#[async_trait]
impl PlaceRepository for SurrealPlaceRepository {
    async fn insert(&self, place: Place) -> AppResult<Place> {
        let content = PlaceContent {
            city: place.city.clone(),
            address: place.address.clone(),
            cap: place.cap.clone(),
            provenance: place.provenance.clone(),
        };

        let created: Option<PlaceRow> = self
            .db
            .create((TABLE, place.id.as_str()))
            .content(content)
            .await?;

        created_or_internal(place, created)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<Place>> {
        let found: Option<PlaceRow> = self.db.select((TABLE, id)).await?;
        Ok(found.map(into_entity))
    }

    async fn find_all(&self) -> AppResult<Vec<Place>> {
        let rows: Vec<PlaceRow> = self.db.select(TABLE).await?;
        Ok(rows.into_iter().map(into_entity).collect())
    }

    async fn replace(&self, id: &str, place: Place) -> AppResult<Option<Place>> {
        let content = PlaceContent {
            city: place.city.clone(),
            address: place.address.clone(),
            cap: place.cap.clone(),
            provenance: place.provenance.clone(),
        };

        let updated: Option<PlaceRow> = self.db.update((TABLE, id)).content(content).await?;
        Ok(updated.map(into_entity))
    }

    async fn remove(&self, id: &str) -> AppResult<bool> {
        let deleted: Option<PlaceRow> = self.db.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}
```

Key points:

- `define_table` (helper in `oxidax-core/src/db/surreal.rs`, re-exported as
  `crate::db::define_table`) runs `DEFINE TABLE IF NOT EXISTS place SCHEMALESS`:
  in SurrealDB 3 tables are not created automatically.
- `record_key_to_string` converts the SurrealDB record id into the string used by
  the API.
- The `created_or_internal` helper turns a missing insert result into an
  `AppError::Internal`.
- Note the `.clone()` on every field when building the `CONTENT`: the content is
  moved into the query while the original value is still needed afterwards (for
  the return value), so cloning is required to avoid a partial move.

---

## 6. The service — `src/api/places/place_service.rs`

The **service** is where domain logic lives. Here there is not much logic: the
DTOs arrive already validated and converted, so the service just orchestrates the
repository and turns absence into a domain error. `more-di`'s `#[injectable]`
generates the constructor that resolves the fields from the container.

```rust
use std::collections::HashMap;
use std::sync::Arc;

use di::injectable;

use crate::api::places::place_dto::{CreatePlaceDto, PlaceResponse, UpdatePlaceDto};
use crate::api::places::place_repository::PlaceRepository;
use crate::errors::{AppError, AppResult};

/// Business rules for places.
///
/// The DTOs arrive already converted and validated (id generated, trimmed,
/// `#[validate(...)]` decorators of the `ValidatedJson` extractors): the
/// service only writes ready-made data to the repository.
#[injectable]
pub struct PlaceService {
    repository: Arc<dyn PlaceRepository>,
}

impl PlaceService {
    pub async fn create(&self, dto: CreatePlaceDto) -> AppResult<PlaceResponse> {
        let place = dto.into_entity();

        self.repository.insert(place).await.map(Into::into)
    }

    pub async fn get(&self, id: &str) -> AppResult<PlaceResponse> {
        self.repository
            .find_by_id(id)
            .await?
            .map(Into::into)
            .ok_or_else(|| not_found(id))
    }

    pub async fn list(&self) -> AppResult<Vec<PlaceResponse>> {
        self.repository
            .find_all()
            .await
            .map(|places| places.into_iter().map(Into::into).collect())
    }

    pub async fn update(&self, id: &str, dto: UpdatePlaceDto) -> AppResult<PlaceResponse> {
        let place = dto.into_entity(id.to_string());

        self.repository
            .replace(id, place)
            .await?
            .map(Into::into)
            .ok_or_else(|| not_found(id))
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        if self.repository.remove(id).await? {
            Ok(())
        } else {
            Err(not_found(id))
        }
    }

    /// Checks whether a place exists: used by the reports service to validate
    /// the relationship before creating or updating a report.
    pub async fn exists(&self, id: &str) -> AppResult<bool> {
        Ok(self.repository.find_by_id(id).await?.is_some())
    }

    /// Retrieves a place without failing if absent: used by the reports service
    /// to nest the `place` object in the response.
    pub async fn find(&self, id: &str) -> AppResult<Option<PlaceResponse>> {
        Ok(self.repository.find_by_id(id).await?.map(Into::into))
    }

    /// `id → place` index: decorates a list of reports with a single read,
    /// avoiding N+1 queries per element.
    pub async fn by_id(&self) -> AppResult<HashMap<String, PlaceResponse>> {
        let mut index = HashMap::new();
        for place in self.list().await? {
            index.insert(place.id.clone(), place);
        }
        Ok(index)
    }
}

fn not_found(id: &str) -> AppError {
    AppError::NotFound(format!("place {id} not found"))
}
```

Note: the three "extra" methods (`exists`, `find`, `by_id`) exist **only** to let
the `reports` feature validate and compose the relation without duplicating
queries. It is the correct way to make two features talk: one service exposes
methods, the other service calls them.

---

## 7. The controller — `src/api/places/place_controller.rs`

The **controller** is only transport: no logic, no domain `if`s. Every handler has
two attributes:

- `#[route]`: **auto-registers** the route and the OpenAPI fragment (it is the
  `oxidax-macro` macro); it must be **above** `#[utoipa::path]`.
- `#[utoipa::path(...)]`: declares the method, path, tag and documented responses.

The handler takes the service with `State<Arc<PlaceService>>` (the only service it
needs) and, when there is a body, `ValidatedJson<...>`.

```rust
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::api::places::place_dto::{CreatePlaceDto, PlaceResponse, UpdatePlaceDto};
use crate::api::places::place_service::PlaceService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::framework::{ValidatedJson, route};

// Route table and OpenAPI spec of the places, defined point-inline: every
// handler is annotated with `#[utoipa::path(...)]` (HTTP method, path and
// responses) and `#[route]`, which self-registers it in the static registry.
// The generic assembly in oxidax-core never changes when a handler is added.

#[route]
#[utoipa::path(
    get,
    path = "/api/places",
    tag = "places",
    responses(
        (status = 200, description = "List of places", body = [PlaceResponse]),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn list_places(
    State(service): State<Arc<PlaceService>>,
) -> AppResult<Json<Vec<PlaceResponse>>> {
    Ok(Json(service.list().await?))
}

#[route]
#[utoipa::path(
    post,
    path = "/api/places",
    tag = "places",
    request_body = CreatePlaceDto,
    responses(
        (status = 201, description = "Place created successfully", body = PlaceResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 422, description = "Field validation failed", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn create_place(
    State(service): State<Arc<PlaceService>>,
    ValidatedJson(dto): ValidatedJson<CreatePlaceDto>,
) -> AppResult<(StatusCode, Json<PlaceResponse>)> {
    let created = service.create(dto).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

#[route]
#[utoipa::path(
    get,
    path = "/api/places/{id}",
    tag = "places",
    params(("id" = String, Path, description = "Place id")),
    responses(
        (status = 200, description = "Place found", body = PlaceResponse),
        (status = 404, description = "Place not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn get_place(
    State(service): State<Arc<PlaceService>>,
    Path(id): Path<String>,
) -> AppResult<Json<PlaceResponse>> {
    Ok(Json(service.get(&id).await?))
}

#[route]
#[utoipa::path(
    put,
    path = "/api/places/{id}",
    tag = "places",
    params(("id" = String, Path, description = "Place id")),
    request_body = UpdatePlaceDto,
    responses(
        (status = 200, description = "Place updated", body = PlaceResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 404, description = "Place not found", body = ErrorResponse),
        (status = 422, description = "Field validation failed", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn update_place(
    State(service): State<Arc<PlaceService>>,
    Path(id): Path<String>,
    ValidatedJson(dto): ValidatedJson<UpdatePlaceDto>,
) -> AppResult<Json<PlaceResponse>> {
    Ok(Json(service.update(&id, dto).await?))
}

#[route]
#[utoipa::path(
    delete,
    path = "/api/places/{id}",
    tag = "places",
    params(("id" = String, Path, description = "Place id")),
    responses(
        (status = 204, description = "Place deleted"),
        (status = 404, description = "Place not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn delete_place(
    State(service): State<Arc<PlaceService>>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    service.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

Key points:

- No import of `Router`/`OpenApiRouter`/`routes!`: the macro adds them.
- `Path(id): Path<String>` for route params; `ValidatedJson` for the body.
- The imports come from the **facade** `crate::framework` (extractors, `route`,
  error types), so all feature controllers are uniform.
- The `tag` groups the endpoints in the Scalar API reference.

---

## 7-bis. Protecting a handler — `#[IsGranted]`

A handler can be protected by the `#[IsGranted]` attribute, which must be placed
**below** `#[route]` (it is the `#[route]` macro that consumes it and generates
the guard; on its own it does not compile):

```rust
#[route]
#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "users",
    responses(
        (status = 200, description = "Profile of the authenticated user", body = UserResponse),
        (status = 401, description = "Missing or invalid access token", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
    security(
        ("accessToken" = []),
    ),
)]
#[IsGranted] // any authenticated caller
pub async fn get_me(
    State(service): State<Arc<UserService>>,
    current: CurrentUser,
) -> AppResult<Json<UserResponse>> {
    Ok(Json(service.me(&current.user_id).await?))
}
```

Allowed forms:

- `#[IsGranted]`: any authenticated caller.
- `#[IsGranted(["admin"])]`: authenticated **and** role among those listed (more
  roles: `["admin", "editor"]`). Strings must not be repeated.

What the macro generates (see `oxidax-macro/src/lib.rs`):

- a `from_fn_with_state((JwtConfig, roles), guard)` layer on the **single route**:
  no global layers, no manual wiring. Without `#[IsGranted]` the route stays
  public and does not depend on `JwtConfig`.
- the guard (`oxidax-core/src/auth/guard.rs`) extracts the bearer token, verifies
  it and inserts `CurrentUser` into the request extensions. Missing/invalid/expired
  token → `401`; disallowed role → `403`.

Inside the handler the current user is read with the `CurrentUser` extractor
(re-exported by `crate::framework`), which exposes `user_id`, `identity_id` and
`role`.

In `#[utoipa::path]` always declare `security(("accessToken" = []))` (the auth
refresh endpoint uses `("refreshToken" = [])`) and the documented `401`/`403`:
the schemes are defined once in `oxidax-core/src/docs.rs`, so here you only need
the reference.

Common errors:

- **`cannot find attribute IsGranted`**: the attribute has no `#[route]` above it
  (or is outside the handler).
- **`expected #[IsGranted] or #[IsGranted(["admin"])]`**: invalid form (e.g.
  `#[IsGranted("admin")]` or lists not made of strings).
- **unexpected `401` on a protected route**: the handler does not use the
  `CurrentUser` extractor… or the token is expired: check `exp`/leeway
  (`jsonwebtoken` grants 60 seconds of leeway by default).

---

## 7-ter. Paginating a list — `PaginationQuery` + `Paged<T>`

Every list in the framework (`categories`, `places`, `reports`, `users`) shares
the same reusable pagination (`oxidax-core/src/pagination.rs`, re-exported as
`crate::lib::pagination`), so there is no need to rewrite it per feature. The
contract is: **body = the usual JSON array**, **total items in the
`X-TOTAL-LIST` header** (no envelope).

On the **repository** side add the two paginated methods to the trait and the
implementation, reusing the helpers in `oxidax-core/src/db/query.rs` (re-exported
as `crate::db::query`):

```rust
// place_repository.rs
async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<Place>>;
async fn count(&self) -> AppResult<u64>;
```

```rust
// surreal_place_repository.rs
async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<Place>> {
    let rows: Vec<PlaceRow> = query::select_page(&self.db, TABLE, offset, limit).await?;
    Ok(rows.into_iter().map(into_entity).collect())
}

async fn count(&self) -> AppResult<u64> {
    query::count(&self.db, TABLE).await
}
```

On the **service** side return `Paged<T>` (items + total):

```rust
use crate::lib::pagination::{Paged, Pagination};

pub async fn list_page(&self, pagination: Pagination) -> AppResult<Paged<PlaceResponse>> {
    let total = self.repository.count().await?;
    let places = self
        .repository
        .find_page(pagination.offset(), pagination.limit())
        .await?;
    Ok(Paged::new(places.into_iter().map(Into::into).collect(), total))
}
```

On the **controller** side add the `PaginationQuery` extractor and call
`into_response()`: the `#[route]` macro does not change, the body stays the array.

```rust
#[route]
#[utoipa::path(
    get,
    path = "/api/places",
    tag = "places",
    params(
        ("page" = Option<u64>, Query, description = "Page, 1-based (default 1)", example = 1),
        ("limit" = Option<u64>, Query, description = "Items per page, 1-100 (default 20)", example = 20),
    ),
    responses(
        (status = 200, description = "Paginated list; total in `X-TOTAL-LIST`",
            body = [PlaceResponse],
            headers(("X-TOTAL-LIST" = u64, description = "Total items across all pages"))),
        (status = 422, description = "Invalid pagination parameters", body = ErrorResponse),
        (status = 503, description = "Database unavailable", body = ErrorResponse),
    ),
)]
pub async fn list_places(
    State(service): State<Arc<PlaceService>>,
    ValidatedQuery(pagination): ValidatedQuery<PaginationQuery>,
) -> AppResult<Response> {
    let page = service.list_page(pagination.pagination()).await?;
    Ok(page.into_response())
}
```

Notes:

- `page`/`limit` are `Option` with defaults (`1` and `20`) applied by
  `Pagination::new`; the range is validated (`422` out of range, `400` if not
  numeric).
- Multiple query extractors coexist in the same handler: filters (e.g. `?role=`)
  use their own DTO (`ValidatedQuery<UserListQueryDto>`) alongside
  `ValidatedQuery<PaginationQuery>`.
- Ordering is not implemented yet: without `ORDER BY` the page order is not
  guaranteed.

---

## 8. The feature wiring — `src/api/places/mod.rs`

This file does **only two things**: (1) declares the submodules; (2) registers the
feature's service in the DI container through `inventory::submit!`. It is the only
"manual" DI line of the feature. The concrete repository is NOT registered here:
it is registered centrally in `src/db.rs` (see §10), so the feature stays
database-agnostic.

```rust
pub mod place_controller;
pub mod place_dto;
pub mod place_entity;
pub mod place_repository;
pub mod place_service;
pub mod surreal_place_repository;

use di::Injectable;
use di::ServiceCollection;

use crate::framework::Feature;
use place_service::PlaceService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `place_controller`. The repository is registered centrally
// in `crate::db`.
fn register(services: &mut ServiceCollection) {
    services.add(PlaceService::singleton());
}

// Zero-touch self-registration in the feature registry.
inventory::submit! {
    Feature::new(1, register)
}
```

Explanation:

- `PlaceService::singleton()` registers the service (built by `#[injectable]`,
  which resolves the repository from the container by asking for
  `Arc<dyn PlaceRepository>`).
- `Feature::new(order, register)`: `order` is the registration order. The
  existing features use `categories = 0`, `auth = 0`, `users = 0`,
  `places = 1`, `reports = 2`. **Set `places = 1` and move `reports = 2`** (see
  §11), so the services used by reports are registered first. In practice
  `more-di` injects lazily (order is tolerant), but keeping it explicit is safer.
- `inventory::submit!` is what makes the feature "zero-touch": the generic
  assembly in `oxidax-core` finds it on its own, with no edits.

---

## 9. Declare the module — `src/api/mod.rs` (manual edit 1)

Add one line, in alphabetical order:

```rust
pub mod auth;
pub mod categories;
pub mod places; // <-- new
pub mod reports;
pub mod users;
```

You can read the need for this line as the price to pay to avoid touching the
generic assembly: Rust must know the module exists.

---

## 10. Register the repository — `src/db.rs` (manual edit 2)

`db::Repositories` is the aggregate of the concrete repositories, and
`register_repositories` is the single wired point that registers each of them
under its trait in the DI container. Three small additions:

```rust
// 1. imports
use crate::api::places::place_repository::PlaceRepository;
use crate::api::places::surreal_place_repository::SurrealPlaceRepository;

// 2. the aggregate gains a field
pub struct Repositories {
    pub reports: Arc<dyn ReportRepository>,
    pub categories: Arc<dyn CategoryRepository>,
    pub places: Arc<dyn PlaceRepository>, // <-- new
    pub users: Arc<dyn UserRepository>,
    pub user_identities: Arc<dyn UserIdentityRepository>,
}

// 3. register_repositories gains one line (the single DI wired point)
pub fn register_repositories(repositories: &Repositories, services: &mut ServiceCollection) {
    register_repository(services, repositories.reports.clone());
    register_repository(services, repositories.categories.clone());
    register_repository(services, repositories.places.clone()); // <-- new
    register_repository(services, repositories.users.clone());
    register_repository(services, repositories.user_identities.clone());
}
```

And in `from_connection`, build it on the shared connection and create its table:

```rust
let places = SurrealPlaceRepository::new(db.clone());
places.ensure_schema().await?;
```

and add it to the returned struct:

```rust
Ok(Repositories {
    reports: Arc::new(reports),
    categories: Arc::new(categories),
    places: Arc::new(places), // <-- new
    users: Arc::new(users),
    user_identities: Arc::new(user_identities),
})
```

Key points:

- Every repository shares the **same connection** (`db.clone()`); the last one
  may take it "owned" (`db`).
- `ensure_schema()` must be called for every new repository: it creates the
  `place` table in SurrealDB.
- `register_repository(services, repositories.places.clone())` registers the
  concrete repository **under its trait** (`Arc<dyn PlaceRepository>`), so the
  service depends on the contract, not the implementation.
- This is, in practice, the only "non zero-touch" point of a feature. If you
  later want to remove it, the step is to make `Repositories` a dynamic map
  (e.g. keyed by `TypeId`) populated by the features themselves.

---

## 11. The relation: `reports` → `places`

A report can now point to a place. The model is **the same** already used for
`categories`: an optional `place_id` field in the entity, validated at the DTO
level (format) and service level (existence), and the full object nested in the
response. The response exposes **only** the nested `place` object (not
`place_id`), as already done for `category`.

### 11.1 `src/api/reports/report_entity.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Id of the associated category (`None` if the report has no category).
    pub category_id: Option<String>,
    /// Id of the associated place (`None` if the report has no place).
    pub place_id: Option<String>,
}
```

### 11.2 `src/api/reports/report_dto.rs`

Add to **`CreateReportDto`** and **`UpdateReportDto`** a `place_id` field
identical (in shape) to `category_id`:

```rust
    /// Id of the associated place (optional; `null` or absent removes the relationship)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    #[validate(custom(
        function = "crate::lib::validators::validate_uuid",
        message = "The place id must be a valid UUID"
    ))]
    pub place_id: Option<String>,
```

In the respective `into_entity`, add `place_id: self.place_id,`.

In **`ReportResponse`** add the nested object (not the id):

```rust
    /// Associated category (full object, present only if the relationship exists)
    pub category: Option<CategoryResponse>,
    /// Associated place (full object, present only if the relationship exists)
    pub place: Option<PlaceResponse>,
```

Add the import at the top of the file:

```rust
use crate::api::places::place_dto::PlaceResponse;
```

And replace the `impl From` with the multi-field version (the real code also
carries `created_by`; keep it and add `place`):

```rust
impl
    From<(
        crate::api::reports::report_entity::Report,
        Option<CategoryResponse>,
        Option<PlaceResponse>,
        Option<UserSummaryResponse>,
    )> for ReportResponse
{
    fn from(
        (report, category, place, created_by): (
            crate::api::reports::report_entity::Report,
            Option<CategoryResponse>,
            Option<PlaceResponse>,
            Option<UserSummaryResponse>,
        ),
    ) -> Self {
        Self {
            id: report.id,
            name: report.name,
            description: report.description,
            category,
            place,
            created_by,
        }
    }
}
```

### 11.3 `src/api/reports/surreal_report_repository.rs`

Add `place_id: Option<String>` to `ReportRow` **and** `ReportContent`, then:

- in `into_entity`: `place_id: row.place_id,`
- in `insert` and `replace`, inside `ReportContent`:
  `place_id: report.place_id.clone(),`

(The `ReportRepository` trait **does not change**: the entity already carries the
field.)

### 11.4 `src/api/reports/report_service.rs`

The reports service now also depends on `PlaceService`. Add the field, the
existence validation, the decoration and the indexing for the list:

```rust
use crate::api::places::place_dto::PlaceResponse;
use crate::api::places::place_service::PlaceService;
// ...

#[injectable]
pub struct ReportService {
    repository: Arc<dyn ReportRepository>,
    categories: Arc<CategoryService>,
    places: Arc<PlaceService>,
    users: Arc<UserService>,
}

impl ReportService {
    pub async fn create(&self, user_id: &str, dto: CreateReportDto) -> AppResult<ReportResponse> {
        let report = dto.into_entity(user_id.to_string());
        self.validate_category_id(&report.category_id).await?;
        self.validate_place_id(&report.place_id).await?;

        let report = self.repository.insert(report).await?;
        self.decorate(report).await
    }

    pub async fn list(&self) -> AppResult<Vec<ReportResponse>> {
        let reports = self.repository.find_all().await?;
        self.decorate_all(reports).await
    }

    pub async fn update(&self, id: &str, dto: UpdateReportDto) -> AppResult<ReportResponse> {
        let report = dto.into_entity(id.to_string());
        self.validate_category_id(&report.category_id).await?;
        self.validate_place_id(&report.place_id).await?;

        let report = self
            .repository
            .replace(id, report)
            .await?
            .ok_or_else(|| not_found(id))?;
        self.decorate(report).await
    }

    // ... unchanged: get, delete ...

    /// Validates the relationship with places (same logic as categories).
    async fn validate_place_id(&self, place_id: &Option<String>) -> AppResult<()> {
        if let Some(id) = place_id
            && !self.places.exists(id).await?
        {
            return Err(AppError::Validation(format!("place {id} not found")));
        }
        Ok(())
    }

    /// Composes the response by nesting the `category`, `place` and
    /// `created_by` objects.
    async fn decorate(&self, report: Report) -> AppResult<ReportResponse> {
        let category = self.category_of(&report).await?;
        let place = self.place_of(&report).await?;
        let created_by = self.creator_of(&report).await?;
        Ok((report, category, place, created_by).into())
    }

    /// Decorates a whole list resolving the relations with one read per table
    /// (index maps) instead of N+1 queries per element.
    async fn decorate_all(&self, reports: Vec<Report>) -> AppResult<Vec<ReportResponse>> {
        let categories = self.categories.by_id().await?;
        let places = self.places.by_id().await?;
        let users = self.users.by_id().await?;

        Ok(reports
            .into_iter()
            .map(|report| {
                let category = report
                    .category_id
                    .as_ref()
                    .and_then(|id| categories.get(id).cloned());
                let place = report
                    .place_id
                    .as_ref()
                    .and_then(|id| places.get(id).cloned());
                let created_by = report
                    .user_id
                    .as_ref()
                    .and_then(|id| users.get(id).cloned());
                (report, category, place, created_by).into()
            })
            .collect())
    }

    async fn place_of(&self, report: &Report) -> AppResult<Option<PlaceResponse>> {
        match &report.place_id {
            Some(place_id) => self.places.find(place_id).await,
            None => Ok(None),
        }
    }
}
```

The reports controller **does not change** (it keeps calling `service.create`,
etc.); the `#[utoipa::path]` already documents `ReportResponse`, which now also
includes `place`.

### 11.5 Order of the `reports` feature

In `src/api/reports/mod.rs` change the order from `1` to `2`, because `reports`
now also depends on `places`:

```rust
// Zero-touch self-registration in the feature registry.
inventory::submit! {
    Feature::new(2, register)
}
```

### Note on service injection

`ReportService` receives `Arc<PlaceService>` and `Arc<CategoryService>`: do **not**
duplicate the place logic inside reports, and do **not** talk directly to other
features' repositories. The rule is: a service depends on other **services** (not
on their repositories) and the dependencies must form an **acyclic** graph.

---

## 12. Tests (recommended)

Integration tests hit real HTTP with the `mem://` backend. Add
`tests/report/places.rs` and declare it in `tests/report/main.rs`:

```rust
mod places;
```

Minimal example (same style as `categories.rs`):

```rust
//! Place CRUD over HTTP and the report → place relation.

use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;

use crate::support::{app, json_body, json_request};

#[tokio::test]
async fn place_full_crud_cycle() {
    let app = app().await;

    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/places",
            json!({
                "city": "Milan",
                "address": "1 Main Street",
                "cap": "20100",
                "provenance": "City of Milan",
            }),
        ))
        .await
        .unwrap();
    let status = response.status();
    let created = json_body(response).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = created["id"].as_str().expect("id present").to_string();

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/reports",
            json!({
                "name": "Report with place",
                "description": "desc",
                "place_id": id,
            }),
        ))
        .await
        .unwrap();
    let created = json_body(response).await;
    assert_eq!(created["place"]["id"], id.as_str());
    assert!(created.get("place_id").is_none());
}
```

---

## 13. Build and run

Inside the dev container (or on any host with the toolchain):

```bash
# compile everything (binaries, tests, examples)
cargo check --workspace --all-targets

# format, lint, test
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Then start the project and open the documentation: the new `places` endpoints are
already in the Scalar API reference at `http://localhost:3000/scalar` (spec at
`/api-docs/openapi.json`), **without having touched** the generic assembly in
`oxidax-core`.

---

## 13-bis. Summary: checklist

1. `src/api/places/` with the 7 files (entity, dto, repository, surreal, service,
   controller, mod) — §2–§8 (or just run `cargo xtask new-feature`).
- [ ] declare `pub mod places;` in `src/api/mod.rs` (§9)
- [ ] add the repository in `src/db.rs` (`Repositories` + `register_repositories` +
  `from_connection`) (§10)
- [ ] connect `reports` (entity, dto, surreal repo, service, `order`) (§11)
- [ ] if the endpoint is protected, `#[IsGranted]` + `security(...)` in the
  `utoipa::path` (§7-bis)
- [ ] if the endpoint is a paginated list, `find_page`/`count` in the repo +
  `Paged<T>` in the service + `PaginationQuery` in the controller (§7-ter)
- [ ] tests + `cargo test`/`clippy`/`fmt` (§12–13)

To recap the **shared** files touched (and why):

| File                | Change                                 | Reason                                       |
| ------------------- | -------------------------------------- | -------------------------------------------- |
| `src/api/mod.rs`    | `pub mod places;`                      | make the module known to the crate           |
| `src/db.rs`         | field + registration + schema          | aggregate of the concrete repositories       |
| `src/api/reports/*` | `place_id` field + decoration          | report → place relation                      |

---

## 13-ter. Common errors

- **`unresolved module places`**: the `pub mod places;` line in
  `src/api/mod.rs` is missing.
- **`missing #[utoipa::path(...)]`** (error from the `#[route]` macro): you put
  `#[route]` without the `#[utoipa::path]` attribute (or below it: it must be
  **above**).
- **`expected a State<Arc<T>> argument`** (error from the `#[route]` macro): the
  handler has no `State<Arc<...>>` parameter.
- **DI: "service not registered"**: the
  `services.add(PlaceService::singleton())` line is missing in the `register` of
  `places/mod.rs`, or the `Feature::new` was not "collected" (check that you added
  `inventory::submit!`).
- **404 on `/api/places`**: the repository is not in `db::Repositories` (§10
  missing) or the table is not created (`ensure_schema`).
- **unexpected 422**: check the `#[validate(...)]` decorators in the DTO (the
  validation messages are in English).
- **`place` field always `null`**: in the reports service the call to
  `place_of`/`by_id` is missing inside `decorate`/`decorate_all`.
- **`cannot find attribute IsGranted`**: the `#[route]` above `#[IsGranted]` is
  missing (see §7-bis).
