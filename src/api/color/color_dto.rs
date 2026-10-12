use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::color::color_entity::Color;
use crate::lib::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateColorDto {
    /// name
    #[schema(min_length = 1, max_length = 255)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The name is required and cannot be empty"
        ),
        length(max = 255, message = "The name must be at most 255 characters")
    )]
    pub name: String,
    /// hex
    #[schema(min_length = 1, max_length = 255)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The hex is required and cannot be empty"
        ),
        length(max = 255, message = "The hex must be at most 255 characters")
    )]
    pub hex: String,
}

impl CreateColorDto {
    /// Converts the DTO into the entity ready for the database:
    /// generates the id and normalizes the string fields (validation
    /// was already done by the extractor).
    pub fn into_entity(self) -> Color {
        Color {
            id: Uuid::new_v4().to_string(),
            name: self.name.trim().to_string(),
            hex: self.hex.trim().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateColorDto {
    /// name
    #[schema(min_length = 1, max_length = 255)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The name is required and cannot be empty"
        ),
        length(max = 255, message = "The name must be at most 255 characters")
    )]
    pub name: String,
    /// hex
    #[schema(min_length = 1, max_length = 255)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The hex is required and cannot be empty"
        ),
        length(max = 255, message = "The hex must be at most 255 characters")
    )]
    pub hex: String,
}

impl UpdateColorDto {
    /// Converts the DTO into the entity ready for the database:
    /// reuses the id from the path and normalizes the string fields
    /// (validation was already done by the extractor).
    pub fn into_entity(self, id: String) -> Color {
        Color {
            id,
            name: self.name.trim().to_string(),
            hex: self.hex.trim().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ColorResponse {
    /// UUID identifier
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    pub name: String,
    pub hex: String,
}

impl From<crate::api::color::color_entity::Color> for ColorResponse {
    fn from(entity: crate::api::color::color_entity::Color) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            hex: entity.hex,
        }
    }
}
