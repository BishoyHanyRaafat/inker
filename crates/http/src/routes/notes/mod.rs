use crate::AppState;
use axum::{
    Router,
    routing::{get, post},
};

pub mod models;
pub mod notes_handler;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(notes_handler::get_notes))
        .route("/", post(notes_handler::create_note))
        .route("/{note_id}", get(notes_handler::get_note_by_id))
        .route(
            "/{note_id}/block",
            get(notes_handler::get_blocks_by_note_id),
        )
        .route("/block/{block_id}", get(notes_handler::get_block_by_id))
        .route("/block", post(notes_handler::create_block))
        .with_state(app_state)
}
