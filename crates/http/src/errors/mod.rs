mod auth_errors;
mod base_errors;
mod notes_errors;
mod youtube_errors;

use crate::responses::ApiResponse;
use axum::{http::StatusCode, response::IntoResponse};
use sea_orm::DbErr;
use std::borrow::Cow;

pub use auth_errors::AuthError;
pub use base_errors::BaseError;
pub use notes_errors::NotesError;
pub use youtube_errors::YoutubeError;

pub trait AppErrorTrait {
    fn status_code(&self) -> StatusCode;
    fn error_message(&self) -> Cow<'static, str>;

    fn get_response(&self) -> ApiResponse<()> {
        ApiResponse::<()>::error_response(self.status_code(), self.error_message().to_string())
    }
}

macro_rules! impl_into_response {
    ($($t:ty),*) => {
        $(
            impl IntoResponse for $t {
                fn into_response(self) -> axum::response::Response {
                    self.get_response().into_response()
                }
            }
        )*
    };
}

impl_into_response!(BaseError, AuthError, NotesError);

pub type AppError = ApiResponse<()>;

impl From<DbErr> for ApiResponse<()> {
    fn from(e: DbErr) -> Self {
        BaseError::DatabaseError.get_response().with_debug(e)
    }
}

impl From<reqwest::Error> for ApiResponse<()> {
    fn from(e: reqwest::Error) -> Self {
        BaseError::InternalServer.get_response().with_debug(e)
    }
}
