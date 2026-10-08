use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::places::place_entity::Place;
use crate::lib::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreatePlaceDto {
    /// City name
    #[schema(example = "Milano", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The city is required and cannot be empty"
        ),
        length(max = 200, message = "The city must be at most 200 characters")
    )]
    pub city: String,
    /// Street address
    #[schema(example = "Via Roma 1", min_length = 1, max_length = 300)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The address is required and cannot be empty"
        ),
        length(max = 300, message = "The address must be at most 300 characters")
    )]
    pub address: String,
    /// Postal code
    #[schema(example = "20100", min_length = 1, max_length = 10)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The postal code is required and cannot be empty"
        ),
        length(max = 10, message = "The postal code must be at most 10 characters")
    )]
    pub cap: String,
    /// Provenance
    #[schema(example = "Comune di Milano", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The provenance is required and cannot be empty"
        ),
        length(max = 200, message = "The provenance must be at most 200 characters")
    )]
    pub provenance: String,
}

impl CreatePlaceDto {
    /// Converts the DTO into the entity ready for the database: generates the
    /// id and normalizes the fields (validation was already done by the
    /// extractor).
    pub fn into_entity(self) -> Place {
        Place {
            id: Uuid::new_v4().to_string(),
            city: self.city.trim().to_string(),
            address: self.address.trim().to_string(),
            cap: self.cap.trim().to_string(),
            provenance: self.provenance.trim().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct UpdatePlaceDto {
    /// City name
    #[schema(example = "Torino", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The city is required and cannot be empty"
        ),
        length(max = 200, message = "The city must be at most 200 characters")
    )]
    pub city: String,
    /// Street address
    #[schema(example = "Corso Francia 10", min_length = 1, max_length = 300)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The address is required and cannot be empty"
        ),
        length(max = 300, message = "The address must be at most 300 characters")
    )]
    pub address: String,
    /// Postal code
    #[schema(example = "10100", min_length = 1, max_length = 10)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The postal code is required and cannot be empty"
        ),
        length(max = 10, message = "The postal code must be at most 10 characters")
    )]
    pub cap: String,
    /// Provenance
    #[schema(example = "Comune di Torino", min_length = 1, max_length = 200)]
    #[validate(
        custom(
            function = "crate::lib::validators::not_blank",
            message = "The provenance is required and cannot be empty"
        ),
        length(max = 200, message = "The provenance must be at most 200 characters")
    )]
    pub provenance: String,
}

impl UpdatePlaceDto {
    /// Converts the DTO into the entity ready for the database: reuses the id
    /// from the path and normalizes the fields (validation was already done by
    /// the extractor).
    pub fn into_entity(self, id: String) -> Place {
        Place {
            id,
            city: self.city.trim().to_string(),
            address: self.address.trim().to_string(),
            cap: self.cap.trim().to_string(),
            provenance: self.provenance.trim().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PlaceResponse {
    /// UUID identifier of the place
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// City name
    #[schema(example = "Milano")]
    pub city: String,
    /// Street address
    #[schema(example = "Via Roma 1")]
    pub address: String,
    /// Postal code
    #[schema(example = "20100")]
    pub cap: String,
    /// Provenance
    #[schema(example = "Comune di Milano")]
    pub provenance: String,
}

impl From<crate::api::places::place_entity::Place> for PlaceResponse {
    fn from(place: crate::api::places::place_entity::Place) -> Self {
        Self {
            id: place.id,
            city: place.city,
            address: place.address,
            cap: place.cap,
            provenance: place.provenance,
        }
    }
}
