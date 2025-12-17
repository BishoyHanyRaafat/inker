use chrono::Duration;
use std::sync::LazyLock;

pub const EXPIRES_IN_JWT_ACCESS_TOKEN: Duration = Duration::days(1);
pub const EXPIRES_IN_JWT_REFRESH_TOKEN: Duration = Duration::weeks(2); // 14 days

/// Debug mode - controls verbose logging and Swagger UI availability
/// Set via DEBUG environment variable (defaults to false in production)
pub static DEBUG: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("DEBUG")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(cfg!(debug_assertions))
});

/// Frontend URL for CORS and redirects
pub static FRONTEND_URL: LazyLock<String> = LazyLock::new(|| {
    std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3001".to_string())
});

/// Backend URL for OAuth callbacks
pub static BACKEND_URL: LazyLock<String> = LazyLock::new(|| {
    std::env::var("BACKEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
});

/// Default redirect after OAuth provider authentication
pub static REDIRECT_DEFAULT_AFTER_PROVIDER: LazyLock<String> = LazyLock::new(|| {
    std::env::var("REDIRECT_DEFAULT_AFTER_PROVIDER").unwrap_or_else(|_| FRONTEND_URL.clone())
});

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
