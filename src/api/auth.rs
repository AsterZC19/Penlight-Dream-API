//! Optional API-key authentication middleware.
//!
//! When `API_KEY` is configured, every `/api/*` request must present the key
//! either in the `X-API-Key` header or as `Authorization: Bearer <key>`.
//! With an empty key the middleware is a no-op.

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::api::SharedState;
use crate::error::{AppError, AppResult};

/// Compares two strings in time independent of their contents using a
/// byte-wise XOR fold. String lengths still leak, which is acceptable for an
/// API key.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

pub async fn require_api_key(State(state): State<SharedState>, request: Request, next: Next) -> AppResult<Response> {
    let expected = state.config.api_key.clone();
    if expected.is_empty() {
        return Ok(next.run(request).await);
    }

    let provided = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            request
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|auth| auth.strip_prefix("Bearer "))
                .map(|rest| rest.trim().to_string())
        });

    if provided.as_deref().is_some_and(|p| constant_time_eq(p, &expected)) {
        Ok(next.run(request).await)
    } else {
        Err(AppError::unauthorized("invalid or missing API key"))
    }
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn constant_time_eq_matches_identical_strings() {
        assert!(constant_time_eq("secret-key", "secret-key"));
        assert!(constant_time_eq("", ""));
    }

    #[test]
    fn constant_time_eq_rejects_any_difference() {
        assert!(!constant_time_eq("secret-key", "secret-keY"));
        assert!(!constant_time_eq("secret-key", "secret-key-longer"));
        assert!(!constant_time_eq("secret-key", ""));
    }
}
