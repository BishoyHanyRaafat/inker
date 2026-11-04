use super::AppErrorTrait;
use axum::http::StatusCode;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotesError {
    #[error("Title is required")]
    TitleRequired,

    #[error("Title too long (max {0} characters)")]
    TitleTooLong(usize),

    #[error("Block content too large (max {0} bytes)")]
    NoteContentTooLarge(usize),

    #[error("Invalid version")]
    InvalidVersion(i32),
}

impl AppErrorTrait for NotesError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::TitleRequired
            | Self::TitleTooLong(_)
            | Self::NoteContentTooLarge(_)
            | Self::InvalidVersion(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_message(&self) -> Cow<'static, str> {
        Cow::Owned(self.to_string())
    }
}
