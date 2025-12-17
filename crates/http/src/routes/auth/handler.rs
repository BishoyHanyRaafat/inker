use super::models::{Auth, LoginUser, SeededUser};
use super::security::validate_user_payload;
use super::verify_user::{DatabaseInsertable, PasswordAuthenticatable, TokenGeneratable};
use crate::AppState;
use crate::errors::{AppError, AppErrorTrait, AuthError};
use crate::responses::ApiResponse;
use crate::routes::extractors::DeviceExtractor;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use chrono::Utc;
use entities::{password, prelude::*};
use entities::{refresh_tokens, user};
use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, ExprTrait, QueryFilter, Set,
    TransactionTrait,
};
use uuid::Uuid;
/// Login endpoint
///
/// Authenticates a user with email and password, returning access and refresh tokens.
#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    tag = "auth",
    request_body = LoginUser,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<Auth>),
        (status = 400, description = "Invalid credentials or missing fields", body = ApiResponse<String>),
        (status = 401, description = "Authentication failed", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn login(
    State(state): State<AppState>,
    user_device_info: DeviceExtractor,
    Json(payload): Json<LoginUser>,
) -> Result<ApiResponse<Auth>, AppError> {
    if payload.password.is_empty() || payload.email.is_empty() {
        return Err(AuthError::MissingCredentials.get_response());
    }

    let user_email = payload.email.clone();
    let (user, pass): (user::Model, Option<password::Model>) = User::find()
        .filter(user::Column::Email.eq(user_email))
        .find_also_related(password::Entity)
        .one(&state.db)
        .await?
        .ok_or_else(|| AuthError::WrongCredentials.get_response())?;

    pass.ok_or_else(|| AuthError::WrongCredentials.get_response())?
        .verify(&payload.password)?;

    let (access_token, ref_token) = user.generate_tokens(user_device_info)?;
    ref_token.insert_refresh_token(&state).await?;

    Ok(ApiResponse::success_response(
        StatusCode::OK,
        "Logged in successfully",
        access_token,
    ))
}

/// Signup endpoint
///
/// Creates a new user account with username, email, and password.
#[utoipa::path(
    post,
    path = "/api/v1/auth/signup",
    tag = "auth",
    request_body = SeededUser,
    responses(
        (status = 200, description = "User created successfully", body = ApiResponse<Auth>),
        (status = 400, description = "Invalid input or user already exists", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn signup(
    State(state): State<AppState>,
    user_device_info: DeviceExtractor,
    Json(payload): Json<SeededUser>,
) -> Result<ApiResponse<Auth>, AppError> {
    let email_to_check = payload.email.clone();
    let username_to_check = payload.username.clone();

    let existing_user = User::find()
        .filter(
            Expr::col(user::Column::Email)
                .eq(email_to_check.clone())
                .or(Expr::col(user::Column::Username).eq(username_to_check.clone())),
        )
        .one(&state.db)
        .await?;

    if let Some(usr) = existing_user {
        return if usr.email == payload.email {
            Err(AuthError::EmailAlreadyExists.get_response())
        } else {
            Err(AuthError::UsernameAlreadyExists.get_response())
        };
    }

    validate_user_payload(&payload.email, &payload.username, &payload.password)?;

    let hashed = password::Model::hash(&payload.password)?;
    let user_am = user::ActiveModel {
        username: Set(payload.username),
        email: Set(payload.email),
        ..Default::default()
    };

    let txn: DatabaseTransaction = state.db.begin().await?;

    // Insert user
    let inserted_user = user_am.insert(&txn).await?;

    // Insert password
    let pass_am = password::ActiveModel {
        user_id: Set(inserted_user.id),
        content: Set(hashed),
        ..Default::default()
    };
    pass_am.insert(&txn).await?;

    // Commit transaction
    txn.commit().await?;

    let (access_token, ref_token) = inserted_user.generate_tokens(user_device_info)?;
    ref_token.insert_refresh_token(&state).await?;

    Ok(ApiResponse::success_response(
        StatusCode::OK,
        "User created successfully",
        access_token,
    ))
}

/// Refresh token endpoint
///
/// Refreshes an access token using a valid refresh token.
/// The old refresh token is invalidated (deleted) and a new one is issued.
#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh-token",
    tag = "auth",
    request_body(content = String, description = "Refresh token UUID", example = "550e8400-e29b-41d4-a716-446655440000"),
    responses(
        (status = 200, description = "Token refreshed successfully", body = ApiResponse<Auth>),
        (status = 400, description = "Invalid or expired refresh token", body = ApiResponse<String>),
        (status = 401, description = "Authentication failed", body = ApiResponse<String>),
    )
)]
pub async fn refreshtoken(
    State(state): State<AppState>,
    user_device_info: DeviceExtractor,
    Json(refresh_token_str): Json<String>,
) -> Result<ApiResponse<Auth>, AppError> {
    let old_token_uuid = Uuid::parse_str(&refresh_token_str)
        .map_err(|e| AuthError::InvalidToken.get_response().with_debug(e))?;

    let old_ref_token: refresh_tokens::Model = RefreshTokens::find()
        .filter(refresh_tokens::Column::Id.eq(old_token_uuid))
        .one(&state.db)
        .await?
        .ok_or_else(|| AuthError::InvalidToken.get_response())?;

    if old_ref_token.expires_at < Utc::now().naive_utc() {
        // Delete expired token for cleanup
        RefreshTokens::delete_by_id(old_token_uuid)
            .exec(&state.db)
            .await?;
        return Err(AuthError::ExpirationError.get_response());
    }

    if old_ref_token.device_type != user_device_info.device_type {
        return Err(AuthError::WrongCredentials
            .get_response()
            .with_debug("Device type is not the same in the refresh token".to_string()));
    }

    let existing_user: user::Model = User::find_by_id(old_ref_token.user_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AuthError::WrongCredentials.get_response())?;

    let (new_auth_token, new_ref_token) = Auth::default(
        existing_user.user_type,
        existing_user.username,
        existing_user.id,
        user_device_info,
    )
    .map_err(|_| AuthError::TokenCreation.get_response())?;

    // Use a transaction to ensure atomicity: delete old token + insert new token
    let txn: DatabaseTransaction = state.db.begin().await?;

    // Delete the old refresh token (invalidate it)
    RefreshTokens::delete_by_id(old_token_uuid)
        .exec(&txn)
        .await?;

    // Insert the new refresh token
    new_ref_token.insert(&txn).await?;

    // Commit the transaction
    txn.commit().await?;

    Ok(ApiResponse::success_response(
        StatusCode::OK,
        "Refresh token updated",
        new_auth_token,
    ))
}
