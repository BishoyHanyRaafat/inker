mod models;

pub use models::*;
use toon_format::decode_default;

#[derive(Clone)]
pub struct GeminiClient {
    client: gemini_rust::Gemini,
}

pub enum Error {
    GeminiError(gemini_rust::ClientError),
    DecodeError(toon_format::ToonError),
}

impl GeminiClient {
    pub fn new(api_key: String) -> Result<Self, gemini_rust::ClientError> {
        Ok(Self {
            client: gemini_rust::Gemini::new(api_key)?,
        })
    }
    pub async fn process_chunk(&self, chunk: Chunk) -> Result<ProcessedChunk, Error> {
        let prompt = create_prompt(chunk);
        let resp = self
            .client
            .generate_content()
            .with_user_message(prompt)
            .execute()
            .await
            .map_err(Error::GeminiError)?;

        decode_default(resp.text().to_string().as_str()).map_err(Error::DecodeError)
    }
}
