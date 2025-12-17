use std::time::Instant;

use crate::errors::YoutubeError;
use ml_processing::ProcessedChunk;
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
}

pub enum YouTubeStreamState {
    Running { position: f64, instant: Instant },
    Paused { position: f64 },
    NotRunning,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum YouTubeStreamResponse {
    Error(YoutubeError),
    Chunk(ProcessedChunk),
}
