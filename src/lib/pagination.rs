//! Reusable pagination.
//!
//! Every list endpoint shares the same pieces, so they live here once instead
//! of being re-implemented per feature:
//!
//! - [`PaginationQuery`]: the `?page=&limit=` query parameters, deserialized
//!   and validated by the `ValidatedQuery` extractor like any other DTO;
//! - [`Pagination`]: the resolved value object (`page`, `limit`, `offset`)
//!   passed to the services and then to the repositories;
//! - [`Paged<T>`]: what a service returns — the plain list of elements **plus**
//!   the total count. The controller turns it into the HTTP response with
//!   [`Paged::into_response`], which keeps the body as a bare JSON array (no
//!   envelope) and sets the total in the `X-TOTAL-LIST` header.
//!
//! A feature only adds one extractor and returns a [`Paged`]:
//!
//! ```ignore
//! pub async fn list_things(
//!     State(service): State<Arc<ThingService>>,
//!     ValidatedQuery(pagination): ValidatedQuery<PaginationQuery>,
//! ) -> AppResult<Response> {
//!     let page = service.list_page(pagination.pagination()).await?;
//!     Ok(page.into_response())
//! }
//! ```

use axum::Json;
use axum::http::HeaderValue;
use axum::http::header::HeaderName;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde::Serialize;
use utoipa::ToSchema;
use validator::Validate;

/// Page used when the `page` parameter is absent.
pub const DEFAULT_PAGE: u64 = 1;
/// Page size used when the `limit` parameter is absent.
pub const DEFAULT_LIMIT: u64 = 20;
/// Hard cap on the page size, so a single request can never ask for the whole
/// table.
pub const MAX_LIMIT: u64 = 100;
/// Header carrying the total number of elements across every page.
pub const TOTAL_HEADER: &str = "x-total-list";

/// Query parameters shared by every paginated list endpoint.
///
/// Both fields are optional: absent means "use the default". They are
/// validated (`422` outside the range) by the `ValidatedQuery` extractor.
#[derive(Debug, Clone, Copy, Default, Deserialize, ToSchema, Validate)]
pub struct PaginationQuery {
    /// 1-based page number
    #[serde(default)]
    #[schema(example = 1, minimum = 1, default = 1)]
    #[validate(range(min = 1, message = "The page must be at least 1"))]
    pub page: Option<u64>,
    /// Number of elements per page
    #[serde(default)]
    #[schema(example = 20, minimum = 1, maximum = 100, default = 20)]
    #[validate(range(min = 1, max = 100, message = "The limit must be between 1 and 100"))]
    pub limit: Option<u64>,
}

impl PaginationQuery {
    /// Resolves the query parameters into a [`Pagination`], applying the
    /// defaults.
    pub fn pagination(&self) -> Pagination {
        Pagination::new(self.page, self.limit)
    }
}

/// Resolved pagination: the query parameters with the defaults applied and
/// clamped to the safe range.
///
/// It is a plain value object: the repositories receive `offset`/`limit`, the
/// services reuse it when slicing and counting.
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    page: u64,
    limit: u64,
}

impl Pagination {
    /// Builds the pagination from the optional query values. `None` falls back
    /// to [`DEFAULT_PAGE`]/[`DEFAULT_LIMIT`]; out-of-range values are clamped
    /// as a defensive measure (they are already rejected by the DTO
    /// validation).
    pub fn new(page: Option<u64>, limit: Option<u64>) -> Self {
        Self {
            page: page.unwrap_or(DEFAULT_PAGE).max(1),
            limit: limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT),
        }
    }

    /// 1-based page number.
    pub fn page(&self) -> u64 {
        self.page
    }

    /// Number of elements per page.
    pub fn limit(&self) -> u64 {
        self.limit
    }

    /// Number of elements to skip: `(page - 1) * limit`.
    pub fn offset(&self) -> u64 {
        (self.page - 1) * self.limit
    }
}

/// List payload returned by the services: the page elements and the total
/// count across every page.
///
/// It is **not** serialized as an envelope: [`Paged::into_response`] sends
/// `items` as a bare JSON array and puts `total` in the [`TOTAL_HEADER`]
/// response header, so list endpoints keep returning the same body shape as
/// before. The controller calls it explicitly, so the response contract stays
/// visible in the handler.
#[derive(Debug, Clone)]
pub struct Paged<T> {
    /// Elements of the current page.
    pub items: Vec<T>,
    /// Total number of elements across every page.
    pub total: u64,
}

impl<T> Paged<T> {
    /// Wraps the page elements together with the total count.
    pub fn new(items: Vec<T>, total: u64) -> Self {
        Self { items, total }
    }

    /// Maps every element to another type, preserving the total: lets a
    /// service build a `Paged<Response>` from entities.
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Paged<U> {
        Paged {
            items: self.items.into_iter().map(&mut f).collect(),
            total: self.total,
        }
    }
}

impl<T: Serialize> Paged<T> {
    /// Builds the HTTP response of a paginated list: the body is the bare JSON
    /// array of the current page and the total number of elements travels in
    /// the [`TOTAL_HEADER`] header. Call it from the controller so the list
    /// endpoints share one response contract.
    pub fn into_response(self) -> Response {
        (
            [(
                HeaderName::from_static(TOTAL_HEADER),
                total_header_value(self.total),
            )],
            Json(self.items),
        )
            .into_response()
    }
}

/// Header values are ASCII digits, so the conversion can never fail: the
/// fallback keeps the code panic-free.
fn total_header_value(total: u64) -> HeaderValue {
    HeaderValue::from_str(&total.to_string()).unwrap_or_else(|_| HeaderValue::from_static("0"))
}
