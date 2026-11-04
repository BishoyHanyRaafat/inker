use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Note
/// Each note has id, title, created_at, updated_at, deleted_at, is_archived, is_pinned and user_id (foreign key to users
/// table)
///
/// Block
/// Each block has id, version, content, note_id (foreign key to notes table)
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
                    .table(NoteContent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NoteContent::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(integer(NoteContent::Version).not_null().default(1))
                    .col(text(NoteContent::Content).not_null())
                    .col(uuid(NoteContent::NoteId).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(NoteContent::Table, NoteContent::NoteId)
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
            .drop_table(Table::drop().table(NoteContent::Table).to_owned())
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
enum NoteContent {
    Table,
    Id,
    Version,
    Content,
    NoteId,
}
