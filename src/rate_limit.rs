use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::warn;

use crate::models::ApiError;

type IpLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Per-IP token-bucket rate limiter.
///
/// Bounded map so a burst of unique IPs cannot exhaust memory.
#[derive(Clone)]
pub struct RateLimitState {
    quota: Quota,
    limiters: Arc<Mutex<HashMap<IpAddr, Arc<IpLimiter>>>>,
    max_entries: usize,
}

impl RateLimitState {
    pub fn new(rps: u32) -> Self {
        let rps = NonZeroU32::new(rps.max(1)).expect("rps >= 1");
        Self {
            quota: Quota::per_second(rps).allow_burst(rps),
            limiters: Arc::new(Mutex::new(HashMap::new())),
            max_entries: 10_000,
        }
    }

    async fn limiter_for(&self, ip: IpAddr) -> Arc<IpLimiter> {
        let mut map = self.limiters.lock().await;
        if let Some(existing) = map.get(&ip) {
            return existing.clone();
        }
        // Bound memory: if full, drop an arbitrary entry before inserting.
        if map.len() >= self.max_entries {
            if let Some(k) = map.keys().next().copied() {
                map.remove(&k);
            }
        }
        let limiter = Arc::new(RateLimiter::direct(self.quota));
        map.insert(ip, limiter.clone());
        limiter
    }
}

/// Middleware enforcing per-IP rate limits.
pub async fn rate_limit(
    State(state): State<RateLimitState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let limiter = state.limiter_for(addr.ip()).await;
    if limiter.check().is_err() {
        warn!(target: "rate_limit", ip = %addr.ip(), "rate limit exceeded");
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [("retry-after", "1")],
            Json(ApiError::new("rate_limited", "request rate exceeded")),
        )
            .into_response();
    }
    next.run(req).await
}
