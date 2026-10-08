use serde::{Deserialize, Serialize};

/// Database-independent entity: any repository must persist and return
/// exactly this type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    pub id: String,
    pub city: String,
    pub address: String,
    pub cap: String,
    pub provenance: String,
}
