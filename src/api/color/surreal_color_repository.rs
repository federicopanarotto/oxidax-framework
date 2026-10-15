use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::SurrealValue;

use crate::api::color::color_entity::Color;
use crate::api::color::color_repository::ColorRepository;
use crate::api::color::color_response::ColorResponse;
use crate::db::{define_table, query};
use crate::errors::AppResult;
use crate::framework::ViewRepository;

const TABLE: &str = "color";

/// Persistable content: the id is managed by the database, it must not be
/// written in the CONTENT.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct ColorContent {
    name: String,
    hex: String,
}

pub struct SurrealColorRepository {
    db: Surreal<Any>,
}

impl SurrealColorRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Creates the table if it does not exist: SurrealDB 3 does not create
    /// tables implicitly.
    pub async fn ensure_schema(&self) -> AppResult<()> {
        define_table(&self.db, TABLE).await
    }
}

/// Enables all the populated reads (`find`, `find_page`, `count`, `delete`) on
/// top of the shared connection.
#[async_trait]
impl ViewRepository for SurrealColorRepository {
    type Response = ColorResponse;

    fn db(&self) -> &Surreal<Any> {
        &self.db
    }
}

fn content(item: Color) -> ColorContent {
    ColorContent {
        name: item.name,
        hex: item.hex,
    }
}

#[async_trait]
impl ColorRepository for SurrealColorRepository {
    async fn insert(&self, item: Color) -> AppResult<ColorResponse> {
        let id = item.id.clone();
        query::insert_view(&self.db, &id, content(item)).await
    }

    async fn replace(&self, id: &str, item: Color) -> AppResult<Option<ColorResponse>> {
        query::replace_view(&self.db, id, content(item)).await
    }
}
