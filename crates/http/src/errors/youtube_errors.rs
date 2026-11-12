use reqwest::StatusCode;
use std::borrow::Cow;
use thiserror::Error;

use crate::errors::AppErrorTrait;

#[derive(Error, Debug)]
pub enum YoutubeError {
    #[error("The requested video was not found.")]
    VideoNotFound,

    #[error("Decode error: {0}")]
    JsonDecodeError(#[from] serde_json::Error),
}

impl AppErrorTrait for YoutubeError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::VideoNotFound => StatusCode::BAD_REQUEST,
            Self::JsonDecodeError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_message(&self) -> Cow<'static, str> {
        Cow::Owned(self.to_string())
    }
}
