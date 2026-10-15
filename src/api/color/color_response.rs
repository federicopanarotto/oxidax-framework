use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::color::color_entity::Color;
use crate::framework::Response;

/// Color as returned to the client. The derive lets the repository project it
/// directly from the `color` table.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Response)]
#[serde(rename_all = "camelCase")]
#[response(entity = Color, table = "color")]
pub struct ColorResponse {
    /// UUID identifier of the color
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// Color name
    #[schema(example = "Rosso")]
    pub name: String,
    /// Hexadecimal value
    #[schema(example = "#FF0000")]
    pub hex: String,
}
