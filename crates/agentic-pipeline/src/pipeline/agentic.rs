use crate::{
    error::PipelineError,
    prompts::{EXTRACTION_SYSTEM_PROMPT, QUESTION_SYSTEM_PROMPT},
    tools::GetTranscriptSection,
};
use rig::{agent::Agent, prelude::*, providers::gemini::Gemini};

use super::config::MODEL_NAME;

pub(super) struct PipelineAgents {
    pub extraction: Agent,
    pub question: Agent,
}

impl PipelineAgents {
    pub fn new(batch_tool: &GetTranscriptSection) -> Result<Self, PipelineError> {
        let model = Gemini::from_env()
            .map_err(|error| PipelineError::Gemini(error.to_string()))?
            .completion(MODEL_NAME);

        let extraction = AgentBuilder::new(model.clone())
            .preamble(EXTRACTION_SYSTEM_PROMPT)
            .temperature(0.2)
            .default_max_turns(2)
            .tool(batch_tool.clone())
            .build();

        let question = AgentBuilder::new(model)
            .preamble(QUESTION_SYSTEM_PROMPT)
            .temperature(0.3)
            .default_max_turns(2)
            .build();

        Ok(Self {
            extraction,
            question,
        })
    }
}
