mod models;

pub use models::*;

#[derive(Clone)]
pub struct GeminiClient {
    client: gemini_rust::Gemini,
}

pub enum Error {
    GeminiError(gemini_rust::ClientError),
    DecodeError(toon_format::ToonError),
    JsonDecodeError(serde_json::Error),
}

impl GeminiClient {
    pub fn new(api_key: String) -> Result<Self, gemini_rust::ClientError> {
        Ok(Self {
            client: gemini_rust::Gemini::new(api_key)?,
        })
    }
    pub async fn process_chunk(&self, chunk: Chunk) -> Result<ProcessedChunk, Error> {
        let prompt = create_prompt_processed_chunk(chunk);
        let resp = self
            .client
            .generate_content()
            .with_user_message(prompt)
            .execute()
            .await
            .map_err(Error::GeminiError)?;
        let text = resp.text();
        tracing::info!("Gemini response: {}", text);

        let text = text.strip_prefix("```json").unwrap_or(&text);
        let text = text.strip_suffix("```").unwrap_or(text);

        // Toon not working properly, using JSON for now
        // LLMS currently don't know much about toon format
        // decode_default(text).map_err(Error::DecodeError)
        serde_json::from_str::<ProcessedChunk>(text).map_err(Error::JsonDecodeError)
    }
}
