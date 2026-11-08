/// DEPRECATED:
/// Not currently used
/// Tokenizer estimates are much faster
///
use std::sync::Arc;

use reqwest::Client;
use serde::Deserialize;

use crate::error::PipelineError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CountTokensResponse {
    total_tokens: usize,
}

#[derive(Clone)]
pub struct GeminiTokenCounter {
    client: Client,
    api_key: Arc<str>,
    model: Arc<str>,
}

impl GeminiTokenCounter {
    pub fn from_env(model: impl Into<String>) -> Result<Self, PipelineError> {
        let api_key =
            std::env::var("GEMINI_API_KEY").map_err(|_| PipelineError::MissingEnv("GEMINI_KEY"))?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(PipelineError::Tokenizer)?;

        Ok(Self {
            client,
            api_key: api_key.into(),
            model: model.into().into(),
        })
    }

    pub async fn count(&self, text: &str) -> Result<usize, PipelineError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:countTokens",
            self.model
        );

        let response = self
            .client
            .post(url)
            .query(&[("key", self.api_key.as_ref())])
            .json(&serde_json::json!({
                "contents": [{
                    "parts": [{
                        "text": text
                    }]
                }]
            }))
            .send()
            .await?
            .error_for_status()?
            .json::<CountTokensResponse>()
            .await?;
        tracing::info!("Token count of the current section is: {:?}", response);

        Ok(response.total_tokens)
    }
}
