use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use subtle::ConstantTimeEq;
use tracing::warn;

use crate::handlers::AppState;
use crate::models::ApiError;

const API_KEY_HEADER: &str = "x-api-key";

/// Middleware that requires a valid API key in the `X-API-Key` header.
///
/// Uses a constant-time comparison to prevent timing attacks against the
/// configured service API key.
pub async fn require_api_key(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let provided = match req.headers().get(API_KEY_HEADER).and_then(|v| v.to_str().ok()) {
        Some(v) => v,
        None => return unauthorized("missing API key"),
    };

    let expected = state.config.api_key();

    // Security: constant-time comparison to prevent timing side-channels.
    // Lengths must match first (length is not secret).
    let matches = provided.len() == expected.len()
        && provided.as_bytes().ct_eq(expected.as_bytes()).into();

    if !matches {
        warn!(target: "auth", "API key authentication failed");
        return unauthorized("invalid API key");
    }

    next.run(req).await
}

fn unauthorized(msg: &'static str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiError::new("unauthorized", msg)),
    )
        .into_response()
}
