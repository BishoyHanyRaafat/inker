use crate::{
    AppState,
    errors::{AppError, AppErrorTrait, BaseError, NotesError},
    responses::ApiResponse,
    routes::{
        auth::models::Claims,
        notes::models::{PatchedNote, SeededNote},
    },
};
use axum::Json;
use axum::extract::{Path, State};
use sea_orm::{
    ActiveValue::{NotSet, Set},
    prelude::*,
};
use uuid::Uuid;

// Validation constants
const MAX_TITLE_LENGTH: usize = 500;
const MAX_NOTE_CONTENT_BYTES: usize = 1024 * 1024 * 20; // 20 MB max for note content

/// Validate note input
fn validate_note(note: &SeededNote) -> Result<(), AppError> {
    let title = note.title.trim();
    validate_title(title)?;
    validate_content(&note.content)?;
    if let Some(version) = note.version.clone() {
        validate_version(version)?;
    }

    Ok(())
}

fn validate_title(title: &str) -> Result<(), AppError> {
    if title.is_empty() {
        return Err(NotesError::TitleRequired.get_response());
    }

    if title.len() > MAX_TITLE_LENGTH {
        return Err(NotesError::TitleTooLong(MAX_TITLE_LENGTH).get_response());
    }

    Ok(())
}

/// Validate content input
fn validate_content(content: &str) -> Result<(), AppError> {
    if content.len() > MAX_NOTE_CONTENT_BYTES {
        return Err(NotesError::NoteContentTooLarge(MAX_NOTE_CONTENT_BYTES).get_response());
    }

    Ok(())
}

fn validate_version(version: String) -> Result<(), AppError> {
    if version != "1" {
        return Err(NotesError::InvalidVersion(version).get_response());
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

/// Update the note
#[utoipa::path(
    patch,
    path = "/api/v1/notes/{note_id}",
    tag = "notes",
    security(
        ("bearer" = [])
    ),
    params(
        ("note_id" = Uuid, Path, description = "Note ID")
    ),
    request_body = PatchedNote,
    responses(
        (status = 201, description = "Note content created successfully", body = ApiResponse<entities::note::Model>),
        (status = 400, description = "Invalid input", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 404, description = "Note not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn patch_note(
    claims: Claims,
    State(app_state): State<AppState>,
    Path(note_id): Path<Uuid>,
    Json(note): Json<PatchedNote>,
) -> Result<ApiResponse<entities::note::Model>, AppError> {
    if let Some(note_content) = &note.content {
        validate_content(note_content)?;
    }
    if let Some(version) = note.version.clone() {
        validate_version(version)?;
    }
    if let Some(title) = &note.title {
        validate_title(title)?;
    }

    // Fetch note content and its associated note
    let fetched_note = entities::note::Entity::find()
        .filter(entities::note::Column::Id.eq(note_id))
        .one(&app_state.db)
        .await
        .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?
        .ok_or_else(|| BaseError::NotFound.get_response())?;

    if fetched_note.user_id != claims.user_id {
        return Err(BaseError::NotFound.get_response());
    }

    // Update content
    let updated = entities::note::ActiveModel {
        id: Set(note_id),
        title: note.title.map_or(NotSet, Set),
        content: note.content.map_or(NotSet, Set),
        version: note.version.map_or(NotSet, Set),
        pinned: note.pinned.map_or(NotSet, Set),
        archived: note.archived.map_or(NotSet, Set),
        ..Default::default()
    }
    .update(&app_state.db)
    .await
    .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?;

    Ok(ApiResponse::success_response(
        axum::http::StatusCode::OK,
        "Updated the content successfully",
        updated,
    ))
}

// TODO:
// Use one table for both the content and the note useless ids
// better manage vectors for the agentic flow
// add caches for the content and extracted stuff
// better notes support more meta data and date time
// Fix OAuth2
// Use sse for and queues for tasks like ai stuff
