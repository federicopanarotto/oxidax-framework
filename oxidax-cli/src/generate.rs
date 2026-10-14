//! `cargo xtask new-feature`: interactive code generator.
//!
//! It asks for a feature name and its fields (name + type selected from a
//! menu), then scaffolds a complete, working CRUD feature following the same
//! conventions of the built-in features:
//!
//! - `src/api/<feature>/` created with entity, DTOs, repository (trait),
//!   SurrealDB repository, service and controller; the module `mod.rs` does
//!   the zero-touch DI registration of the service.
//! - `src/api/mod.rs` gets the new `pub mod <feature>;`.
//! - `src/db.rs` gets the new repository wired in `Repositories`, in
//!   `from_connection` (imports + schema + construction) and registered in
//!   `register_repositories`.
//! - `tests/report/<feature>.rs` gets a full CRUD test, registered in
//!   `tests/report/main.rs`.
//!
//! Complex or custom logic (relations, custom validation, extra endpoints) is
//! intentionally out of scope: the scaffolding is meant to be extended, and
//! opencode can understand both the generated code and the cli scripts to
//! evolve it.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use super::ExitCode;

/// Supported scalar types of an entity field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldType {
    Str,
    Bool,
    Int,
    Uint,
    Float,
    StrList,
}

impl FieldType {
    const ALL: [Self; 6] = [
        Self::Str,
        Self::Bool,
        Self::Int,
        Self::Uint,
        Self::Float,
        Self::StrList,
    ];

    /// Rust type name used in the entity and DTOs.
    fn rust(self) -> &'static str {
        match self {
            Self::Str => "String",
            Self::Bool => "bool",
            Self::Int => "i64",
            Self::Uint => "u64",
            Self::Float => "f64",
            Self::StrList => "Vec<String>",
        }
    }

    /// Whether the type supports the `Eq` derive (`f64` does not).
    fn is_eq(self) -> bool {
        !matches!(self, Self::Float)
    }

    fn label(self) -> &'static str {
        match self {
            Self::Str => "String",
            Self::Bool => "bool",
            Self::Int => "i64 (integer)",
            Self::Uint => "u64 (positive integer)",
            Self::Float => "f64 (decimal)",
            Self::StrList => "Vec<String> (list of strings)",
        }
    }

    /// JSON literal used as a sample value in the generated test.
    fn sample(self) -> &'static str {
        match self {
            Self::Str => "\"sample\"",
            Self::Bool => "true",
            Self::Int => "1",
            Self::Uint => "1",
            Self::Float => "1.5",
            Self::StrList => "[\"a\", \"b\"]",
        }
    }

    /// Selects from a numbered menu; invalid input reprompts.
    fn prompt() -> Result<Self, ExitCode> {
        loop {
            println!("  field type:");
            for (index, option) in Self::ALL.iter().enumerate() {
                println!("    {} = {}", index + 1, option.label());
            }
            let input = prompt("  > ")?;
            match input.trim().parse::<usize>() {
                Ok(index) if (1..=Self::ALL.len()).contains(&index) => {
                    return Ok(Self::ALL[index - 1]);
                }
                _ => eprintln!("  invalid choice, pick a number from the list"),
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Field {
    name: String,
    ty: FieldType,
}

/// Scaffolds a complete CRUD feature interactively.
pub fn new_feature() -> ExitCode {
    let name = match prompt_feature_name() {
        Ok(name) => name,
        Err(code) => return code,
    };
    let pascal = to_pascal(&name);
    let plural = pluralize(&name);

    println!("Fields (empty name = done).");
    let mut fields = Vec::new();
    let add_fields = match prompt_bool("Add fields now? [Y/n] ") {
        Ok(value) => value,
        Err(code) => return code,
    };
    if add_fields {
        loop {
            let raw = match prompt("field name (snake_case, empty = finish) > ") {
                Ok(value) => value,
                Err(code) => return code,
            };
            let field_name = raw.trim().to_string();
            if field_name.is_empty() {
                break;
            }
            if !is_valid_field_name(&field_name) {
                eprintln!("  invalid field name: only lowercase letters and underscores");
                continue;
            }
            let ty = match FieldType::prompt() {
                Ok(ty) => ty,
                Err(code) => return code,
            };
            fields.push(Field {
                name: field_name,
                ty,
            });
            println!("  added {}: {}", fields.last().unwrap().name, ty.rust());
        }
    }

    if fields.is_empty() {
        eprintln!("at least one field is required before generating");
        return ExitCode::from(2);
    }

    generate(&name, &pascal, &plural, &fields)
}

fn prompt_feature_name() -> Result<String, ExitCode> {
    loop {
        let raw = prompt("feature name (snake_case, singular) > ")?;
        let name = raw.trim().to_string();
        if is_valid_feature_name(&name) {
            return Ok(name);
        }
        eprintln!(
            "  invalid name: only lowercase letters and underscores, no leading/trailing '_'"
        );
    }
}

fn is_valid_feature_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('_')
        && !name.ends_with('_')
        && name.chars().all(|c| c.is_ascii_lowercase() || c == '_')
}

fn is_valid_field_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c == '_')
}

/// Reads a line from stdin.
fn prompt(question: &str) -> Result<String, ExitCode> {
    let mut stdout = io::stdout();
    stdout.write_all(question.as_bytes()).map_err(|err| {
        eprintln!("cannot write to stdout: {err}");
        ExitCode::from(2)
    })?;
    stdout.flush().map_err(|err| {
        eprintln!("cannot flush stdout: {err}");
        ExitCode::from(2)
    })?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|err| {
        eprintln!("cannot read stdin: {err}");
        ExitCode::from(2)
    })?;
    Ok(input.trim_end_matches(['\n', '\r']).to_string())
}

/// Yes/no prompt, `n` as default (only when the answer starts with `y`).
fn prompt_bool(question: &str) -> Result<bool, ExitCode> {
    let input = prompt(question)?;
    Ok(input.trim().starts_with(['y', 'Y']))
}

fn to_pascal(snake: &str) -> String {
    snake
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn pluralize(singular: &str) -> String {
    if singular.ends_with('s') {
        singular.to_string()
    } else {
        format!("{singular}s")
    }
}

/// Writes the whole feature plus the wiring edits.
fn generate(name: &str, pascal: &str, plural: &str, fields: &[Field]) -> ExitCode {
    let api_dir = Path::new("src/api").join(name);
    if api_dir.exists() {
        eprintln!("feature `{name}` already exists at {}", api_dir.display());
        return ExitCode::from(2);
    }
    if !Path::new("src/api/mod.rs").exists() || !Path::new("src/db.rs").exists() {
        eprintln!("run `cargo xtask new-feature` from the repository root");
        return ExitCode::from(2);
    }

    fs::create_dir_all(&api_dir).expect("create feature directory");

    let entity_file = format!("{name}_entity.rs");
    let repo_file = format!("{name}_repository.rs");
    let surreal_file = format!("surreal_{name}_repository.rs");
    let service_file = format!("{name}_service.rs");
    let dto_file = format!("{name}_dto.rs");
    let controller_file = format!("{name}_controller.rs");

    write(&api_dir.join(&entity_file), &entity_source(pascal, fields));
    write(&api_dir.join(&dto_file), &dto_source(name, pascal, fields));
    write(&api_dir.join(&repo_file), &repository_source(name, pascal));
    write(
        &api_dir.join(&surreal_file),
        &surreal_repository_source(name, pascal, fields),
    );
    write(&api_dir.join(&service_file), &service_source(name, pascal));
    write(
        &api_dir.join(&controller_file),
        &controller_source(name, pascal, plural),
    );
    write(&api_dir.join("mod.rs"), &mod_source(name, pascal));

    wire_api_mod(name);
    wire_db_mod(name, pascal);
    write(
        &Path::new("tests/report").join(format!("{name}.rs")),
        &test_source(name, fields),
    );
    wire_test_mod(name);

    println!();
    println!("feature `{name}` generated:");
    for entry in fs::read_dir(&api_dir).unwrap() {
        println!("  created {}", entry.unwrap().path().display());
    }
    println!("  updated src/api/mod.rs");
    println!("  updated src/db.rs");
    println!("  created tests/report/{name}.rs");
    println!("  updated tests/report/main.rs");
    println!();
    println!("next: cargo fmt && cargo test");
    println!();
    println!(
        "The generated code is a minimal, compiling CRUD feature. Use opencode\n\
         to evolve it (relations, custom validation, extra endpoints): it\n\
         understands both the generated code and the cli scripts in oxidax-cli/."
    );
    ExitCode::SUCCESS
}

fn write(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap_or_else(|err| {
        panic!("cannot write {}: {err}", path.display());
    });
}

fn entity_source(pascal: &str, fields: &[Field]) -> String {
    let eq = if fields.iter().all(|field| field.ty.is_eq()) {
        ", Eq"
    } else {
        ""
    };
    let mut body = String::new();
    for field in fields {
        body.push_str(&format!("    pub {}: {},\n", field.name, field.ty.rust()));
    }

    format!(
        "use serde::{{Deserialize, Serialize}};\n\
         \n\
         /// Database-independent entity: any repository must persist and return\n\
         /// exactly this type.\n\
         #[derive(Debug, Clone, PartialEq{eq}, Serialize, Deserialize)]\n\
         pub struct {pascal} {{\n\
         \x20   pub id: String,\n\
         {body}}}\n"
    )
}

fn dto_source(name: &str, pascal: &str, fields: &[Field]) -> String {
    let mut decls = String::new();
    let mut create_assigns = String::new();
    let mut update_assigns = String::new();
    let mut response_fields = String::new();
    let mut response_assigns = String::new();
    for field in fields {
        decls.push_str(&dto_field_decl(field));
        if field.ty == FieldType::Str {
            create_assigns.push_str(&format!(
                "            {}: self.{}.trim().to_string(),\n",
                field.name, field.name
            ));
            update_assigns.push_str(&format!(
                "            {}: self.{}.trim().to_string(),\n",
                field.name, field.name
            ));
        } else {
            create_assigns.push_str(&format!(
                "            {}: self.{},\n",
                field.name, field.name
            ));
            update_assigns.push_str(&format!(
                "            {}: self.{},\n",
                field.name, field.name
            ));
        }
        response_fields.push_str(&format!("    pub {}: {},\n", field.name, field.ty.rust()));
        response_assigns.push_str(&format!(
            "            {}: entity.{},\n",
            field.name, field.name
        ));
    }

    format!(
        "use serde::{{Deserialize, Serialize}};\n\
         use utoipa::ToSchema;\n\
         use uuid::Uuid;\n\
         \n\
         use crate::api::{name}::{name}_entity::{pascal};\n\
         use crate::lib::Validate;\n\
         \n\
         #[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]\n\
         pub struct Create{pascal}Dto {{\n\
         {decls}}}\n\
         \n\
         impl Create{pascal}Dto {{\n\
         \x20   /// Converts the DTO into the entity ready for the database:\n\
         \x20   /// generates the id and normalizes the string fields (validation\n\
         \x20   /// was already done by the extractor).\n\
         \x20   pub fn into_entity(self) -> {pascal} {{\n\
         \x20       {pascal} {{\n\
         \x20           id: Uuid::new_v4().to_string(),\n\
         {create_assigns}\x20       }}\n\
         \x20   }}\n\
         }}\n\
         \n\
         #[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]\n\
         pub struct Update{pascal}Dto {{\n\
         {decls}}}\n\
         \n\
         impl Update{pascal}Dto {{\n\
         \x20   /// Converts the DTO into the entity ready for the database:\n\
         \x20   /// reuses the id from the path and normalizes the string fields\n\
         \x20   /// (validation was already done by the extractor).\n\
         \x20   pub fn into_entity(self, id: String) -> {pascal} {{\n\
         \x20       {pascal} {{\n\
         \x20           id,\n\
         {update_assigns}\x20       }}\n\
         \x20   }}\n\
         }}\n\
         \n\
         #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]\n\
         pub struct {pascal}Response {{\n\
         \x20   /// UUID identifier\n\
         \x20   #[schema(example = \"550e8400-e29b-41d4-a716-446655440000\")]\n\
         \x20   pub id: String,\n\
         {response_fields}}}\n\
         \n\
         impl From<crate::api::{name}::{name}_entity::{pascal}> for {pascal}Response {{\n\
         \x20   fn from(entity: crate::api::{name}::{name}_entity::{pascal}) -> Self {{\n\
         \x20       Self {{\n\
         \x20           id: entity.id,\n\
         {response_assigns}\x20       }}\n\
         \x20   }}\n\
         }}\n"
    )
}

/// Declaration of a field (with validations) in the create/update DTOs.
fn dto_field_decl(field: &Field) -> String {
    if field.ty == FieldType::Str {
        let human = field.name.replace('_', " ");
        format!(
            "    /// {human}\n\
             \x20   #[schema(min_length = 1, max_length = 255)]\n\
             \x20   #[validate(\n\
             \x20       custom(\n\
             \x20           function = \"crate::lib::validators::not_blank\",\n\
             \x20           message = \"The {human} is required and cannot be empty\"\n\
             \x20       ),\n\
             \x20       length(max = 255, message = \"The {human} must be at most 255 characters\")\n\
             \x20   )]\n\
             \x20   pub {}: String,\n",
            field.name
        )
    } else {
        format!("    pub {}: {},\n", field.name, field.ty.rust())
    }
}

fn repository_source(name: &str, pascal: &str) -> String {
    format!(
        "use async_trait::async_trait;\n\
         \n\
         use crate::api::{name}::{name}_entity::{pascal};\n\
         use crate::errors::AppResult;\n\
         \n\
         /// Persistence contract for {name}s.\n\
         ///\n\
         /// It is the only point that depends on the database: implementing this\n\
         /// trait on a new backend leaves the service and controllers unchanged.\n\
         #[async_trait]\n\
         pub trait {pascal}Repository: Send + Sync {{\n\
         \x20   async fn insert(&self, item: {pascal}) -> AppResult<{pascal}>;\n\
         \x20   async fn find_by_id(&self, id: &str) -> AppResult<Option<{pascal}>>;\n\
         \x20   async fn find_all(&self) -> AppResult<Vec<{pascal}>>;\n\
         \x20   /// One page: `offset` skipped, at most `limit` returned.\n\
         \x20   async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<{pascal}>>;\n\
         \x20   /// Total number of rows, used to fill the pagination metadata.\n\
         \x20   async fn count(&self) -> AppResult<u64>;\n\
         \x20   async fn replace(&self, id: &str, item: {pascal}) -> AppResult<Option<{pascal}>>;\n\
         \x20   async fn remove(&self, id: &str) -> AppResult<bool>;\n\
         }}\n"
    )
}

fn surreal_repository_source(name: &str, pascal: &str, fields: &[Field]) -> String {
    let mut row_fields = String::new();
    let mut content_fields = String::new();
    let mut entity_assigns = String::new();
    let mut content_assigns = String::new();
    for field in fields {
        row_fields.push_str(&format!("    {}: {},\n", field.name, field.ty.rust()));
        content_fields.push_str(&format!("    {}: {},\n", field.name, field.ty.rust()));
        entity_assigns.push_str(&format!("        {}: row.{},\n", field.name, field.name));
        content_assigns.push_str(&format!(
            "            {}: item.{}.clone(),\n",
            field.name, field.name
        ));
    }

    format!(
        "use async_trait::async_trait;\n\
         use serde::{{Deserialize, Serialize}};\n\
         use surrealdb::Surreal;\n\
         use surrealdb::engine::any::Any;\n\
         use surrealdb::types::{{RecordId, SurrealValue}};\n\
         \n\
         use crate::api::{name}::{name}_entity::{pascal};\n\
         use crate::api::{name}::{name}_repository::{pascal}Repository;\n\
         use crate::db::{{define_table, query, record_key_to_string}};\n\
         use crate::errors::{{AppError, AppResult}};\n\
         \n\
         const TABLE: &str = \"{name}\";\n\
         \n\
         /// Row as it arrives from the database (the id is a SurrealDB record id).\n\
         #[derive(Debug, Serialize, Deserialize, SurrealValue)]\n\
         struct {pascal}Row {{\n\
         \x20   id: RecordId,\n\
         {row_fields}}}\n\
         \n\
         /// Persistable content: the id is managed by the database.\n\
         #[derive(Debug, Serialize, Deserialize, SurrealValue)]\n\
         struct {pascal}Content {{\n\
         {content_fields}}}\n\
         \n\
         pub struct Surreal{pascal}Repository {{\n\
         \x20   db: Surreal<Any>,\n\
         }}\n\
         \n\
         impl Surreal{pascal}Repository {{\n\
         \x20   pub fn new(db: Surreal<Any>) -> Self {{\n\
         \x20       Self {{ db }}\n\
         \x20   }}\n\
         \n\
         \x20   /// Creates the table if it does not exist: SurrealDB 3 does not\n\
         \x20   /// create tables implicitly.\n\
         \x20   pub async fn ensure_schema(&self) -> AppResult<()> {{\n\
         \x20       define_table(&self.db, TABLE).await\n\
         \x20   }}\n\
         }}\n\
         \n\
         /// The API works with the record id key alone (`uuid`), not `table:uuid`.\n\
         fn into_entity(row: {pascal}Row) -> {pascal} {{\n\
         \x20   {pascal} {{\n\
         \x20       id: record_key_to_string(row.id.key),\n\
         {entity_assigns}\x20   }}\n\
         }}\n\
         \n\
         fn created_or_internal(item: {pascal}, created: Option<{pascal}Row>) -> AppResult<{pascal}> {{\n\
         \x20   if created.is_some() {{\n\
         \x20       Ok(item)\n\
         \x20   }} else {{\n\
         \x20       Err(AppError::Internal(format!(\"failed to create {name} {{}}\", item.id)))\n\
         \x20   }}\n\
         }}\n\
         \n\
         #[async_trait]\n\
         impl {pascal}Repository for Surreal{pascal}Repository {{\n\
         \x20   async fn insert(&self, item: {pascal}) -> AppResult<{pascal}> {{\n\
         \x20       let content = {pascal}Content {{\n\
         {content_assigns}\x20       }};\n\
         \n\
         \x20       let created: Option<{pascal}Row> = self\n\
         \x20           .db\n\
         \x20           .create((TABLE, item.id.as_str()))\n\
         \x20           .content(content)\n\
         \x20           .await?;\n\
         \n\
         \x20       created_or_internal(item, created)\n\
         \x20   }}\n\
         \n\
         \x20   async fn find_by_id(&self, id: &str) -> AppResult<Option<{pascal}>> {{\n\
         \x20       let found: Option<{pascal}Row> = self.db.select((TABLE, id)).await?;\n\
         \x20       Ok(found.map(into_entity))\n\
         \x20   }}\n\
         \n\
         \x20   async fn find_all(&self) -> AppResult<Vec<{pascal}>> {{\n\
         \x20       let rows: Vec<{pascal}Row> = self.db.select(TABLE).await?;\n\
         \x20       Ok(rows.into_iter().map(into_entity).collect())\n\
         \x20   }}\n\
         \n\
         \x20   async fn find_page(&self, offset: u64, limit: u64) -> AppResult<Vec<{pascal}>> {{\n\
         \x20       let rows: Vec<{pascal}Row> = query::select_page(&self.db, TABLE, offset, limit).await?;\n\
         \x20       Ok(rows.into_iter().map(into_entity).collect())\n\
         \x20   }}\n\
         \n\
         \x20   async fn count(&self) -> AppResult<u64> {{\n\
         \x20       query::count(&self.db, TABLE).await\n\
         \x20   }}\n\
         \n\
         \x20   async fn replace(&self, id: &str, item: {pascal}) -> AppResult<Option<{pascal}>> {{\n\
         \x20       let content = {pascal}Content {{\n\
         {content_assigns}\x20       }};\n\
         \n\
         \x20       let updated: Option<{pascal}Row> = self.db.update((TABLE, id)).content(content).await?;\n\
         \x20       Ok(updated.map(into_entity))\n\
         \x20   }}\n\
         \n\
         \x20   async fn remove(&self, id: &str) -> AppResult<bool> {{\n\
         \x20       let deleted: Option<{pascal}Row> = self.db.delete((TABLE, id)).await?;\n\
         \x20       Ok(deleted.is_some())\n\
         \x20   }}\n\
         }}\n"
    )
}

fn service_source(name: &str, pascal: &str) -> String {
    format!(
        "use std::sync::Arc;\n\
         \n\
         use di::injectable;\n\
         \n\
         use crate::api::{name}::{name}_dto::{{Create{pascal}Dto, {pascal}Response, Update{pascal}Dto}};\n\
         use crate::api::{name}::{name}_repository::{pascal}Repository;\n\
         use crate::errors::{{AppError, AppResult}};\n\
         use crate::lib::pagination::{{Paged, Pagination}};\n\
         \n\
         /// Business rules for {name}s.\n\
         #[injectable]\n\
         pub struct {pascal}Service {{\n\
         \x20   repository: Arc<dyn {pascal}Repository>,\n\
         }}\n\
         \n\
         impl {pascal}Service {{\n\
         \x20   pub async fn create(&self, dto: Create{pascal}Dto) -> AppResult<{pascal}Response> {{\n\
         \x20       let item = dto.into_entity();\n\
         \x20       self.repository.insert(item).await.map(Into::into)\n\
         \x20   }}\n\
         \n\
         \x20   pub async fn get(&self, id: &str) -> AppResult<{pascal}Response> {{\n\
         \x20       self.repository\n\
         \x20           .find_by_id(id)\n\
         \x20           .await?\n\
         \x20           .map(Into::into)\n\
         \x20           .ok_or_else(|| not_found(id))\n\
         \x20   }}\n\
         \n\
         \x20   pub async fn list(&self) -> AppResult<Vec<{pascal}Response>> {{\n\
         \x20       self.repository\n\
         \x20           .find_all()\n\
         \x20           .await\n\
         \x20           .map(|items| items.into_iter().map(Into::into).collect())\n\
         \x20   }}\n\
         \n\
         \x20   /// One page of {name}s with the total count (sent as `X-TOTAL-LIST`).\n\
         \x20   pub async fn list_page(&self, pagination: Pagination) -> AppResult<Paged<{pascal}Response>> {{\n\
         \x20       let total = self.repository.count().await?;\n\
         \x20       let items = self\n\
         \x20           .repository\n\
         \x20           .find_page(pagination.offset(), pagination.limit())\n\
         \x20           .await?;\n\
         \n\
         \x20       let items = items.into_iter().map(Into::into).collect();\n\
         \x20       Ok(Paged::new(items, total))\n\
         \x20   }}\n\
         \n\
         \x20   pub async fn update(&self, id: &str, dto: Update{pascal}Dto) -> AppResult<{pascal}Response> {{\n\
         \x20       let item = dto.into_entity(id.to_string());\n\
         \x20       self.repository\n\
         \x20           .replace(id, item)\n\
         \x20           .await?\n\
         \x20           .map(Into::into)\n\
         \x20           .ok_or_else(|| not_found(id))\n\
         \x20   }}\n\
         \n\
         \x20   pub async fn delete(&self, id: &str) -> AppResult<()> {{\n\
         \x20       if self.repository.remove(id).await? {{\n\
         \x20           Ok(())\n\
         \x20       }} else {{\n\
         \x20           Err(not_found(id))\n\
         \x20       }}\n\
         \x20   }}\n\
         }}\n\
         \n\
         fn not_found(id: &str) -> AppError {{\n\
         \x20   AppError::NotFound(format!(\"{name} {{id}} not found\"))\n\
         }}\n"
    )
}

fn controller_source(name: &str, pascal: &str, plural: &str) -> String {
    format!(
        "use std::sync::Arc;\n\
         \n\
         use axum::Json;\n\
         use axum::extract::{{Path, State}};\n\
         use axum::http::StatusCode;\n\
         use axum::response::Response;\n\
         \n\
         use crate::api::{name}::{name}_dto::{{Create{pascal}Dto, {pascal}Response, Update{pascal}Dto}};\n\
         use crate::api::{name}::{name}_service::{pascal}Service;\n\
         use crate::errors::AppResult;\n\
         use crate::framework::{{PaginationQuery, ValidatedJson, ValidatedQuery, route}};\n\
         \n\
         // Route table and OpenAPI spec of the {name} feature, defined point-inline:\n\
         // every handler is annotated with `#[utoipa::path(...)]` and `#[route]`,\n\
         // which self-registers it in the static registry.\n\
         \n\
         #[route]\n\
         #[utoipa::path(\n\
         \x20   get,\n\
         \x20   path = \"/api/{name}\",\n\
         \x20   tag = \"{name}\",\n\
         \x20   params(\n\
         \x20       (\"page\" = Option<u64>, Query, description = \"Page number, 1-based (default 1)\", example = 1),\n\
         \x20       (\"limit\" = Option<u64>, Query, description = \"Elements per page, 1-100 (default 20)\", example = 20),\n\
         \x20   ),\n\
         \x20   responses(\n\
         \x20       (status = 200, description = \"Paginated list of {plural}; the total is returned in the `X-TOTAL-LIST` header\", body = [{pascal}Response], headers((\"X-TOTAL-LIST\" = u64, description = \"Total number of {plural} across every page\"))),\n\
         \x20       (status = 422, description = \"Invalid pagination parameters\"),\n\
         \x20       (status = 503, description = \"Database unavailable\"),\n\
         \x20   ),\n\
         )]\n\
         pub async fn list_{plural}(\n\
         \x20   State(service): State<Arc<{pascal}Service>>,\n\
         \x20   ValidatedQuery(pagination): ValidatedQuery<PaginationQuery>,\n\
         ) -> AppResult<Response> {{\n\
         \x20   let page = service.list_page(pagination.pagination()).await?;\n\
         \x20   Ok(page.into_response())\n\
         }}\n\
         \n\
         #[route]\n\
         #[utoipa::path(\n\
         \x20   post,\n\
         \x20   path = \"/api/{name}\",\n\
         \x20   tag = \"{name}\",\n\
         \x20   request_body = Create{pascal}Dto,\n\
         \x20   responses(\n\
         \x20       (status = 201, description = \"{pascal} created successfully\", body = {pascal}Response),\n\
         \x20       (status = 400, description = \"Invalid request: malformed JSON body\"),\n\
         \x20       (status = 422, description = \"Field validation failed\"),\n\
         \x20       (status = 503, description = \"Database unavailable\"),\n\
         \x20   ),\n\
         )]\n\
         pub async fn create_{name}(\n\
         \x20   State(service): State<Arc<{pascal}Service>>,\n\
         \x20   ValidatedJson(dto): ValidatedJson<Create{pascal}Dto>,\n\
         ) -> AppResult<(StatusCode, Json<{pascal}Response>)> {{\n\
         \x20   let created = service.create(dto).await?;\n\
         \x20   Ok((StatusCode::CREATED, Json(created)))\n\
         }}\n\
         \n\
         #[route]\n\
         #[utoipa::path(\n\
         \x20   get,\n\
         \x20   path = \"/api/{name}/{{id}}\",\n\
         \x20   tag = \"{name}\",\n\
         \x20   params((\"id\" = String, Path, description = \"{pascal} id\")),\n\
         \x20   responses(\n\
         \x20       (status = 200, description = \"{pascal} found\", body = {pascal}Response),\n\
         \x20       (status = 404, description = \"{pascal} not found\"),\n\
         \x20       (status = 503, description = \"Database unavailable\"),\n\
         \x20   ),\n\
         )]\n\
         pub async fn get_{name}(\n\
         \x20   State(service): State<Arc<{pascal}Service>>,\n\
         \x20   Path(id): Path<String>,\n\
         ) -> AppResult<Json<{pascal}Response>> {{\n\
         \x20   Ok(Json(service.get(&id).await?))\n\
         }}\n\
         \n\
         #[route]\n\
         #[utoipa::path(\n\
         \x20   put,\n\
         \x20   path = \"/api/{name}/{{id}}\",\n\
         \x20   tag = \"{name}\",\n\
         \x20   params((\"id\" = String, Path, description = \"{pascal} id\")),\n\
         \x20   request_body = Update{pascal}Dto,\n\
         \x20   responses(\n\
         \x20       (status = 200, description = \"{pascal} updated\", body = {pascal}Response),\n\
         \x20       (status = 400, description = \"Invalid request: malformed JSON body\"),\n\
         \x20       (status = 404, description = \"{pascal} not found\"),\n\
         \x20       (status = 422, description = \"Field validation failed\"),\n\
         \x20       (status = 503, description = \"Database unavailable\"),\n\
         \x20   ),\n\
         )]\n\
         pub async fn update_{name}(\n\
         \x20   State(service): State<Arc<{pascal}Service>>,\n\
         \x20   Path(id): Path<String>,\n\
         \x20   ValidatedJson(dto): ValidatedJson<Update{pascal}Dto>,\n\
         ) -> AppResult<Json<{pascal}Response>> {{\n\
         \x20   Ok(Json(service.update(&id, dto).await?))\n\
         }}\n\
         \n\
         #[route]\n\
         #[utoipa::path(\n\
         \x20   delete,\n\
         \x20   path = \"/api/{name}/{{id}}\",\n\
         \x20   tag = \"{name}\",\n\
         \x20   params((\"id\" = String, Path, description = \"{pascal} id\")),\n\
         \x20   responses(\n\
         \x20       (status = 204, description = \"{pascal} deleted\"),\n\
         \x20       (status = 404, description = \"{pascal} not found\"),\n\
         \x20       (status = 503, description = \"Database unavailable\"),\n\
         \x20   ),\n\
         )]\n\
         pub async fn delete_{name}(\n\
         \x20   State(service): State<Arc<{pascal}Service>>,\n\
         \x20   Path(id): Path<String>,\n\
         ) -> AppResult<StatusCode> {{\n\
         \x20   service.delete(&id).await?;\n\
         \x20   Ok(StatusCode::NO_CONTENT)\n\
         }}\n"
    )
}

fn mod_source(name: &str, pascal: &str) -> String {
    format!(
        "pub mod {name}_controller;\n\
         pub mod {name}_dto;\n\
         pub mod {name}_entity;\n\
         pub mod {name}_repository;\n\
         pub mod {name}_service;\n\
         pub mod surreal_{name}_repository;\n\
         \n\
         use di::Injectable;\n\
         use di::ServiceCollection;\n\
         \n\
         use crate::framework::Feature;\n\
         use {name}_service::{pascal}Service;\n\
         \n\
         // All the DI wiring lives here; every route is auto-registered by\n\
         // `#[route]` on the handlers in `{name}_controller`. The repository is\n\
         // registered centrally in `crate::db`.\n\
         fn register(services: &mut ServiceCollection) {{\n\
         \x20   services.add({pascal}Service::singleton());\n\
         }}\n\
         \n\
         // Zero-touch self-registration in the feature registry.\n\
         inventory::submit! {{\n\
         \x20   Feature::new(0, register)\n\
         }}\n"
    )
}

/// Appends `pub mod <name>;` to `src/api/mod.rs`.
fn wire_api_mod(name: &str) {
    let path = Path::new("src/api/mod.rs");
    let mut contents = fs::read_to_string(path).expect("read src/api/mod.rs");
    let marker = format!("pub mod {name};");
    if contents.contains(&marker) {
        return;
    }
    if !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str(&marker);
    contents.push('\n');
    fs::write(path, contents).expect("write src/api/mod.rs");
}

/// Wires the repository into `src/db.rs` (imports, struct field, schema,
/// construction and central registration).
fn wire_db_mod(name: &str, pascal: &str) {
    let path = Path::new("src/db.rs");
    let contents = fs::read_to_string(path).expect("read src/db.rs");
    if contents.contains(&format!("pub {name}: Arc<dyn {pascal}Repository>")) {
        eprintln!("`{name}` seems already wired in src/db.rs; skipping");
        return;
    }

    let import_anchor = "use crate::api::users::user_repository::UserRepository;\n";
    let imports = format!(
        "use crate::api::{name}::{name}_repository::{pascal}Repository;\n\
         use crate::api::{name}::surreal_{name}_repository::Surreal{pascal}Repository;\n"
    );
    let struct_anchor = "    pub places: Arc<dyn PlaceRepository>,\n";
    let struct_field = format!("    pub {name}: Arc<dyn {pascal}Repository>,\n");
    let schema_anchor = "    places.ensure_schema().await?;\n";
    let schema = format!(
        "\n    let {name} = Surreal{pascal}Repository::new(db.clone());\n\
         \x20   {name}.ensure_schema().await?;\n"
    );
    let construct_anchor = "        places: Arc::new(places),\n";
    let construct = format!("        {name}: Arc::new({name}),\n");
    let register_anchor = "    register_repository(services, repositories.places.clone());\n";
    let register = format!("    register_repository(services, repositories.{name}.clone());\n");

    let contents = replace_once(
        contents,
        import_anchor,
        &format!("{import_anchor}{imports}"),
    );
    let contents = replace_once(
        contents,
        struct_anchor,
        &format!("{struct_anchor}{struct_field}"),
    );
    let contents = replace_once(contents, schema_anchor, &format!("{schema_anchor}{schema}"));
    let contents = replace_once(
        contents,
        construct_anchor,
        &format!("{construct_anchor}{construct}"),
    );
    let contents = replace_once(
        contents,
        register_anchor,
        &format!("{register_anchor}{register}"),
    );

    fs::write(path, contents).expect("write src/db.rs");
}

/// Appends `mod <name>;` to `tests/report/main.rs`.
fn wire_test_mod(name: &str) {
    let path = Path::new("tests/report/main.rs");
    let mut contents = fs::read_to_string(path).expect("read tests/report/main.rs");
    let marker = format!("mod {name};");
    if contents.contains(&marker) {
        return;
    }
    if !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str(&marker);
    contents.push('\n');
    fs::write(path, contents).expect("write tests/report/main.rs");
}

/// Replaces the first occurrence of `anchor`, panicking if not found.
fn replace_once(contents: String, anchor: &str, replacement: &str) -> String {
    if !contents.contains(anchor) {
        panic!("cannot find anchor in src/db.rs: {anchor:?}");
    }
    contents.replacen(anchor, replacement, 1)
}

fn test_source(name: &str, fields: &[Field]) -> String {
    let mut create_pairs = String::new();
    for field in fields {
        create_pairs.push_str(&format!(
            "            \"{}\": {},\n",
            field.name,
            field.ty.sample()
        ));
    }

    format!(
        "//! Full CRUD cycle over HTTP for the generated `{name}` feature.\n\
         \n\
         use axum::body::Body;\n\
         use axum::http::{{Request, StatusCode}};\n\
         use serde_json::json;\n\
         use tower::ServiceExt;\n\
         \n\
         use crate::support::{{app, json_body, json_request}};\n\
         \n\
         #[tokio::test]\n\
         async fn {name}_full_crud_cycle() {{\n\
         \x20   let app = app().await;\n\
         \n\
         \x20   let response = app\n\
         \x20       .clone()\n\
         \x20       .oneshot(json_request(\n\
         \x20           \"POST\",\n\
         \x20           \"/api/{name}\",\n\
         \x20           json!({{\n\
         {create_pairs}\x20           }}),\n\
         \x20       ))\n\
         \x20       .await\n\
         \x20       .unwrap();\n\
         \x20   let status = response.status();\n\
         \x20   let created = json_body(response).await;\n\
         \x20   assert_eq!(status, StatusCode::CREATED, \"{{created}}\");\n\
         \x20   let id = created[\"id\"].as_str().expect(\"id\").to_string();\n\
         \n\
         \x20   let response = app\n\
         \x20       .clone()\n\
         \x20       .oneshot(\n\
         \x20           Request::builder()\n\
         \x20               .uri(format!(\"/api/{name}/{{id}}\"))\n\
         \x20               .body(Body::empty())\n\
         \x20               .unwrap(),\n\
         \x20       )\n\
         \x20       .await\n\
         \x20       .unwrap();\n\
         \x20   assert_eq!(response.status(), StatusCode::OK);\n\
         \n\
         \x20   let response = app\n\
         \x20       .clone()\n\
         \x20       .oneshot(\n\
         \x20           Request::builder()\n\
         \x20               .uri(\"/api/{name}?page=1&limit=10\")\n\
         \x20               .body(Body::empty())\n\
         \x20               .unwrap(),\n\
         \x20       )\n\
         \x20       .await\n\
         \x20       .unwrap();\n\
         \x20   let status = response.status();\n\
         \x20   let total = response\n\
         \x20       .headers()\n\
         \x20       .get(\"x-total-list\")\n\
         \x20       .expect(\"X-TOTAL-LIST header\")\n\
         \x20       .to_str()\n\
         \x20       .unwrap()\n\
         \x20       .to_string();\n\
         \x20   let list = json_body(response).await;\n\
         \x20   assert_eq!(status, StatusCode::OK, \"{{list}}\");\n\
         \x20   assert_eq!(total, \"1\");\n\
         \x20   assert_eq!(list.as_array().unwrap().len(), 1);\n\
         \n\
         \x20   let response = app\n\
         \x20       .clone()\n\
         \x20       .oneshot(json_request(\n\
         \x20           \"PUT\",\n\
         \x20           &format!(\"/api/{name}/{{id}}\"),\n\
         \x20           json!({{\n\
         {create_pairs}\x20           }}),\n\
         \x20       ))\n\
         \x20       .await\n\
         \x20       .unwrap();\n\
         \x20   assert_eq!(response.status(), StatusCode::OK);\n\
         \n\
         \x20   let response = app\n\
         \x20       .clone()\n\
         \x20       .oneshot(\n\
         \x20           Request::builder()\n\
         \x20               .method(\"DELETE\")\n\
         \x20               .uri(format!(\"/api/{name}/{{id}}\"))\n\
         \x20               .body(Body::empty())\n\
         \x20               .unwrap(),\n\
         \x20       )\n\
         \x20       .await\n\
         \x20       .unwrap();\n\
         \x20   assert_eq!(response.status(), StatusCode::NO_CONTENT);\n\
         \n\
         \x20   let response = app\n\
         \x20       .oneshot(\n\
         \x20           Request::builder()\n\
         \x20               .uri(format!(\"/api/{name}/{{id}}\"))\n\
         \x20               .body(Body::empty())\n\
         \x20               .unwrap(),\n\
         \x20       )\n\
         \x20       .await\n\
         \x20       .unwrap();\n\
         \x20   assert_eq!(response.status(), StatusCode::NOT_FOUND);\n\
         }}\n"
    )
}
