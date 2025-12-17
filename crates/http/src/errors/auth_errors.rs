use super::AppErrorTrait;
use axum::http::StatusCode;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Wrong credentials")]
    WrongCredentials,

    #[error("Missing credentials")]
    MissingCredentials,

    #[error("Token creation error")]
    TokenCreation,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Username already exists")]
    UsernameAlreadyExists,

    #[error("Email already in use")]
    EmailAlreadyExists,

    #[error("Token expired")]
    ExpirationError,

    #[error("Couldn't validate the submitted form")]
    ValidationError,

    #[error("{0}")]
    InvalidProvider(String),
}

impl AppErrorTrait for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::WrongCredentials => StatusCode::UNAUTHORIZED,
            Self::MissingCredentials => StatusCode::BAD_REQUEST,
            Self::TokenCreation => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InvalidToken => StatusCode::BAD_REQUEST,
            Self::UsernameAlreadyExists => StatusCode::CONFLICT,
            Self::EmailAlreadyExists => StatusCode::CONFLICT,
            Self::ExpirationError => StatusCode::BAD_REQUEST,
            Self::ValidationError => StatusCode::BAD_REQUEST,
            Self::InvalidProvider(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_message(&self) -> Cow<'static, str> {
        match self {
            Self::InvalidProvider(s) => Cow::Owned(s.clone()),
            other => Cow::Owned(other.to_string()),
        }
    }
}
