use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};

use crate::client::{ClientError, SuccessFactorsClient};
use crate::config::{Config, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, MAX_SKIP};
use crate::models::*;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub client: SuccessFactorsClient,
    pub config: Arc<Config>,
}

/// Raw pagination input from the URL. Validated via `Pagination::parse`.
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default)]
    pub top: Option<u32>,
    #[serde(default)]
    pub skip: Option<u32>,
}

/// Validated pagination values clamped to safe bounds.
pub struct Pagination {
    pub top: u32,
    pub skip: u32,
}

impl PaginationParams {
    /// Clamp incoming pagination to safe server-side bounds.
    /// Rejects obviously malicious values (skip beyond MAX_SKIP).
    pub fn parse(self) -> Result<Pagination, (StatusCode, Json<ApiError>)> {
        let top = self.top.unwrap_or(DEFAULT_PAGE_SIZE);
        let skip = self.skip.unwrap_or(0);

        if top == 0 {
            return Err(bad_request("top must be >= 1"));
        }
        if top > MAX_PAGE_SIZE {
            return Err(bad_request("top exceeds maximum page size"));
        }
        if skip > MAX_SKIP {
            return Err(bad_request("skip exceeds maximum allowed"));
        }

        Ok(Pagination { top, skip })
    }
}

/// Max length of a user ID accepted on the path.
const MAX_USER_ID_LEN: usize = 128;

/// Validate a user ID path parameter. Only accepts a conservative charset
/// commonly used for SuccessFactors user IDs.
fn validate_user_id(id: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
    if id.is_empty() || id.len() > MAX_USER_ID_LEN {
        return Err(bad_request("user_id has invalid length"));
    }
    // Allow alphanumerics, plus common safe separators. Blocks quotes, slashes, ()
    // which could attempt OData or URL structure tampering.
    let ok = id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@'));
    if !ok {
        return Err(bad_request("user_id contains disallowed characters"));
    }
    Ok(())
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    })
}

pub async fn test_connection(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    info!("testing SuccessFactors connection");

    match state.client.test_connection().await {
        Ok(true) => Ok(Json(serde_json::json!({
            "status": "connected"
        }))),
        Ok(false) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError::new("connection_failed", "upstream unavailable")),
        )),
        Err(e) => {
            error!(error = %e, "connection test failed");
            Err(map_error(e))
        }
    }
}

pub async fn get_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<User>>, (StatusCode, Json<ApiError>)> {
    let p = params.parse()?;
    state
        .client
        .get_users(p.top, p.skip)
        .await
        .map(Json)
        .map_err(map_error)
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<User>, (StatusCode, Json<ApiError>)> {
    validate_user_id(&user_id)?;
    state
        .client
        .get_user(&user_id)
        .await
        .map(Json)
        .map_err(map_error)
}

pub async fn get_personal_info(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<PerPersonal>>, (StatusCode, Json<ApiError>)> {
    let p = params.parse()?;
    state
        .client
        .get_per_personal(p.top, p.skip)
        .await
        .map(Json)
        .map_err(map_error)
}

pub async fn get_job_info(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<EmpJob>>, (StatusCode, Json<ApiError>)> {
    let p = params.parse()?;
    state
        .client
        .get_emp_job(p.top, p.skip)
        .await
        .map(Json)
        .map_err(map_error)
}

pub async fn get_employment_info(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<EmpEmployment>>, (StatusCode, Json<ApiError>)> {
    let p = params.parse()?;
    state
        .client
        .get_emp_employment(p.top, p.skip)
        .await
        .map(Json)
        .map_err(map_error)
}

pub async fn get_emails(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<PerEmail>>, (StatusCode, Json<ApiError>)> {
    let p = params.parse()?;
    state
        .client
        .get_per_email(p.top, p.skip)
        .await
        .map(Json)
        .map_err(map_error)
}

pub async fn get_phones(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<PerPhone>>, (StatusCode, Json<ApiError>)> {
    let p = params.parse()?;
    state
        .client
        .get_per_phone(p.top, p.skip)
        .await
        .map(Json)
        .map_err(map_error)
}

fn bad_request(msg: &'static str) -> (StatusCode, Json<ApiError>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError::new("bad_request", msg)),
    )
}

/// Maps upstream errors to sanitized client responses.
///
/// Security: never forward raw upstream error bodies or status codes verbatim.
/// Upstream 401/403 becomes an opaque 502 because the service - not the
/// caller - is misconfigured, and a mismatch between upstream auth and our
/// own auth should not leak to the client.
fn map_error(err: ClientError) -> (StatusCode, Json<ApiError>) {
    error!(error = %err, "upstream client error");
    match err {
        ClientError::Api { status } => match status {
            404 => (
                StatusCode::NOT_FOUND,
                Json(ApiError::new("not_found", "resource not found")),
            ),
            400 => (
                StatusCode::BAD_REQUEST,
                Json(ApiError::new("bad_request", "invalid request")),
            ),
            429 => (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ApiError::new("upstream_throttled", "try again later")),
            ),
            _ => (
                StatusCode::BAD_GATEWAY,
                Json(ApiError::new("upstream_error", "upstream request failed")),
            ),
        },
        ClientError::ResponseTooLarge => (
            StatusCode::BAD_GATEWAY,
            Json(ApiError::new("upstream_error", "response too large")),
        ),
        ClientError::Request(_) | ClientError::HttpClient(_) => (
            StatusCode::BAD_GATEWAY,
            Json(ApiError::new("upstream_error", "upstream request failed")),
        ),
        ClientError::Parse => (
            StatusCode::BAD_GATEWAY,
            Json(ApiError::new("upstream_error", "invalid upstream response")),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_user_ids_accepted() {
        assert!(validate_user_id("abc123").is_ok());
        assert!(validate_user_id("first.last").is_ok());
        assert!(validate_user_id("user@company.com").is_ok());
        assert!(validate_user_id("user_1-2").is_ok());
    }

    #[test]
    fn malicious_user_ids_rejected() {
        assert!(validate_user_id("").is_err());
        assert!(validate_user_id("ab'c").is_err());
        assert!(validate_user_id("../etc/passwd").is_err());
        assert!(validate_user_id("user')%20or%20('1'='1").is_err());
        assert!(validate_user_id(&"x".repeat(200)).is_err());
    }

    #[test]
    fn pagination_validated() {
        assert!(PaginationParams { top: Some(0), skip: None }.parse().is_err());
        assert!(PaginationParams { top: Some(MAX_PAGE_SIZE + 1), skip: None }.parse().is_err());
        assert!(PaginationParams { top: None, skip: Some(MAX_SKIP + 1) }.parse().is_err());
        assert!(PaginationParams { top: Some(10), skip: Some(0) }.parse().is_ok());
    }
}
