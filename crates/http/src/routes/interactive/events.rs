use std::time::Instant;

use crate::errors::YoutubeError;
use agentic_pipeline::pipeline::SectionQuestions;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum YouTubeStreamEventUpdate {
    Start { id: String },
    Seeked { position: f64 },
    Paused,
    Resumed,
    PositionUpdated { position: f64 },
}

pub enum YouTubeStreamState {
    Running { position: f64, instant: Instant },
    Paused { position: f64 },
    NotRunning,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum YouTubeStreamResponse {
    #[schema(example = "some error", value_type=String)]
    Error(YoutubeError),
    Question(SectionQuestions),
    Ready,
}

impl From<YoutubeError> for YouTubeStreamResponse {
    fn from(error: YoutubeError) -> Self {
        YouTubeStreamResponse::Error(error)
    }
}

impl From<serde_json::Error> for YouTubeStreamResponse {
    fn from(error: serde_json::Error) -> Self {
        YouTubeStreamResponse::Error(YoutubeError::from(error))
    }
}

impl From<SectionQuestions> for YouTubeStreamResponse {
    fn from(section: SectionQuestions) -> Self {
        YouTubeStreamResponse::Question(section)
    }
}
