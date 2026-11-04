use std::sync::Arc;

use rig::{prelude::*, tool::ToolContext};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;
use tokio::sync::Mutex;

use yt_processing::Caption;

pub const MAX_SECTION_TOKENS: usize = 2_700;
pub const MAX_SECTIONS_PER_BATCH: usize = 5;

#[derive(Debug, Clone)]
struct PreparedSegment {
    segment: Caption,
    token_count: usize,
}

#[derive(Debug, Default)]
struct SectionState {
    cursor: usize,
    calls_in_batch: usize,
}

#[derive(Debug, Error)]
pub enum SectionToolError {
    #[error("No more transcript sections are available")]
    TranscriptExhausted,

    #[error("Batch limit reached; generate the outline before continuing")]
    BatchLimitReached,
}

#[derive(Debug, Deserialize)]
pub struct GetSectionArgs {}

#[derive(Debug, Serialize)]
pub struct SectionResult {
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub content: String,
    pub estimated_tokens: usize,
    pub batch_complete: bool,
    pub has_more: bool,
}

#[derive(Clone)]
pub struct GetTranscriptSection {
    segments: Arc<Vec<PreparedSegment>>,
    state: Arc<Mutex<SectionState>>,
}

impl GetTranscriptSection {
    pub fn new(segments: Vec<(Caption, usize)>) -> Self {
        let segments = segments
            .into_iter()
            .map(|(segment, token_count)| PreparedSegment {
                segment,
                token_count,
            })
            .collect();

        Self {
            segments: Arc::new(segments),
            state: Arc::new(Mutex::new(SectionState::default())),
        }
    }

    pub async fn reset_batch(&self) {
        self.state.lock().await.calls_in_batch = 0;
    }

    pub async fn has_more(&self) -> bool {
        let state = self.state.lock().await;
        state.cursor < self.segments.len()
    }
}

impl Tool for GetTranscriptSection {
    const NAME: &'static str = "get_next_transcript_section";

    type Args = GetSectionArgs;
    type Output = SectionResult;
    type Error = SectionToolError;

    fn description(&self) -> String {
        format!(
            "Retrieve the next sequential section of the YouTube transcript. \
             Each section contains approximately {MAX_SECTION_TOKENS} tokens. \
             Retrieve sections until batch_complete is true. \
             When batch_complete is true, stop calling this tool and update \
             your outline before requesting more sections."
        )
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "required": []
        })
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        _args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        let mut state = self.state.lock().await;

        if state.cursor >= self.segments.len() {
            return Err(SectionToolError::TranscriptExhausted);
        }

        if state.calls_in_batch >= MAX_SECTIONS_PER_BATCH {
            return Err(SectionToolError::BatchLimitReached);
        }

        let mut content = String::new();
        let mut token_count = 0;

        let mut start_seconds = 0.0;
        let mut end_seconds = 0.0;

        while state.cursor < self.segments.len() {
            let item = &self.segments[state.cursor];

            if !content.is_empty() && token_count + item.token_count > MAX_SECTION_TOKENS {
                break;
            }

            if content.is_empty() {
                start_seconds = item.segment.start;
            }

            content.push_str(&item.segment.text);
            content.push('\n');

            token_count += item.token_count;
            end_seconds = item.segment.end;

            state.cursor += 1;
        }

        state.calls_in_batch += 1;

        let batch_complete = state.calls_in_batch >= MAX_SECTIONS_PER_BATCH;

        let has_more = state.cursor < self.segments.len();

        Ok(SectionResult {
            start_seconds,
            end_seconds,
            content,
            estimated_tokens: token_count,
            batch_complete,
            has_more,
        })
    }
}
