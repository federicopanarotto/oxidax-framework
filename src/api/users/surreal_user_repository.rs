use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::{RecordId, SurrealValue};

use crate::api::users::user_entity::User;
use crate::api::users::user_repository::UserRepository;
use crate::db::{define_table, record_key_to_string};
use crate::errors::{AppError, AppResult};

const TABLE: &str = "user";

/// Row as it arrives from the database (the id is a SurrealDB record id).
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct UserRow {
    id: RecordId,
    firstname: String,
    lastname: String,
    role: String,
}

/// Persistable content: the id is managed by the database, it must not be
/// written in the CONTENT.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct UserContent {
    firstname: String,
    lastname: String,
    role: String,
}

pub struct SurrealUserRepository {
    db: Surreal<Any>,
}

impl SurrealUserRepository {
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
fn into_entity(row: UserRow) -> User {
    User {
        id: record_key_to_string(row.id.key),
        firstname: row.firstname,
        lastname: row.lastname,
        role: row.role,
    }
}

fn created_or_internal(user: User, created: Option<UserRow>) -> AppResult<User> {
    if created.is_some() {
        Ok(user)
    } else {
        Err(AppError::Internal(format!(
            "failed to create user {}",
            user.id
        )))
    }
}

#[async_trait]
impl UserRepository for SurrealUserRepository {
    async fn insert(&self, user: User) -> AppResult<User> {
        let content = UserContent {
            firstname: user.firstname.clone(),
            lastname: user.lastname.clone(),
            role: user.role.clone(),
        };

        let created: Option<UserRow> = self
            .db
            .create((TABLE, user.id.as_str()))
            .content(content)
            .await?;

        created_or_internal(user, created)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<User>> {
        let found: Option<UserRow> = self.db.select((TABLE, id)).await?;
        Ok(found.map(into_entity))
    }

    async fn find_all(&self) -> AppResult<Vec<User>> {
        let rows: Vec<UserRow> = self.db.select(TABLE).await?;
        Ok(rows.into_iter().map(into_entity).collect())
    }

    async fn remove(&self, id: &str) -> AppResult<bool> {
        let deleted: Option<UserRow> = self.db.delete((TABLE, id)).await?;
        Ok(deleted.is_some())
    }
}
