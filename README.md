# axum-api — un framework personale per API in Rust

> **Progetto sperimentale.** Questo repository è un "campo di prova" dove sto
> costruendo, pezzo per pezzo, un **mio** framework personale per scrivere API
> in Rust. L'obiettivo non è inventare tutto da zero, ma portare in Rust lo
> stile di sviluppo che mi trovavo bene con **Express / Node**, approfittando
> della velocità e della sicurezza di Rust e di un database performante come
> **SurrealDB**.

## Filosofia

1. **Riusare le librerie esistenti.** In Rust esistono già ecosistemi maturi per
   praticamente ogni tassello di un'API: routing, gestione errori, validazione,
   documentazione OpenAPI, dependency injection. Il framework sceglie le
   migliori e le *combina* in una convenzione di progetto comoda, invece di
   riscriverle.
2. **Zero-touch.** Aggiungere una feature (= un dominio dell'API) significa
   solo creare i suoi moduli. I file di "cablaggio" (`features.rs`, `router.rs`,
   `docs.rs`) sono generici e **non si toccano mai**.
3. **Stile decorator.** Gli handler si dichiarano come funzioni con attributi:
   la rotta e la documentazione si scrivono "sulla" funzione (`#[route]` +
   `#[utoipa::path]`), i DTO si validano con decoratori (`#[validate]` dentro
   `ValidatedJson<...>`), esattamente come con i decoratori a cui ero abituato.

## Stack

| Tassello      | Crate scelto                                       | Perché                                                       |
| ------------- | -------------------------------------------------- | ------------------------------------------------------------ |
| HTTP/rotta    | [axum](https://github.com/tokio-rs/axum)           | performante, macro-free, ecosistema tower                    |
| OpenAPI/UI    | [utoipa](https://github.com/juhaku/utoipa) + `utoipa-axum` + `utoipa-swagger-ui` | decoratori `#[utoipa::path]`, Swagger UI pronta               |
| Rotta self-registrata | `framework-macros` (mia proc-macro `#[route]`) + [inventory](https://github.com/dtolnay/inventory) | discovery statica delle rotte a compile-time                  |
| Validazione   | [validator](https://github.com/Keats/validator)    | decoratori di validazione già maturi                          |
| DI            | [more-di](https://github.com/AndyShi-more/MoreDi)  | iniezione lazy + `ServiceProvider`                            |
| DB            | [SurrealDB](https://surrealdb.com) + `surrealdb`   | database performante, `mem://` per i test, schema-less        |
| Auth          | `jsonwebtoken` + `argon2`                           | JWT HS256 (access/refresh) + hash password Argon2id           |
| Errori        | `thiserror` + pipeline unificata                   | contratto di errore uniforme in tutta l'API                   |

## Struttura del progetto

```
src/
├── api/                      # le feature dell'API
│   ├── features.rs           # generico: registra le feature nel container DI
│   ├── router.rs             # generico: assembla il router finale + Swagger UI
│   ├── docs.rs               # generico: merge degli spec OpenAPI delle feature
│   └── categories/           # feature "categories"
│       ├── mod.rs            # wiring DI (repository + service) + registrazione
│       ├── category_dto.rs   # DTO presenti in body/query/params
│       ├── category_entity.rs# modello di persistenza
│       ├── category_repository.rs   # trait del repository
│       ├── surreal_category_repository.rs  # implementazione per SurrealDB
│       ├── category_service.rs       # logica di dominio
│       └── category_controller.rs    # gli handler HTTP con #[route]
│   ├── places/               # feature "places" (city, address, cap, provenance)
│   ├── reports/              # feature "reports" (dipende da categories, places e users)
│   ├── users/                # feature "users" (/api/users/me, lista admin)
│   └── auth/                 # feature "auth" (register/login/refresh/logout)
├── lib/                      # il "core" del framework
│   ├── registry.rs           # registri statici Feature (DI) e Route
│   ├── validators.rs         # estrattori ValidatedJson/Query/Params
│   ├── auth/                 # auth core: JWT, password, identità, guard/extractor
│   └── middlewares/          # middleware estraibili, stile Express
├── errors/                   # contratto errori + pipeline HTTP
├── framework.rs              # facade: import unico (`crate::framework::*`) per le feature
├── config.rs                 # configurazione da env
└── db/                       # connessione SurrealDB + aggregatore repository
framework-macros/             # la proc-macro #[route]
tests/report/                 # integration test end-to-end (HTTP reali)
```

### Come nasce una feature

Una nuova feature è *solo* il suo modulo. Presa "categories":

1. **`mod.rs`** — il wiring DI (l'unica riga "manuale" di tutto il framework):
   ```rust
   fn register(repositories: &db::Repositories, services: &mut ServiceCollection) {
       register_repository(services, repositories.categories.clone());
       services.add(CategoryService::singleton());
   }

   inventory::submit! { Feature::new(0, register) }
   ```
   L'`order` serve a garantire l'ordine di registrazione quando una feature
   dipende da un'altra (reports → categories).

2. **`category_controller.rs`** — gli handler: niente oggetto controller, solo
   funzioni con due attributi:
   ```rust
   #[route]                                    // auto-registra rotta + OpenAPI
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

3. **`category_dto.rs`** — i DTO con i decoratori di validazione:
   ```rust
   #[derive(Debug, Serialize, Deserialize, ToSchema)]
   #[serde(deny_unknown_fields)]
   pub struct CreateCategoryDto {
       #[validate(length(min = 3, max = 64, message = "Il nome è obbligatorio e deve avere almeno 3 caratteri"))]
       #[schema(max_length = 64)]
       pub name: String,
   }
   ```

Nessuna altra modifica: `features.rs`, `router.rs`, `docs.rs` iterano i
registri statici e raccolgono automaticamente rotte e specifica OpenAPI.

### Proteggere un handler

Un handler autenticato (o riservato a dei ruoli) aggiunge un attributo,
consumato da `#[route]`:

```rust
#[route]                                  // solo utenti autenticati
#[utoipa::path(get, path = "/api/users/me", ...)]
#[IsGranted]
pub async fn get_me(service: ..., current: CurrentUser) -> ... { ... }

#[route]                                  // solo ruoli ammessi
#[IsGranted(["admin"])]
pub async fn list_users(...) -> ... { ... }
```

La macro genera il layer con la guardia (`401` senza token valido, `403` col
ruolo sbagliato) e l'estrattore `CurrentUser` espone `user_id`, `identity_id`
e `role`: anche l'auth resta zero-touch, senza layer globali da configurare.

## Contratto delle risposte

- **Successo**: corpo JSON del DTO di risposta (o `204`).
- **Errore di dominio/pipeline**: sempre `{code, error, message}` con status
  HTTP significativo (`not_found`, `database_unavailable`, ...).
- **Errore di validazione**: sempre `{error, details}` con `422`; body/query
  malformati danno `400`. Il campo `details` elenca i problemi campo per campo,
  formato "Gentile e professionale" pensato anche per l'utente finale.

## Autenticazione

JWT firmati in HS256 (`JWT_SECRET`) con due token distinti: un access token
breve e un refresh token lungo e **monouso** (rotato a ogni refresh, bruciato
dal logout).

| Endpoint                 | Effetto                                                             |
| ------------------------ | ------------------------------------------------------------------- |
| `POST /api/auth/register` | `201 {message}`: nessun token; l'email è l'username (`role = user`) |
| `POST /api/auth/login`   | `200 {access_token, refresh_token, token_type, expires_in}`         |
| `POST /api/auth/refresh` | nuovo pair; il vecchio refresh token smette di funzionare           |
| `POST /api/auth/logout`  | `204`: azzera il `jti` → il refresh token outstandig è revocato     |
| `GET /api/users/me`      | profilo del chiamante con email                                      |
| `GET /api/users`         | lista utenti con email, solo `admin`                                 |

Errori **generici** per design: email ignota e password sbagliata danno lo
stesso `401 invalid credentials`, il doppio register dà `409` generico (niente
enumerazione account). Il primo admin si assegna direttamente sul database.
In Swagger i due scheme `accessToken`/`refreshToken` (bearer JWT) sono pronti
da usare con "Authorize".

## Documentazione

La UI Swagger è servita, live, a **`/swagger-ui`** (spec completa a
`/api-docs/openapi.json`). Niente file di export: è il framework che compone lo
spec OpenAPI a partire dalle macro mentre monta le rotte.

## Configurazione

La configurazione arriva **solo** da variabili d'ambiente: nessun valore è
hardcoded nel codice. In locale i valori sono letti dal file `.env`
(git-ignorato) tramite `dotenvy`; le variabili già presenti nell'ambiente hanno
precedenza su quelle del file.

```bash
cp .env.example .env   # template con tutte le variabili richieste
```

Variabili richieste: `SERVER_ADDR`, `DB_BACKEND` (`surreal` o `memory`),
`SURREAL_URL`, `SURREAL_NS`, `SURREAL_DB`, `SURREAL_USER`, `SURREAL_PASS`,
`JWT_SECRET`, `ACCESS_TOKEN_DURATION` (secondi), `REFRESH_TOKEN_DURATION`
(secondi). Se una variabile manca l'avvio fallisce con un messaggio esplicito.

## Avvio

```bash
# Dev container (SurrealDB + toolchain inclusi)
docker compose -f .devcontainer/docker-compose.yml up -d

# Backend in-memory, senza SurrealDB esterno
DB_BACKEND=memory cargo run

# Backend su SurrealDB (valori dal .env)
cargo run
```

## Test

```bash
cargo test          # integration test end-to-end su HTTP reali (34 test)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Stato

- [x] Routing + DI + OpenAPI con discovery **zero-touch** (`#[route]` su ogni
      handler, registri `inventory`)
- [x] Validazione decorator su body/query/params con errori `422 {error, details}`
- [x] Pipeline errori uniforme + fallback 404
- [x] Feature di esempio: `categories`, `reports` e `places` su SurrealDB
      (`mem://` nei test)
- [x] Swagger UI live
- [x] Access log (middleware esterno) + autenticazione JWT con guard
      **per-rotta** (`#[IsGranted]`, ruoli inclusi)
- [x] Report con creatore: `POST /api/reports` autenticato, l'utente del token
      viene salvato e restituito come `created_by` nelle letture
- [ ] Paginazione/ordinamento e altre utilità da riusare tra le feature

> **Nota**: il framework è in costruzione e serve a imparare: l'API pubblica
> delle macro e la struttura dei registri possono cambiare.