use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use tracing::{error, info};

use crate::client::{ClientError, SuccessFactorsClient};
use crate::models::*;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub client: SuccessFactorsClient,
}

/// Query parameters for pagination
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub top: Option<u32>,
    pub skip: Option<u32>,
}

/// Health check endpoint
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Test SuccessFactors connection
pub async fn test_connection(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ApiError>)> {
    info!("Testing SuccessFactors connection");

    match state.client.test_connection().await {
        Ok(true) => Ok(Json(serde_json::json!({
            "status": "connected",
            "message": "Successfully connected to SuccessFactors API"
        }))),
        Ok(false) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiError::new("connection_failed", "Failed to connect to SuccessFactors API")),
        )),
        Err(e) => {
            error!("Connection test failed: {}", e);
            Err(handle_client_error(e))
        }
    }
}

/// Get all users
pub async fn get_users(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<User>>, (StatusCode, Json<ApiError>)> {
    info!("Fetching users with params: {:?}", params);

    state
        .client
        .get_users(params.top, params.skip)
        .await
        .map(Json)
        .map_err(handle_client_error)
}

/// Get a specific user by ID
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<User>, (StatusCode, Json<ApiError>)> {
    info!("Fetching user: {}", user_id);

    state
        .client
        .get_user(&user_id)
        .await
        .map(Json)
        .map_err(handle_client_error)
}

/// Get employee personal information
pub async fn get_personal_info(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<PerPersonal>>, (StatusCode, Json<ApiError>)> {
    info!("Fetching personal info with params: {:?}", params);

    state
        .client
        .get_per_personal(params.top, params.skip)
        .await
        .map(Json)
        .map_err(handle_client_error)
}

/// Get employee job information
pub async fn get_job_info(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<EmpJob>>, (StatusCode, Json<ApiError>)> {
    info!("Fetching job info with params: {:?}", params);

    state
        .client
        .get_emp_job(params.top, params.skip)
        .await
        .map(Json)
        .map_err(handle_client_error)
}

/// Get employee employment information
pub async fn get_employment_info(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<EmpEmployment>>, (StatusCode, Json<ApiError>)> {
    info!("Fetching employment info with params: {:?}", params);

    state
        .client
        .get_emp_employment(params.top, params.skip)
        .await
        .map(Json)
        .map_err(handle_client_error)
}

/// Get employee email information
pub async fn get_emails(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<PerEmail>>, (StatusCode, Json<ApiError>)> {
    info!("Fetching email info with params: {:?}", params);

    state
        .client
        .get_per_email(params.top, params.skip)
        .await
        .map(Json)
        .map_err(handle_client_error)
}

/// Get employee phone information
pub async fn get_phones(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<PerPhone>>, (StatusCode, Json<ApiError>)> {
    info!("Fetching phone info with params: {:?}", params);

    state
        .client
        .get_per_phone(params.top, params.skip)
        .await
        .map(Json)
        .map_err(handle_client_error)
}

/// Convert client errors to HTTP responses
fn handle_client_error(error: ClientError) -> (StatusCode, Json<ApiError>) {
    let status_code = error.status_code();
    let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let api_error = match &error {
        ClientError::Api { message, .. } => ApiError::new("api_error", message),
        ClientError::Request(msg) => ApiError::new("request_error", msg),
        ClientError::HttpClient(msg) => ApiError::new("client_error", msg),
        ClientError::Parse(msg) => ApiError::new("parse_error", msg),
    };
    (status, Json(api_error))
}
