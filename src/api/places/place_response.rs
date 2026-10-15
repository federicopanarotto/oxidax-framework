use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::places::place_entity::Place;
use crate::framework::Response;

/// Place as returned to the client. It is also nested inside the report
/// response: the derive teaches it how to project itself from the `place`
/// table, so a report can resolve it in the same query.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Response)]
#[serde(rename_all = "camelCase")]
#[response(entity = Place, table = "place")]
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
