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

    if provided.as_deref() == Some(expected.as_str()) {
        Ok(next.run(request).await)
    } else {
        Err(AppError::unauthorized("invalid or missing API key"))
    }
}
