use rig::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Concepts {
    concepts: Vec<Concept>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Concept {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct ExtractedConcept {
    pub id: i32,
    pub concepts: Concepts,
    pub meta: ExtractedConceptMetaData,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct QuestionedExtractedConcept {
    pub extracted_concept: ExtractedConcept,
    pub section_question: SectionQuestions,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, Default)]
pub struct ExtractedConceptMetaData {
    pub transcript_content: String,
    pub start_time: f64,
    pub end_time: f64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, utoipa::ToSchema)]
pub struct ClozeQuestion {
    pub text: String,
    pub answers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, utoipa::ToSchema)]
pub struct OpenEndedQuestion {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, utoipa::ToSchema)]
pub struct FlashCardQuestion {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, utoipa::ToSchema)]
pub struct SectionQuestions {
    pub cloze: Vec<ClozeQuestion>,
    pub flashcards: Vec<FlashCardQuestion>,
    pub open_ended: Vec<OpenEndedQuestion>,
}
