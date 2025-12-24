mod client;
mod config;
mod handlers;
mod models;

use axum::{
    routing::get,
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::client::SuccessFactorsClient;
use crate::config::Config;
use crate::handlers::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "successfactors_microservice=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Create SuccessFactors client
    let sf_client = SuccessFactorsClient::new(&config)?;
    info!("SuccessFactors client initialized");

    // Create application state
    let state = AppState { client: sf_client };

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        // Health endpoints
        .route("/health", get(handlers::health))
        .route("/api/test-connection", get(handlers::test_connection))
        // User endpoints
        .route("/api/users", get(handlers::get_users))
        .route("/api/users/:user_id", get(handlers::get_user))
        // Employee Central endpoints
        .route("/api/employees/personal", get(handlers::get_personal_info))
        .route("/api/employees/jobs", get(handlers::get_job_info))
        .route("/api/employees/employment", get(handlers::get_employment_info))
        .route("/api/employees/emails", get(handlers::get_emails))
        .route("/api/employees/phones", get(handlers::get_phones))
        // Add middleware
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = config.server_addr();
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
