use crate::pipeline::{ExtractedConceptMetaData, MAX_SECTION_TOKENS, MAX_SECTIONS_PER_BATCH};
use std::sync::Arc;

use rig::{prelude::*, tool::ToolContext};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;
use tokio::sync::Mutex;

use yt_processing::Caption;

#[derive(Debug, Clone)]
struct PreparedSegment {
    segment: Caption,
    token_count: usize,
}

#[derive(Debug, Default)]
struct SectionState {
    cursor: usize,
    calls_in_batch: usize,
    tool_calls_count: usize,
    batch_metadata: ExtractedConceptMetaData,
}

#[derive(Debug, Error)]
pub enum SectionToolError {
    #[error("No more transcript sections are available")]
    TranscriptExhausted,

    #[error("Batch limit reached; generate the outline before continuing")]
    BatchLimitReached,

    #[error("This segment is too big")]
    OversizedSegment,
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

#[derive(Clone, Debug)]
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
        let mut state = self.state.lock().await;
        state.calls_in_batch = 0;
        state.batch_metadata.start_time = 0.0;
        state.batch_metadata.end_time = 0.0;
        state.batch_metadata.transcript_content = String::new();
    }

    pub async fn get_batch_meta(&self) -> ExtractedConceptMetaData {
        self.state.lock().await.batch_metadata.clone()
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
             Each section contains at most {MAX_SECTION_TOKENS} estimated tokens. \
             Retrieve sections until batch_complete is true. \
             When batch_complete is true, stop calling this tool and update \
             your outline before requesting more sections. \
             If has_more is false, the transcript is exhausted."
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

            if item.token_count > MAX_SECTION_TOKENS {
                return Err(SectionToolError::OversizedSegment);
            }

            if !content.is_empty() && token_count + item.token_count > MAX_SECTION_TOKENS {
                break;
            }

            if content.is_empty() {
                start_seconds = item.segment.start;
            } else {
                content.push(' ');
            }

            content.push_str(&item.segment.text);

            token_count += item.token_count;
            end_seconds = item.segment.end;

            state.cursor += 1;
        }

        state.tool_calls_count += 1;
        state.calls_in_batch += 1;

        let has_more = state.cursor < self.segments.len();

        let batch_complete = state.calls_in_batch >= MAX_SECTIONS_PER_BATCH;

        tracing::info!("Sending from {} to {}", start_seconds, end_seconds,);

        state.batch_metadata.transcript_content.push_str(&content);
        state.batch_metadata.end_time = end_seconds;
        if state.batch_metadata.start_time == 0.0 {
            state.batch_metadata.start_time = start_seconds;
        }
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
