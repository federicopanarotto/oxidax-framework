use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::categories::category_entity::Category;
use crate::lib::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateCategoryDto {
    /// Nome della categoria
    #[schema(example = "Vendite", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "Il nome è obbligatorio e non può essere vuoto"
        ),
        length(max = 200, message = "Il nome deve essere di massimo 200 caratteri")
    )]
    pub name: String,
}

impl CreateCategoryDto {
    /// Converte il DTO nell'entità pronta per il database: genera l'id e
    /// normalizza il nome (la validazione è già stata fatta dall'extractor).
    pub fn into_entity(self) -> Category {
        Category {
            id: Uuid::new_v4().to_string(),
            name: self.name.trim().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdateCategoryDto {
    /// Nuovo nome della categoria
    #[schema(example = "Marketing", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "Il nome è obbligatorio e non può essere vuoto"
        ),
        length(max = 200, message = "Il nome deve essere di massimo 200 caratteri")
    )]
    pub name: String,
}

impl UpdateCategoryDto {
    /// Converte il DTO nell'entità pronta per il database: riusa l'id del path
    /// e normalizza il nome (la validazione è già stata fatta dall'extractor).
    pub fn into_entity(self, id: String) -> Category {
        Category {
            id,
            name: self.name.trim().to_string(),
        }
    }
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
