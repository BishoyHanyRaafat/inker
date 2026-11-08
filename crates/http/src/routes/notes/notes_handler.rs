use crate::{
    AppState,
    errors::{AppError, AppErrorTrait, BaseError, NotesError},
    responses::ApiResponse,
    routes::{
        auth::models::Claims,
        notes::models::{PatchNoteContent, SeededNote, SeededNoteContent},
    },
};
use axum::Json;
use axum::extract::{Path, State};
use sea_orm::{
    ActiveValue::{NotSet, Set},
    Condition, QuerySelect, QueryTrait,
    prelude::*,
};
use uuid::Uuid;

// Validation constants
const MAX_TITLE_LENGTH: usize = 500;
const MAX_NOTE_CONTENT_BYTES: usize = 1024 * 1024 * 20; // 20 MB max for note content

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

/// Validate content input
fn validate_content(content: &str, version: Option<i32>) -> Result<(), AppError> {
    let version = version.unwrap_or(1);
    if version != 1 {
        return Err(NotesError::InvalidVersion(version).get_response());
    }

    if content.len() > MAX_NOTE_CONTENT_BYTES {
        return Err(NotesError::NoteContentTooLarge(MAX_NOTE_CONTENT_BYTES).get_response());
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

/// Get Content by note ID
///
/// Retrieves the note content for a specific note that belongs to the authenticated user.
#[utoipa::path(
    get,
    path = "/api/v1/notes/{note_id}/content",
    tag = "notes",
    security(
        ("bearer" = [])
    ),
    params(
        ("note_id" = Uuid, Path, description = "Note ID")
    ),
    responses(
        (status = 200, description = "Content retrieved successfully", body = ApiResponse<entities::note_content::Model>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 404, description = "Note not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn get_note_content_by_note_id(
    State(app_state): State<AppState>,
    Path(note_id): Path<Uuid>,
    claims: Claims,
) -> Result<ApiResponse<entities::note_content::Model>, AppError> {
    use sea_orm::QueryFilter;
    use sea_orm::entity::*;

    // fetch note content for the given note_id that belongs to this user
    let note_content = entities::note_content::Entity::find()
        .filter(entities::note_content::Column::NoteId.eq(note_id))
        .filter(
            entities::note_content::Column::NoteId.in_subquery(
                entities::note::Entity::find()
                    .select_only()
                    .column(entities::note::Column::Id)
                    .filter(entities::note::Column::UserId.eq(claims.user_id))
                    .into_query(),
            ),
        )
        .one(&app_state.db)
        .await
        .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?
        .ok_or_else(|| BaseError::NotFound.get_response())?;

    Ok(ApiResponse::success_response(
        axum::http::StatusCode::OK,
        "Note content retrieved successfully",
        note_content,
    ))
}

/// Get note content by ID
///
/// Retrieves a specific note content by its ID. The note content must belong to a note owned by the authenticated user.
#[utoipa::path(
    get,
    path = "/api/v1/notes/content/{content_id}",
    tag = "notes",
    security(
        ("bearer" = [])
    ),
    params(
        ("note_content_id" = Uuid, Path, description = "Note content ID")
    ),
    responses(
        (status = 200, description = "Note Content retrieved successfully", body = ApiResponse<entities::note_content::Model>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 404, description = "Note content not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn get_note_content_by_id(
    claims: Claims,
    Path(note_content_id): Path<Uuid>,
    State(app_state): State<AppState>,
) -> Result<ApiResponse<entities::note_content::Model>, AppError> {
    let (content, note) = entities::note_content::Entity::find_by_id(note_content_id)
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
        "Note content retrieved successfully",
        content,
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

/// Create a new note content for a note
#[utoipa::path(
    post,
    path = "/api/v1/notes/content",
    tag = "notes",
    security(
        ("bearer" = [])
    ),
    request_body = SeededNoteContent,
    responses(
        (status = 201, description = "Note content created successfully", body = ApiResponse<SeededNoteContent>),
        (status = 400, description = "Invalid input", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 404, description = "Note not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn create_note_content(
    claims: Claims,
    State(app_state): State<AppState>,
    Json(content): Json<SeededNoteContent>,
) -> Result<ApiResponse<entities::note_content::Model>, AppError> {
    // Validate input
    validate_content(&content.content, content.version)?;

    // Verify the note exists and belongs to the user
    if entities::note::Entity::find()
        .filter(entities::note::Column::Id.eq(content.note_id))
        .filter(entities::note::Column::UserId.eq(claims.user_id))
        .one(&app_state.db)
        .await
        .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?
        .is_none()
    {
        return Err(BaseError::NotFound.get_response());
    }

    if entities::note_content::Entity::find()
        .filter(entities::note_content::Column::NoteId.eq(content.note_id))
        .one(&app_state.db)
        .await
        .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?
        .is_none()
    {
        return Err(BaseError::InternalServer
            .get_response()
            .with_debug("Error creating duplicated content"));
    }

    let inserted_note_content = entities::note_content::ActiveModel {
        note_id: Set(content.note_id),
        content: Set(content.content),
        version: Set(content.version.unwrap_or(1)),
        ..Default::default()
    }
    .insert(&app_state.db)
    .await
    .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?;

    Ok(ApiResponse::success_response(
        axum::http::StatusCode::CREATED,
        "Note content created successfully",
        inserted_note_content,
    ))
}

/// Update the note content
#[utoipa::path(
    patch,
    path = "/api/v1/notes/content",
    tag = "notes",
    security(
        ("bearer" = [])
    ),
    request_body = PatchNoteContent,
    responses(
        (status = 201, description = "Note content created successfully", body = ApiResponse<entities::note_content::Model>),
        (status = 400, description = "Invalid input", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 404, description = "Note not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>),
    )
)]
#[axum::debug_handler]
pub async fn patch_note_content(
    claims: Claims,
    State(app_state): State<AppState>,
    Json(content): Json<PatchNoteContent>,
) -> Result<ApiResponse<entities::note_content::Model>, AppError> {
    validate_content(&content.content, content.version)?;

    // Fetch note content and its associated note
    let (note_content, note) = entities::note_content::Entity::find()
        .filter(
            Condition::any()
                .add(entities::note_content::Column::Id.eq(content.id))
                .add(entities::note_content::Column::NoteId.eq(content.id)),
        )
        .find_also_related(entities::note::Entity)
        .one(&app_state.db)
        .await
        .map_err(|e| BaseError::DatabaseError.get_response().with_debug(e))?
        .ok_or_else(|| BaseError::NotFound.get_response())?;

    // Verify ownership
    let note = note.ok_or_else(|| BaseError::NotFound.get_response())?;

    if note.user_id != claims.user_id {
        return Err(BaseError::NotFound.get_response());
    }

    // Update content
    let updated = entities::note_content::ActiveModel {
        id: Set(note_content.id),
        content: Set(content.content),
        version: content.version.map_or(NotSet, Set),
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
