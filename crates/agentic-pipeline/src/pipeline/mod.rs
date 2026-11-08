mod agentic;
mod config;
mod extraction;
mod models;
mod questions;

use crate::{error::PipelineError, tools::GetTranscriptSection};
pub use config::{
    MAX_AGENT_RETRIES, MAX_CONCURRENT_QUESTION_REQUESTS, MAX_SECTION_TOKENS,
    MAX_SECTIONS_PER_BATCH, MODEL_NAME,
};
pub use models::{ExtractedConceptMetaData, QuestionedExtractedConcept, SectionQuestions};

use agentic::PipelineAgents;
use yt_processing::Transcript;

pub struct AgenticPipeline {
    agents: PipelineAgents,
    batch_tool: GetTranscriptSection,
    video_title: String,
    video_description: String,
}

impl AgenticPipeline {
    pub async fn new(transcript: Transcript) -> Result<Self, PipelineError> {
        if transcript.is_empty() {
            return Err(PipelineError::EmptyTranscript);
        }
        let video_title = transcript.title;
        let video_description = transcript.description;

        let mut prepared_segments = Vec::with_capacity(transcript.captions.segments.len());

        for segment in transcript.captions.segments {
            let estimated_tokens = segment.text.len().div_ceil(4);

            if estimated_tokens > MAX_SECTION_TOKENS {
                return Err(PipelineError::OversizedSegment);
            }

            prepared_segments.push((segment, estimated_tokens));
        }

        let batch_tool = GetTranscriptSection::new(prepared_segments);

        let agents = PipelineAgents::new(&batch_tool)?;

        Ok(Self {
            agents,
            batch_tool,
            video_title,
            video_description,
        })
    }

    pub async fn run(&self) -> Result<Vec<QuestionedExtractedConcept>, PipelineError> {
        // Stage 1: Extract concepts independently from transcript batches.
        let concepts = extraction::extract_concepts(
            &self.agents.extraction,
            self.batch_tool.clone(),
            &self.video_title,
            &self.video_description,
        )
        .await?;

        // Stage 2: Merge concepts into the final outline.
        // let mut outline = merging::merge_concepts(&self.agents.merge, &concepts).await?;

        // Stage 3: Generate questions from the final outline.
        let questioned_extracted =
            questions::generate_questions(&self.agents.question, concepts).await?;

        Ok(questioned_extracted)
    }
}
