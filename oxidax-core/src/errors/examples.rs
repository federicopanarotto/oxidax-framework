//! Automatic OpenAPI examples for the error responses.
//!
//! The runtime body of every error is [`crate::errors::ErrorResponse`], but the
//! generated document should show, for each status, an example that actually
//! matches the response (`code`, `error` and a human message). Writing that
//! example by hand in every `#[utoipa::path]` is noisy and, worse, easy to get
//! wrong (all the responses end up showing the same `not_found` example).
//!
//! This module centralizes the information instead: [`error_spec`] knows the
//! machine `code` of every documented error status, and [`ErrorExamples`]
//! injects the shared `ErrorResponse` schema plus the matching example into the
//! final document. Controllers only declare `(status = ..., description = "...")`
//! and the example follows automatically, using the description as the message.
//!
//! [`ErrorExamples`] is applied by [`crate::docs::build`] **after** the feature
//! fragments have been merged, so it sees every path.

use serde_json::json;
use utoipa::Modify;
use utoipa::openapi::path::{Operation, PathItem};
use utoipa::openapi::{Content, OpenApi, Ref, RefOr};

/// Injects the canonical `ErrorResponse` body and example into every documented
/// error status. The response `description` is reused as the example message, so
/// it is the only thing controllers need to write.
pub struct ErrorExamples;

/// Canonical machine `code` and status line of a documented error status.
struct ErrorSpec {
    code: &'static str,
    status: &'static str,
}

/// The single source of truth for the error statuses exposed in the document.
///
/// Extend this table (and nothing else) when a new error status is documented.
fn error_spec(status: &str) -> Option<ErrorSpec> {
    let spec = match status {
        "400" => ErrorSpec {
            code: "bad_request",
            status: "400 Bad Request",
        },
        "401" => ErrorSpec {
            code: "unauthorized",
            status: "401 Unauthorized",
        },
        "403" => ErrorSpec {
            code: "forbidden",
            status: "403 Forbidden",
        },
        "404" => ErrorSpec {
            code: "not_found",
            status: "404 Not Found",
        },
        "409" => ErrorSpec {
            code: "conflict",
            status: "409 Conflict",
        },
        "413" => ErrorSpec {
            code: "payload_too_large",
            status: "413 Payload Too Large",
        },
        "415" => ErrorSpec {
            code: "unsupported_media_type",
            status: "415 Unsupported Media Type",
        },
        "422" => ErrorSpec {
            code: "validation_error",
            status: "422 Unprocessable Entity",
        },
        "500" => ErrorSpec {
            code: "internal_error",
            status: "500 Internal Server Error",
        },
        "503" => ErrorSpec {
            code: "database_unavailable",
            status: "503 Service Unavailable",
        },
        _ => return None,
    };
    Some(spec)
}

impl Modify for ErrorExamples {
    fn modify(&self, openapi: &mut OpenApi) {
        for path_item in openapi.paths.paths.values_mut() {
            for_each_operation(path_item, |operation| {
                for (status, response) in operation.responses.responses.iter_mut() {
                    let Some(spec) = error_spec(status) else {
                        continue;
                    };
                    let RefOr::T(response) = response else {
                        continue;
                    };

                    let message = if response.description.is_empty() {
                        spec.status.to_string()
                    } else {
                        response.description.clone()
                    };
                    let example = json!({
                        "code": spec.code,
                        "error": spec.status,
                        "message": message,
                    });

                    let mut content = Content::new(Some(Ref::from_schema_name("ErrorResponse")));
                    content.example = Some(example);
                    response
                        .content
                        .insert("application/json".to_string(), RefOr::T(content));
                }
            });
        }
    }
}

/// Visits every [`Operation`] of a [`PathItem`] (including the less common
/// methods and any additional one).
fn for_each_operation(path_item: &mut PathItem, mut visit: impl FnMut(&mut Operation)) {
    let operations = [
        &mut path_item.get,
        &mut path_item.put,
        &mut path_item.post,
        &mut path_item.delete,
        &mut path_item.options,
        &mut path_item.head,
        &mut path_item.patch,
        &mut path_item.trace,
        &mut path_item.query,
    ];
    for operation in operations.into_iter().flatten() {
        visit(operation);
    }
    for operation in path_item.additional_operations.values_mut() {
        visit(operation);
    }
}
