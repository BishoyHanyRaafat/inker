use crate::{
    error::PipelineError,
    prompts::{
        EXTRACTION_SYSTEM_PROMPT, QUESTION_SYSTEM_PROMPT, SECTION_WRITER_PROMPT,
        SUMMARY_PLANNER_PROMPT,
    },
    tools::{GetTranscriptSection, get_concept::GetConceptSection},
};
use rig::{
    agent::Agent,
    prelude::*,
    providers::gemini::{Gemini, completion::GenerateContent},
};

use super::config::MODEL_NAME;

pub(super) struct PipelineAgents {
    pub extraction: Agent,
    pub question: Agent,
    pub planner: Agent,
    pub model: Model<GenerateContent>,
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

        let question = AgentBuilder::new(model.clone())
            .preamble(QUESTION_SYSTEM_PROMPT)
            .temperature(0.3)
            .default_max_turns(2)
            .build();

        let planner = AgentBuilder::new(model.clone())
            .preamble(SUMMARY_PLANNER_PROMPT)
            .temperature(0.3)
            .default_max_turns(2)
            .build();
        Ok(Self {
            extraction,
            question,
            planner,
            model,
        })
    }
    pub fn get_material_note(&self, tool: GetConceptSection) -> Agent {
        AgentBuilder::new(self.model.clone())
            .preamble(SECTION_WRITER_PROMPT)
            .temperature(0.3)
            .tool(tool)
            .default_max_turns(2)
            .build()
    }
}
