use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::types::{RecordId, SurrealValue};

use crate::auth::identity_repository::UserIdentityRepository;
use crate::auth::user_identity::{AuthProvider, UserIdentity};
use crate::db::{define_table, record_key_to_string};
use crate::errors::{AppError, AppResult};

const TABLE: &str = "user_identity";

/// Row as it arrives from the database (the id is a SurrealDB record id).
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct UserIdentityRow {
    id: RecordId,
    user_id: String,
    provider: String,
    identifier: String,
    password_hash: String,
    #[serde(default)]
    refresh_jti: Option<String>,
    #[serde(default)]
    refresh_expires_at: Option<usize>,
}

/// Persistable content: the id is managed by the database, it must not be
/// written in the CONTENT.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct UserIdentityContent {
    user_id: String,
    provider: String,
    identifier: String,
    password_hash: String,
    refresh_jti: Option<String>,
    refresh_expires_at: Option<usize>,
}

/// Partial update of the rotating session.
#[derive(Debug, Serialize, Deserialize, SurrealValue)]
struct RefreshPatch {
    refresh_jti: Option<String>,
    refresh_expires_at: Option<usize>,
}

pub struct SurrealUserIdentityRepository {
    db: Surreal<Any>,
}

impl SurrealUserIdentityRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self { db }
    }

    /// Creates the table if it does not exist: SurrealDB 3 does not create
    /// tables implicitly.
    pub async fn ensure_schema(&self) -> AppResult<()> {
        define_table(&self.db, TABLE).await
    }

    async fn find_one(&self, statement: &str, value: &str) -> AppResult<Option<UserIdentity>> {
        let mut result = self.db.query(statement).bind(("value", value)).await?;
        let rows: Vec<UserIdentityRow> = result.take(0)?;
        Ok(rows.into_iter().next().map(into_entity))
    }
}

/// The API works with the record id key alone (`uuid`), not with `table:uuid`.
fn into_entity(row: UserIdentityRow) -> UserIdentity {
    UserIdentity {
        id: record_key_to_string(row.id.key),
        user_id: row.user_id,
        provider: AuthProvider::parse(&row.provider).unwrap_or(AuthProvider::Local),
        identifier: row.identifier,
        password_hash: row.password_hash,
        refresh_jti: row.refresh_jti,
        refresh_expires_at: row.refresh_expires_at,
    }
}

fn created_or_internal(
    identity: UserIdentity,
    created: Option<UserIdentityRow>,
) -> AppResult<UserIdentity> {
    if created.is_some() {
        Ok(identity)
    } else {
        Err(AppError::Internal(format!(
            "failed to create identity {}",
            identity.id
        )))
    }
}

fn patch_applied(updated: Option<UserIdentityRow>, id: &str) -> AppResult<bool> {
    if updated.is_some() {
        Ok(true)
    } else {
        Err(AppError::NotFound(format!("identity {id} not found")))
    }
}

#[async_trait]
impl UserIdentityRepository for SurrealUserIdentityRepository {
    async fn insert(&self, identity: UserIdentity) -> AppResult<UserIdentity> {
        let content = UserIdentityContent {
            user_id: identity.user_id.clone(),
            provider: identity.provider.as_str().to_string(),
            identifier: identity.identifier.clone(),
            password_hash: identity.password_hash.clone(),
            refresh_jti: identity.refresh_jti.clone(),
            refresh_expires_at: identity.refresh_expires_at,
        };

        let created: Option<UserIdentityRow> = self
            .db
            .create((TABLE, identity.id.as_str()))
            .content(content)
            .await?;

        created_or_internal(identity, created)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<UserIdentity>> {
        let found: Option<UserIdentityRow> = self.db.select((TABLE, id)).await?;
        Ok(found.map(into_entity))
    }

    async fn find_by_identifier(&self, identifier: &str) -> AppResult<Option<UserIdentity>> {
        self.find_one(
            "SELECT * FROM user_identity WHERE identifier = $value",
            identifier,
        )
        .await
    }

    async fn find_by_user_id(&self, user_id: &str) -> AppResult<Option<UserIdentity>> {
        self.find_one(
            "SELECT * FROM user_identity WHERE user_id = $value",
            user_id,
        )
        .await
    }

    async fn find_all(&self) -> AppResult<Vec<UserIdentity>> {
        let rows: Vec<UserIdentityRow> = self.db.select(TABLE).await?;
        Ok(rows.into_iter().map(into_entity).collect())
    }

    async fn set_refresh(&self, id: &str, jti: &str, expires_at: usize) -> AppResult<bool> {
        let patch = RefreshPatch {
            refresh_jti: Some(jti.to_string()),
            refresh_expires_at: Some(expires_at),
        };
        let updated: Option<UserIdentityRow> = self.db.update((TABLE, id)).merge(patch).await?;
        patch_applied(updated, id)
    }

    async fn clear_refresh(&self, id: &str) -> AppResult<bool> {
        let patch = RefreshPatch {
            refresh_jti: None,
            refresh_expires_at: None,
        };
        let updated: Option<UserIdentityRow> = self.db.update((TABLE, id)).merge(patch).await?;
        patch_applied(updated, id)
    }
}
