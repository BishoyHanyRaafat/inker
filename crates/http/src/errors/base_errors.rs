use super::AppErrorTrait;
use axum::http::StatusCode;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BaseError {
    #[error("Database error")]
    DatabaseError,

    #[error("Internal Server Error")]
    InternalServer,

    #[error("Missing Parameters")]
    MissingParams,

    #[error("Resource Not Found")]
    NotFound,
}

impl AppErrorTrait for BaseError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::DatabaseError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InternalServer => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingParams => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_message(&self) -> Cow<'static, str> {
        Cow::Owned(self.to_string())
    }
}
