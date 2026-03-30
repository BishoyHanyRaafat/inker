use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub struct Chunk {
    title: String,
    content: String,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl From<String> for Chunk {
    fn from(value: String) -> Self {
        Self {
            content: value,
            title: String::new(),
        }
    }
}

impl Chunk {
    pub fn new(content: String) -> Self {
        Self {
            content,
            title: String::new(),
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, ToSchema)]
pub struct ProcessedChunk {
    summary: String,
    keypoints: Vec<String>,
    cloze_questions: Vec<ClozeQuestion>,
    flashcards: Vec<FlashCard>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, ToSchema)]
pub struct ClozeQuestion {
    text: String,
    answers: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, ToSchema)]
pub struct FlashCard {
    question: String,
    answer: String,
}

pub fn create_prompt_processed_chunk(transcript_chunk: Chunk) -> String {
    let title = transcript_chunk.title.clone();
    format!(
        r#"
You are an intelligent assistant that processes a transcript chunk from a lecture/video.
You must output exactly one object, in Json format, matching this schema:

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
- Output **only** the Json data (no explanation, commentary, or extra text).
- YOU MUST RETURN Json DATA NOT ANY OTHER TYPE
- Here is the title of the lecture/video: "{title}"
- Try to get relevant contents not useless stuff like "What is the discord server link?" or "Subscribe to my channel" etc...
"#,
        transcript_chunk = transcript_chunk,
        title = title
    )
}

pub fn create_prompt_table_of_content_chunk(transcript_chunk: Vec<ProcessedChunk>) -> String {
    let mut chunks = String::new();
    for i in 1..=transcript_chunk.len() {
        let chunk = &transcript_chunk[i - 1];
        chunks += &format!("Chunk {} Summary: {}\n", i, chunk.summary);
    }

    format!(
        r#"
You are an intelligent assistant that summarizes multiple processed transcript chunks from a lecture/video.
Generate a concise table of contents based on the summaries of the chunks provided.
you must return the output as json formatted as array of strings.

{transcript_chunk}
"#,
        transcript_chunk = chunks
    )
}

pub fn create_part_prompt(part: String) -> String {
    // TODO: make sure to use the notes struct
    format!(
        r#"
Now, based on the table of contents
let's work on part {part}
You must return a detailed explanation of this part.
You must not include any other information. excpet from the summaries above.
The output must be formatted using this struct:

"#,
        part = part
    )
}
