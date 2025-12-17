use crate::config::VERSION;
use crate::responses::ApiResponse;
use crate::routes::auth::handler;
use crate::routes::auth::models::{
    Auth, LoginUser, ProviderPath, ProviderQuery, SeededUser, Token,
};
use crate::routes::auth::provider;
use crate::routes::notes::models::{
    Block, DividerBlock, ImageBlock, NoteBlock, SeededBlock, SeededNote, TableBlock, TableCell,
    TextBlock, TextMark, TextSegment, TodoBlock,
};
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
        notes_handler::get_blocks_by_note_id,
        notes_handler::get_block_by_id,
        notes_handler::create_note,
        notes_handler::create_block,
    ),
    components(schemas(
        ApiResponse<Auth>,
        ApiResponse<String>,
        ApiResponse<Vec<entities::note::Model>>,
        ApiResponse<entities::note::Model>,
        ApiResponse<Vec<NoteBlock>>,
        ApiResponse<NoteBlock>,
        Auth,
        Token,
        SeededUser,
        LoginUser,
        ProviderPath,
        ProviderQuery,
        OauthProvider,
        entities::note::Model,
        NoteBlock,
        Block,
        TextBlock,
        TextSegment,
        TextMark,
        ImageBlock,
        TodoBlock,
        TableBlock,
        TableCell,
        DividerBlock,
        SeededBlock,
        SeededNote,
    )),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "notes", description = "Notes and blocks management endpoints")
    ),
    info(
        title = "Inker API",
        description = "API documentation for Inker service",
        version = VERSION,
    )
)]
pub struct ApiDoc;
