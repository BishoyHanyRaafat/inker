use futures::future::join_all;
use rig::agent::Agent;

use crate::{
    error::PipelineError,
    pipeline::models::{
        ExtractedConceptsStripped, MaterialPlanOutline, MaterialSectionContent, OutlineSection,
    },
};

use super::config::{MAX_AGENT_RETRIES, MAX_CONCURRENT_QUESTION_REQUESTS, RUN_AGENTS_IN_PARALLEL};

pub(super) async fn generate_plan(
    agent: &Agent,
    extracted_concepts: &Vec<ExtractedConceptsStripped>,
) -> Result<MaterialPlanOutline, PipelineError> {
    let prompt = format!(
        r#"
            Generate study plan for the following concepts.

            Concepts:
            {:?}
            "#,
        extracted_concepts
    );

    let response = agent
        .prompt_typed::<MaterialPlanOutline>(&prompt)
        .retries(MAX_AGENT_RETRIES)
        .await
        .map_err(|error| PipelineError::QuestionAgent(error.to_string()))?;

    Ok(response.output)
}
pub(super) async fn generate_planned_section(
    agent: &Agent,
    outline_section: &Vec<OutlineSection>,
) -> Result<Vec<MaterialSectionContent>, PipelineError> {
    let mut out: Vec<MaterialSectionContent> = Vec::new();
    if RUN_AGENTS_IN_PARALLEL {
        for chunk in outline_section
            .iter()
            .collect::<Vec<_>>()
            .chunks(MAX_CONCURRENT_QUESTION_REQUESTS)
        {
            let futures = chunk
                .iter()
                .cloned()
                .map(|node| process_node_material(agent, node));

            let results = join_all(futures).await;

            for result in results {
                out.push(result?);
            }
        }
    } else {
        for node in outline_section {
            out.push(process_node_material(agent, node).await?);
        }
    }

    Ok(out)
}

async fn process_node_material(
    agent: &Agent,
    node: &OutlineSection,
) -> Result<MaterialSectionContent, PipelineError> {
    let prompt = format!(
        r#"
            Generate the following study material according to the given planned outline

            Planned Outline:
            {:?}
            "#,
        node
    );

    let response = agent
        .prompt_typed::<MaterialSectionContent>(&prompt)
        .retries(MAX_AGENT_RETRIES)
        .await
        .map_err(|error| PipelineError::QuestionAgent(error.to_string()))?;

    tracing::info!("Generate section with title: {}", node.title);
    Ok(response.output)
}
