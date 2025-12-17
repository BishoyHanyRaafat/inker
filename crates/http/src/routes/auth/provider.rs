use super::models::{Auth, GitHubEmail, GitHubUser, GoogleUser, OauthProfile};
use super::security::{extract_redirect_and_csrf, validate_csrf};
use super::verify_user::find_or_create_user;
use crate::config::{BACKEND_URL, REDIRECT_DEFAULT_AFTER_PROVIDER};
use crate::errors::{AppError, AppErrorTrait, AuthError, BaseError};
use crate::responses::ApiResponse;
use crate::routes::auth::models::{ProviderPath, ProviderQuery};
use crate::routes::auth::verify_user::DatabaseInsertable;
use crate::routes::extractors::DeviceExtractor;
use crate::{AppState, OAuthProviderInfo};
use axum::debug_handler;
use axum::extract::{Path, Query, State};
use axum::response::Redirect;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use entities::sea_orm_active_enums::OauthProvider;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, TokenUrl, basic::BasicClient,
};
use oauth2::{AuthorizationCode, EndpointNotSet, EndpointSet, TokenResponse};
use reqwest::StatusCode;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use std::collections::HashMap;

fn try_from_str_provider(value: &str) -> Result<OauthProvider, String> {
    match value {
        "google" => Ok(OauthProvider::Google),
        "facebook" => Ok(OauthProvider::Facebook),
        "twitter" => Ok(OauthProvider::Twitter),
        "github" => Ok(OauthProvider::Github),
        other => Err(format!("Unknown OAuth provider: {}", other)),
    }
}
async fn client_factory(
    oauth_provider: &OAuthProviderInfo,
    redirect_path: &str,
) -> Result<
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    AppError,
> {
    Ok(
        BasicClient::new(ClientId::new(oauth_provider.client_id.to_string()))
            .set_client_secret(ClientSecret::new(oauth_provider.client_secret.to_string()))
            .set_auth_uri(
                AuthUrl::new(oauth_provider.auth_url.to_string())
                    .map_err(|e| BaseError::InternalServer.get_response().with_debug(e))?,
            )
            .set_token_uri(
                TokenUrl::new(oauth_provider.token_url.to_string())
                    .map_err(|e| BaseError::InternalServer.get_response().with_debug(e))?,
            )
            .set_redirect_uri(
                RedirectUrl::new(redirect_path.to_string())
                    .map_err(|e| BaseError::InternalServer.get_response().with_debug(e))?,
            ),
    )
}

/// OAuth provider redirect endpoint
///
/// Initiates OAuth flow by redirecting to the provider's authorization page.
/// Supported providers: google, github, facebook, twitter
#[utoipa::path(
    get,
    path = "/api/v1/auth/oauth/{provider}",
    tag = "auth",
    params(
        ("provider" = OauthProvider, Path, description = "OAuth provider name", example = "google"),
        ("redirect" = Option<String>, Query, description = "Optional redirect URL after OAuth completion", example = "https://example.com/dashboard")
    ),
    responses(
        (status = 302, description = "Redirects to OAuth provider authorization page"),
        (status = 400, description = "Invalid provider", body = ApiResponse<String>),
    )
)]
#[debug_handler]
pub async fn provider_handler(
    State(state): State<AppState>,
    Path(path): Path<ProviderPath>,
    Query(query): Query<ProviderQuery>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), AppError> {
    let user_provider = &path.provider;
    let redirect_after_provider = query
        .redirect
        .unwrap_or_else(|| REDIRECT_DEFAULT_AFTER_PROVIDER.clone());
    let provider_str = match user_provider {
        OauthProvider::Google => "google",
        OauthProvider::Github => "github",
        OauthProvider::Facebook => "facebook",
        OauthProvider::Twitter => "twitter",
    };
    // OAuth callback goes to backend, not frontend
    let redirect_path = format!(
        "{}/api/v1/auth/oauth/{}/callback",
        BACKEND_URL.as_str(),
        provider_str
    );

    let oauth_provider = state.providers.get_oauth_provider(user_provider);
    let csrf = CsrfToken::new_random().secret().clone();
    let client = client_factory(oauth_provider, redirect_path.as_str()).await?;

    let state = format!("csrf={csrf}&redirect={redirect_after_provider}");
    let mut auth_request = client.authorize_url(|| CsrfToken::new(state.clone()));

    for scope in &oauth_provider.scopes {
        auth_request = auth_request.add_scope(oauth2::Scope::new(scope.clone()));
    }

    let (auth_url, _) = auth_request.url();
    let csrf_cookie = Cookie::build(("oauth_csrf", csrf))
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Lax);

    let crsf_jar = jar.add(csrf_cookie);
    Ok((crsf_jar, Redirect::to(auth_url.as_str())))
}

#[debug_handler]
pub async fn oauth_callback(
    Query(params): Query<HashMap<String, String>>,
    Path(path_params): Path<HashMap<String, String>>,
    user_device_info: DeviceExtractor,
    jar: CookieJar,
    State(state): State<AppState>,
) -> Result<axum::response::Response, AppError> {
    use axum::response::IntoResponse;
    let provider_str = path_params.get("provider").ok_or_else(|| {
        BaseError::MissingParams
            .get_response()
            .with_error_detail("provider", "Missing parameter")
    })?;

    let user_provider = try_from_str_provider(provider_str.as_str())
        .map_err(|_| AuthError::InvalidProvider(provider_str.clone()).get_response())?;

    let state_param = params
        .get("state")
        .ok_or_else(|| BaseError::MissingParams.get_response())?;

    let (csrf_from_state, redirect_path) = extract_redirect_and_csrf(state_param);
    let csrf_cookie = jar
        .get("oauth_csrf")
        .map(|c| c.value().to_string())
        .ok_or_else(|| {
            BaseError::InternalServer
                .get_response()
                .with_debug("Cookies not set")
        })?;

    validate_csrf(csrf_from_state, &csrf_cookie)?;

    let redirect_path = redirect_path.unwrap_or_else(|| REDIRECT_DEFAULT_AFTER_PROVIDER.as_str());

    let code = params
        .get("code")
        .ok_or_else(|| BaseError::MissingParams.get_response())?;

    // OAuth callback goes to backend, not frontend
    let redirect_path_client = format!(
        "{}/api/v1/auth/oauth/{}/callback",
        BACKEND_URL.as_str(),
        provider_str
    );

    let oauth_provider = state.providers.get_oauth_provider(&user_provider);
    let client = client_factory(oauth_provider, &redirect_path_client).await?;

    let token_result = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .request_async(&state.http_client)
        .await
        .map_err(|e| BaseError::InternalServer.get_response().with_debug(e))?;

    let access_token = token_result.access_token().secret();
    let scopes = token_result
        .scopes()
        .map_or_else(Vec::new, |v| v.iter().map(|x| x.to_string()).collect());

    let oauth_profile = get_provider_api(access_token.clone(), &state.http_client, &user_provider)
        .await?
        .ok_or_else(|| {
            BaseError::InternalServer
                .get_response()
                .with_debug("Couldn't get profile from provider")
        })?;

    let user =
        find_or_create_user(&state, &oauth_profile, user_provider, scopes, token_result).await?;

    let (token, ref_token) =
        Auth::default(user.user_type, user.username, user.id, user_device_info)
            .map_err(|_| AuthError::TokenCreation.get_response())?;

    ref_token.insert_refresh_token(&state).await?;

    // If there's a redirect path, redirect and set the access token as a cookie
    if !redirect_path.is_empty() {
        let access_token_cookie = Cookie::build(("access_token", token.access_token.token.clone()))
            .path(redirect_path.to_string())
            .secure(true)
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax);

        let jar = jar.add(access_token_cookie);
        return Ok((jar, Redirect::to(redirect_path)).into_response());
    }

    Ok(
        ApiResponse::success_response(StatusCode::OK, "Logged in successfully", token)
            .into_response(),
    )
}

pub async fn get_provider_api(
    access_token: String,
    http_client: &reqwest::Client,
    provider: &OauthProvider,
) -> Result<Option<OauthProfile>, AppError> {
    match provider {
        OauthProvider::Github => {
            let user: GitHubUser = http_client
                .get("https://api.github.com/user")
                .header(USER_AGENT, "basic_rust")
                .header(AUTHORIZATION, format!("Bearer {}", access_token))
                .send()
                .await?
                .json()
                .await?;

            let emails: Vec<GitHubEmail> = http_client
                .get("https://api.github.com/user/emails")
                .header(USER_AGENT, "basic_rust")
                .header(AUTHORIZATION, format!("Bearer {}", access_token))
                .send()
                .await?
                .json()
                .await?;

            let primary_email = emails
                .into_iter()
                .find(|e| e.primary && e.verified)
                .map(|e| e.email);

            match primary_email {
                Some(_email) => Ok(Some(OauthProfile {
                    username: user.login,
                    email: _email,
                    id: user.id.to_string(),
                })),
                None => Ok(None),
            }
        }

        OauthProvider::Google => {
            let user: GoogleUser = http_client
                .get("https://www.googleapis.com/oauth2/v3/userinfo")
                .header(USER_AGENT, "basic_rust")
                .header(AUTHORIZATION, format!("Bearer {}", access_token))
                .send()
                .await?
                .json()
                .await?;

            if user.email_verified {
                Ok(Some(OauthProfile {
                    username: user.name,
                    email: user.email,
                    id: user.sub,
                }))
            } else {
                Ok(None)
            }
        }
        _ => Err(BaseError::InternalServer
            .get_response()
            .with_debug("Provider not implemented")),
    }
}
