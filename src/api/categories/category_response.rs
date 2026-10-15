use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::categories::category_entity::Category;
use crate::framework::Response;

/// Category as returned to the client. It is also nested inside the report
/// response: the derive teaches it how to project itself from the `category`
/// table, so a report can resolve it in the same query.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Response)]
#[serde(rename_all = "camelCase")]
#[response(entity = Category, table = "category")]
pub struct CategoryResponse {
    /// UUID identifier of the category
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// Category name
    #[schema(example = "Vendite")]
    pub name: String,
}
