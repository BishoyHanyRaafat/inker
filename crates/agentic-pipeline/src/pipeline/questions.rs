use futures::future::join_all;
use rig::agent::Agent;

use crate::{error::PipelineError, pipeline::models::ExtractedConcept};

use super::{
    config::{MAX_AGENT_RETRIES, MAX_CONCURRENT_QUESTION_REQUESTS, RUN_AGENTS_IN_PARALLEL},
    models::SectionQuestions,
};

pub(super) async fn generate_questions(
    agent: &Agent,
    extracted_concepts: &Vec<ExtractedConcept>,
) -> Result<Vec<SectionQuestions>, PipelineError> {
    let mut out: Vec<SectionQuestions> = Vec::new();
    if RUN_AGENTS_IN_PARALLEL {
        for chunk in extracted_concepts
            .iter()
            .collect::<Vec<_>>()
            .chunks(MAX_CONCURRENT_QUESTION_REQUESTS)
        {
            let futures = chunk
                .iter()
                .cloned()
                .map(|node| process_node_questions(agent, node));

            let results = join_all(futures).await;

            for result in results {
                out.push(result?);
            }
        }
    } else {
        for node in extracted_concepts {
            out.push(process_node_questions(agent, node).await?);
        }
    }

    Ok(out)
}

async fn process_node_questions(
    agent: &Agent,
    node: &ExtractedConcept,
) -> Result<SectionQuestions, PipelineError> {
    let prompt = format!(
        r#"
            Generate study questions for the following section.

            Concepts:
            {:?}

            Transcript:
            {}
            "#,
        node.concepts, node.meta.transcript_content
    );

    let response = agent
        .prompt_typed::<SectionQuestions>(&prompt)
        .retries(MAX_AGENT_RETRIES)
        .await
        .map_err(|error| PipelineError::QuestionAgent(error.to_string()))?;

    Ok(response.output)
}
