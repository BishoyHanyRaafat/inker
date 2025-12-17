#![allow(dead_code)]
//! Common test utilities and helpers for integration tests.
//!
//! This module provides reusable functions for creating test app states,
//! mock databases, and test servers.

use axum::http::StatusCode;
use axum_test::TestServer;
use inker_http::{AppState, OAuthProviderInfo, Providers};
use sea_orm::{DatabaseBackend, DatabaseConnection, MockDatabase};
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use user_agent_parser::UserAgentParser;
use uuid::Uuid;

/// Test configuration defaults
pub mod defaults {
    pub const JWT_SECRET: &str = "test_jwt_secret_key_for_testing_only_32bytes!";
    pub const GEMINI_API_KEY: &str = "test_gemini_api_key";
    pub const DEBUG: &str = "1";
}

/// Builder for creating test AppState with customizable components
pub struct TestAppStateBuilder {
    db: Option<DatabaseConnection>,
    ua_parser: Option<Arc<UserAgentParser>>,
    providers: Option<Providers>,
    http_client: Option<reqwest::Client>,
}

impl Default for TestAppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TestAppStateBuilder {
    /// Create a new builder with default values
    pub fn new() -> Self {
        Self {
            db: None,
            ua_parser: None,
            providers: None,
            http_client: None,
        }
    }

    /// Set a custom database connection
    pub fn with_db(mut self, db: DatabaseConnection) -> Self {
        self.db = Some(db);
        self
    }

    /// Set a custom UserAgentParser
    pub fn with_ua_parser(mut self, parser: Arc<UserAgentParser>) -> Self {
        self.ua_parser = Some(parser);
        self
    }

    /// Set custom OAuth providers
    pub fn with_providers(mut self, providers: Providers) -> Self {
        self.providers = Some(providers);
        self
    }

    /// Set a custom HTTP client
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Build the AppState
    pub fn build(self) -> AppState {
        AppState::new(
            self.db.unwrap_or_else(create_mock_db),
            self.ua_parser.unwrap_or_else(create_test_ua_parser),
            self.providers.unwrap_or_else(create_test_providers),
            self.http_client.unwrap_or_default(),
            create_test_gemini_client(),
            yt_processing::YouTubeCaptions::default(),
        )
    }
}

/// Creates a mock database connection for testing.
///
/// This creates an empty mock database with PostgreSQL backend.
/// Use `create_mock_db_with_results` for tests that need specific query results.
pub fn create_mock_db() -> DatabaseConnection {
    MockDatabase::new(DatabaseBackend::Postgres).into_connection()
}

/// Creates a mock database with predefined query results.
///
/// # Example
/// ```ignore
/// use entities::user;
/// let db = create_mock_db_with_results(vec![
///     vec![user::Model { id: 1, name: "test".into() }]
/// ]);
/// ```
pub fn create_mock_db_builder() -> MockDatabase {
    MockDatabase::new(DatabaseBackend::Postgres)
}

/// Creates a minimal UserAgentParser for testing.
///
/// Uses empty rules which is sufficient for most tests.
pub fn create_test_ua_parser() -> Arc<UserAgentParser> {
    // Use the same upstream UAP regexes file the binary uses in `main.rs`.
    // `user_agent_parser` expects that exact schema (our previous “empty YAML” fails with IncorrectSource).
    let yaml = include_str!("../../../resources/user_agent_regexes.yaml");
    Arc::new(UserAgentParser::from_str(yaml).expect("Failed to create test UA parser"))
}

/// Creates test OAuth providers with dummy credentials.
///
/// These providers are configured with test values and won't work
/// for actual OAuth flows, but are sufficient for testing route behavior.
pub fn create_test_providers() -> Providers {
    Providers {
        github: OAuthProviderInfo {
            auth_url: "https://github.com/login/oauth/authorize".into(),
            token_url: "https://github.com/login/oauth/access_token".into(),
            client_id: "test_github_client_id".into(),
            client_secret: "test_github_client_secret".into(),
            scopes: vec!["read:user".into(), "user:email".into()],
        },
        google: OAuthProviderInfo {
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token_url: "https://oauth2.googleapis.com/token".into(),
            client_id: "test_google_client_id".into(),
            client_secret: "test_google_client_secret".into(),
            scopes: vec!["profile".into(), "email".into()],
        },
    }
}

/// Creates a test Gemini client with a dummy API key.
pub fn create_test_gemini_client() -> ml_processing::GeminiClient {
    ml_processing::GeminiClient::new(defaults::GEMINI_API_KEY.into())
        .expect("Failed to create test Gemini client")
}

/// Creates a complete test AppState with sensible defaults.
///
/// This is the simplest way to get a working AppState for tests.
/// For more control, use `TestAppStateBuilder`.
pub fn create_test_app_state() -> AppState {
    TestAppStateBuilder::new().build()
}

/// Creates a test AppState with a custom mock database.
///
/// Useful when you need to set up specific query/exec results.
pub fn create_test_app_state_with_db(db: DatabaseConnection) -> AppState {
    TestAppStateBuilder::new().with_db(db).build()
}

/// Creates the application router for testing.
pub fn create_test_app(app_state: AppState) -> axum::Router {
    inker_http::create_app(app_state)
}

/// Creates a TestServer from an AppState.
///
/// This is a convenience function that combines app creation and server setup.
pub fn create_test_server(app_state: AppState) -> TestServer {
    let app = create_test_app(app_state);
    // `tower_governor` (used by auth routes) expects `ConnectInfo<SocketAddr>` so it can
    // extract a client key (IP) for rate limiting. axum-test's mock transport doesn't
    // provide that, so we run a real in-process HTTP server with connect-info enabled.
    let app = app.into_make_service_with_connect_info::<SocketAddr>();
    TestServer::new_with_config(
        app,
        axum_test::TestServerConfig {
            transport: Some(axum_test::Transport::HttpRandomPort),
            ..Default::default()
        },
    )
    .expect("Failed to create test server")
}

/// Creates a TestServer with default test configuration.
///
/// Sets up JWT environment variable and creates a server with mock database.
pub fn create_default_test_server() -> TestServer {
    setup_test_env();
    create_test_server(create_test_app_state())
}

/// Creates a TestServer with a custom mock database.
pub fn create_test_server_with_db(db: DatabaseConnection) -> TestServer {
    setup_test_env();
    create_test_server(create_test_app_state_with_db(db))
}

/// Sets up required environment variables for testing.
///
/// Call this at the start of each test that uses JWT functionality.
pub fn setup_test_env() {
    // Rust 2024: mutating process environment is `unsafe` due to potential UB with
    // concurrent environment access. Tests are single-process and we control usage here.
    unsafe {
        std::env::set_var("JWT", defaults::JWT_SECRET);
        std::env::set_var("DEBUG", defaults::DEBUG);
    }
}

/// Create a valid JWT access token (raw token string) for a user.
///
/// Notes:
/// - This only generates a signed JWT; it does **not** insert anything into the database.
/// - Callers can attach it with `axum_test::TestRequest::authorization_bearer(token)`.
pub fn valid_access_token(user_id: Uuid) -> String {
    use entities::sea_orm_active_enums::UserType;
    use inker_http::routes::auth::models::Claims;

    // Ensure JWT env var is set before `KEYS` is initialized.
    setup_test_env();

    Claims::default(UserType::Regular, "test-user".to_string(), user_id)
        .token()
        .expect("Failed to create JWT access token for tests")
        .token
}

/// Convenience helper to build the full `Authorization` header value.
pub fn valid_bearer(user_id: Uuid) -> String {
    format!("Bearer {}", valid_access_token(user_id))
}

/// Assert a response status and return the JSON body.
///
/// If the status doesn't match, this will `panic!` while printing:
/// - request method + URL
/// - response status + headers
/// - full response body
/// - the `debug` field (see `crates/http/src/responses.rs`, around lines 24-25)
pub fn assert_status_json(response: axum_test::TestResponse, expected: StatusCode) -> Value {
    let status = response.status_code();
    let method = response.request_method();
    let url = response.request_url();
    let headers = response.headers();
    let body_text = response.text();

    let parsed_json: Option<Value> = serde_json::from_str(&body_text).ok();
    let debug = parsed_json
        .as_ref()
        .and_then(|v| v.get("debug"))
        .cloned()
        .unwrap_or(Value::Null);

    if status != expected {
        let pretty_json = parsed_json
            .as_ref()
            .and_then(|v| serde_json::to_string_pretty(v).ok())
            .unwrap_or_else(|| "<non-json body>".to_string());

        panic!(
            "\nASSERT STATUS FAILED\n\
             Expected: {expected}\n\
             Actual:   {status}\n\
             Request:  {method} {url}\n\
             Headers:  {headers:?}\n\
             Debug:    {debug}\n\
             Body(raw):\n{body_text}\n\
             Body(json pretty):\n{pretty_json}\n"
        );
    }

    // If it's not JSON and status matched, still panic since callers expect JSON.
    parsed_json.unwrap_or_else(|| {
        let e = serde_json::from_str::<Value>(&body_text).unwrap_err();
        panic!(
            "\nRESPONSE JSON PARSE FAILED\n\
             Status:   {status}\n\
             Request:  {method} {url}\n\
             Headers:  {headers:?}\n\
             Error:    {e}\n\
             Body(raw):\n{body_text}\n"
        )
    })
}

/// Helper struct for building mock database with query results
pub struct MockDbBuilder {
    mock: MockDatabase,
}

impl MockDbBuilder {
    /// Create a new mock database builder
    pub fn new() -> Self {
        Self {
            mock: MockDatabase::new(DatabaseBackend::Postgres),
        }
    }

    /// Append single-entity query results (existing method)
    pub fn with_query_results<M>(mut self, results: Vec<Vec<M>>) -> Self
    where
        M: sea_orm::IntoMockRow,
    {
        self.mock = self.mock.append_query_results(results);
        self
    }

    /// Append tuple query results (for find_also_related)
    pub fn with_tuple_query_results<M, R>(mut self, results: Vec<Vec<(M, Option<R>)>>) -> Self
    where
        M: sea_orm::IntoMockRow,
        R: sea_orm::IntoMockRow,
        (M, Option<R>): sea_orm::IntoMockRow,
    {
        self.mock = self.mock.append_query_results(results);
        self
    }

    /// Append execution results to the mock database
    pub fn with_exec_results(mut self, results: Vec<sea_orm::MockExecResult>) -> Self {
        self.mock = self.mock.append_exec_results(results);
        self
    }

    /// Build the database connection
    pub fn build(self) -> DatabaseConnection {
        self.mock.into_connection()
    }
}

impl Default for MockDbBuilder {
    fn default() -> Self {
        Self::new()
    }
}
