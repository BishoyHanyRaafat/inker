use crate::AppState;
use axum::{Router, routing::get};

pub mod events;
pub mod yt_websocket;

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/yt/ws", get(yt_websocket::ws_handler))
        .with_state(app_state)
}
