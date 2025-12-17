pub mod handler;
pub mod models;
pub mod provider;
pub mod security;
pub mod verify_user;

use crate::AppState;
use axum::{Router, routing::get, routing::post};
use handler::{login, refreshtoken, signup};
use provider::{oauth_callback, provider_handler};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};

pub fn router(app_state: AppState) -> Router {
    // Strict rate limit for login/signup: 5 requests allowed, refills 1 per 12 seconds
    // This prevents brute force attacks while allowing legitimate retries
    let auth_governor_conf = GovernorConfigBuilder::default()
        .per_second(12) // 1 token per 12 seconds refill rate
        .burst_size(5) // Allow burst of 5 requests
        .finish()
        .expect("Failed to build auth rate limiter config");

    // More lenient rate limit for token refresh: 10 requests allowed, refills faster
    let refresh_governor_conf = GovernorConfigBuilder::default()
        .per_second(6) // 1 token per 6 seconds refill rate
        .burst_size(10) // Allow burst of 10 requests
        .finish()
        .expect("Failed to build refresh rate limiter config");

    // Rate-limited auth routes (login, signup)
    let auth_routes = Router::new()
        .route("/login", post(login))
        .route("/signup", post(signup))
        .layer(GovernorLayer::new(auth_governor_conf));

    // Rate-limited refresh token route
    let refresh_routes = Router::new()
        .route("/refresh-token", post(refreshtoken))
        .layer(GovernorLayer::new(refresh_governor_conf));

    // OAuth routes (less strict - they redirect to external providers)
    let oauth_routes = Router::new()
        .route("/oauth/{provider}", get(provider_handler))
        .route("/oauth/{provider}/callback", get(oauth_callback));

    Router::new()
        .merge(auth_routes)
        .merge(refresh_routes)
        .merge(oauth_routes)
        .with_state(app_state)
}
