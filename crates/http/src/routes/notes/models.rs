use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeededNote {
    #[schema(default = "1")]
    pub version: Option<String>,
    pub content: String,
    pub title: String,
    #[schema(default = false)]
    pub archived: Option<bool>,
    #[schema(default = false)]
    pub pinned: Option<bool>,
    pub coverage_notes: Option<Vec<String>>,
    pub description: Option<String>,
    pub subject: Option<String>,
    pub tags: Option<Vec<String>>,
    pub learning_objectives: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PatchedNote {
    pub version: Option<String>,
    pub content: Option<String>,
    pub title: Option<String>,
    pub archived: Option<bool>,
    pub pinned: Option<bool>,
    pub coverage_notes: Option<Vec<String>>,
    pub description: Option<String>,
    pub subject: Option<String>,
    pub tags: Option<Vec<String>>,
    pub learning_objectives: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Note {
    pub id: Uuid,
    pub version: String,
    pub title: String,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
    pub archived: bool,
    pub pinned: bool,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub learning_objectives: Option<Vec<String>>,
    pub coverage_notes: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

impl TryFrom<entities::note::Model> for Note {
    type Error = serde_json::Error;
    fn try_from(value: entities::note::Model) -> Result<Self, serde_json::Error> {
        let learning_objectives: Option<Vec<String>> = value
            .learning_objectives
            .map(serde_json::from_value)
            .transpose()?;

        let tags: Option<Vec<String>> = value.tags.map(serde_json::from_value).transpose()?;

        let coverage_notes: Option<Vec<String>> = value
            .coverage_notes
            .map(serde_json::from_value)
            .transpose()?;

        Ok(Note {
            id: value.id,
            version: value.version,
            title: value.title,
            content: value.content,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            deleted_at: value.deleted_at,
            archived: value.archived,
            pinned: value.pinned,
            subject: value.subject,
            description: value.description,
            learning_objectives,
            coverage_notes,
            tags,
        })
    }
}

impl From<Note> for entities::note::Model {
    fn from(val: Note) -> Self {
        Self {
            id: val.id,
            version: val.version,
            title: val.title,
            content: val.content,
            user_id: val.user_id,
            created_at: val.created_at,
            updated_at: val.updated_at,
            deleted_at: val.deleted_at,
            archived: val.archived,
            pinned: val.pinned,
            subject: val.subject,
            description: val.description,
            learning_objectives: val.learning_objectives.map(Into::into),
            coverage_notes: val.coverage_notes.map(Into::into),
            tags: val.tags.map(Into::into),
        }
    }
}
