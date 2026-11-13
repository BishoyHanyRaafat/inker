use rig::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Concepts {
    pub concepts: Vec<Concept>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Concept {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct ExtractedConcept {
    pub id: u32,
    pub concepts: Concepts,
    pub meta: ExtractedConceptMetaData,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct ExtractedConceptsStripped {
    pub id: u32,
    pub concepts: Concepts,
}

impl From<ExtractedConcept> for ExtractedConceptsStripped {
    fn from(value: ExtractedConcept) -> Self {
        Self {
            id: value.id,
            concepts: value.concepts,
        }
    }
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
/// TODO: Add reference to the transcription
pub struct SectionQuestions {
    pub cloze: Vec<ClozeQuestion>,
    pub flashcards: Vec<FlashCardQuestion>,
    pub open_ended: Vec<OpenEndedQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Material<T> {
    pub title: String,
    pub description: String,
    pub subject: String,
    pub learning_objectives: Vec<String>,
    pub sections: Vec<T>,
    pub coverage_notes: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ContentType {
    Markdown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MaterialSectionContent {
    pub content: String,
    #[serde(rename = "type")]
    pub content_type: ContentType,
}

pub type MaterialPlanOutline = Material<OutlineSection>;
pub type MaterialNote = Material<MaterialSectionContent>;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OutlineSection {
    pub title: String,
    pub purpose: String,
    pub concept_refs: Vec<ConceptRef>,
    pub key_points: Vec<String>,
    pub content_requirements: Vec<String>,
    pub recommended_visuals: Vec<RecommendedVisual>,
    pub children: Vec<OutlineSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConceptRef {
    pub batch_id: u32,
    pub concept_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum VisualTypes {
    Digram,
    FlowChart,
    Table,
    Graph,
    MindMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecommendedVisual {
    #[serde(rename = "type")]
    pub visual_type: VisualTypes,
    pub purpose: String,
}

// TODO: Maybe add a review if needed
// pub struct ReviewResult {
//     pub passed: bool,
//     pub issues: Vec<ReviewIssue>,
//     pub missing_concepts: Vec<ConceptRef>,
// }
// pub struct ReviewIssue {
//     pub section_id: String,
//     pub issue_type: String,
//     pub description: String,
//     pub suggested_fix: String,
// }
