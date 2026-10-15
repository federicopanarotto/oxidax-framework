use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::categories::category_entity::Category;
use crate::lib::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateCategoryDto {
    /// Category name
    #[schema(example = "Vendite", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The name is required and cannot be empty"
        ),
        length(max = 200, message = "The name must be at most 200 characters")
    )]
    pub name: String,
}

impl CreateCategoryDto {
    /// Converts the DTO into the entity ready for the database: generates the
    /// id and normalizes the name (validation was already done by the
    /// extractor).
    pub fn into_entity(self) -> Category {
        Category {
            id: Uuid::new_v4().to_string(),
            name: self.name.trim().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateCategoryDto {
    /// New category name
    #[schema(example = "Marketing", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The name is required and cannot be empty"
        ),
        length(max = 200, message = "The name must be at most 200 characters")
    )]
    pub name: String,
}

impl UpdateCategoryDto {
    /// Converts the DTO into the entity ready for the database: reuses the id
    /// from the path and normalizes the name (validation was already done by
    /// the extractor).
    pub fn into_entity(self, id: String) -> Category {
        Category {
            id,
            name: self.name.trim().to_string(),
        }
    }
}
