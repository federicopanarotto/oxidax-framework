# Aggiungere una feature — guida passo-passo (esempio: `Places`)

Questa guida spiega, componente per componente, come si aggiunge una nuova
feature all'API. Come esempio concreto costruiamo la feature **`Places`**
(un luogo: `city`, `address`, `cap`, `provenance`) e la colleghiamo alla feature
esistente **`reports`** con una relazione opzionale (un report può puntare a un
place), esattamente come già avviene con `categories`.

> Convenzione del progetto: la prosa dei documenti è in italiano, ma **i
> commenti nel codice e i messaggi di errore/log sono in inglese**.

---

## 0. Modello mentale: i layer di una feature

Una feature è composta sempre dagli stessi pezzi, impilati dal basso verso
l'alto. Ogni layer dipende **solo** da quello sotto (mai il contrario):

```
                 HTTP (axum)
   place_controller.rs   ← handler #[route] + #[utoipa::path]; parla SOLO col service
            │
   place_service.rs      ← logica di dominio; parla col repository (e con altri service)
            │
   place_repository.rs   ← trait: il contratto di persistenza (DB-agnostico)
            │
   surreal_place_repository.rs ← implementazione per SurrealDB (l'unico punto DB-specific)
            │
   place_entity.rs       ← il modello puro che il repository persiste/restituisce
            │
   place_dto.rs          ← i DTO di ingresso/uscita (validazione, serializzazione JSON)
```

Regole d'oro:

- Il **controller** non contiene logica: chiama **una** funzione del service.
- Il **service** è l'unico a orchestrare; se gli servono altri dati, ne parla con
  altri service iniettati via DI (come `ReportService` fa con `CategoryService`).
- Il **repository** è l'unico punto che "conosce" il database.

### Cosa è automatico e cosa è manuale

Grazie al framework (macro `#[route]` + `inventory`) **non tocchi mai**
`features.rs`, `router.rs`, `docs.rs`. Le uniche cose **manuali** quando aggiungi
una feature sono:

1. creare i file della feature;
2. dichiarare il modulo in `src/api/mod.rs` (una riga);
3. registrare il repository in `src/db/mod.rs` (l'aggregatore dei repository: è
   l'unico punto che conosce i repository concreti).

Nella feature, la riga `inventory::submit!` in `mod.rs` auto-registra il wiring
DI; i `#[route]` sugli handler auto-registrano rotte e OpenAPI.

---

## 1. Struttura dei file da creare

Crea la cartella `src/api/places/` con questi file:

```
src/api/places/
├── mod.rs                        # wiring DI + inventory::submit!  (auto-registrazione)
├── place_entity.rs               # entità di dominio/persistenza
├── place_dto.rs                  # DTO di ingresso (validate) e di uscita (response)
├── place_repository.rs           # trait del repository
├── surreal_place_repository.rs   # implementazione SurrealDB del trait
├── place_service.rs              # logica di dominio
└── place_controller.rs           # handler HTTP con #[route] + #[utoipa::path]
```

Poi si modificano due file condivisi (`src/api/mod.rs`, `src/db/mod.rs`) e la
feature `reports` per aggiungere la relazione.

---

## 2. L'entità — `src/api/places/place_entity.rs`

L'**entity** è il modello puro, indipendente dal database: qualunque repository
(che sia SurrealDB, Postgres, un mock in memoria…) deve persistere e restituire
esattamente questo tipo. Non contiene né validazione né logica.

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

Perché separarla dai DTO? Perché il DTO descrive ciò che **entra** dall'HTTP (con
le sue regole di validazione), mentre l'entity descrive ciò che vive nel
database. Tenerle separate evita di far "trapelare" dettagli HTTP nel dominio e
viceversa.

---

## 3. I DTO e la validazione — `src/api/places/place_dto.rs`

Qui vivono tre tipi:

- `CreatePlaceDto` / `UpdatePlaceDto`: cosa arriva nel body (con i decoratori
  `#[validate(...)]`);
- `PlaceResponse`: cosa torna al client;
- la conversione DTO → entity (`into_entity`) e entity → response (`From`).

I campi sono **privat**? No: in questo progetto i DTO hanno campi `pub` (comodi
per i test e per i service); la validazione avviene prima, nell'estrattore
`ValidatedJson`.

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::places::place_entity::Place;
use crate::lib::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreatePlaceDto {
    /// City name
    #[schema(example = "Milano", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The city is required and cannot be empty"
        ),
        length(max = 200, message = "The city must be at most 200 characters")
    )]
    pub city: String,

    /// Street address
    #[schema(example = "Via Roma 1", min_length = 1, max_length = 300)]
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
    #[schema(example = "Comune di Milano", min_length = 1, max_length = 200)]
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
    #[schema(example = "Torino", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The city is required and cannot be empty"
        ),
        length(max = 200, message = "The city must be at most 200 characters")
    )]
    pub city: String,

    /// New street address
    #[schema(example = "Corso Francia 10", min_length = 1, max_length = 300)]
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
    #[schema(example = "Comune di Torino", min_length = 1, max_length = 200)]
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
    #[schema(example = "Milano")]
    pub city: String,
    /// Street address
    #[schema(example = "Via Roma 1")]
    pub address: String,
    /// Postal code
    #[schema(example = "20100")]
    pub cap: String,
    /// Provenance
    #[schema(example = "Comune di Milano")]
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

Punti chiave:

- `#[derive(Validate)]` + `#[validate(...)]` sono ciò che l'estrattore
  `ValidatedJson` esegue **prima** che l'handler venga chiamato: se falliscono,
  risponde `422 {error, details}` senza mai entrare nel service.
- `not_blank` è un validator condiviso (`crate::lib::validators`): rifiuta
  stringhe vuote o fatte solo di spazi.
- `into_entity` normalizza (`.trim()`) e genera l'`id`: così il service riceve
  dati già pronti.
- `ToSchema` rende il DTO visibile in Swagger.

---

## 4. Il repository: trait — `src/api/places/place_repository.rs`

Il **trait** è il contratto di persistenza. È l'unico punto che il service
conosce del database: cambiare backend = implementare questo trait, senza
toccare service e controller.

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

Note:

- `#[async_trait]` permette i metodi `async` nei trait (in Rust sono "nativi" solo
  con GAT; `async-trait` è il compromesso usato qui).
- `Send + Sync` è richiesto perché il repository vive in un `Arc` condiviso tra i
  task di tokio.
- I metodi restituiscono `AppResult<...>`: gli errori DB vengono convertiti in
  `AppError` centralmente (vedi `impl From<surrealdb::Error> for AppError`).

---

## 5. Il repository: implementazione SurrealDB — `src/api/places/surreal_place_repository.rs`

Qui c'è **l'unico codice DB-specific** della feature. Struttura tipica (identica
a `surreal_category_repository.rs`):

- una costante `TABLE`;
- un `...Row`: come i dati arrivano dal DB (l'`id` è un `RecordId` di SurrealDB);
- un `...Content`: cosa si scrive nel `CONTENT` (senza `id`, gestito dal DB);
- `new`, `ensure_schema`, una `into_entity` (record id → `uuid` dell'API) e i
  metodi del trait.

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

Punti chiave:

- `define_table` (helper in `src/db/surreal.rs`) esegue
  `DEFINE TABLE IF NOT EXISTS place SCHEMALESS`: in SurrealDB 3 le tabelle non
  nascono automaticamente.
- `record_key_to_string` converte il record id di SurrealDB nella stringa usata
  dall'API.
- L'`?` sugli errori di SurrealDB sfrutta la conversione automatica in
  `AppError::Database` (status `503`).

---

## 6. Il service — `src/api/places/place_service.rs`

Il **service** è dove vive la logica di dominio. Qui però non c'è molta logica:
i DTO arrivano già validati e convertiti, quindi il service si limita a
orchestrare il repository e a tradurre l'assenza in un errore di dominio. Il
`#[injectable]` di `more-di` genera il costruttore che risolve i campi dal
container.

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

Nota bene: i tre metodi "extra" (`exists`, `find`, `by_id`) esistono **solo** per
permettere alla feature `reports` di validare e comporre la relazione senza
duplicare query. È il modo corretto di far parlare due feature: un service
espone metodi, l'altro service li chiama.

---

## 7. Il controller — `src/api/places/place_controller.rs`

Il **controller** è solo trasporto: niente logica, niente `if` di dominio.
Ogni handler ha due attributi:

- `#[route]`: **auto-registra** la rotta e il frammento OpenAPI (è la macro di
  `framework-macros`); deve stare **sopra** `#[utoipa::path]`.
- `#[utoipa::path(...)]`: dichiara metodo, path, tag e risposte documentate.

L'handler prende il service con `State<Arc<PlaceService>>` (l'unico service che
gli serve) e, quando c'è un body, `ValidatedJson<...>`.

```rust
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use crate::api::places::place_dto::{CreatePlaceDto, PlaceResponse, UpdatePlaceDto};
use crate::api::places::place_service::PlaceService;
use crate::errors::{AppResult, BadRequestResponse, ErrorResponse};
use crate::framework::{ValidatedJson, ValidationErrorResponse, route};

// Route table and OpenAPI spec of the places, defined point-inline: every
// handler is annotated with `#[utoipa::path(...)]` (HTTP method, path and
// responses) and `#[route]`, which self-registers it in the static registry.
// `features.rs`/`router.rs`/`docs.rs` never change when a handler is added.

#[route]
#[utoipa::path(
    post,
    path = "/api/places",
    tag = "places",
    request_body = CreatePlaceDto,
    responses(
        (status = 201, description = "Place created successfully", body = PlaceResponse),
        (status = 400, description = "Invalid request: malformed JSON body", body = BadRequestResponse),
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
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
        (status = 422, description = "Field validation failed", body = ValidationErrorResponse),
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

Punti chiave:

- Niente import di `Router`/`OpenApiRouter`/`routes!`: li mette la macro.
- `Path(id): Path<String>` per i parametri di route; `ValidatedJson` per il body.
- Gli import vengono dalla **facade** `crate::framework` (estrattori, `route`,
  tipi di errore), così i controller di tutte le feature sono uniformi.
- Il `tag` raggruppa gli endpoint in Swagger UI.

---

## 8. Il wiring della feature — `src/api/places/mod.rs`

Questo file fa **due sole cose**: (1) dichiara i sottomoduli; (2) registra la
feature nel container DI tramite `inventory::submit!`. È l'unica riga "manuale"
di DI della feature.

```rust
pub mod place_controller;
pub mod place_dto;
pub mod place_entity;
pub mod place_repository;
pub mod place_service;
pub mod surreal_place_repository;

use di::Injectable;
use di::ServiceCollection;

use crate::db;
use crate::framework::{Feature, register_repository};
use place_service::PlaceService;

// All the DI wiring lives here; every route is auto-registered by `#[route]`
// on the handlers in `place_controller`.
fn register(repositories: &db::Repositories, services: &mut ServiceCollection) {
    register_repository(services, repositories.places.clone());
    services.add(PlaceService::singleton());
}

// Zero-touch self-registration in the feature registry.
inventory::submit! {
    Feature::new(1, register)
}
```

Spiegazione:

- `register_repository` registra il repository concreto **sotto il suo trait**
  (`Arc<dyn PlaceRepository>`), così il service dipende dal contratto, non
  dall'implementazione.
- `PlaceService::singleton()` registra il service (costruito da `#[injectable]`).
- `Feature::new(order, register)`: `order` è l'ordine di registrazione. Le
  feature esistenti usano `categories = 0`, `reports = 1`.
  **Mettiamo `places = 1` e spostiamo `reports = 2`** (vedi §11), così i
  repository/service usati dai report sono registrati prima. In pratica
  `more-di` inietta in modo lazy (l'ordine è tollerante), ma tenerlo esplicito
  è più sicuro.
- L'`inventory::submit!` è ciò che rende la feature "zero-touch":
  `features.rs` la trova da solo al prossimo avvio, senza modifiche.

---

## 9. Dichiarare il modulo — `src/api/mod.rs` (modifica manuale 1)

Aggiungi una riga, in ordine alfabetico:

```rust
pub mod categories;
pub mod docs;
pub mod features;
pub mod places; // <-- nuovo
pub mod reports;
pub mod router;
```

Puoi vedere la necessità di questa riga come il prezzo da pagare per non dover
toccare `features.rs`/`router.rs`: Rust deve sapere che esiste il modulo.

---

## 10. Registrare il repository — `src/db/mod.rs` (modifica manuale #2)

`db::Repositories` è l'aggregatore dei repository concreti. È l'unico punto
condiviso che va aggiornato, perché è lui che viene passato a ogni
`Feature::register`. Nuova versione:

```rust
mod surreal;

use std::sync::Arc;

use surrealdb::Surreal;
use surrealdb::engine::any::Any;

use crate::api::categories::category_repository::CategoryRepository;
use crate::api::categories::surreal_category_repository::SurrealCategoryRepository;
use crate::api::places::place_repository::PlaceRepository;
use crate::api::places::surreal_place_repository::SurrealPlaceRepository;
use crate::api::reports::report_repository::ReportRepository;
use crate::api::reports::surreal_report_repository::SurrealReportRepository;
use crate::config::{Config, DbBackend};
use crate::errors::AppResult;

pub use surreal::{define_table, record_key_to_string};

/// All the application repositories, built on the same connection.
pub struct Repositories {
    pub reports: Arc<dyn ReportRepository>,
    pub categories: Arc<dyn CategoryRepository>,
    pub places: Arc<dyn PlaceRepository>,
}

/// Creates the repositories based on the configured backend.
///
/// Switching database means implementing the `ReportRepository` and
/// `CategoryRepository` traits and adding a new branch here: the rest of the
/// application (services, controllers) does not change.
pub async fn repositories(config: &Config) -> AppResult<Repositories> {
    let url = match config.db_backend {
        DbBackend::Memory => "mem://",
        DbBackend::Surreal => config.surreal_url.as_str(),
    };

    from_connection(surreal::connect(url, config).await?).await
}

/// Embedded in-memory repositories (SurrealDB `mem://`): used by the tests and
/// available at runtime with `DB_BACKEND=memory`.
pub async fn memory_repositories() -> AppResult<Repositories> {
    repositories(&Config::memory()).await
}

async fn from_connection(db: Surreal<Any>) -> AppResult<Repositories> {
    let reports = SurrealReportRepository::new(db.clone());
    reports.ensure_schema().await?;

    let categories = SurrealCategoryRepository::new(db.clone());
    categories.ensure_schema().await?;

    let places = SurrealPlaceRepository::new(db);
    places.ensure_schema().await?;

    Ok(Repositories {
        reports: Arc::new(reports),
        categories: Arc::new(categories),
        places: Arc::new(places),
    })
}
```

Punti chiave:

- Ogni repository condivide la **stessa connessione** (`db.clone()`); l'ultimo
  può prenderla "owned" (`db`).
- `ensure_schema()` va chiamato per ogni nuovo repository: crea la tabella
  `place` in SurrealDB.
- Questo è, di fatto, l'unico punto "non zero-touch" delle feature. Se in
  futuro vorrai eliminarlo, il passo è rendere `Repositories` una mappa
  dinamica (es. per `TypeId`) popolata dalle feature stesse.

---

## 10-bis. Dichiarare il modulo `places` anche per la collection

Ricordati che va dichiarato `pub mod places;` in `src/api/mod.rs` (§9) **prima**
che `src/db/mod.rs` possa riferirsi a `crate::api::places::...`. Se compili dopo
aver fatto solo §8, l'errore sarà "unresolved module `places`": è normale,
manca §9.

---

## 11. La relazione: `reports` → `places`

Un report può ora puntare a un place. Il modello è **lo stesso** già usato per
`categories`: un campo opzionale `place_id` nell'entity, validato a livello DTO
(formato) e service (esistenza), e l'oggetto completo annidato nella risposta.
La risposta espone **solo** l'oggetto annidato `place` (non `place_id`), come già
fatto per `category`.

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

Aggiungi a **`CreateReportDto`** e **`UpdateReportDto`** un campo `place_id`
identico (per forma) a `category_id`:

```rust
    /// Id of the associated place (optional; `null` or absent removes the relationship)
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    #[validate(custom(
        function = "crate::lib::validators::validate_uuid",
        message = "The place id must be a valid UUID"
    ))]
    pub place_id: Option<String>,
```

Nel rispettivo `into_entity`, aggiungi `place_id: self.place_id,`.

Nella **`ReportResponse`** aggiungi l'oggetto annidato (non l'id):

```rust
    /// Associated category (full object, present only if the relationship exists)
    pub category: Option<CategoryResponse>,
    /// Associated place (full object, present only if the relationship exists)
    pub place: Option<PlaceResponse>,
```

Aggiungi l'import in cima al file:

```rust
use crate::api::places::place_dto::PlaceResponse;
```

E sostituisci l'`impl From` con la versione a tre elementi:

```rust
impl
    From<(
        crate::api::reports::report_entity::Report,
        Option<CategoryResponse>,
        Option<PlaceResponse>,
    )> for ReportResponse
{
    fn from(
        (report, category, place): (
            crate::api::reports::report_entity::Report,
            Option<CategoryResponse>,
            Option<PlaceResponse>,
        ),
    ) -> Self {
        Self {
            id: report.id,
            name: report.name,
            description: report.description,
            category,
            place,
        }
    }
}
```

### 11.3 `src/api/reports/surreal_report_repository.rs`

Aggiungi `place_id: Option<String>` a `ReportRow` **e** `ReportContent`, poi:

- in `into_entity`: `place_id: row.place_id,`
- in `insert` e `replace`, dentro `ReportContent`: `place_id: report.place_id.clone(),`

(Il trait `ReportRepository` **non cambia**: l'entity porta già il campo.)

### 11.4 `src/api/reports/report_service.rs`

Il service dei report ora dipende anche da `PlaceService`. Aggiungi il campo,
la validazione dell'esistenza, la decorazione e l'indicizzazione per la lista:

```rust
use crate::api::places::place_dto::PlaceResponse;
use crate::api::places::place_service::PlaceService;
// ...

#[injectable]
pub struct ReportService {
    repository: Arc<dyn ReportRepository>,
    categories: Arc<CategoryService>,
    places: Arc<PlaceService>,
}

impl ReportService {
    pub async fn create(&self, dto: CreateReportDto) -> AppResult<ReportResponse> {
        let report = dto.into_entity();
        self.validate_category_id(&report.category_id).await?;
        self.validate_place_id(&report.place_id).await?;

        let report = self.repository.insert(report).await?;
        self.decorate(report).await
    }

    pub async fn list(&self) -> AppResult<Vec<ReportResponse>> {
        let reports = self.repository.find_all().await?;
        let categories = self.categories.by_id().await?;
        let places = self.places.by_id().await?;

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
                (report, category, place).into()
            })
            .collect())
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

    // ... invariate: get, delete ...

    /// Validates the relationship with places (same logic as categories).
    async fn validate_place_id(&self, place_id: &Option<String>) -> AppResult<()> {
        if let Some(id) = place_id
            && !self.places.exists(id).await?
        {
            return Err(AppError::Validation(format!("place {id} not found")));
        }
        Ok(())
    }

    /// Composes the response by nesting the `category` and `place` objects.
    async fn decorate(&self, report: Report) -> AppResult<ReportResponse> {
        let category = self.category_of(&report).await?;
        let place = self.place_of(&report).await?;
        Ok((report, category, place).into())
    }

    async fn place_of(&self, report: &Report) -> AppResult<Option<PlaceResponse>> {
        match &report.place_id {
            Some(place_id) => self.places.find(place_id).await,
            None => Ok(None),
        }
    }
}
```

Il controller dei report **non cambia** (continua a chiamare `service.create`,
ecc.); il `#[utoipa::path]` già documenta `ReportResponse`, che ora include
anche `place`.

### 11.5 Ordine della feature `reports`

In `src/api/reports/mod.rs` cambia l'ordine da `1` a `2`, perché ora `reports`
dipende anche da `places`:

```rust
// Zero-touch self-registration in the feature registry.
inventory::submit! {
    Feature::new(2, register)
}
```

### Nota sull'iniezione dei service

`ReportService` riceve `Arc<PlaceService>` e `Arc<CategoryService>`: **non**
duplicare la logica dei place nei report, e **non** parlare direttamente dei
repository altrui. La regola è: un service dipende da altri **service** (non dai
loro repository) e le dipendenze devono formare un grafo **aciclico**.

---

## 12. Test (consigliati)

I test di integrazione colpiscono l'HTTP reale con backend `mem://`. Aggiungi
`tests/report/places.rs` e dichiaralo in `tests/report/main.rs`:

```rust
mod places;
```

Esempio minimo (stesso stile di `categories.rs`):

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
                "city": "Milano",
                "address": "Via Roma 1",
                "cap": "20100",
                "provenance": "Comune di Milano",
            }),
        ))
        .await
        .unwrap();
    let status = response.status();
    let created = json_body(response).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let id = created["id"].as_str().expect("id presente").to_string();

    let response = app
        .oneshot(json_request(
            "POST",
            "/api/reports",
            json!({
                "name": "Report con luogo",
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

## 13. Verifica e avvio

```bash
# compila tutto (binari, test, esempi)
docker exec -u vscode -w /workspace axum-api_devcontainer-app-1 cargo check --all-targets

# format, lint, test
docker exec -u vscode -w /workspace axum-api_devcontainer-app-1 cargo fmt
docker exec -u vscode -w /workspace axum-api_devcontainer-app-1 cargo clippy --all-targets -- -D warnings
docker exec -u vscode -w /workspace axum-api_devcontainer-app-1 cargo test
```

Poi avvia e apri la documentazione: i nuovi endpoint `places` saranno già in
Swagger UI su `http://localhost:3000/swagger-ui` (spec a
`/api-docs/openapi.json`), **senza aver toccato** `features.rs`, `router.rs` o
`docs.rs`.

---

## 13-bis. Riepilogo: checklist

1. `src/api/places/` con i 7 file (entity, dto, repository, surreal, service,
   controller, mod) — §2–§8.
- [ ] dichiara `pub mod places;` in `src/api/mod.rs` (§9)
- [ ] aggiungi il repository in `src/db/mod.rs` (`Repositories` + `from_connection`) (§10)
- [ ] collega `reports` (entity, dto, surreal repo, service, `order`) (§11)
- [ ] test + `cargo test`/`clippy`/`fmt` (§12–13)

Ricapitolando i file **condivisi** toccati (e perché):

| File                 | Modifica                        | Motivo                                              |
| -------------------- | ------------------------------- | --------------------------------------------------- |
| `src/api/mod.rs`     | `pub mod places;`               | far conoscere il modulo al crate                    |
| `src/db/mod.rs`      | campo + costruzione + schema    | aggregatore dei repository concreti                 |
| `src/api/reports/*`  | campo `place_id` + decorazione  | relazione report → place                            |

---

## 13-ter. Errori comuni

- **`unresolved module places`**: manca `pub mod places;` in `src/api/mod.rs`.
- **`missing #[utoipa::path(...)]`** (errore della macro `#[route]`): hai messo
  `#[route]` senza l'attributo `#[utoipa::path]` (oppure sotto di esso: deve
  stare **sopra**).
- **`expected a State<Arc<T>> argument`** (errore della macro `#[route]`):
  l'handler non ha il parametro `State<Arc<...>>`.
- **DI: "service not registered"**: manca `services.add(PlaceService::singleton())`
  nel `register` di `places/mod.rs`, oppure il `Feature::new` non è stato
  "raccolto" (controlla di aver messo `inventory::submit!`).
- **404 su `/api/places`**: il repository non è in `db::Repositories` (manca §10)
  o la tabella non viene creata (`ensure_schema`).
- **422 inatteso**: controlla i decoratori `#[validate(...)]` nel DTO (i
  messaggi di validazione sono in inglese).
- **Campo `place` sempre `null`**: nel service dei report manca la chiamata a
  `place_of`/`by_id` dentro `decorate`/`list`.