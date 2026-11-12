use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PatchedNote {
    pub version: Option<String>,
    pub content: Option<String>,
    pub title: Option<String>,
    pub archived: Option<bool>,
    pub pinned: Option<bool>,
}
