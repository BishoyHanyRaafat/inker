use std::sync::Arc;

use crate::{
    AppState,
    errors::{AppError, AppErrorTrait, AuthError},
};
use axum::{
    RequestPartsExt,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    extract::CookieJar,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{Validation, decode};
use user_agent_parser::UserAgentParser;

use super::auth::models::{Claims, KEYS};

pub struct DeviceExtractor {
    pub os: Option<String>,
    pub device_type: Option<String>,
    pub engine: Option<String>,
    pub user_agent: String,
}

impl DeviceExtractor {
    #[allow(dead_code)]
    pub fn new(device_type: String, os: String, engine: String, user_agent: String) -> Self {
        DeviceExtractor {
            device_type: Some(device_type),
            os: Some(os),
            engine: Some(engine),
            user_agent,
        }
    }

    /// Creates a DeviceExtractor with only the raw user agent string.
    /// Used as a fallback when UA parsing fails.
    #[allow(dead_code)]
    pub fn unknown(user_agent: String) -> Self {
        DeviceExtractor {
            device_type: None,
            os: None,
            engine: None,
            user_agent,
        }
    }

    /// Attempts to parse the user agent string into device info.
    /// Returns a fully populated DeviceExtractor if all fields can be parsed,
    /// otherwise returns a DeviceExtractor with only the fields that could be parsed.
    pub fn from_ua_parser(user_agent_parser: &Arc<UserAgentParser>, user_agent_str: &str) -> Self {
        let device_type = user_agent_parser
            .parse_device(user_agent_str)
            .name
            .map(|s| s.to_string());
        let os = user_agent_parser
            .parse_os(user_agent_str)
            .name
            .map(|s| s.to_string());
        let engine = user_agent_parser
            .parse_engine(user_agent_str)
            .name
            .map(|s| s.to_string());

        DeviceExtractor {
            device_type,
            os,
            engine,
            user_agent: user_agent_str.to_string(),
        }
    }
}

impl<S> FromRequestParts<S> for DeviceExtractor
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let state = AppState::from_ref(state);
        let user_agent = parts
            .headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("Unknown")
            .to_string();

        async move {
            // Always succeed - gracefully handle unknown user agents
            Ok(DeviceExtractor::from_ua_parser(
                &state.ua_parser,
                &user_agent,
            ))
        }
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|e| AuthError::InvalidToken.get_response().with_debug(e))?;
        decode_claims_from_token(bearer.token())
    }
}

fn decode_claims_from_token(token: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(token, &KEYS.decoding, &Validation::default())
        .map_err(|e| AuthError::InvalidToken.get_response().with_debug(e))?;
    Ok(token_data.claims)
}

/// WebSocket-friendly claims extractor.
///
/// Browsers cannot attach custom `Authorization` headers when creating a `WebSocket`,
/// so we accept the access token via:
/// - `Authorization: Bearer <token>` (non-browser clients)
/// - HttpOnly cookie `access_token` (browser clients)
///
/// NOTE: This extractor is only meant for WebSocket endpoints.
pub struct WsClaims(pub Claims);

impl<S> FromRequestParts<S> for WsClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header_token = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .ok()
            .map(|TypedHeader(Authorization(bearer))| bearer.token().to_string());

        let cookie_token = parts
            .extract::<CookieJar>()
            .await
            .ok()
            .and_then(|jar| jar.get("access_token").map(|c| c.value().to_string()));

        let token = header_token
            .or(cookie_token)
            .ok_or_else(|| AuthError::InvalidToken.get_response())?;

        Ok(WsClaims(decode_claims_from_token(&token)?))
    }
}
