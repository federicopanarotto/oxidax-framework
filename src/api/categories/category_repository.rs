use async_trait::async_trait;

use crate::api::categories::category_entity::Category;
use crate::errors::AppResult;

/// Contratto di persistenza per le categorie.
///
/// È l'unico punto che dipende dal database: implementando questo trait su un
/// nuovo backend (Postgres, SQLite, MongoDB...) il service e i controller
/// restano invariati.
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn insert(&self, category: Category) -> AppResult<Category>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Category>>;
    async fn find_all(&self) -> AppResult<Vec<Category>>;
    async fn replace(&self, id: &str, category: Category) -> AppResult<Option<Category>>;
    async fn remove(&self, id: &str) -> AppResult<bool>;
}
