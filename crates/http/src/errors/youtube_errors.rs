#![allow(dead_code)]
use serde::{Serialize, Serializer};
use std::borrow::Cow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum YoutubeError {
    #[error("The requested video was not found.")]
    VideoNotFound,

    #[error("Decode error: {0}")]
    JsonDecodeError(#[from] serde_json::Error),

    #[error("{0}")]
    Message(String, bool),
}

// Implement AppErrorTrait
impl YoutubeError {
    fn error_message(&self) -> Cow<'static, str> {
        match self {
            YoutubeError::VideoNotFound => Cow::from("The requested video was not found."),
            YoutubeError::JsonDecodeError(e) => {
                Cow::from(format!("Failed to decode response: {}", e))
            }
            YoutubeError::Message(e, b) => Cow::from(format!("is_fatal={}, message={}", b, e)),
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
