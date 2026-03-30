use crate::AppState;
use axum::Router;

pub mod auth;
pub mod extractors;
pub mod interactive;
pub mod notes;

pub fn get_route(appstate: AppState) -> Router {
    let api_routes = Router::new()
        .nest("/auth", auth::router(appstate.clone()))
        .nest("/notes", notes::router(appstate.clone()))
        .nest("/interactive", interactive::router(appstate.clone()));

    Router::new().nest("/api/v1", api_routes)
}
