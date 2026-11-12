use agentic_pipeline::pipeline::SectionQuestions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct YouTubeQuestionsRequest {
    pub url: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ScheduledQuestion {
    pub trigger_at: f64,
    pub questions: SectionQuestions,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct YouTubeQuestionsResponse {
    pub questions: Vec<ScheduledQuestion>,
}
