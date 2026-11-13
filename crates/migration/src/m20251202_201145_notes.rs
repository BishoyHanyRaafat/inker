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
                    .col(
                        ColumnDef::new(Note::Version)
                            .string()
                            .not_null()
                            .default("1"),
                    )
                    .col(string(Note::Title).not_null())
                    .col(uuid(Note::UserId).not_null())
                    .col(text(Note::Content).not_null())
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
                    .col(string(Note::Subject).null())
                    .col(string(Note::Description).null())
                    .col(json_binary(Note::LearningObjectives).null())
                    .col(json_binary(Note::CoverageNotes).null())
                    .col(json_binary(Note::Tags).null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Note::Table, Note::UserId)
                            .to("user", "id")
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Note::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Note {
    Table,
    Id,
    Version,
    Title,
    UserId,
    Content,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Archived,
    Pinned,
    Subject,
    Description,
    LearningObjectives,
    CoverageNotes,
    Tags,
}
