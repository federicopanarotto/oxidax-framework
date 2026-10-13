//! CORS middleware.
//!
//! The allowed origins are configured as a list of regex patterns matched
//! against the request's `Origin` header (scheme + host + port), so a single
//! entry can cover several subdomains, e.g. `^https://(.+\.)?example\.com$`.
//!
//! `tower-http`'s `CorsLayer` only supports exact origins out of the box; the
//! `AllowOrigin::predicate` hook lets us apply the regex allow-list instead.
//! The layer also answers preflight (`OPTIONS`) requests and exposes the
//! pagination header (`X-TOTAL-LIST`) to browsers.

use axum::http::{HeaderName, HeaderValue, Method, header, request::Parts};
use regex::Regex;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Builds the CORS layer for the given origin patterns.
///
/// Returns `None` when the list is empty, which means CORS is disabled: the
/// browser receives no CORS headers and blocks cross-origin requests.
pub fn layer(patterns: &[Regex]) -> Option<CorsLayer> {
    if patterns.is_empty() {
        return None;
    }

    let patterns = patterns.to_vec();

    Some(
        CorsLayer::new()
            .allow_origin(AllowOrigin::predicate(
                move |origin: &HeaderValue, _parts: &Parts| {
                    origin
                        .to_str()
                        .map(|value| patterns.iter().any(|pattern| pattern.is_match(value)))
                        .unwrap_or(false)
                },
            ))
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
            .expose_headers([HeaderName::from_static("x-total-list")]),
    )
}
