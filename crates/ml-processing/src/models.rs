use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

pub struct Chunk {
    content: String,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ProcessedChunk {
    summary: String,
    keypoints: Vec<String>,
    cloze_questions: Vec<ClozeQuestion>,
    flashcards: Vec<FlashCard>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ClozeQuestion {
    text: String,
    answers: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct FlashCard {
    question: String,
    answer: String,
}

pub fn create_prompt(transcript_chunk: Chunk) -> String {
    format!(
        r#"
You are an intelligent assistant that processes a transcript chunk from a lecture/video.
You must output exactly one object, in TOON (Token-Oriented Object Notation) format, matching this schema:

ProcessedChunk{{
  summary: String,
  keypoints: [String],
  cloze_questions: [{{ text: String, answers: [String] }}],
  flashcards: [{{ question: String, answer: String }}]
}}

Here is the transcript chunk you must analyze:
"""
{transcript_chunk}
"""

=== Requirements ===
- `summary` should be a short but informative summary (1–3 sentences) of the chunk.
- `keypoints` should list the most important statements / facts / concepts from the chunk.
- For `cloze_questions`, generate 0 to 3 fill-in-the-blank questions derived from key concepts in the chunk.
  - For each cloze, `text` is the sentence with a blank (use “<BLANK>”  for the blank),
  - `answers` is a list containing the correct fill(s).
- For `flashcards`, generate 0 or more cards: each with a `question` (a concise prompt) and `answer` (the correct answer).  
- Do **not** add any extra fields.  
- Output **only** the TOON data (no explanation, commentary, or extra text).
"#,
        transcript_chunk = transcript_chunk
    )
}
