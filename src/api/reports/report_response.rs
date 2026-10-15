use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::api::categories::category_response::CategoryResponse;
use crate::api::places::place_response::PlaceResponse;
use crate::api::reports::report_entity::Report;
use crate::api::users::user_response::UserSummaryResponse;
use crate::framework::Response;

/// Report as returned to the client.
///
/// The relations (`category`, `place`, `created_by`) are nested objects
/// resolved by the database in the same query: the derive turns every
/// `#[response(relation = ...)]` field into a nested projection, so listing N
/// reports is still a single `SELECT VALUE` instead of N+1 queries. The
/// foreign keys are never exposed, only the resolved objects (null when the
/// relationship is absent or broken).
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Response)]
#[response(entity = Report, table = "report")]
pub struct ReportResponse {
    /// UUID identifier of the report
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub id: String,
    /// Report name
    #[schema(example = "Vendite mensili")]
    pub name: String,
    /// Report description
    #[schema(example = "Report sulle vendite del mese corrente")]
    pub description: String,
    /// Associated category (full object, present only if the relationship exists)
    #[response(relation = "category_id", table = "category")]
    pub category: Option<CategoryResponse>,
    /// Associated place (full object, present only if the relationship exists)
    #[response(relation = "place_id", table = "place")]
    pub place: Option<PlaceResponse>,
    /// User who created the report (profile without the email, `null` only
    /// for legacy rows created before authentication existed)
    #[response(relation = "user_id", table = "user")]
    pub created_by: Option<UserSummaryResponse>,
}
