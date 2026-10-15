use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::SurrealValue;

use crate::api::users::user_entity::User;
use crate::api::users::user_repository::UserRepository;
use crate::api::users::user_response::UserSummaryResponse;
use crate::db::{define_table, query};
use crate::errors::AppResult;
use crate::framework::ViewRepository;

const TABLE: &str = "user";

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

/// Enables all the populated reads (`find`, `find_page`, `find_page_by`,
/// `count`, `count_by`, `delete`) on top of the shared connection.
#[async_trait]
impl ViewRepository for SurrealUserRepository {
    type Response = UserSummaryResponse;

    fn db(&self) -> &Surreal<Any> {
        &self.db
    }
}

#[async_trait]
impl UserRepository for SurrealUserRepository {
    async fn insert(&self, user: User) -> AppResult<UserSummaryResponse> {
        let id = user.id.clone();
        let content = UserContent {
            firstname: user.firstname,
            lastname: user.lastname,
            role: user.role,
        };

        query::insert_view(&self.db, &id, content).await
    }
}
