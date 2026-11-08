use thiserror::Error;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Missing environment variable: {0}")]
    MissingEnv(&'static str),

    #[error("Gemini initialization failed: {0}")]
    Gemini(String),

    #[error("Gemini token counting request failed: {0}")]
    Tokenizer(#[from] reqwest::Error),

    #[error("Invalid token counting response: {0}")]
    InvalidTokenResponse(#[from] serde_json::Error),

    #[error("Transcript is empty")]
    EmptyTranscript,

    #[error("A transcript segment exceeds the section token budget")]
    OversizedSegment,

    #[error("Outline agent execution failed: {0}")]
    OutlineAgent(String),

    #[error("Merge agent execution failed: {0}")]
    MergeAgent(String),

    #[error("Question agent execution failed: {0}")]
    QuestionAgent(String),
}
