use crate::AppState;
use axum::{Router, routing::post};

pub mod models;
pub mod routes;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/questions", post(routes::yt_question_handler))
        .route("/summary", post(routes::yt_summary_handler))
        .with_state(app_state)
}
