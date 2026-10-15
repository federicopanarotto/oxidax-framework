use async_trait::async_trait;

use crate::api::categories::category_entity::Category;
use crate::api::categories::category_response::CategoryResponse;
use crate::errors::AppResult;
use crate::framework::ViewRepository;

/// Persistence contract for categories.
///
/// It inherits the populated reads (`find`, `find_page`, `count`, `delete`)
/// from [`ViewRepository`] and only declares the aggregate-specific writes.
#[async_trait]
pub trait CategoryRepository: ViewRepository<Response = CategoryResponse> {
    async fn insert(&self, category: Category) -> AppResult<CategoryResponse>;
    async fn replace(&self, id: &str, category: Category) -> AppResult<Option<CategoryResponse>>;
}
