use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeededNote {
    pub title: String,
    #[schema(default = false)]
    pub archived: Option<bool>,
    #[schema(default = false)]
    pub pinned: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeededNoteContent {
    #[schema(default = 1)]
    pub version: Option<i32>,
    /// This is a markdown
    ///
    /// We can change it in the feature to use MDX to support more components
    pub content: String,
    pub note_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PatchNoteContent {
    pub version: Option<i32>,
    pub content: String,
    /// This can be either content id or the note id
    pub id: uuid::Uuid,
}
