use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCategoryDto {
    /// Nome della categoria
    #[schema(example = "Vendite")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCategoryDto {
    /// Nuovo nome della categoria
    #[schema(example = "Marketing")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategoryResponse {
    /// Identificativo UUID della categoria
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// Nome della categoria
    #[schema(example = "Vendite")]
    pub name: String,
}

impl From<crate::api::categories::category_entity::Category> for CategoryResponse {
    fn from(category: crate::api::categories::category_entity::Category) -> Self {
        Self {
            id: category.id,
            name: category.name,
        }
    }
}
