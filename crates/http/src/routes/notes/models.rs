/// Jsonb content pub structure:
/// ID: block_type_UUID
/// data can be one of the following:
/// TextBlock
///     List of segments field
///     each segment has text and marks
///     marks is a list of: bold, italic, underline, strike through, code, link,
///     highlight, mention (If the segment is mention the text will be the user_id), color(starts with # for hex color)
/// ImageBlock
///     Just an url to the image
/// TODOBlock
///     List of text
/// TableBlock
///      List of ordered Blocks
/// DividerBlock
///     Nothing we don't need any field for divider
///
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NoteBlock {
    id: String,
    block: Block,
    note_id: uuid::Uuid,
    order: f32,
    version: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum Block {
    Text(TextBlock),
    Image(ImageBlock),
    Todo(TodoBlock),
    Table(TableBlock),
    Divider(DividerBlock),
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum TableCellContent {
    Text(TextBlock),
    Image(ImageBlock),
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TextBlock {
    segments: Vec<TextSegment>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TextSegment {
    text: String,
    marks: Vec<TextMark>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum TextMark {
    Bold,
    Italic,
    Underline,
    StrikeThrough,
    Code,
    Link,
    Highlight,
    Mention,
    Color,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageBlock {
    url: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TodoBlock {
    items: Vec<TodoItem>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TodoItem {
    text: String,
    completed: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TableBlock {
    cells: Vec<TableCell>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TableCell {
    row: usize,
    column: usize,
    block: TableCellContent,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DividerBlock;

impl TryFrom<entities::block::Model> for NoteBlock {
    type Error = serde_json::Error;
    fn try_from(model: entities::block::Model) -> Result<Self, Self::Error> {
        Ok(NoteBlock {
            id: model.id.to_string(),
            block: serde_json::from_value(model.content)?,
            version: model.version,
            note_id: model.note_id,
            order: model.order,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeededNote {
    pub title: String,
    #[schema(default = false)]
    pub archived: Option<bool>,
    #[schema(default = false)]
    pub pinned: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SeededBlock {
    pub content: Block,
    pub note_id: uuid::Uuid,
    pub order: f32,
    #[schema(default = 1)]
    pub version: Option<i32>,
}
