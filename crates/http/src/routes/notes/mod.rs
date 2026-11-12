use crate::AppState;
use axum::{
    Router,
    routing::{get, patch, post},
};

pub mod models;
pub mod notes_handler;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(notes_handler::get_notes))
        .route("/", post(notes_handler::create_note))
        .route("/{note_id}", get(notes_handler::get_note_by_id))
        .route("/{note_id}", patch(notes_handler::patch_note))
        .with_state(app_state)
}
