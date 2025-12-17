#![allow(clippy::result_large_err)]

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::Response;
use dotenvy::dotenv;
use http_body_util::BodyExt;
use reqwest::Method;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use sea_orm::{Database, DatabaseConnection};
use std::fs;
use std::{env, sync::Arc};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use user_agent_parser::UserAgentParser;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::DEBUG;
use crate::responses::ApiResponse;
pub use inker_http::OAuthProviderInfo;
use inker_http::{AppState, Providers};

mod config;
mod errors;
mod openapi;
mod printer;
mod responses;
mod routes;

/// Debug middleware that logs request method/uri and response body (only in DEBUG mode)
async fn debug_logger(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    if *DEBUG {
        let (parts, body) = response.into_parts();
        let bytes = body
            .collect()
            .await
            .map(|b| b.to_bytes())
            .unwrap_or_default();

        // Try to parse as JSON for pretty printing
        let body_str = String::from_utf8_lossy(&bytes);
        let formatted_body = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str)
        {
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| body_str.to_string())
        } else {
            // Truncate non-JSON responses (like HTML) to avoid spam
            if body_str.len() > 200 {
                format!("{}... [truncated]", &body_str[..200])
            } else {
                body_str.to_string()
            }
        };

        tracing::info!(target: "http::debug",
            "\n[{}] {} -> {} {}",
            method.to_string(),
            uri.to_string(),
            parts.status.to_string(),
            formatted_body.to_string()
        );

        Response::from_parts(parts, Body::from(bytes))
    } else {
        response
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "http=info,http::debug=info".into()),
        )
        .init();
    let yaml_content = fs::read_to_string("resources/user_agent_regexes.yaml")
        .expect("Failed to read user agent regexes file");
    let ua_parser = Arc::new(UserAgentParser::from_str(yaml_content).unwrap());
    let db: DatabaseConnection = Database::connect(database_url)
        .await
        .expect("Couldn't connect to the database");

    // Create the application state
    let app_state = AppState::new(
        db,
        ua_parser,
        Providers::default(),
        reqwest::Client::new(),
        ml_processing::GeminiClient::new(
            env::var("GOOGLE_GEMINI_API_KEY").expect("GOOGLE_GEMINI_API_KEY must be set"),
        )
        .expect("Failed to create Gemini client"),
        yt_processing::YouTubeCaptions::default(),
    );
    let default_route = Router::new();

    let http_router = crate::routes::get_route(app_state.clone());

    // Swagger UI configuration
    let app = {
        let base = default_route.merge(http_router).fallback(|| async {
            ApiResponse::<()>::error_response(
                StatusCode::NOT_FOUND,
                "The requested resource was not found".to_string(),
            )
        });

        if *DEBUG {
            base.merge(
                SwaggerUi::new("/swagger-ui")
                    .url("/api-doc/openapi.json", openapi::ApiDoc::openapi()),
            )
        } else {
            base
        }
    };
    let cors = CorsLayer::new()
        .allow_origin([config::FRONTEND_URL
            .parse::<HeaderValue>()
            .expect("Invalid FRONTEND_URL")])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .allow_credentials(true);

    let app_layer = app.layer(middleware::from_fn(debug_logger)).layer(cors);

    printer::print_routes(&app_layer);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    // Use into_make_service_with_connect_info to enable IP-based rate limiting
    axum::serve(
        listener,
        app_layer.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .unwrap();
}

// For audio support later
// For later use
// use anyhow::Result;
// use kalosm_streams::text_stream::TextStream;
// use rodio::Decoder;
// use rwhisper::WhisperBuilder;
// use std::fs::File;
// use std::io::BufReader as StdBufReader;
//
// #[tokio::main]
// async fn main() -> Result<()> {
//     // Load the Whisper model (Tiny English for speed)
//     let model = WhisperBuilder::default()
//         .with_source(rwhisper::WhisperSource::TinyEn)
//         .build()
//         .await?;
//
//     // Open an audio file
//     let file = File::open("/Users/bishoy/Documents/rust/inker/crates/http/audio_example.wav")?;
//     let buf = StdBufReader::new(file);
//     let decoder = Decoder::new(buf)?; // rodio::Decoder
//
//     // Convert decoder to a source compatible with transcribe
//     let mut transcription = model.transcribe(decoder);
//
//     // Print the transcription as it comes
//     transcription.to_std_out().await?;
//
//     Ok(())
// }
