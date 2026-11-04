use rig::schemars;
use rig::schemars::JsonSchema;
use rig::{agent::Agent, prelude::*, providers::gemini::Gemini};
use serde::{Deserialize, Serialize};

use crate::tools::get_section::MAX_SECTION_TOKENS;
use crate::{
    error::PipelineError, tokenizer::GeminiTokenCounter, tools::GetTranscriptSection,
    transcript::Transcript,
};

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct OutlineNode {
    pub title: String,
    pub children: Option<Vec<OutlineNode>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Outline {
    pub nodes: Vec<OutlineNode>,
}

pub struct AgenticPipeline {
    agent: Agent,
    section_tool: GetTranscriptSection,
}

impl AgenticPipeline {
    pub async fn new(
        model_name: impl Into<String>,
        transcript: Transcript,
    ) -> Result<Self, PipelineError> {
        let model_name = model_name.into();

        if transcript.is_empty() {
            return Err(PipelineError::EmptyTranscript);
        }

        let counter = GeminiTokenCounter::from_env(&model_name)?;

        let mut prepared_segments = Vec::with_capacity(transcript.captions.segments.len());

        for segment in transcript.captions.segments {
            let token_count = counter.count(&segment.text).await?;

            if token_count > MAX_SECTION_TOKENS {
                return Err(PipelineError::OversizedSegment);
            }

            prepared_segments.push((segment, token_count));
        }

        let section_tool = GetTranscriptSection::new(prepared_segments);

        let model = Gemini::from_env()
            .map_err(|error| PipelineError::Gemini(error.to_string()))?
            .completion(model_name);

        let agent = AgentBuilder::new(model)
            .preamble(
                "You are a YouTube video analysis assistant.

                Your task is to build a comprehensive and accurate outline
                of the video using its transcript.

                You have access to get_next_transcript_section.

                The outline should be up to 3 levels deep (Topic -> Sub-topic -> Detail).

                Rules:
                - Read transcript sections sequentially.
                - Never invent information.
                - Preserve important concepts, examples, and explanations.
                - Include timestamps for important topics within the title.
                - When batch_complete is true, stop retrieving sections.
                - After each batch, update the existing outline while maintaining the structured format.
                - Do not discard information from previous batches.
                - When has_more is false, produce the final structured outline.",
            )
            .temperature(0.2)
            .default_max_turns(8)
            .tool(section_tool.clone())
            .build();

        Ok(Self {
            agent,
            section_tool,
        })
    }

    pub async fn summarize(&self) -> Result<Vec<OutlineNode>, PipelineError> {
        let mut current_outline: Option<Outline> = None;

        loop {
            self.section_tool.reset_batch().await;

            let prompt = format!(
                "Read the next transcript batch using \
                 get_next_transcript_section.

                 Previous outline:
                 {}

                 Retrieve sections until batch_complete is true,
                 or the transcript is exhausted.

                 Then update the outline using the new information.

                 If the transcript is exhausted, return the complete
                 final structured outline.",
                if let Some(ref outline) = current_outline {
                    serde_json::to_string(outline)
                        .unwrap_or_else(|_| "Error serializing outline".to_string())
                } else {
                    "No outline has been created yet.".to_string()
                }
            );

            let response = self
                .agent
                .prompt_typed::<Outline>(&prompt)
                .max_turns(8)
                .await
                .map_err(|error| PipelineError::Agent(error.to_string()))?;

            current_outline = Some(response.output);

            if !self.section_tool.has_more().await {
                return Ok(current_outline.map(|o| o.nodes).unwrap_or_default());
            }
        }
    }
}
