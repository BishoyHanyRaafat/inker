use crate::{
    AppState,
    errors::{AppError, AppErrorTrait, BaseError, NotesError},
    responses::ApiResponse,
    routes::{
        auth::models::Claims,
        notes::models::{NoteBlock, SeededBlock, SeededNote},
    },
};
use axum::Json;
use axum::extract::{Path, State};
use sea_orm::{ActiveValue::Set, QuerySelect, QueryTrait, prelude::*};
use uuid::Uuid;

// Validation constants
const MAX_TITLE_LENGTH: usize = 500;
const MAX_BLOCK_CONTENT_BYTES: usize = 1024 * 1024; // 1 MB max for block content

/// Validate note input
fn validate_note(note: &SeededNote) -> Result<(), AppError> {
    let title = note.title.trim();

    if title.is_empty() {
        return Err(NotesError::TitleRequired.get_response());
    }

    if title.len() > MAX_TITLE_LENGTH {
        return Err(NotesError::TitleTooLong(MAX_TITLE_LENGTH).get_response());
    }

    Ok(())
}

/// Validate block input
fn validate_block(block: &SeededBlock) -> Result<(), AppError> {
    // Check content size by serializing to JSON
    let content_json = serde_json::to_string(&block.content)
        .map_err(|e| BaseError::InternalServer.get_response().with_debug(e))?;

    if content_json.len() > MAX_BLOCK_CONTENT_BYTES {
        return Err(NotesError::BlockContentTooLarge(MAX_BLOCK_CONTENT_BYTES).get_response());
    }

    Ok(())
}

/// Get all notes
///
/// Retrieves all notes for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/v1/notes",
    tag = "notes",
    security(
        ("bearer" = [])
    ),
    responses(
        (status = 200, description = "Notes retrieved successfully", body = ApiResponse<Vec<entities::note::Model>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn get_notes(
    State(app_state): State<AppState>,
    claims: Claims,
) -> Result<ApiResponse<Vec<entities::note::Model>>, AppError> {
    let entities = entities::note::Entity::find()
        .filter(entities::note::Column::UserId.eq(claims.user_id))
        .all(&app_state.db)
        .await
        .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?;

    Ok(ApiResponse::success_response(
        axum::http::StatusCode::OK,
        "All notes retrieved successfully",
        entities,
    ))
}

/// Get note by ID
///
/// Retrieves a specific note by its ID for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/v1/notes/{note_id}",
    tag = "notes",
    security(
        ("bearer" = [])
    ),
    params(
        ("note_id" = Uuid, Path, description = "Note ID")
    ),
    responses(
        (status = 200, description = "Note retrieved successfully", body = ApiResponse<entities::note::Model>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 404, description = "Note not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn get_note_by_id(
    State(app_state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<ApiResponse<entities::note::Model>, AppError> {
    let entity = entities::note::Entity::find()
        .filter(entities::note::Column::Id.eq(id))
        .filter(entities::note::Column::UserId.eq(claims.user_id))
        .one(&app_state.db)
        .await
        .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?
        .ok_or_else(|| BaseError::NotFound.get_response())?;

    Ok(ApiResponse::success_response(
        axum::http::StatusCode::OK,
        "note retrieved successfully",
        entity,
    ))
}

/// Get blocks by note ID
///
/// Retrieves all blocks for a specific note that belongs to the authenticated user.
#[utoipa::path(
    get,
    path = "/api/v1/notes/{note_id}/block",
    tag = "notes",
    security(
        ("bearer" = [])
    ),
    params(
        ("note_id" = Uuid, Path, description = "Note ID")
    ),
    responses(
        (status = 200, description = "Blocks retrieved successfully", body = ApiResponse<Vec<NoteBlock>>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 404, description = "Note not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn get_blocks_by_note_id(
    State(app_state): State<AppState>,
    Path(note_id): Path<Uuid>,
    claims: Claims,
) -> Result<ApiResponse<Vec<NoteBlock>>, AppError> {
    use sea_orm::QueryFilter;
    use sea_orm::entity::*;

    // fetch all blocks for the given note_id that belongs to this user
    let blocks = entities::block::Entity::find()
        .filter(entities::block::Column::NoteId.eq(note_id))
        .filter(
            entities::block::Column::NoteId.in_subquery(
                entities::note::Entity::find()
                    .select_only()
                    .column(entities::note::Column::Id)
                    .filter(entities::note::Column::UserId.eq(claims.user_id))
                    .into_query(),
            ),
        )
        .all(&app_state.db)
        .await
        .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?;

    let note_blocks: Vec<NoteBlock> = blocks
        .into_iter()
        .map(|x| {
            NoteBlock::try_from(x)
                .map_err(|e| NotesError::NoteMalFormed.get_response().with_debug(e))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ApiResponse::success_response(
        axum::http::StatusCode::OK,
        "Blocks retrieved successfully",
        note_blocks,
    ))
}

/// Get block by ID
///
/// Retrieves a specific block by its ID. The block must belong to a note owned by the authenticated user.
#[utoipa::path(
    get,
    path = "/api/v1/notes/block/{block_id}",
    tag = "notes",
    security(
        ("bearer" = [])
    ),
    params(
        ("block_id" = Uuid, Path, description = "Block ID")
    ),
    responses(
        (status = 200, description = "Block retrieved successfully", body = ApiResponse<NoteBlock>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 404, description = "Block not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn get_block_by_id(
    claims: Claims,
    Path(block_id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> Result<ApiResponse<NoteBlock>, AppError> {
    let (block, note) = entities::block::Entity::find_by_id(block_id)
        .find_also_related(entities::note::Entity)
        .one(&app_state.db)
        .await
        .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?
        .ok_or(BaseError::NotFound.get_response())?;

    if (note.ok_or(BaseError::NotFound.get_response())?).user_id != claims.user_id {
        return Err(BaseError::NotFound.get_response());
    }
    Ok(ApiResponse::success_response(
        axum::http::StatusCode::OK,
        "Block retrieved successfully",
        NoteBlock::try_from(block)
            .map_err(|e| NotesError::NoteMalFormed.get_response().with_debug(e))?,
    ))
}

/// Create a new note
#[utoipa::path(
    post,
    path = "/api/v1/notes",
    tag = "notes",
    security(
        ("bearer" = [])
    ),
    request_body = SeededNote,
    responses(
        (status = 201, description = "Note created successfully", body = ApiResponse<entities::note::Model>),
        (status = 400, description = "Invalid input", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn create_note(
    claims: Claims,
    State(app_state): State<AppState>,
    Json(note): Json<SeededNote>,
) -> Result<ApiResponse<entities::note::Model>, AppError> {
    // Validate input
    validate_note(&note)?;

    let inserted_note = entities::note::ActiveModel {
        user_id: Set(claims.user_id),
        title: Set(note.title.trim().to_string()),
        archived: Set(note.archived.unwrap_or(false)),
        pinned: Set(note.pinned.unwrap_or(false)),
        ..Default::default()
    }
    .insert(&app_state.db)
    .await
    .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?;

    Ok(ApiResponse::success_response(
        axum::http::StatusCode::CREATED,
        "Note created successfully",
        inserted_note,
    ))
}

/// Create a new block for a note
#[utoipa::path(
    post,
    path = "/api/v1/notes/block",
    tag = "notes",
    security(
        ("bearer" = [])
    ),
    request_body = SeededBlock,
    responses(
        (status = 201, description = "Block created successfully", body = ApiResponse<NoteBlock>),
        (status = 400, description = "Invalid input", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 404, description = "Note not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn create_block(
    claims: Claims,
    State(app_state): State<AppState>,
    Json(block): Json<SeededBlock>,
) -> Result<ApiResponse<NoteBlock>, AppError> {
    // Validate input
    validate_block(&block)?;

    // Verify the note exists and belongs to the user
    if entities::note::Entity::find()
        .filter(entities::note::Column::Id.eq(block.note_id))
        .filter(entities::note::Column::UserId.eq(claims.user_id))
        .one(&app_state.db)
        .await
        .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?
        .is_none()
    {
        return Err(BaseError::NotFound.get_response());
    }

    let inserted_block_entity = entities::block::ActiveModel {
        note_id: Set(block.note_id),
        content: Set(serde_json::to_value(&block.content)
            .map_err(|e| BaseError::InternalServer.get_response().with_debug(e))?),
        version: Set(block.version.unwrap_or(1)),
        order: Set(block.order),
        ..Default::default()
    }
    .insert(&app_state.db)
    .await
    .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?;

    Ok(ApiResponse::success_response(
        axum::http::StatusCode::CREATED,
        "Block created successfully",
        NoteBlock::try_from(inserted_block_entity)
            .map_err(|e| NotesError::NoteMalFormed.get_response().with_debug(e))?,
    ))
}
