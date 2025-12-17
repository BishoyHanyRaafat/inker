use super::AppErrorTrait;
use axum::http::StatusCode;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotesError {
    #[error("Note is malformed")]
    NoteMalFormed,

    #[error("Title is required")]
    TitleRequired,

    #[error("Title too long (max {0} characters)")]
    TitleTooLong(usize),

    #[error("Block content too large (max {0} bytes)")]
    BlockContentTooLarge(usize),
}

impl AppErrorTrait for NotesError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NoteMalFormed => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TitleRequired | Self::TitleTooLong(_) | Self::BlockContentTooLarge(_) => {
                StatusCode::BAD_REQUEST
            }
        }
    }

    fn error_message(&self) -> Cow<'static, str> {
        Cow::Owned(self.to_string())
    }
}
