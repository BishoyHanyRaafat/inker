mod agentic;
mod config;
mod extraction;
mod models;
mod questions;
mod summary;

use crate::{
    error::PipelineError,
    pipeline::models::{ExtractedConceptsStripped, MaterialPlanOutline},
    tools::{GetTranscriptSection, get_concept::GetConceptSection},
};
pub use config::{
    MAX_AGENT_RETRIES, MAX_CONCURRENT_QUESTION_REQUESTS, MAX_SECTION_TOKENS,
    MAX_SECTIONS_PER_BATCH, MODEL_NAME,
};
pub use models::{
    Concept, ConceptRef, Concepts, ExtractedConcept, ExtractedConceptMetaData, MaterialNote,
    SectionQuestions,
};

use agentic::PipelineAgents;
use yt_processing::Transcript;

/// Represent an Empty agent with no state
pub struct Empty;

/// Represent an agent that extracted the core concepts
pub struct Extracted {
    pub concepts: Vec<ExtractedConcept>,
}

/// Represent an agent that is has questions generated to it
pub struct Questioned {
    pub concepts: Vec<ExtractedConcept>,
    pub questioned: Vec<SectionQuestions>,
}

/// Represent an agent that entered the plan mode of a summary
pub struct Planned {
    pub concepts: Vec<ExtractedConcept>,
    pub outline: MaterialPlanOutline,
}

/// Represent an agent that entered that generated a summary
pub struct Summarized {
    pub concepts: Vec<ExtractedConcept>,
    pub summary: MaterialNote,
}

pub struct AgenticPipeline<S> {
    agents: PipelineAgents,
    batch_tool: GetTranscriptSection,
    video_title: String,
    video_description: String,
    state: S,
}

impl AgenticPipeline<Empty> {
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
            state: Empty,
        })
    }

    pub async fn extract(self) -> Result<AgenticPipeline<Extracted>, PipelineError> {
        tracing::info!("Extracting");
        // Stage 1: Extract concepts independently from transcript batches.
        let concepts = extraction::extract_concepts(
            &self.agents.extraction,
            self.batch_tool.clone(),
            &self.video_title,
            &self.video_description,
        )
        .await?;

        // std::fs::write(
        //     "output.json",
        //     serde_json::to_string_pretty(&concepts).unwrap(),
        // );

        // For testing purposes to avoid token loss
        // let concepts: Vec<ExtractedConcept> =
        //     serde_json::from_str(&std::fs::read_to_string("output.json").unwrap()).unwrap();

        tracing::info!(
            "Collected {} concepts",
            concepts
                .iter()
                .map(|concepts| concepts.concepts.concepts.len())
                .sum::<usize>()
        );

        Ok(AgenticPipeline {
            agents: self.agents,
            batch_tool: self.batch_tool,
            video_title: self.video_title,
            video_description: self.video_description,
            state: Extracted { concepts },
        })
    }
}

impl<S: HasExtractedConcepts> AgenticPipeline<S> {
    pub async fn generate_questions(self) -> Result<AgenticPipeline<Questioned>, PipelineError> {
        tracing::info!("Generating Questions");
        let questioned =
            questions::generate_questions(&self.agents.question, self.state.concepts()).await?;

        let concepts = self.state.into_concepts();
        Ok(AgenticPipeline {
            agents: self.agents,
            batch_tool: self.batch_tool,
            video_title: self.video_title,
            video_description: self.video_description,
            state: Questioned {
                concepts,
                questioned,
            },
        })
    }

    pub async fn generate_notes_plan(self) -> Result<AgenticPipeline<Planned>, PipelineError> {
        tracing::info!("Planning Note");
        let concepts: Vec<ExtractedConceptsStripped> = self
            .state
            .concepts()
            .clone()
            .into_iter()
            .map(Into::into)
            .collect();
        let outline = summary::generate_plan(&self.agents.planner, &concepts).await?;
        tracing::info!(
            "Generated a plan\ntitle: {}\nsubject: {}\ntags: {:?}\ncoverage_notes {:?}\nlearning_objectives {:?}",
            outline.title,
            outline.subject,
            outline.tags,
            outline.coverage_notes,
            outline.learning_objectives,
        );
        tracing::info!("Generated a plan for {} sections", outline.sections.len());
        Ok(AgenticPipeline {
            agents: self.agents,
            batch_tool: self.batch_tool,
            video_title: self.video_title,
            video_description: self.video_description,
            state: Planned {
                concepts: self.state.into_concepts(),
                outline,
            },
        })
    }
}

impl AgenticPipeline<Planned> {
    pub async fn generate_notes(self) -> Result<AgenticPipeline<Summarized>, PipelineError> {
        tracing::info!("Noting, Take a breath...");
        let tool = GetConceptSection::new(self.state.concepts());
        let material = summary::generate_planned_section(
            &self.agents.get_material_note(tool),
            &self.state.outline.sections,
        )
        .await?;
        tracing::info!("NOICE, {} note sections generated", material.len());
        Ok(AgenticPipeline {
            agents: self.agents,
            batch_tool: self.batch_tool,
            video_title: self.video_title,
            video_description: self.video_description,
            state: Summarized {
                concepts: self.state.concepts,
                summary: MaterialNote {
                    title: self.state.outline.title,
                    description: self.state.outline.description,
                    subject: self.state.outline.subject,
                    learning_objectives: self.state.outline.learning_objectives,
                    sections: material,
                    coverage_notes: self.state.outline.coverage_notes,
                    tags: self.state.outline.tags,
                },
            },
        })
    }
}

pub trait HasExtractedConcepts {
    fn concepts(&self) -> &Vec<ExtractedConcept>;
    fn into_concepts(self) -> Vec<ExtractedConcept>;
}

impl HasExtractedConcepts for Extracted {
    fn concepts(&self) -> &Vec<ExtractedConcept> {
        &self.concepts
    }

    fn into_concepts(self) -> Vec<ExtractedConcept> {
        self.concepts
    }
}

impl HasExtractedConcepts for Questioned {
    fn concepts(&self) -> &Vec<ExtractedConcept> {
        &self.concepts
    }

    fn into_concepts(self) -> Vec<ExtractedConcept> {
        self.concepts
    }
}

impl HasExtractedConcepts for Planned {
    fn concepts(&self) -> &Vec<ExtractedConcept> {
        &self.concepts
    }

    fn into_concepts(self) -> Vec<ExtractedConcept> {
        self.concepts
    }
}

impl AgenticPipeline<Questioned> {
    pub fn questions(&self) -> &Vec<SectionQuestions> {
        &self.state.questioned
    }
    pub fn concepts(&self) -> &Vec<ExtractedConcept> {
        &self.state.concepts
    }
}

impl AgenticPipeline<Summarized> {
    pub fn summary(&self) -> &MaterialNote {
        &self.state.summary
    }

    pub fn into_summary(self) -> MaterialNote {
        self.state.summary
    }
}
