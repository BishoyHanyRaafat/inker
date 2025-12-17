#![allow(clippy::result_large_err)]

use entities::sea_orm_active_enums::OauthProvider;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use user_agent_parser::UserAgentParser;

pub mod config;
pub mod errors;
pub mod responses;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub ua_parser: Arc<UserAgentParser>,
    pub providers: Providers,
    pub http_client: reqwest::Client,
    pub gemini_client: ml_processing::GeminiClient,
    pub youtube_captions: yt_processing::YouTubeCaptions,
}

impl AppState {
    pub fn new(
        db: DatabaseConnection,
        ua_parser: Arc<UserAgentParser>,
        providers: Providers,
        http_client: reqwest::Client,
        gemini_client: ml_processing::GeminiClient,
        youtube_captions: yt_processing::YouTubeCaptions,
    ) -> Self {
        Self {
            db,
            ua_parser,
            providers,
            http_client,
            gemini_client,
            youtube_captions,
        }
    }
}

#[derive(Clone)]
pub struct Providers {
    pub github: OAuthProviderInfo,
    pub google: OAuthProviderInfo,
}

#[derive(Clone, Debug)]
pub struct OAuthProviderInfo {
    pub auth_url: String,
    pub token_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub scopes: Vec<String>,
}

impl Default for Providers {
    fn default() -> Self {
        let github = OAuthProviderInfo {
            auth_url: "https://github.com/login/oauth/authorize".into(),
            token_url: "https://github.com/login/oauth/access_token".into(),
            client_id: std::env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set"),
            client_secret: std::env::var("GITHUB_CLIENT_SECRET")
                .expect("GITHUB_CLIENT_SECRET must be set"),
            scopes: vec!["read:user".into(), "user:email".into()],
        };

        let google = OAuthProviderInfo {
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token_url: "https://oauth2.googleapis.com/token".into(),
            client_id: std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
            client_secret: std::env::var("GOOGLE_CLIENT_SECRET")
                .expect("GOOGLE_CLIENT_SECRET must be set"),
            scopes: vec!["profile".into(), "email".into()],
        };
        Self { google, github }
    }
}

impl Providers {
    pub fn get_oauth_provider(&self, provider: &OauthProvider) -> &OAuthProviderInfo {
        match provider {
            OauthProvider::Google => &self.google,
            OauthProvider::Github => &self.github,
            OauthProvider::Facebook => unimplemented!(),
            OauthProvider::Twitter => unimplemented!(),
        }
    }
}

/// Create the application router with all routes configured.
///
/// This is primarily intended for tests and other internal callers; the main
/// binary can still compose middleware/layers as needed.
pub fn create_app(app_state: AppState) -> axum::Router {
    routes::get_route(app_state)
}
