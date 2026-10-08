use serde::{Deserialize, Serialize};

/// Database-independent entity: any repository must persist and return
/// exactly this type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Id of the associated category (`None` if the report has no category).
    pub category_id: Option<String>,
    /// Id of the associated place (`None` if the report has no place).
    pub place_id: Option<String>,
}
