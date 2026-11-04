use crate::config::VERSION;
use crate::responses::ApiResponse;
use crate::routes::auth::handler;
use crate::routes::auth::models::{
    Auth, LoginUser, ProviderPath, ProviderQuery, SeededUser, Token,
};
use crate::routes::auth::provider;
use crate::routes::interactive::{
    events::YouTubeStreamEventUpdate, events::YouTubeStreamResponse, yt_websocket,
};
use crate::routes::notes::models::{PatchNoteContent, SeededNote, SeededNoteContent};
use crate::routes::notes::notes_handler;
use entities::sea_orm_active_enums::OauthProvider;
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
        notes_handler::get_note_content_by_note_id,
        notes_handler::get_note_content_by_id,
        notes_handler::create_note,
        notes_handler::create_note_content,
        notes_handler::patch_note_content,
        yt_websocket::ws_handler,
    ),
    components(schemas(
        ApiResponse<Auth>,
        ApiResponse<String>,
        ApiResponse<Vec<entities::note::Model>>,
        ApiResponse<entities::note::Model>,
        ApiResponse<entities::note_content::Model>,
        Auth,
        Token,
        SeededUser,
        LoginUser,
        ProviderPath,
        ProviderQuery,
        OauthProvider,
        entities::note::Model,
        SeededNote,
        SeededNoteContent,
        PatchNoteContent,
        YouTubeStreamResponse,
        YouTubeStreamEventUpdate
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
