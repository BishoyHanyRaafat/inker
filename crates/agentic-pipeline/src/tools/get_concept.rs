use crate::pipeline::{Concept, ConceptRef, Concepts, ExtractedConcept};
use std::{collections::HashMap, sync::Arc};

use rig::{prelude::*, tool::ToolContext};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConceptRetrievalError {
    #[error("Concept not found")]
    NotFound,
}

#[derive(Clone, Debug)]
pub struct GetConceptSection {
    concepts: Arc<HashMap<(u32, usize), Concept>>,
}

impl GetConceptSection {
    pub fn new(batches: &[ExtractedConcept]) -> Self {
        let concepts = batches
            .iter()
            .flat_map(|batch| {
                let batch_id = batch.id;

                batch
                    .concepts
                    .concepts
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(move |(index, concept)| ((batch_id, index), concept))
            })
            .collect();

        Self {
            concepts: Arc::new(concepts),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetConceptSectionArgs {
    pub references: Vec<ConceptRef>,
}

impl Tool for GetConceptSection {
    const NAME: &'static str = "get_concept_by_ref";

    type Args = GetConceptSectionArgs;
    type Output = Concepts;
    type Error = ConceptRetrievalError;

    fn description(&self) -> String {
        "Get the related concept by the ref, A concept is a content which the was stated in the material \
        Use the batch id with the concept index to retrieve the concept"
        .to_string()
    }

    fn parameters(&self) -> Value {
        serde_json::to_value(schema_for!(GetConceptSectionArgs)).unwrap()
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        tracing::info!("Retriving {:?}", args);
        let concepts = args
            .references
            .iter()
            .map(|reference| {
                self.concepts
                    .get(&(reference.batch_id, reference.concept_index))
                    .cloned()
                    .ok_or(ConceptRetrievalError::NotFound)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Concepts { concepts })
    }
}
