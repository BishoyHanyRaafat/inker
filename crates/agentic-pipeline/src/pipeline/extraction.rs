use rig::agent::Agent;

use crate::error::PipelineError;
use crate::pipeline::models::Concepts;
use crate::tools::GetTranscriptSection;

use super::models::ExtractedConcept;

pub(super) async fn extract_concepts(
    agent: &Agent,
    batch_tool: GetTranscriptSection,
    video_title: &str,
    video_description: &str,
) -> Result<Vec<ExtractedConcept>, PipelineError> {
    let mut current_outline: Vec<ExtractedConcept> = Vec::new();
    let mut outline_index = 0;

    loop {
        batch_tool.reset_batch().await;
        let prompt = format!(
            "
                You are a YouTube video analysis assistant.
                Here is the Video title: {video_title}
                Here is the Video descriptions: {video_description}

                Your task is to build a comprehensive and accurate outline
                of the video using its transcript.

                You have access to get_next_transcript_section.

                The outline should be up to 3 levels deep (Topic -> Sub-topic -> Detail).",
        );

        let response = agent
            .prompt_typed::<Concepts>(prompt)
            .max_turns(8)
            .await
            .map_err(|error| PipelineError::OutlineAgent(error.to_string()))?;
        outline_index += 1;
        current_outline.push(ExtractedConcept {
            id: outline_index,
            concepts: response.output,
            meta: batch_tool.get_batch_meta().await,
        });

        if !batch_tool.has_more().await {
            return Ok(current_outline);
        }
    }
}
