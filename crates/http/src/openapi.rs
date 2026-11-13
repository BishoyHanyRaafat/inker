use crate::config::VERSION;
use crate::responses::ApiResponse;
use crate::routes::auth::handler;
use crate::routes::auth::models::{
    Auth, LoginUser, ProviderPath, ProviderQuery, SeededUser, Token,
};
use crate::routes::auth::provider;
use crate::routes::notes::models::{PatchedNote, SeededNote};
use crate::routes::notes::notes_handler;
use crate::routes::yt_material::models::{
    ScheduledQuestion, YouTubeQuestionsRequest, YouTubeQuestionsResponse,
};
use crate::routes::yt_material::routes;
use entities::sea_orm_active_enums::OauthProvider;
use inker_http::routes::notes::models::Note;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handler::login,
        handler::signup,
        handler::refreshtoken,
        provider::provider_handler,
        notes_handler::get_notes,
        notes_handler::get_note_by_id,
        notes_handler::create_note,
        notes_handler::patch_note,
        routes::yt_question_handler,
        routes::yt_summary_handler
    ),
    components(schemas(
        ApiResponse<Auth>,
        ApiResponse<String>,
        ApiResponse<Vec<Note>>,
        ApiResponse<Note>,
        PatchedNote,
        Auth,
        Token,
        SeededUser,
        LoginUser,
        ProviderPath,
        ProviderQuery,
        OauthProvider,
        SeededNote,
        YouTubeQuestionsRequest,
        YouTubeQuestionsResponse,
        ScheduledQuestion
    )),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "notes", description = "Notes and blocks management endpoints"),
        (name = "interactive", description = "Interactive streaming endpoints"),
        (name = "ws", description = "WebSocket endpoints"),
    ),
    info(
        title = "Inker API",
        description = "API documentation for Inker service",
        version = VERSION,
    )
)]
pub struct ApiDoc;
