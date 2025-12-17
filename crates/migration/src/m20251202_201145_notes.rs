use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Note
/// Each note has id, title, created_at, updated_at, deleted_at, is_archived, is_pinned and user_id (foreign key to users
/// table)
///
/// Block
/// Each block has id, version, content, order(float), note_id (foreign key to notes table)
/// content is a jsonb value that can store different types of blocks (text, image, todo, etc.)
///
///
/// Jsonb content structure:
/// ID: block_type_UUID
/// data can be one of the following:
/// TextBlock
///     List of segments field
///     each segment has text and marks
///     marks is a list of: bold, italic, underline, strike through, code, link,
///     highlight, mention (If the segment is mention the text will be the user_id), color(starts with # for hex color)
/// ImageBlock
///     Just an url to the image
/// TODOBlock
///     List of text
/// TableBlock
///      List of ordered Blocks
/// DividerBlock
///     Nothing we don't need any field for divider
///
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(Note::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Note::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(string(Note::Title).not_null())
                    .col(uuid(Note::UserId).not_null())
                    .col(
                        timestamp(Note::CreatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(Note::UpdatedAt)
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".into())),
                    )
                    .col(
                        timestamp(Note::DeletedAt)
                            .null()
                            .default(None::<prelude::NaiveDateTime>),
                    )
                    .col(
                        ColumnDef::new(Note::Archived)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Note::Pinned)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Note::Table, Note::UserId)
                            .to("user", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Block::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Block::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(integer(Block::Version).not_null().default(1))
                    .col(json_binary(Block::Content).not_null())
                    .col(float(Block::Order).not_null())
                    .col(uuid(Block::NoteId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Block::Table, Block::NoteId)
                            .to(Note::Table, Note::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Block::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Note::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Note {
    Table,
    Id,
    Title,
    UserId,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Archived,
    Pinned,
}

#[derive(DeriveIden)]
enum Block {
    Table,
    Id,
    Version,
    Content,
    Order,
    NoteId,
}
