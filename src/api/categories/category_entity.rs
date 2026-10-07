use serde::{Deserialize, Serialize};

/// Entità indipendente dal database: qualsiasi repository deve persistere
/// e restituire esattamente questo tipo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
}
