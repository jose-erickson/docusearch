mod auth;
mod client;
mod config;
mod handlers;
mod models;
mod rate_limit;

use axum::{
    http::{header, HeaderValue, Method},
    middleware,
    routing::get,
    Router,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::client::SuccessFactorsClient;
use crate::config::Config;
use crate::handlers::AppState;
use crate::rate_limit::RateLimitState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // JSON-structured logs by default for easier SIEM ingestion.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,successfactors_microservice=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    info!(addr = %config.server_addr(), "configuration loaded");

    let sf_client = SuccessFactorsClient::new(&config)?;
    let config = Arc::new(config);
    let state = AppState {
        client: sf_client,
        config: config.clone(),
    };

    let rate_state = RateLimitState::new(config.rate_limit_rps);

    // CORS: restrictive by default; only allow origins explicitly listed.
    let cors = if config.cors_allowed_origins.is_empty() {
        CorsLayer::new()
    } else {
        let origins: Vec<HeaderValue> = config
            .cors_allowed_origins
            .iter()
            .filter_map(|o| HeaderValue::from_str(o).ok())
            .collect();
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([Method::GET])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::HeaderName::from_static("x-api-key")])
            .max_age(Duration::from_secs(600))
    };

    // Protected routes require a valid API key.
    let protected = Router::new()
        .route("/api/test-connection", get(handlers::test_connection))
        .route("/api/users", get(handlers::get_users))
        .route("/api/users/:user_id", get(handlers::get_user))
        .route("/api/employees/personal", get(handlers::get_personal_info))
        .route("/api/employees/jobs", get(handlers::get_job_info))
        .route("/api/employees/employment", get(handlers::get_employment_info))
        .route("/api/employees/emails", get(handlers::get_emails))
        .route("/api/employees/phones", get(handlers::get_phones))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_api_key,
        ));

    let app = Router::new()
        .route("/health", get(handlers::health))
        .merge(protected)
        .layer(middleware::from_fn_with_state(
            rate_state.clone(),
            rate_limit::rate_limit,
        ))
        // Security headers applied to every response.
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
        ))
        .layer(cors)
        .layer(RequestBodyLimitLayer::new(config.max_body_bytes))
        .layer(TimeoutLayer::new(Duration::from_secs(
            config.request_timeout_secs,
        )))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = config.server_addr().parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(bind = %addr, "server listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    info!("server shut down cleanly");
    Ok(())
}

/// Graceful shutdown on SIGINT/SIGTERM - drain in-flight requests.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = signal::ctrl_c().await {
            warn!(error = %e, "failed to install SIGINT handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(e) => warn!(error = %e, "failed to install SIGTERM handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received SIGINT, shutting down"),
        _ = terminate => info!("received SIGTERM, shutting down"),
    }
}
