#![allow(dead_code)]
use serde::{Serialize, Serializer};
use std::borrow::Cow;
use thiserror::Error;

use ml_processing::Error as CommonError;

#[derive(Error, Debug)]
pub enum YoutubeError {
    #[error("The requested video was not found.")]
    VideoNotFound,

    #[error("Gemini client error: {0}")]
    GeminiError(#[from] gemini_rust::ClientError),

    #[error("Decode error: {0}")]
    ToonDecodeError(#[from] toon_format::ToonError),

    #[error("Decode error: {0}")]
    JsonDecodeError(#[from] serde_json::Error),
}

// Implement AppErrorTrait
impl YoutubeError {
    fn error_message(&self) -> Cow<'static, str> {
        match self {
            YoutubeError::VideoNotFound => Cow::from("The requested video was not found."),
            YoutubeError::GeminiError(e) => Cow::from(e.to_string()),
            YoutubeError::JsonDecodeError(e) => {
                Cow::from(format!("Failed to decode response: {}", e))
            }
            YoutubeError::ToonDecodeError(e) => {
                Cow::from(format!("Failed to decode format: {}", e))
            }
        }
    }
}

impl Serialize for YoutubeError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert Cow<'static, str> to &str and serialize
        serializer.serialize_str(self.error_message().as_ref())
    }
}

impl From<CommonError> for YoutubeError {
    fn from(err: CommonError) -> Self {
        match err {
            CommonError::GeminiError(e) => YoutubeError::GeminiError(e),
            CommonError::DecodeError(e) => YoutubeError::ToonDecodeError(e),
            CommonError::JsonDecodeError(e) => YoutubeError::JsonDecodeError(e),
        }
    }
}
