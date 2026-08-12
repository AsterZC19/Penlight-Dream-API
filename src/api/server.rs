//! Server-region scoping for the HTTP API.
//!
//! Routes are scoped as `/api/{server}/...`. This project serves the JP region
//! only, so `{server}` must resolve to `jp` and anything else is rejected with
//! a 400. Keeping the segment in the path makes adding another region later a
//! config change rather than a route change.

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::error::{AppError, AppResult};

/// The only configured game server region.
pub const JP: &str = "jp";

/// Validates that a path segment names a configured server region.
pub fn resolve_server(server: &str) -> AppResult<()> {
    if server.eq_ignore_ascii_case(JP) {
        Ok(())
    } else {
        Err(AppError::bad_request(format!(
            "unsupported server \"{server}\", only \"{JP}\" is configured"
        )))
    }
}

/// Extracts the first path segment and validates it as a server region. The
/// layer runs on the nested API router, whose path already has the API prefix
/// stripped, so the server is always the leading segment.
pub async fn validate_server(request: Request, next: Next) -> AppResult<Response> {
    let path = request.uri().path();
    let server = path
        .split('/')
        .find(|s| !s.is_empty())
        .unwrap_or_default();
    resolve_server(server)?;
    Ok(next.run(request).await)
}
